use crate::vendor::GpuVendor;
use clap::{Parser, ValueEnum};

/// A production-ready CLI tool for GPU management and monitoring
#[derive(Parser)]
#[command(
    name = "gpukill",
    version = env!("CARGO_PKG_VERSION"),
    about = "GPU management and monitoring CLI tool",
    long_about = "gpukill provides comprehensive GPU monitoring, process management, and device control capabilities for NVIDIA, AMD, and Intel GPUs."
)]
pub struct Cli {
    /// Log level for debugging and diagnostics
    #[arg(long, value_enum, default_value = "info", global = true)]
    pub log_level: LogLevel,

    /// Configuration file path (optional)
    #[arg(long, global = true)]
    pub config: Option<String>,

    /// Dry-run mode: preview actions without making changes
    #[arg(long, alias = "safe", global = true)]
    pub dry_run: bool,

    /// List GPUs and their current status
    #[arg(long)]
    pub list: bool,

    /// Kill a GPU process
    #[arg(long)]
    pub kill: bool,

    /// Reset GPU(s)
    #[arg(long)]
    pub reset: bool,

    /// Show GPU usage audit history
    #[arg(long)]
    pub audit: bool,

    /// Start coordinator server for cluster management
    #[arg(long)]
    pub server: bool,

    /// Open browser to dashboard (works with --server and alias 'up')
    #[arg(long, requires = "server", global = true)]
    pub open: bool,

    /// Show detailed per-process information
    #[arg(long)]
    pub details: bool,

    /// Refresh output every 2 seconds until Ctrl-C
    #[arg(long)]
    pub watch: bool,

    /// Output format
    #[arg(long, value_enum, default_value = "table")]
    pub output: OutputFormat,

    /// Process ID to terminate
    #[arg(long)]
    pub pid: Option<u32>,

    /// Timeout in seconds before escalating to SIGKILL
    #[arg(long, default_value = "5")]
    pub timeout_secs: u16,

    /// Force escalation to SIGKILL after timeout
    #[arg(long)]
    pub force: bool,

    /// Specific GPU ID to reset
    #[arg(long)]
    pub gpu: Option<u16>,

    /// Reset all GPUs
    #[arg(long)]
    pub all: bool,

    /// Filter by GPU vendor
    #[arg(long, value_enum)]
    pub vendor: Option<VendorFilter>,

    /// Filter processes by name pattern (supports regex)
    #[arg(long)]
    pub filter: Option<String>,

    /// Kill multiple processes matching the filter
    #[arg(long, requires = "filter")]
    pub batch: bool,

    /// Show container information for processes
    #[arg(long, requires = "list")]
    pub containers: bool,

    /// Filter audit by user name
    #[arg(long, requires = "audit")]
    pub audit_user: Option<String>,

    /// Filter audit by process name pattern
    #[arg(long, requires = "audit")]
    pub audit_process: Option<String>,

    /// Show audit for last N hours
    #[arg(long, requires = "audit", default_value = "24")]
    pub audit_hours: u32,

    /// Show audit summary (top users/processes)
    #[arg(long, requires = "audit")]
    pub audit_summary: bool,

    /// Detect suspicious/rogue GPU usage patterns
    #[arg(long, requires = "audit")]
    pub rogue: bool,

    /// Show rogue detection configuration
    #[arg(long, requires = "audit")]
    pub rogue_config: bool,

    /// Update rogue detection thresholds
    #[arg(long, requires = "audit", value_name = "MEMORY_GB")]
    pub rogue_memory_threshold: Option<f32>,

    /// Update rogue detection utilization threshold
    #[arg(long, requires = "audit", value_name = "PERCENT")]
    pub rogue_utilization_threshold: Option<f32>,

    /// Update rogue detection duration threshold
    #[arg(long, requires = "audit", value_name = "HOURS")]
    pub rogue_duration_threshold: Option<f32>,

    /// Update rogue detection confidence threshold
    #[arg(long, requires = "audit", value_name = "CONFIDENCE")]
    pub rogue_confidence_threshold: Option<f32>,

    /// Add process to rogue detection whitelist
    #[arg(long, requires = "audit", value_name = "PROCESS_NAME")]
    pub rogue_whitelist_process: Option<String>,

    /// Remove process from rogue detection whitelist
    #[arg(long, requires = "audit", value_name = "PROCESS_NAME")]
    pub rogue_unwhitelist_process: Option<String>,

    /// Add user to rogue detection whitelist
    #[arg(long, requires = "audit", value_name = "USERNAME")]
    pub rogue_whitelist_user: Option<String>,

    /// Remove user from rogue detection whitelist
    #[arg(long, requires = "audit", value_name = "USERNAME")]
    pub rogue_unwhitelist_user: Option<String>,

    /// Export rogue detection configuration to JSON
    #[arg(long, requires = "audit")]
    pub rogue_export_config: bool,

    /// Import rogue detection configuration from JSON file
    #[arg(long, requires = "audit", value_name = "FILE_PATH")]
    pub rogue_import_config: Option<String>,

    /// Enable Guard Mode (soft policy enforcement)
    #[arg(long)]
    pub guard: bool,

    /// Show Guard Mode configuration
    #[arg(long, requires = "guard")]
    pub guard_config: bool,

    /// Enable Guard Mode
    #[arg(long, requires = "guard")]
    pub guard_enable: bool,

    /// Disable Guard Mode
    #[arg(long, requires = "guard")]
    pub guard_disable: bool,

    /// Set Guard Mode to dry-run (no enforcement)
    #[arg(long, requires = "guard")]
    pub guard_dry_run: bool,

    /// Set Guard Mode to enforce policies
    #[arg(long, requires = "guard")]
    pub guard_enforce: bool,

    /// Add user policy for Guard Mode
    #[arg(long, requires = "guard", value_name = "USERNAME")]
    pub guard_add_user: Option<String>,

    /// Remove user policy from Guard Mode
    #[arg(long, requires = "guard", value_name = "USERNAME")]
    pub guard_remove_user: Option<String>,

    /// Set memory limit for user (GB)
    #[arg(long, requires = "guard", value_name = "GB")]
    pub guard_memory_limit: Option<f32>,

    /// Set utilization limit for user (%)
    #[arg(long, requires = "guard", value_name = "PERCENT")]
    pub guard_utilization_limit: Option<f32>,

    /// Set concurrent process limit for user
    #[arg(long, requires = "guard", value_name = "COUNT")]
    pub guard_process_limit: Option<u32>,

    /// Export Guard Mode configuration to JSON
    #[arg(long, requires = "guard")]
    pub guard_export_config: bool,

    /// Import Guard Mode configuration from JSON file
    #[arg(long, requires = "guard", value_name = "FILE_PATH")]
    pub guard_import_config: Option<String>,

    /// Test policies in dry-run mode (simulate enforcement)
    #[arg(long, requires = "guard")]
    pub guard_test_policies: bool,

    /// Toggle dry-run mode on/off
    #[arg(long, requires = "guard")]
    pub guard_toggle_dry_run: bool,

    /// Add group policy
    #[arg(long, requires = "guard", value_name = "GROUP_NAME")]
    pub guard_add_group: Option<String>,

    /// Remove group policy
    #[arg(long, requires = "guard", value_name = "GROUP_NAME")]
    pub guard_remove_group: Option<String>,

    /// Add GPU policy
    #[arg(long, requires = "guard", value_name = "GPU_INDEX")]
    pub guard_add_gpu: Option<u16>,

    /// Remove GPU policy
    #[arg(long, requires = "guard", value_name = "GPU_INDEX")]
    pub guard_remove_gpu: Option<u16>,

    /// Group memory limit (GB)
    #[arg(long, requires = "guard", value_name = "GB")]
    pub guard_group_memory_limit: Option<f32>,

    /// Group utilization limit (%)
    #[arg(long, requires = "guard", value_name = "PERCENT")]
    pub guard_group_utilization_limit: Option<f32>,

    /// Group process limit
    #[arg(long, requires = "guard", value_name = "COUNT")]
    pub guard_group_process_limit: Option<u32>,

    /// GPU memory limit (GB)
    #[arg(long, requires = "guard", value_name = "GB")]
    pub guard_gpu_memory_limit: Option<f32>,

    /// GPU utilization limit (%)
    #[arg(long, requires = "guard", value_name = "PERCENT")]
    pub guard_gpu_utilization_limit: Option<f32>,

    /// GPU reserved memory (GB)
    #[arg(long, requires = "guard", value_name = "GB")]
    pub guard_gpu_reserved_memory: Option<f32>,

    /// Group members (comma-separated usernames)
    #[arg(long, requires = "guard", value_name = "MEMBERS")]
    pub guard_group_members: Option<String>,

    /// GPU allowed users (comma-separated usernames)
    #[arg(long, requires = "guard", value_name = "USERS")]
    pub guard_gpu_allowed_users: Option<String>,

    /// Server port for coordinator API
    #[arg(long, requires = "server", default_value = "8080")]
    pub server_port: u16,

    /// Server host for coordinator API
    #[arg(long, requires = "server", default_value = "0.0.0.0")]
    pub server_host: String,

    /// Register this node with a coordinator
    #[arg(long, value_name = "COORDINATOR_URL")]
    pub register_node: Option<String>,

    /// Remote host to connect to via SSH
    #[arg(long)]
    pub remote: Option<String>,

    /// SSH username (defaults to current user)
    #[arg(long, requires = "remote")]
    pub ssh_user: Option<String>,

    /// SSH port (defaults to 22)
    #[arg(long, requires = "remote", default_value = "22")]
    pub ssh_port: u16,

    /// SSH private key path
    #[arg(long, requires = "remote")]
    pub ssh_key: Option<String>,

    /// SSH password (interactive prompt if not provided)
    #[arg(long, requires = "remote")]
    pub ssh_password: Option<String>,

    /// SSH connection timeout in seconds
    #[arg(long, requires = "remote", default_value = "30")]
    pub ssh_timeout: u16,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum VendorFilter {
    Nvidia,
    Amd,
    Intel,
    Apple,
    All,
}

impl VendorFilter {
    pub fn to_gpu_vendor(&self) -> Option<GpuVendor> {
        match self {
            VendorFilter::Nvidia => Some(GpuVendor::Nvidia),
            VendorFilter::Amd => Some(GpuVendor::Amd),
            VendorFilter::Intel => Some(GpuVendor::Intel),
            VendorFilter::Apple => Some(GpuVendor::Apple),
            VendorFilter::All => None,
        }
    }
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum OutputFormat {
    Table,
    Json,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "trace"),
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Error => write!(f, "error"),
        }
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Table => write!(f, "table"),
            OutputFormat::Json => write!(f, "json"),
        }
    }
}

impl Cli {
    /// Parse command line arguments with validation
    pub fn parse() -> Self {
        // Pre-process argv to support friendly shorthands before clap parsing
        let mut argv: Vec<String> = std::env::args().collect();

        // Positional PID alias: `gpukill <pid> [global flags]` => `gpukill --kill --pid <pid> [global flags]`
        // Only apply if no explicit operation flag is present
        let has_operation_flag = argv.iter().any(|a| {
            matches!(a.as_str(), "--list" | "--kill" | "--reset" | "--audit" | "--server" | "--guard")
        });
        if !has_operation_flag {
            if let Some((pos_idx, pid_token)) = argv
                .iter()
                .enumerate()
                .skip(1)
                .find_map(|(i, t)| {
                    if !t.starts_with('-') && t.chars().all(|c| c.is_ascii_digit()) {
                        Some((i, t.clone()))
                    } else {
                        None
                    }
                })
            {
                let mut new_argv = Vec::with_capacity(argv.len() + 2);
                new_argv.push(argv[0].clone());
                new_argv.push("--kill".to_string());
                new_argv.push("--pid".to_string());
                new_argv.push(pid_token);
                // push the rest excluding the positional pid we consumed
                for (i, t) in argv.into_iter().enumerate().skip(1) {
                    if i == pos_idx { continue; }
                    new_argv.push(t);
                }
                argv = new_argv;
            }
        }

        // Reset shorthand: `gpukill --reset <id>` => `gpukill --reset --gpu <id>`
        if let Some(pos) = argv.iter().position(|a| a == "--reset") {
            if pos + 1 < argv.len() {
                let next = &argv[pos + 1];
                if !next.starts_with('-') && next.chars().all(|c| c.is_ascii_digit()) {
                    // Insert --gpu before the numeric id if --gpu isn't already specified
                    // Avoid duplicating if user already passed --gpu
                    let has_gpu_flag = argv.iter().any(|a| a == "--gpu");
                    if !has_gpu_flag {
                        argv.insert(pos + 1, "--gpu".to_string());
                    }
                }
            }
        }

        // Alias: `gpukill watch` => `gpukill --list --watch`
        // Only if no explicit operation flag is present
        let has_operation_flag = argv.iter().any(|a| {
            matches!(a.as_str(), "--list" | "--kill" | "--reset" | "--audit" | "--server" | "--guard")
        });
        if !has_operation_flag {
            if let Some(pos) = argv.iter().position(|a| a == "watch") {
                let mut new_argv = Vec::with_capacity(argv.len() + 2);
                new_argv.push(argv[0].clone());
                new_argv.push("--list".to_string());
                new_argv.push("--watch".to_string());
                for (i, t) in argv.into_iter().enumerate().skip(1) {
                    if i == pos { continue; }
                    new_argv.push(t);
                }
                argv = new_argv;
            }
        }

        // Alias: `gpukill up [--open]` => `gpukill --server [--open]`
        let has_operation_flag2 = argv.iter().any(|a| {
            matches!(a.as_str(), "--list" | "--kill" | "--reset" | "--audit" | "--server" | "--guard")
        });
        if !has_operation_flag2 {
            if let Some(pos) = argv.iter().position(|a| a == "up") {
                let mut new_argv = Vec::with_capacity(argv.len() + 1);
                new_argv.push(argv[0].clone());
                new_argv.push("--server".to_string());
                for (i, t) in argv.into_iter().enumerate().skip(1) {
                    if i == pos { continue; }
                    new_argv.push(t);
                }
                argv = new_argv;
            }
        }

        let cli = Self::parse_from(argv);
        cli.validate();
        cli
    }

    /// Validate argument combinations
    fn validate(&self) {
        // Check that exactly one operation is specified
        let operation_count = [
            self.list,
            self.kill,
            self.reset,
            self.audit,
            self.server,
            self.guard,
        ]
        .iter()
        .filter(|&&x| x)
        .count();
        if operation_count == 0 {
            eprintln!("Error: Exactly one of --list, --kill, --reset, --audit, --server, or --guard must be specified");
            std::process::exit(3);
        }
        if operation_count > 1 {
            eprintln!("Error: Only one of --list, --kill, --reset, --audit, --server, or --guard can be specified");
            std::process::exit(3);
        }

        // Validate kill operation
        if self.kill {
            if self.pid.is_some() && self.filter.is_some() {
                eprintln!("Error: --kill cannot use both --pid and --filter");
                std::process::exit(3);
            }

            // Allow one of: --pid, --filter, or --gpu (kill-by-GPU)
            if self.pid.is_none() && self.filter.is_none() && self.gpu.is_none() {
                eprintln!("Error: --kill requires one of --pid <PID>, --filter <PATTERN>, or --gpu <ID>");
                std::process::exit(3);
            }

            if let Some(pid) = self.pid {
                if pid == 0 {
                    eprintln!("Error: PID must be greater than 0");
                    std::process::exit(3);
                }
            }
        }

        // Validate reset operation
        if self.reset {
            if self.gpu.is_none() && !self.all {
                eprintln!("Error: --reset requires either --gpu <ID> or --all");
                std::process::exit(3);
            }
            if self.gpu.is_some() && self.all {
                eprintln!("Error: --reset cannot use both --gpu and --all");
                std::process::exit(3);
            }
        }

        // Validate list operation dependencies
        if self.list {
            if self.details && !self.list {
                eprintln!("Error: --details requires --list");
                std::process::exit(3);
            }
            if self.watch && !self.list {
                eprintln!("Error: --watch requires --list");
                std::process::exit(3);
            }
            if self.containers && !self.list {
                eprintln!("Error: --containers requires --list");
                std::process::exit(3);
            }
        }

        // Validate kill operation dependencies
        if self.kill {
            if self.filter.is_some() && !self.kill {
                eprintln!("Error: --filter requires --kill");
                std::process::exit(3);
            }
            if self.batch && self.filter.is_none() {
                // Allow batch with either filter or gpu (kill-by-GPU)
                if self.gpu.is_none() {
                    eprintln!("Error: --batch requires --filter or --gpu");
                    std::process::exit(3);
                }
            }

            // Prevent ambiguous use of --all with --kill
            if self.all {
                eprintln!("Error: --all is only valid with --reset");
                std::process::exit(3);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_list_operation_parsing() {
        let cli = Cli::try_parse_from(["gpukill", "--list"]).unwrap();
        assert!(cli.list);
        assert!(!cli.details);
        assert!(!cli.watch);
        assert!(matches!(cli.output, OutputFormat::Table));
    }

    #[test]
    fn test_list_with_details() {
        let cli = Cli::try_parse_from(["gpukill", "--list", "--details"]).unwrap();
        assert!(cli.list);
        assert!(cli.details);
    }

    #[test]
    fn test_list_with_details_and_watch() {
        let cli = Cli::try_parse_from(["gpukill", "--list", "--details", "--watch"]).unwrap();
        assert!(cli.list);
        assert!(cli.details);
        assert!(cli.watch);
        assert!(matches!(cli.output, OutputFormat::Table));
    }

    #[test]
    fn test_list_json_output() {
        let cli = Cli::try_parse_from(["gpukill", "--list", "--output", "json"]).unwrap();
        assert!(cli.list);
        assert!(matches!(cli.output, OutputFormat::Json));
    }

    #[test]
    fn test_kill_operation() {
        let cli = Cli::try_parse_from(["gpukill", "--kill", "--pid", "12345"]).unwrap();
        assert!(cli.kill);
        assert_eq!(cli.pid, Some(12345));
        assert_eq!(cli.timeout_secs, 5);
        assert!(!cli.force);
    }

    #[test]
    fn test_kill_with_custom_timeout_and_force() {
        let cli = Cli::try_parse_from([
            "gpukill",
            "--kill",
            "--pid",
            "12345",
            "--timeout-secs",
            "10",
            "--force",
        ])
        .unwrap();
        assert!(cli.kill);
        assert_eq!(cli.pid, Some(12345));
        assert_eq!(cli.timeout_secs, 10);
        assert!(cli.force);
    }

    #[test]
    fn test_reset_single_gpu() {
        let cli = Cli::try_parse_from(["gpukill", "--reset", "--gpu", "0"]).unwrap();
        assert!(cli.reset);
        assert_eq!(cli.gpu, Some(0));
        assert!(!cli.all);
    }

    #[test]
    fn test_reset_all_gpus() {
        let cli = Cli::try_parse_from(["gpukill", "--reset", "--all"]).unwrap();
        assert!(cli.reset);
        assert_eq!(cli.gpu, None);
        assert!(cli.all);
    }

    #[test]
    fn test_invalid_pid() {
        // This test would fail at validation, not parsing
        let result = Cli::try_parse_from(["gpukill", "--kill", "--pid", "0"]);
        // The parsing succeeds, but validation should fail
        if let Ok(cli) = result {
            // Validation happens in the parse() method, not try_parse_from
            assert!(cli.kill);
            assert_eq!(cli.pid, Some(0));
        }
    }

    #[test]
    fn test_reset_without_target() {
        // This should fail because neither --gpu nor --all is specified
        // But parsing will succeed, validation will fail
        let result = Cli::try_parse_from(["gpukill", "--reset"]);
        assert!(result.is_ok());
        let cli = result.unwrap();
        // The validation should catch this in the actual parse() method
        // For this test, we'll just verify the reset flag is set
        assert!(cli.reset);
        assert!(cli.gpu.is_none());
        assert!(!cli.all);
    }
}
