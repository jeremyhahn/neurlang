//! Remote SSH session management for training

use anyhow::{Context, Result};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

/// Remote session configuration
#[derive(Debug, Clone)]
pub struct RemoteConfig {
    /// SSH host (user@hostname)
    pub host: String,
    /// Remote working directory
    pub work_dir: String,
    /// SSH connection timeout (seconds)
    pub timeout: u32,
    /// SSH identity file (optional)
    pub identity_file: Option<String>,
    /// SSH port (default 22)
    pub port: u16,
}

impl Default for RemoteConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            work_dir: "~/neurlang".to_string(),
            timeout: 30,
            identity_file: None,
            port: 22,
        }
    }
}

/// Training status reported from remote
#[derive(Debug, Clone)]
pub struct TrainingStatus {
    pub epoch: usize,
    pub total_epochs: usize,
    pub train_loss: f64,
    pub val_loss: Option<f64>,
    pub accuracy: Option<f64>,
    pub eta_seconds: Option<u64>,
}

/// Remote SSH session for training operations
pub struct RemoteSession {
    config: RemoteConfig,
}

impl RemoteSession {
    /// Create a new remote session
    pub fn new(config: RemoteConfig) -> Self {
        Self { config }
    }

    /// Test SSH connectivity
    pub fn test_connection(&self) -> Result<bool> {
        let output = self.ssh_command_output("echo ok")?;
        Ok(output.trim() == "ok")
    }

    /// Get GPU information from remote
    pub fn get_gpu_info(&self) -> Result<String> {
        self.ssh_command_output("nvidia-smi --query-gpu=name,memory.total --format=csv,noheader 2>/dev/null || echo 'No GPU found'")
    }

    /// Get Python version from remote
    pub fn get_python_version(&self) -> Result<String> {
        self.ssh_command_output("python3 --version 2>&1 || echo 'Python not found'")
    }

    /// Check if venv exists on remote
    pub fn has_venv(&self) -> Result<bool> {
        let output =
            self.ssh_command_output("test -f ~/venv/bin/python && echo 'yes' || echo 'no'")?;
        Ok(output.trim() == "yes")
    }

    /// Check PyTorch installation
    pub fn get_pytorch_version(&self) -> Result<String> {
        self.ssh_command_output(
            "source ~/venv/bin/activate 2>/dev/null && python -c 'import torch; print(torch.__version__)' 2>/dev/null || echo 'Not installed'"
        )
    }

    /// Create remote directory
    pub fn mkdir(&self, path: &str) -> Result<()> {
        self.ssh_command(&format!("mkdir -p {}", path))?;
        Ok(())
    }

    /// Check if file exists on remote
    pub fn file_exists(&self, path: &str) -> Result<bool> {
        let output =
            self.ssh_command_output(&format!("test -f {} && echo 'yes' || echo 'no'", path))?;
        Ok(output.trim() == "yes")
    }

    /// Get file size on remote
    pub fn file_size(&self, path: &str) -> Result<u64> {
        let output =
            self.ssh_command_output(&format!("stat -c%s {} 2>/dev/null || echo '0'", path))?;
        output.trim().parse().context("Failed to parse file size")
    }

    /// Run SSH command and get output
    fn ssh_command_output(&self, cmd: &str) -> Result<String> {
        let output = Command::new("ssh")
            .args(self.ssh_args())
            .arg(&self.config.host)
            .arg(cmd)
            .output()
            .context("Failed to execute SSH command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("SSH command failed: {}", stderr);
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Run SSH command (no output capture)
    fn ssh_command(&self, cmd: &str) -> Result<()> {
        let status = Command::new("ssh")
            .args(self.ssh_args())
            .arg(&self.config.host)
            .arg(cmd)
            .status()
            .context("Failed to execute SSH command")?;

        if !status.success() {
            anyhow::bail!("SSH command failed with status: {}", status);
        }

        Ok(())
    }

    /// Run SSH command with streaming output (for training)
    pub fn run_streaming(&self, cmd: &str) -> Result<()> {
        let mut child = Command::new("ssh")
            .args(self.ssh_args())
            .args(["-t", "-t"]) // Force PTY
            .arg(&self.config.host)
            .arg(cmd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn SSH command")?;

        // Stream stdout
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            for line in reader.lines().flatten() {
                println!("{}", line);
            }
        }

        let status = child.wait()?;
        if !status.success() {
            anyhow::bail!("SSH command failed with status: {}", status);
        }

        Ok(())
    }

    /// Get SSH arguments based on config
    fn ssh_args(&self) -> Vec<String> {
        let mut args = vec![
            "-o".to_string(),
            "StrictHostKeyChecking=no".to_string(),
            "-o".to_string(),
            "BatchMode=yes".to_string(),
            "-o".to_string(),
            format!("ConnectTimeout={}", self.config.timeout),
        ];

        if self.config.port != 22 {
            args.push("-p".to_string());
            args.push(self.config.port.to_string());
        }

        if let Some(ref identity) = self.config.identity_file {
            args.push("-i".to_string());
            args.push(identity.clone());
        }

        args
    }
}

/// Check if rsync is available on this system
pub fn rsync_available() -> bool {
    Command::new("rsync")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
