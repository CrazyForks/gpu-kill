use anyhow::{Context, Result};
use std::{
    process::{Command, Stdio},
    time::Duration,
};
use tracing::{debug, info, warn};

/// SSH connection configuration
#[derive(Debug, Clone)]
pub struct SshConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub key_path: Option<String>,
    pub password: Option<String>,
    pub timeout: Duration,
}

impl SshConfig {
    /// Create a new SSH configuration
    pub fn new(host: String, port: u16, username: String) -> Self {
        Self {
            host,
            port,
            username,
            key_path: None,
            password: None,
            timeout: Duration::from_secs(30),
        }
    }

    /// Set SSH key path
    pub fn with_key_path(mut self, key_path: String) -> Self {
        self.key_path = Some(key_path);
        self
    }

    /// Set SSH password
    pub fn with_password(mut self, password: String) -> Self {
        self.password = Some(password);
        self
    }

    /// Set connection timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

/// SSH remote connection manager using system SSH
pub struct SshRemote {
    config: SshConfig,
}

impl SshRemote {
    /// Create a new SSH remote connection
    pub fn new(config: SshConfig) -> Self {
        Self { config }
    }

    /// Execute a command on the remote host
    pub fn execute_command(&self, command: &str) -> Result<String> {
        debug!("Executing remote command: {}", command);
        
        let mut ssh_cmd = Command::new("ssh");
        
        // Add SSH options
        ssh_cmd.arg("-o")
               .arg("ConnectTimeout=30")
               .arg("-o")
               .arg("StrictHostKeyChecking=no")
               .arg("-o")
               .arg("UserKnownHostsFile=/dev/null")
               .arg("-o")
               .arg("LogLevel=ERROR");
        
        // Add port if not default
        if self.config.port != 22 {
            ssh_cmd.arg("-p").arg(self.config.port.to_string());
        }
        
        // Add key file if specified
        if let Some(key_path) = &self.config.key_path {
            ssh_cmd.arg("-i").arg(key_path);
        }
        
        // Add password authentication if specified
        if self.config.password.is_some() {
            ssh_cmd.arg("-o").arg("PasswordAuthentication=yes");
        }
        
        // Add host and command
        let host_spec = format!("{}@{}", self.config.username, self.config.host);
        ssh_cmd.arg(host_spec).arg(command);
        
        // Set up input for password if needed
        if let Some(_password) = &self.config.password {
            ssh_cmd.stdin(Stdio::piped());
        }
        
        debug!("Running SSH command: {:?}", ssh_cmd);
        
        let mut child = ssh_cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn SSH command")?;
        
        // Send password if provided
        if let Some(password) = &self.config.password {
            if let Some(stdin) = child.stdin.as_mut() {
                use std::io::Write;
                stdin.write_all(password.as_bytes())
                    .context("Failed to write password to SSH stdin")?;
                stdin.write_all(b"\n")
                    .context("Failed to write newline to SSH stdin")?;
            }
        }
        
        let output = child.wait_with_output()
            .context("Failed to wait for SSH command")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "SSH command failed with exit code {}: {}",
                output.status.code().unwrap_or(-1),
                stderr
            ));
        }
        
        let stdout = String::from_utf8(output.stdout)
            .context("Failed to decode SSH command output as UTF-8")?;
        
        debug!("Command executed successfully, output length: {} bytes", stdout.len());
        Ok(stdout)
    }

    /// Execute gpukill command on remote host
    pub fn execute_gpukill(&self, args: &[String]) -> Result<String> {
        let command = format!("gpukill {}", args.join(" "));
        self.execute_command(&command)
    }

    /// Check if gpukill is available on remote host
    pub fn check_gpukill_availability(&self) -> Result<bool> {
        match self.execute_command("which gpukill") {
            Ok(output) => {
                let available = !output.trim().is_empty();
                if available {
                    info!("gpukill is available on remote host");
                } else {
                    warn!("gpukill not found on remote host");
                }
                Ok(available)
            }
            Err(_) => {
                warn!("Failed to check gpukill availability on remote host");
                Ok(false)
            }
        }
    }

    /// Get remote host information
    pub fn get_host_info(&self) -> Result<RemoteHostInfo> {
        let hostname = self.execute_command("hostname")?.trim().to_string();
        let os_info = self.execute_command("uname -a")?.trim().to_string();
        let gpu_info = self.execute_command("nvidia-smi --query-gpu=name --format=csv,noheader,nounits 2>/dev/null || echo 'No NVIDIA GPUs'")?.trim().to_string();
        
        Ok(RemoteHostInfo {
            hostname,
            os_info,
            gpu_info,
        })
    }
}

/// Information about the remote host
#[derive(Debug, Clone)]
pub struct RemoteHostInfo {
    pub hostname: String,
    pub os_info: String,
    #[allow(dead_code)]
    pub gpu_info: String,
}

/// Execute a local gpukill command with remote forwarding
pub fn execute_remote_operation(
    config: SshConfig,
    local_args: &[String],
) -> Result<()> {
    let remote = SshRemote::new(config);
    
    // Check if gpukill is available on remote host
    if !remote.check_gpukill_availability()? {
        return Err(anyhow::anyhow!(
            "gpukill is not available on the remote host. Please install gpukill on the remote host first."
        ));
    }
    
    // Get remote host info
    let host_info = remote.get_host_info()?;
    info!("Remote host: {} ({})", host_info.hostname, host_info.os_info);
    
    // Execute the command on remote host
    let output = remote.execute_gpukill(local_args)?;
    
    // Print the output
    print!("{}", output);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_ssh_config_creation() {
        let config = SshConfig::new("localhost".to_string(), 22, "testuser".to_string());
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 22);
        assert_eq!(config.username, "testuser");
        assert_eq!(config.timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_ssh_config_with_options() {
        let config = SshConfig::new("localhost".to_string(), 22, "testuser".to_string())
            .with_key_path("/path/to/key".to_string())
            .with_password("password".to_string())
            .with_timeout(Duration::from_secs(60));
        
        assert_eq!(config.key_path, Some("/path/to/key".to_string()));
        assert_eq!(config.password, Some("password".to_string()));
        assert_eq!(config.timeout, Duration::from_secs(60));
    }
}