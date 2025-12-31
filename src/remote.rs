use anyhow::{Context, Result};
use shell_escape::escape;
use std::{
    borrow::Cow,
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
    ///
    /// # Authentication
    /// - SSH key authentication is preferred and works automatically
    /// - Password authentication requires `sshpass` to be installed on the system,
    ///   as SSH requires a TTY for interactive password prompts
    pub fn execute_command(&self, command: &str) -> Result<String> {
        debug!("Executing remote command: {}", command);

        // Build SSH options that are common to both sshpass and direct SSH
        let timeout_secs = self.config.timeout.as_secs();
        let host_spec = format!("{}@{}", self.config.username, self.config.host);

        // If password is provided, use sshpass (SSH requires TTY for password prompts)
        let output = if let Some(password) = &self.config.password {
            self.execute_with_sshpass(command, password, &host_spec, timeout_secs)?
        } else {
            self.execute_with_ssh(command, &host_spec, timeout_secs)?
        };

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

        debug!(
            "Command executed successfully, output length: {} bytes",
            stdout.len()
        );
        Ok(stdout)
    }

    /// Execute command using sshpass for password authentication
    ///
    /// SSH does not read passwords from stdin - it requires a controlling TTY.
    /// sshpass is a utility that provides password to SSH in a non-interactive way.
    fn execute_with_sshpass(
        &self,
        command: &str,
        password: &str,
        host_spec: &str,
        timeout_secs: u64,
    ) -> Result<std::process::Output> {
        // Check if sshpass is available
        let sshpass_check = Command::new("which").arg("sshpass").output();

        if sshpass_check.is_err() || !sshpass_check.unwrap().status.success() {
            return Err(anyhow::anyhow!(
                "Password authentication requires 'sshpass' to be installed. \
                SSH requires a TTY for password prompts, which is not available in this context. \
                Please install sshpass (e.g., 'apt install sshpass' or 'brew install sshpass') \
                or use SSH key authentication instead (--ssh-key)."
            ));
        }

        let mut cmd = Command::new("sshpass");
        cmd.arg("-p").arg(password);
        cmd.arg("ssh");

        // Add SSH options
        cmd.arg("-o")
            .arg(format!("ConnectTimeout={}", timeout_secs))
            .arg("-o")
            .arg("StrictHostKeyChecking=no")
            .arg("-o")
            .arg("UserKnownHostsFile=/dev/null")
            .arg("-o")
            .arg("LogLevel=ERROR")
            .arg("-o")
            .arg("PasswordAuthentication=yes")
            .arg("-o")
            .arg("PubkeyAuthentication=no");

        // Add port if not default
        if self.config.port != 22 {
            cmd.arg("-p").arg(self.config.port.to_string());
        }

        // Add host and command
        cmd.arg(host_spec).arg(command);

        debug!("Running SSH command with sshpass: {:?}", cmd);

        cmd.stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .context("Failed to execute sshpass command")
    }

    /// Execute command using direct SSH (key-based or agent authentication)
    fn execute_with_ssh(
        &self,
        command: &str,
        host_spec: &str,
        timeout_secs: u64,
    ) -> Result<std::process::Output> {
        let mut ssh_cmd = Command::new("ssh");

        // Add SSH options
        ssh_cmd
            .arg("-o")
            .arg(format!("ConnectTimeout={}", timeout_secs))
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

        // Add host and command
        ssh_cmd.arg(host_spec).arg(command);

        debug!("Running SSH command: {:?}", ssh_cmd);

        ssh_cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .context("Failed to execute SSH command")
    }

    /// Execute gpukill command on remote host
    ///
    /// # Security
    /// All arguments are properly shell-escaped to prevent command injection attacks.
    /// User-controlled input (e.g., --filter, --audit-user, --audit-process) is safely
    /// quoted before being passed to the remote shell.
    pub fn execute_gpukill(&self, args: &[String]) -> Result<String> {
        // Escape each argument to prevent shell metacharacter injection
        // This prevents attacks like: --filter "python; rm -rf /"
        let escaped_args: Vec<Cow<str>> = args
            .iter()
            .map(|arg| escape(Cow::Borrowed(arg.as_str())))
            .collect();
        let command = format!("gpukill {}", escaped_args.join(" "));
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
pub fn execute_remote_operation(config: SshConfig, local_args: &[String]) -> Result<()> {
    let remote = SshRemote::new(config);

    // Check if gpukill is available on remote host
    if !remote.check_gpukill_availability()? {
        return Err(anyhow::anyhow!(
            "gpukill is not available on the remote host. Please install gpukill on the remote host first."
        ));
    }

    // Get remote host info
    let host_info = remote.get_host_info()?;
    info!(
        "Remote host: {} ({})",
        host_info.hostname, host_info.os_info
    );

    // Execute the command on remote host
    let output = remote.execute_gpukill(local_args)?;

    // Print the output
    print!("{}", output);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;
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

    #[test]
    fn test_shell_escape_prevents_command_injection() {
        // Test that shell metacharacters are properly escaped
        let malicious_args = [
            "--filter".to_string(),
            "python; rm -rf /".to_string(), // Command injection attempt
        ];

        let escaped_args: Vec<Cow<str>> = malicious_args
            .iter()
            .map(|arg| escape(Cow::Borrowed(arg.as_str())))
            .collect();
        let command = format!("gpukill {}", escaped_args.join(" "));

        // The semicolon and dangerous command should be escaped/quoted
        // so they are treated as literal strings, not shell commands
        assert!(
            command.contains("'python; rm -rf /'") || command.contains("python\\; rm -rf /"),
            "Command injection attempt should be escaped: {}",
            command
        );
        // The command should NOT contain an unquoted semicolon that would
        // allow command chaining
        assert!(
            !command.contains(" python; rm"),
            "Unescaped semicolon would allow command injection: {}",
            command
        );
    }

    #[test]
    fn test_shell_escape_various_metacharacters() {
        // Test various shell metacharacters that could be used for injection
        let test_cases = vec![
            "test; whoami",             // Command chaining
            "test | cat /etc/passwd",   // Pipe
            "test && touch /tmp/pwned", // AND operator
            "test || touch /tmp/pwned", // OR operator
            "$(whoami)",                // Command substitution
            "`whoami`",                 // Backtick substitution
            "test > /tmp/file",         // Output redirection
            "test < /etc/passwd",       // Input redirection
            "$HOME",                    // Variable expansion
            "test\nwhoami",             // Newline injection
        ];

        for malicious_input in test_cases {
            let escaped = escape(Cow::Borrowed(malicious_input));
            // After escaping, the string should be safe to pass to a shell
            // It should either be quoted or have metacharacters escaped
            assert!(
                escaped.starts_with('\'') || escaped.contains('\\'),
                "Input '{}' should be escaped, got: {}",
                malicious_input,
                escaped
            );
        }
    }
}
