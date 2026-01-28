//! GPU-specific training profiles
//!
//! Optimized configurations for different GPU architectures:
//! - H100: Hopper architecture, 80GB HBM3
//! - H200: Hopper with 141GB HBM3e
//! - B300: Blackwell architecture, 288GB HBM3e

use std::fmt;

/// GPU profile for optimized training
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuProfile {
    /// NVIDIA H100 80GB (Hopper)
    H100,
    /// NVIDIA H200 141GB (Hopper with HBM3e)
    H200,
    /// NVIDIA B300 288GB (Blackwell)
    B300,
    /// NVIDIA L40S 48GB (Ada Lovelace)
    L40S,
    /// NVIDIA RTX A6000 48GB (Ampere)
    A6000,
    /// NVIDIA A100 80GB (Ampere)
    A100,
    /// NVIDIA A100 40GB (Ampere)
    A100_40GB,
    /// Generic CUDA GPU (conservative settings)
    Generic,
    /// CPU only (for testing)
    Cpu,
}

impl GpuProfile {
    /// Parse profile from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "h100" => Some(Self::H100),
            "h200" => Some(Self::H200),
            "b300" | "b200" | "blackwell" => Some(Self::B300),
            "l40s" | "l40" => Some(Self::L40S),
            "a6000" | "rtx-a6000" => Some(Self::A6000),
            "a100" | "a100-80gb" => Some(Self::A100),
            "a100-40gb" => Some(Self::A100_40GB),
            "generic" | "cuda" => Some(Self::Generic),
            "cpu" => Some(Self::Cpu),
            _ => None,
        }
    }

    /// Profile name
    pub fn name(&self) -> &'static str {
        match self {
            Self::H100 => "h100",
            Self::H200 => "h200",
            Self::B300 => "b300",
            Self::L40S => "l40s",
            Self::A6000 => "a6000",
            Self::A100 => "a100",
            Self::A100_40GB => "a100-40gb",
            Self::Generic => "generic",
            Self::Cpu => "cpu",
        }
    }

    /// Human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::H100 => "NVIDIA H100 80GB HBM3 (Hopper)",
            Self::H200 => "NVIDIA H200 141GB HBM3e (Hopper)",
            Self::B300 => "NVIDIA B300 288GB HBM3e (Blackwell)",
            Self::L40S => "NVIDIA L40S 48GB (Ada Lovelace)",
            Self::A6000 => "NVIDIA RTX A6000 48GB GDDR6 (Ampere)",
            Self::A100 => "NVIDIA A100 80GB HBM2e (Ampere)",
            Self::A100_40GB => "NVIDIA A100 40GB HBM2e (Ampere)",
            Self::Generic => "Generic CUDA GPU (conservative)",
            Self::Cpu => "CPU only (for testing)",
        }
    }

    /// VRAM in GB
    pub fn vram_gb(&self) -> usize {
        match self {
            Self::H100 => 80,
            Self::H200 => 141,
            Self::B300 => 288,
            Self::L40S => 48,
            Self::A6000 => 48,
            Self::A100 => 80,
            Self::A100_40GB => 40,
            Self::Generic => 24,
            Self::Cpu => 0,
        }
    }

    /// Optimal batch size for this GPU
    /// Note: B300 uses smaller batch due to manual attention (no flash attn support)
    pub fn batch_size(&self) -> usize {
        match self {
            Self::H100 => 4096,      // 80GB, fast HBM3
            Self::H200 => 6144,      // 141GB, HBM3e
            Self::B300 => 512,       // 288GB, Blackwell - reduced for manual attention
            Self::L40S => 2048,      // 48GB, good performance
            Self::A6000 => 2048,     // 48GB GDDR6
            Self::A100 => 4096,      // 80GB
            Self::A100_40GB => 2048, // 40GB
            Self::Generic => 512,    // Conservative
            Self::Cpu => 32,         // CPU-only
        }
    }

    /// Number of transformer layers
    pub fn n_layer(&self) -> usize {
        match self {
            Self::H100 | Self::H200 | Self::B300 => 12,
            Self::A100 | Self::L40S | Self::A6000 => 10,
            Self::A100_40GB => 8,
            Self::Generic => 6,
            Self::Cpu => 4,
        }
    }

    /// Number of attention heads
    pub fn n_head(&self) -> usize {
        match self {
            Self::H100 | Self::H200 | Self::B300 => 12,
            Self::A100 | Self::L40S | Self::A6000 => 10,
            Self::A100_40GB => 8,
            Self::Generic => 8,
            Self::Cpu => 4,
        }
    }

    /// Embedding dimension
    pub fn n_embd(&self) -> usize {
        match self {
            Self::H100 | Self::H200 | Self::B300 => 768,
            Self::A100 | Self::L40S | Self::A6000 => 640,
            Self::A100_40GB => 512,
            Self::Generic => 384,
            Self::Cpu => 256,
        }
    }

    /// Extra CLI arguments for this profile
    pub fn extra_args(&self) -> &'static str {
        match self {
            Self::H100 => "--bf16",
            Self::H200 => "--bf16",
            Self::B300 => "--bf16 --no-flash --grad-accum 16", // Blackwell: no flash attn, use grad accum for effective batch
            Self::L40S => "--bf16",
            Self::A6000 => "--bf16",
            Self::A100 => "--bf16",
            Self::A100_40GB => "--bf16",
            Self::Generic => "",
            Self::Cpu => "",
        }
    }

    /// Recommended learning rate
    pub fn learning_rate(&self) -> f64 {
        match self {
            Self::H100 | Self::H200 | Self::B300 => 3e-4,
            Self::A100 | Self::L40S | Self::A6000 => 3e-4,
            Self::A100_40GB => 2e-4,
            Self::Generic => 1e-4,
            Self::Cpu => 1e-4,
        }
    }

    /// Whether to use flash attention
    pub fn use_flash_attention(&self) -> bool {
        match self {
            Self::H100 | Self::H200 => true, // Native support
            Self::B300 => false,             // Blackwell uses SDPA
            Self::A100 | Self::L40S | Self::A6000 => true,
            Self::A100_40GB => true,
            Self::Generic => false,
            Self::Cpu => false,
        }
    }

    /// Whether to use torch.compile
    pub fn use_compile(&self) -> bool {
        match self {
            Self::H100 | Self::H200 => true,
            Self::B300 => false, // cuDNN compatibility issues
            Self::A100 | Self::L40S | Self::A6000 => true,
            Self::A100_40GB => true,
            Self::Generic => false,
            Self::Cpu => false,
        }
    }

    /// CUDA architecture string for torch
    pub fn cuda_arch(&self) -> &'static str {
        match self {
            Self::H100 | Self::H200 => "sm_90",
            Self::B300 => "sm_100", // Blackwell
            Self::A100 | Self::A100_40GB => "sm_80",
            Self::A6000 => "sm_86", // Ampere GA102
            Self::L40S => "sm_89",  // Ada Lovelace
            Self::Generic | Self::Cpu => "sm_75",
        }
    }

    /// PyTorch CUDA index string
    pub fn cuda_version(&self) -> &'static str {
        match self {
            Self::B300 => "cu128", // Blackwell needs CUDA 12.8+
            _ => "cu124",          // CUDA 12.4 for others
        }
    }

    /// Provisioner script path
    pub fn provisioner_script(&self) -> &'static str {
        match self {
            Self::B300 => "scripts/setup-b300.sh",
            Self::H100 | Self::H200 => "scripts/setup-hopper.sh",
            Self::A100 | Self::A100_40GB => "scripts/setup-ampere.sh",
            Self::A6000 => "scripts/setup-a6000.sh",
            Self::L40S => "scripts/setup-ada.sh",
            _ => "scripts/setup-remote.sh",
        }
    }
}

impl fmt::Display for GpuProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Get a profile by name
pub fn get_profile(name: &str) -> Option<GpuProfile> {
    GpuProfile::from_str(name)
}

/// List all available profiles
pub fn list_profiles() -> Vec<GpuProfile> {
    vec![
        GpuProfile::H100,
        GpuProfile::H200,
        GpuProfile::B300,
        GpuProfile::L40S,
        GpuProfile::A6000,
        GpuProfile::A100,
        GpuProfile::A100_40GB,
        GpuProfile::Generic,
        GpuProfile::Cpu,
    ]
}

/// Print profile comparison table
pub fn print_profile_table() {
    println!("GPU Training Profiles:");
    println!("======================");
    println!();
    println!(
        "{:<12} {:>8} {:>8} {:>8} {:>8} {:>10}",
        "Profile", "VRAM", "Batch", "Layers", "Heads", "Embed"
    );
    println!(
        "{:-<12} {:->8} {:->8} {:->8} {:->8} {:->10}",
        "", "", "", "", "", ""
    );

    for profile in list_profiles() {
        println!(
            "{:<12} {:>6}GB {:>8} {:>8} {:>8} {:>10}",
            profile.name(),
            profile.vram_gb(),
            profile.batch_size(),
            profile.n_layer(),
            profile.n_head(),
            profile.n_embd()
        );
    }

    println!();
    println!("Usage: nl train --profile h100 --remote user@host");
}
