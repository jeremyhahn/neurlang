//! Remote training infrastructure for Neurlang
//!
//! Provides SSH-based remote training capabilities with GPU-optimized profiles.
//! Supports H100, H200, and B300 GPUs with automatic provisioning.

mod profiles;
mod remote;
mod sync;

pub use profiles::{get_profile, list_profiles, print_profile_table, GpuProfile};
pub use remote::{RemoteConfig, RemoteSession, TrainingStatus};
pub use sync::{sync_from_remote, sync_to_remote, SyncConfig};

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Training backend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TrainingBackend {
    /// PyTorch training scripts (default, full-featured)
    #[default]
    PyTorch,
    /// ONNX Runtime training (simpler, cross-platform)
    Onnx,
}

impl TrainingBackend {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "pytorch" | "torch" | "pt" => Some(Self::PyTorch),
            "onnx" | "onnxruntime" | "ort" => Some(Self::Onnx),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::PyTorch => "pytorch",
            Self::Onnx => "onnx",
        }
    }
}

/// Training configuration
#[derive(Debug, Clone)]
pub struct TrainingConfig {
    /// Path to training data
    pub data_path: PathBuf,
    /// Output model path (local)
    pub output_path: PathBuf,
    /// GPU profile to use
    pub profile: GpuProfile,
    /// Training backend
    pub backend: TrainingBackend,
    /// Remote host (user@host)
    pub remote_host: String,
    /// Remote working directory
    pub remote_dir: String,
    /// Path to provisioner script
    pub provisioner: Option<PathBuf>,
    /// Number of training epochs
    pub epochs: usize,
    /// Early stopping patience
    pub patience: usize,
    /// Use cross-validation
    pub cross_validate: bool,
    /// Number of CV folds
    pub folds: usize,
    /// Verbose output
    pub verbose: bool,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            data_path: PathBuf::from("train/training_data.jsonl"),
            output_path: PathBuf::from("train/model.pt"),
            profile: GpuProfile::H100,
            backend: TrainingBackend::default(),
            remote_host: String::new(),
            remote_dir: "~/neurlang".to_string(),
            provisioner: Some(PathBuf::from("scripts/setup-remote.sh")),
            epochs: 20,
            patience: 5,
            cross_validate: false,
            folds: 5,
            verbose: false,
        }
    }
}

/// Run training on a remote GPU server
pub fn run_remote_training(config: &TrainingConfig) -> Result<()> {
    println!("=== Neurlang Remote Training ===");
    println!();

    // Validate configuration
    if config.remote_host.is_empty() {
        anyhow::bail!("Remote host is required (--remote user@host)");
    }

    if !config.data_path.exists() {
        anyhow::bail!("Training data not found: {}", config.data_path.display());
    }

    let profile = &config.profile;
    println!("Configuration:");
    println!("  Remote Host: {}", config.remote_host);
    println!("  Remote Dir:  {}", config.remote_dir);
    println!(
        "  GPU Profile: {} ({})",
        profile.name(),
        profile.description()
    );
    println!("  Data Path:   {}", config.data_path.display());
    println!("  Output:      {}", config.output_path.display());
    println!("  Epochs:      {}", config.epochs);
    println!("  Patience:    {}", config.patience);
    println!();

    // Step 1: Provision remote environment
    if let Some(ref provisioner) = config.provisioner {
        println!("[1/5] Provisioning remote environment...");
        provision_remote(
            &config.remote_host,
            &config.remote_dir,
            provisioner,
            config.verbose,
        )?;
    } else {
        println!("[1/5] Skipping provisioning (--no-provision)");
    }

    // Step 2: Sync training files to remote
    println!("[2/5] Syncing training files to remote...");
    sync_training_files(config)?;

    // Step 3: Run training on remote
    println!("[3/5] Starting remote training...");
    run_training_command(config)?;

    // Step 4: Download trained model
    println!("[4/5] Downloading trained model...");
    download_model(config)?;

    // Step 5: Verify model
    println!("[5/5] Verifying model...");
    verify_model(&config.output_path)?;

    println!();
    println!("=== Training Complete ===");
    println!("Model saved to: {}", config.output_path.display());

    Ok(())
}

fn provision_remote(host: &str, remote_dir: &str, provisioner: &Path, verbose: bool) -> Result<()> {
    if !provisioner.exists() {
        anyhow::bail!("Provisioner script not found: {}", provisioner.display());
    }

    // Create remote directory
    let mkdir_cmd = format!("mkdir -p {}", remote_dir);
    run_ssh_command(host, &mkdir_cmd, verbose)?;

    // Copy provisioner script
    let remote_script = format!("{}/.provisioner.sh", remote_dir);
    run_scp_to_remote(host, provisioner, &remote_script, verbose)?;

    // Run provisioner
    let run_cmd = format!("chmod +x {} && bash {}", remote_script, remote_script);
    run_ssh_command(host, &run_cmd, verbose)?;

    Ok(())
}

fn sync_training_files(config: &TrainingConfig) -> Result<()> {
    let host = &config.remote_host;
    let remote_dir = &config.remote_dir;
    let verbose = config.verbose;

    // Files to sync - parallel model architecture
    let files = vec![
        ("train/parallel/model.py", "train/parallel/model.py"),
        ("train/parallel/train.py", "train/parallel/train.py"),
        (
            "train/parallel/generate_balanced_data.py",
            "train/parallel/generate_balanced_data.py",
        ),
        (
            "train/parallel/export_onnx.py",
            "train/parallel/export_onnx.py",
        ),
        (
            config.data_path.to_str().unwrap(),
            "train/training_data.jsonl",
        ),
    ];

    // Create remote train directory structure
    run_ssh_command(
        host,
        &format!("mkdir -p {}/train/parallel", remote_dir),
        verbose,
    )?;

    // Sync each file
    for (local, remote) in files {
        let local_path = Path::new(local);
        if local_path.exists() {
            let remote_path = format!("{}/{}", remote_dir, remote);
            println!("  Syncing {} -> {}", local, remote_path);
            run_scp_to_remote(host, local_path, &remote_path, verbose)?;
        }
    }

    Ok(())
}

fn run_training_command(config: &TrainingConfig) -> Result<()> {
    let host = &config.remote_host;
    let remote_dir = &config.remote_dir;
    let profile = &config.profile;

    let train_cmd = match config.backend {
        TrainingBackend::PyTorch => {
            // PyTorch training with parallel instruction model
            format!(
                r#"cd {remote_dir}/train && \
source ~/venv/bin/activate && \
PYTHONPATH=. python -u parallel/train.py \
  --data training_data.jsonl \
  --output models/model.pt \
  --device cuda \
  --epochs {epochs} \
  --patience {patience} \
  --batch-size {batch_size} \
  --lr {learning_rate} \
  --mixed-precision \
  --num-workers 8 \
  2>&1 | tee training.log"#,
                remote_dir = remote_dir,
                epochs = config.epochs,
                patience = config.patience,
                batch_size = profile.batch_size(),
                learning_rate = profile.learning_rate(),
            )
        }
        TrainingBackend::Onnx => {
            // ONNX export after PyTorch training
            format!(
                r#"cd {remote_dir}/train && \
source ~/venv/bin/activate && \
PYTHONPATH=. python -u parallel/export_onnx.py \
  --model models/model.pt \
  --output models/parallel.onnx \
  2>&1 | tee export.log"#,
                remote_dir = remote_dir,
            )
        }
    };

    println!("  Using {} backend", config.backend.name());

    // Run with real-time output
    run_ssh_command_streaming(host, &train_cmd)?;

    Ok(())
}

fn download_model(config: &TrainingConfig) -> Result<()> {
    let host = &config.remote_host;
    let remote_dir = &config.remote_dir;
    let verbose = config.verbose;

    // Download the model from the models directory
    let remote_model = format!("{}/train/models/model.pt", remote_dir);
    run_scp_from_remote(host, &remote_model, &config.output_path, verbose)?;

    // Also download config if exists
    let remote_config = format!("{}/train/models/model.config.json", remote_dir);
    let local_config = config.output_path.with_extension("config.json");
    if let Err(_) = run_scp_from_remote(host, &remote_config, &local_config, verbose) {
        // Config is optional, don't fail
        println!("  (config file not found, skipping)");
    }

    Ok(())
}

fn verify_model(model_path: &Path) -> Result<()> {
    if !model_path.exists() {
        anyhow::bail!("Model file not found after download");
    }

    let metadata = std::fs::metadata(model_path)?;
    println!(
        "  Model size: {:.2} MB",
        metadata.len() as f64 / 1_000_000.0
    );

    Ok(())
}

// SSH/SCP helper functions

fn run_ssh_command(host: &str, cmd: &str, verbose: bool) -> Result<()> {
    let output = Command::new("ssh")
        .args(["-o", "StrictHostKeyChecking=no"])
        .args(["-o", "BatchMode=yes"])
        .arg(host)
        .arg(cmd)
        .output()
        .context("Failed to execute SSH command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("SSH command failed: {}", stderr);
    }

    if verbose {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.is_empty() {
            println!("{}", stdout);
        }
    }

    Ok(())
}

fn run_ssh_command_streaming(host: &str, cmd: &str) -> Result<()> {
    use std::fs::OpenOptions;
    use std::io::{BufRead, BufReader, Write};
    use std::process::Stdio;

    // Open local log file
    let log_path = "train/training.log";
    let mut log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)
        .context("Failed to create local log file")?;

    println!("  Streaming output to console and {}", log_path);
    println!();

    // Start SSH process with piped stdout
    let mut child = Command::new("ssh")
        .args(["-o", "StrictHostKeyChecking=no"])
        .args(["-o", "BatchMode=yes"])
        .arg(host)
        .arg(cmd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to start SSH process")?;

    // Stream stdout in real-time
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line) = line {
                // Print to console
                println!("{}", line);
                // Write to log file
                writeln!(log_file, "{}", line).ok();
                log_file.flush().ok();
            }
        }
    }

    // Also capture stderr
    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        for line in reader.lines().flatten() {
            eprintln!("{}", line);
            writeln!(log_file, "[stderr] {}", line).ok();
        }
    }

    let status = child.wait().context("Failed to wait for SSH process")?;

    if !status.success() {
        anyhow::bail!("Training failed with status: {}", status);
    }

    Ok(())
}

fn run_scp_to_remote(host: &str, local: &Path, remote: &str, verbose: bool) -> Result<()> {
    let remote_path = format!("{}:{}", host, remote);

    let mut cmd = Command::new("scp");
    cmd.args(["-o", "StrictHostKeyChecking=no"]);
    if !verbose {
        cmd.arg("-q");
    }
    cmd.arg(local);
    cmd.arg(&remote_path);

    let output = cmd.output().context("Failed to execute SCP command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("SCP upload failed: {}", stderr);
    }

    Ok(())
}

fn run_scp_from_remote(host: &str, remote: &str, local: &Path, verbose: bool) -> Result<()> {
    let remote_path = format!("{}:{}", host, remote);

    // Ensure local directory exists
    if let Some(parent) = local.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut cmd = Command::new("scp");
    cmd.args(["-o", "StrictHostKeyChecking=no"]);
    if !verbose {
        cmd.arg("-q");
    }
    cmd.arg(&remote_path);
    cmd.arg(local);

    let output = cmd.output().context("Failed to execute SCP command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("SCP download failed: {}", stderr);
    }

    Ok(())
}
