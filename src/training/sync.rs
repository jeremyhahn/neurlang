//! File synchronization for remote training

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Sync configuration
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// Use rsync if available, fall back to scp
    pub prefer_rsync: bool,
    /// Show progress during sync
    pub show_progress: bool,
    /// Compress during transfer
    pub compress: bool,
    /// SSH port
    pub port: u16,
    /// SSH identity file
    pub identity_file: Option<String>,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            prefer_rsync: true,
            show_progress: true,
            compress: true,
            port: 22,
            identity_file: None,
        }
    }
}

/// Sync files to remote host
pub fn sync_to_remote(
    host: &str,
    local_path: &Path,
    remote_path: &str,
    config: &SyncConfig,
) -> Result<()> {
    if config.prefer_rsync && super::remote::rsync_available() {
        rsync_to_remote(host, local_path, remote_path, config)
    } else {
        scp_to_remote(host, local_path, remote_path, config)
    }
}

/// Sync files from remote host
pub fn sync_from_remote(
    host: &str,
    remote_path: &str,
    local_path: &Path,
    config: &SyncConfig,
) -> Result<()> {
    if config.prefer_rsync && super::remote::rsync_available() {
        rsync_from_remote(host, remote_path, local_path, config)
    } else {
        scp_from_remote(host, remote_path, local_path, config)
    }
}

fn rsync_to_remote(
    host: &str,
    local_path: &Path,
    remote_path: &str,
    config: &SyncConfig,
) -> Result<()> {
    let remote_full = format!("{}:{}", host, remote_path);

    let mut cmd = Command::new("rsync");
    cmd.arg("-avz");

    if config.show_progress {
        cmd.arg("--progress");
    }

    if !config.compress {
        cmd.arg("--no-compress");
    }

    // SSH options
    let mut ssh_opts = format!("ssh -o StrictHostKeyChecking=no -p {}", config.port);
    if let Some(ref identity) = config.identity_file {
        ssh_opts.push_str(&format!(" -i {}", identity));
    }
    cmd.arg("-e").arg(&ssh_opts);

    cmd.arg(local_path);
    cmd.arg(&remote_full);

    let status = cmd.status().context("Failed to execute rsync")?;

    if !status.success() {
        anyhow::bail!("rsync failed with status: {}", status);
    }

    Ok(())
}

fn rsync_from_remote(
    host: &str,
    remote_path: &str,
    local_path: &Path,
    config: &SyncConfig,
) -> Result<()> {
    let remote_full = format!("{}:{}", host, remote_path);

    // Ensure local directory exists
    if let Some(parent) = local_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut cmd = Command::new("rsync");
    cmd.arg("-avz");

    if config.show_progress {
        cmd.arg("--progress");
    }

    if !config.compress {
        cmd.arg("--no-compress");
    }

    // SSH options
    let mut ssh_opts = format!("ssh -o StrictHostKeyChecking=no -p {}", config.port);
    if let Some(ref identity) = config.identity_file {
        ssh_opts.push_str(&format!(" -i {}", identity));
    }
    cmd.arg("-e").arg(&ssh_opts);

    cmd.arg(&remote_full);
    cmd.arg(local_path);

    let status = cmd.status().context("Failed to execute rsync")?;

    if !status.success() {
        anyhow::bail!("rsync failed with status: {}", status);
    }

    Ok(())
}

fn scp_to_remote(
    host: &str,
    local_path: &Path,
    remote_path: &str,
    config: &SyncConfig,
) -> Result<()> {
    let remote_full = format!("{}:{}", host, remote_path);

    let mut cmd = Command::new("scp");
    cmd.arg("-o").arg("StrictHostKeyChecking=no");

    if config.port != 22 {
        cmd.arg("-P").arg(config.port.to_string());
    }

    if let Some(ref identity) = config.identity_file {
        cmd.arg("-i").arg(identity);
    }

    if config.compress {
        cmd.arg("-C");
    }

    // Recursive for directories
    if local_path.is_dir() {
        cmd.arg("-r");
    }

    cmd.arg(local_path);
    cmd.arg(&remote_full);

    let status = cmd.status().context("Failed to execute scp")?;

    if !status.success() {
        anyhow::bail!("scp failed with status: {}", status);
    }

    Ok(())
}

fn scp_from_remote(
    host: &str,
    remote_path: &str,
    local_path: &Path,
    config: &SyncConfig,
) -> Result<()> {
    let remote_full = format!("{}:{}", host, remote_path);

    // Ensure local directory exists
    if let Some(parent) = local_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut cmd = Command::new("scp");
    cmd.arg("-o").arg("StrictHostKeyChecking=no");

    if config.port != 22 {
        cmd.arg("-P").arg(config.port.to_string());
    }

    if let Some(ref identity) = config.identity_file {
        cmd.arg("-i").arg(identity);
    }

    if config.compress {
        cmd.arg("-C");
    }

    cmd.arg(&remote_full);
    cmd.arg(local_path);

    let status = cmd.status().context("Failed to execute scp")?;

    if !status.success() {
        anyhow::bail!("scp failed with status: {}", status);
    }

    Ok(())
}
