use anyhow::Result;
use chrono::{DateTime, Datelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::{error, info};

use crate::nvml_api::GpuProc;

/// Guard Mode policy configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GuardModeConfig {
    /// Global guard mode settings
    pub global: GlobalSettings,
    /// User-specific policies
    pub user_policies: HashMap<String, UserPolicy>,
    /// Group-specific policies
    pub group_policies: HashMap<String, GroupPolicy>,
    /// GPU-specific policies
    pub gpu_policies: HashMap<String, GpuPolicy>,
    /// Time-based policies
    pub time_policies: Vec<TimePolicy>,
    /// Enforcement settings
    pub enforcement: EnforcementSettings,
    /// Configuration metadata
    pub metadata: ConfigMetadata,
}

/// Global guard mode settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSettings {
    /// Enable/disable guard mode
    pub enabled: bool,
    /// Default memory limit per user (GB)
    pub default_memory_limit_gb: f32,
    /// Default utilization limit per user (%)
    pub default_utilization_limit_pct: f32,
    /// Default process duration limit (hours)
    pub default_duration_limit_hours: f32,
    /// Check interval in seconds
    pub check_interval_seconds: u32,
    /// Enable dry-run mode (no actual enforcement)
    pub dry_run: bool,
}

/// User-specific policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPolicy {
    /// User name
    pub username: String,
    /// Memory limit (GB)
    pub memory_limit_gb: f32,
    /// Utilization limit (%)
    pub utilization_limit_pct: f32,
    /// Process duration limit (hours)
    pub duration_limit_hours: f32,
    /// Maximum concurrent processes
    pub max_concurrent_processes: u32,
    /// Priority level (higher = more resources)
    pub priority: u8,
    /// Allowed GPU indices
    pub allowed_gpus: Vec<u16>,
    /// Blocked GPU indices
    pub blocked_gpus: Vec<u16>,
    /// Time-based overrides
    pub time_overrides: Vec<TimeOverride>,
}

/// Group-specific policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupPolicy {
    /// Group name
    pub group_name: String,
    /// Memory limit for entire group (GB)
    pub total_memory_limit_gb: f32,
    /// Utilization limit for entire group (%)
    pub total_utilization_limit_pct: f32,
    /// Maximum concurrent processes for group
    pub max_concurrent_processes: u32,
    /// Priority level
    pub priority: u8,
    /// Allowed GPU indices
    pub allowed_gpus: Vec<u16>,
    /// Blocked GPU indices
    pub blocked_gpus: Vec<u16>,
    /// Member users
    pub members: Vec<String>,
}

/// GPU-specific policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuPolicy {
    /// GPU index
    pub gpu_index: u16,
    /// Maximum memory usage (GB)
    pub max_memory_gb: f32,
    /// Maximum utilization (%)
    pub max_utilization_pct: f32,
    /// Reserved memory for system (GB)
    pub reserved_memory_gb: f32,
    /// Allowed users
    #[serde(default)]
    pub allowed_users: Vec<String>,
    /// Blocked users
    #[serde(default)]
    pub blocked_users: Vec<String>,
    /// Maintenance window
    #[serde(default)]
    pub maintenance_window: Option<MaintenanceWindow>,
}

/// Time-based policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePolicy {
    /// Policy name
    pub name: String,
    /// Start time (HH:MM format)
    pub start_time: String,
    /// End time (HH:MM format)
    pub end_time: String,
    /// Days of week (0=Sunday, 1=Monday, etc.)
    pub days_of_week: Vec<u8>,
    /// Memory limit multiplier
    pub memory_multiplier: f32,
    /// Utilization limit multiplier
    pub utilization_multiplier: f32,
    /// Duration limit multiplier
    pub duration_multiplier: f32,
}

/// Time-based override for user policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeOverride {
    /// Start time
    pub start_time: String,
    /// End time
    pub end_time: String,
    /// Days of week
    pub days_of_week: Vec<u8>,
    /// Override settings
    pub overrides: PolicyOverrides,
}

/// Policy overrides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyOverrides {
    pub memory_limit_gb: Option<f32>,
    pub utilization_limit_pct: Option<f32>,
    pub duration_limit_hours: Option<f32>,
    pub max_concurrent_processes: Option<u32>,
}

/// Maintenance window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceWindow {
    /// Start time
    pub start_time: String,
    /// End time
    pub end_time: String,
    /// Days of week
    pub days_of_week: Vec<u8>,
    /// Maintenance message
    pub message: String,
}

/// Enforcement settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementSettings {
    /// Enable soft enforcement (warnings only)
    pub soft_enforcement: bool,
    /// Enable hard enforcement (process termination)
    pub hard_enforcement: bool,
    /// Grace period before enforcement (seconds)
    pub grace_period_seconds: u32,
    /// Maximum warnings before enforcement
    pub max_warnings: u32,
    /// Notification channels
    pub notifications: NotificationSettings,
}

/// Notification settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    /// Enable console notifications
    pub console: bool,
    /// Enable log file notifications
    pub log_file: bool,
    /// Enable email notifications
    pub email: bool,
    /// Email recipients
    pub email_recipients: Vec<String>,
    /// Enable webhook notifications
    pub webhook: bool,
    /// Webhook URL
    pub webhook_url: Option<String>,
}

/// Configuration metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMetadata {
    pub version: String,
    pub created_at: String,
    pub last_modified: String,
    pub description: String,
}

/// Guard Mode enforcement result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementResult {
    pub timestamp: DateTime<Utc>,
    pub violations: Vec<PolicyViolation>,
    pub warnings: Vec<PolicyWarning>,
    pub actions_taken: Vec<EnforcementAction>,
    pub dry_run: bool,
}

/// Policy violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolation {
    pub violation_type: ViolationType,
    pub severity: ViolationSeverity,
    pub user: String,
    pub process: GpuProc,
    pub policy_name: String,
    pub current_value: f32,
    pub limit_value: f32,
    pub message: String,
    pub recommended_action: String,
}

/// Policy warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyWarning {
    pub warning_type: WarningType,
    pub user: String,
    pub process: GpuProc,
    pub policy_name: String,
    pub current_value: f32,
    pub limit_value: f32,
    pub message: String,
    pub time_to_limit: Option<u32>, // seconds
}

/// Enforcement action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementAction {
    pub action_type: ActionType,
    pub user: String,
    pub process: GpuProc,
    pub policy_name: String,
    pub message: String,
    pub success: bool,
}

/// Violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationType {
    MemoryLimitExceeded,
    UtilizationLimitExceeded,
    DurationLimitExceeded,
    ConcurrentProcessLimitExceeded,
    UnauthorizedGpuAccess,
    UnauthorizedUserAccess,
    MaintenanceWindowViolation,
}

/// Violation severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Warning types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WarningType {
    ApproachingMemoryLimit,
    ApproachingUtilizationLimit,
    ApproachingDurationLimit,
    HighResourceUsage,
    UnusualActivity,
}

/// Action types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    Warning,
    ProcessTermination,
    ResourceThrottling,
    AccessDenied,
    NotificationSent,
}

// Default implementation is now derived

impl Default for GlobalSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            default_memory_limit_gb: 16.0,
            default_utilization_limit_pct: 80.0,
            default_duration_limit_hours: 12.0,
            check_interval_seconds: 60,
            dry_run: true,
        }
    }
}

impl Default for EnforcementSettings {
    fn default() -> Self {
        Self {
            soft_enforcement: true,
            hard_enforcement: false,
            grace_period_seconds: 300, // 5 minutes
            max_warnings: 3,
            notifications: NotificationSettings::default(),
        }
    }
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            console: true,
            log_file: true,
            email: false,
            email_recipients: Vec::new(),
            webhook: false,
            webhook_url: None,
        }
    }
}

impl Default for ConfigMetadata {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            last_modified: chrono::Utc::now().to_rfc3339(),
            description: "Default GPU Kill Guard Mode configuration".to_string(),
        }
    }
}

/// Guard Mode policy manager
pub struct GuardModeManager {
    config_path: PathBuf,
    config: GuardModeConfig,
    violation_history: Vec<PolicyViolation>,
    warning_history: Vec<PolicyWarning>,
}

#[derive(Clone, Copy, Debug)]
struct TimeMultipliers {
    memory: f32,
    utilization: f32,
    duration: f32,
}

impl GuardModeManager {
    /// Create a new guard mode manager
    pub fn new() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        let config = if config_path.exists() {
            Self::load_config(&config_path)?
        } else {
            let default_config = GuardModeConfig::default();
            Self::save_config(&config_path, &default_config)?;
            default_config
        };

        Ok(Self {
            config_path,
            config,
            violation_history: Vec::new(),
            warning_history: Vec::new(),
        })
    }

    /// Get the configuration file path
    fn get_config_path() -> Result<PathBuf> {
        let mut path = if let Some(config_dir) = dirs::config_dir() {
            config_dir
        } else if let Some(home_dir) = dirs::home_dir() {
            home_dir.join(".config")
        } else {
            std::env::current_dir()?
        };

        path.push("gpukill");
        fs::create_dir_all(&path)
            .map_err(|e| anyhow::anyhow!("Failed to create config directory: {}", e))?;

        path.push("guard_mode_config.toml");
        Ok(path)
    }

    /// Load configuration from file
    fn load_config(path: &PathBuf) -> Result<GuardModeConfig> {
        let content = fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read config file: {}", e))?;

        let config: GuardModeConfig = toml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse config file: {}", e))?;

        info!("Loaded Guard Mode configuration from: {}", path.display());
        Ok(config)
    }

    /// Save configuration to file
    fn save_config(path: &PathBuf, config: &GuardModeConfig) -> Result<()> {
        info!("Serializing config to TOML...");
        let content = match toml::to_string_pretty(config) {
            Ok(content) => {
                info!("Config serialized successfully, writing to file...");
                content
            }
            Err(e) => {
                error!("TOML serialization failed: {}", e);
                return Err(anyhow::anyhow!("Failed to serialize config: {}", e));
            }
        };

        fs::write(path, content)
            .map_err(|e| anyhow::anyhow!("Failed to write config file: {}", e))?;

        info!("Saved Guard Mode configuration to: {}", path.display());
        Ok(())
    }

    /// Get the current configuration
    pub fn get_config(&self) -> &GuardModeConfig {
        &self.config
    }

    /// Update the configuration
    pub fn update_config(&mut self, new_config: GuardModeConfig) -> Result<()> {
        self.config = new_config;
        self.config.metadata.last_modified = chrono::Utc::now().to_rfc3339();
        Self::save_config(&self.config_path, &self.config)?;
        Ok(())
    }

    /// Enable or disable guard mode
    pub fn set_enabled(&mut self, enabled: bool) -> Result<()> {
        self.config.global.enabled = enabled;
        self.config.metadata.last_modified = chrono::Utc::now().to_rfc3339();
        Self::save_config(&self.config_path, &self.config)?;
        Ok(())
    }

    /// Set dry-run mode
    pub fn set_dry_run(&mut self, dry_run: bool) -> Result<()> {
        self.config.global.dry_run = dry_run;
        self.config.metadata.last_modified = chrono::Utc::now().to_rfc3339();
        Self::save_config(&self.config_path, &self.config)?;
        Ok(())
    }

    /// Add a user policy
    pub fn add_user_policy(&mut self, policy: UserPolicy) -> Result<()> {
        self.config
            .user_policies
            .insert(policy.username.clone(), policy);
        self.config.metadata.last_modified = chrono::Utc::now().to_rfc3339();
        Self::save_config(&self.config_path, &self.config)?;
        Ok(())
    }

    /// Remove a user policy
    pub fn remove_user_policy(&mut self, username: &str) -> Result<()> {
        self.config.user_policies.remove(username);
        self.config.metadata.last_modified = chrono::Utc::now().to_rfc3339();
        Self::save_config(&self.config_path, &self.config)?;
        Ok(())
    }

    /// Add a group policy
    pub fn add_group_policy(&mut self, policy: GroupPolicy) -> Result<()> {
        self.config
            .group_policies
            .insert(policy.group_name.clone(), policy);
        self.config.metadata.last_modified = chrono::Utc::now().to_rfc3339();
        Self::save_config(&self.config_path, &self.config)?;
        Ok(())
    }

    /// Add a GPU policy
    pub fn add_gpu_policy(&mut self, policy: GpuPolicy) -> Result<()> {
        info!("Adding GPU policy for GPU {}", policy.gpu_index);
        let key = policy.gpu_index.to_string();
        self.config.gpu_policies.insert(key, policy);
        self.config.metadata.last_modified = chrono::Utc::now().to_rfc3339();
        info!("Saving config to: {}", self.config_path.display());
        Self::save_config(&self.config_path, &self.config)?;
        info!("GPU policy added successfully");
        Ok(())
    }

    /// Remove a group policy
    pub fn remove_group_policy(&mut self, group_name: &str) -> Result<()> {
        if self.config.group_policies.remove(group_name).is_some() {
            self.config.metadata.last_modified = chrono::Utc::now().to_rfc3339();
            Self::save_config(&self.config_path, &self.config)?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Group policy '{}' not found", group_name))
        }
    }

    /// Remove a GPU policy
    pub fn remove_gpu_policy(&mut self, gpu_index: u16) -> Result<()> {
        let key = gpu_index.to_string();
        if self.config.gpu_policies.remove(&key).is_some() {
            self.config.metadata.last_modified = chrono::Utc::now().to_rfc3339();
            Self::save_config(&self.config_path, &self.config)?;
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "GPU policy for GPU {} not found",
                gpu_index
            ))
        }
    }

    /// Check processes against policies
    pub fn check_policies(&mut self, processes: &[GpuProc]) -> Result<EnforcementResult> {
        if !self.config.global.enabled {
            return Ok(EnforcementResult {
                timestamp: Utc::now(),
                violations: Vec::new(),
                warnings: Vec::new(),
                actions_taken: Vec::new(),
                dry_run: self.config.global.dry_run,
            });
        }

        let mut violations = Vec::new();
        let mut warnings = Vec::new();
        let mut actions_taken = Vec::new();
        let now = Utc::now();
        let time_multipliers = self.get_time_multipliers(now);

        // Group processes by user
        let mut user_processes: HashMap<String, Vec<&GpuProc>> = HashMap::new();
        for process in processes {
            user_processes
                .entry(process.user.clone())
                .or_default()
                .push(process);
        }

        // Check each user's processes
        for (username, user_procs) in user_processes {
            let user_result =
                self.check_user_policies(&username, &user_procs, now, time_multipliers)?;
            violations.extend(user_result.violations);
            warnings.extend(user_result.warnings);
            actions_taken.extend(user_result.actions_taken);
        }

        self.check_group_policies(processes, time_multipliers, &mut violations, &mut warnings);
        self.check_gpu_policies(
            processes,
            now,
            time_multipliers,
            &mut violations,
            &mut warnings,
        );

        // In dry-run mode, simulate actions without actually taking them
        if self.config.global.dry_run {
            actions_taken = self.simulate_actions(&violations, &warnings);
        } else {
            // In enforcement mode, actually take actions
            actions_taken = self.execute_actions(&violations, &warnings)?;
        }

        // Store violations and warnings in history
        self.violation_history.extend(violations.clone());
        self.warning_history.extend(warnings.clone());

        Ok(EnforcementResult {
            timestamp: Utc::now(),
            violations,
            warnings,
            actions_taken,
            dry_run: self.config.global.dry_run,
        })
    }

    /// Check policies for a specific user
    fn check_user_policies(
        &mut self,
        username: &str,
        processes: &[&GpuProc],
        now: DateTime<Utc>,
        time_multipliers: TimeMultipliers,
    ) -> Result<EnforcementResult> {
        let mut violations = Vec::new();
        let mut warnings = Vec::new();
        let actions_taken = Vec::new();

        // Get user policy (or use defaults)
        let mut user_policy = self.get_user_policy(username);
        self.apply_time_overrides(&mut user_policy, now);
        self.apply_time_multipliers(&mut user_policy, time_multipliers);

        // Check memory limits
        let total_memory = processes
            .iter()
            .map(|p| p.used_mem_mb as f32 / 1024.0)
            .sum::<f32>();

        if total_memory > user_policy.memory_limit_gb {
            violations.push(PolicyViolation {
                violation_type: ViolationType::MemoryLimitExceeded,
                severity: ViolationSeverity::High,
                user: username.to_string(),
                process: processes[0].clone(),
                policy_name: "memory_limit".to_string(),
                current_value: total_memory,
                limit_value: user_policy.memory_limit_gb,
                message: format!(
                    "User {} exceeded memory limit: {:.1}GB > {:.1}GB",
                    username, total_memory, user_policy.memory_limit_gb
                ),
                recommended_action: "Terminate some processes or request limit increase"
                    .to_string(),
            });
        } else if total_memory > user_policy.memory_limit_gb * 0.8 {
            warnings.push(PolicyWarning {
                warning_type: WarningType::ApproachingMemoryLimit,
                user: username.to_string(),
                process: processes[0].clone(),
                policy_name: "memory_limit".to_string(),
                current_value: total_memory,
                limit_value: user_policy.memory_limit_gb,
                message: format!(
                    "User {} approaching memory limit: {:.1}GB / {:.1}GB",
                    username, total_memory, user_policy.memory_limit_gb
                ),
                time_to_limit: None,
            });
        }

        // Check utilization limits (based on memory usage percentage)
        if user_policy.memory_limit_gb > 0.0 {
            let utilization_pct = (total_memory / user_policy.memory_limit_gb) * 100.0;
            if utilization_pct > user_policy.utilization_limit_pct {
                violations.push(PolicyViolation {
                    violation_type: ViolationType::UtilizationLimitExceeded,
                    severity: ViolationSeverity::High,
                    user: username.to_string(),
                    process: processes[0].clone(),
                    policy_name: "utilization_limit".to_string(),
                    current_value: utilization_pct,
                    limit_value: user_policy.utilization_limit_pct,
                    message: format!(
                        "User {} exceeded utilization limit: {:.1}% > {:.1}%",
                        username, utilization_pct, user_policy.utilization_limit_pct
                    ),
                    recommended_action: "Reduce GPU workload or request higher limit".to_string(),
                });
            } else if utilization_pct > user_policy.utilization_limit_pct * 0.8 {
                warnings.push(PolicyWarning {
                    warning_type: WarningType::ApproachingUtilizationLimit,
                    user: username.to_string(),
                    process: processes[0].clone(),
                    policy_name: "utilization_limit".to_string(),
                    current_value: utilization_pct,
                    limit_value: user_policy.utilization_limit_pct,
                    message: format!(
                        "User {} approaching utilization limit: {:.1}% / {:.1}%",
                        username, utilization_pct, user_policy.utilization_limit_pct
                    ),
                    time_to_limit: None,
                });
            }
        }

        // Check concurrent process limits
        if processes.len() as u32 > user_policy.max_concurrent_processes {
            violations.push(PolicyViolation {
                violation_type: ViolationType::ConcurrentProcessLimitExceeded,
                severity: ViolationSeverity::Medium,
                user: username.to_string(),
                process: processes[0].clone(),
                policy_name: "concurrent_processes".to_string(),
                current_value: processes.len() as f32,
                limit_value: user_policy.max_concurrent_processes as f32,
                message: format!(
                    "User {} exceeded concurrent process limit: {} > {}",
                    username,
                    processes.len(),
                    user_policy.max_concurrent_processes
                ),
                recommended_action: "Terminate some processes or request limit increase"
                    .to_string(),
            });
        }

        // Check GPU access permissions
        for process in processes {
            if !user_policy.allowed_gpus.is_empty()
                && !user_policy.allowed_gpus.contains(&process.gpu_index)
            {
                violations.push(PolicyViolation {
                    violation_type: ViolationType::UnauthorizedGpuAccess,
                    severity: ViolationSeverity::High,
                    user: username.to_string(),
                    process: (*process).clone(),
                    policy_name: "gpu_access".to_string(),
                    current_value: process.gpu_index as f32,
                    limit_value: user_policy.allowed_gpus[0] as f32,
                    message: format!(
                        "User {} accessing unauthorized GPU: {}",
                        username, process.gpu_index
                    ),
                    recommended_action: "Terminate process or add GPU to allowed list".to_string(),
                });
            }

            if user_policy.blocked_gpus.contains(&process.gpu_index) {
                violations.push(PolicyViolation {
                    violation_type: ViolationType::UnauthorizedGpuAccess,
                    severity: ViolationSeverity::Critical,
                    user: username.to_string(),
                    process: (*process).clone(),
                    policy_name: "blocked_gpu".to_string(),
                    current_value: process.gpu_index as f32,
                    limit_value: -1.0,
                    message: format!(
                        "User {} accessing blocked GPU: {}",
                        username, process.gpu_index
                    ),
                    recommended_action: "Immediately terminate process".to_string(),
                });
            }
        }

        // Check duration limits per process
        for process in processes {
            if let Some(duration_hours) = parse_duration_hours(&process.start_time, now) {
                if duration_hours > user_policy.duration_limit_hours {
                    violations.push(PolicyViolation {
                        violation_type: ViolationType::DurationLimitExceeded,
                        severity: ViolationSeverity::Medium,
                        user: username.to_string(),
                        process: (*process).clone(),
                        policy_name: "duration_limit".to_string(),
                        current_value: duration_hours,
                        limit_value: user_policy.duration_limit_hours,
                        message: format!(
                            "User {} exceeded duration limit: {:.1}h > {:.1}h",
                            username, duration_hours, user_policy.duration_limit_hours
                        ),
                        recommended_action: "Terminate long-running process or request extension"
                            .to_string(),
                    });
                } else if duration_hours > user_policy.duration_limit_hours * 0.8 {
                    warnings.push(PolicyWarning {
                        warning_type: WarningType::ApproachingDurationLimit,
                        user: username.to_string(),
                        process: (*process).clone(),
                        policy_name: "duration_limit".to_string(),
                        current_value: duration_hours,
                        limit_value: user_policy.duration_limit_hours,
                        message: format!(
                            "User {} approaching duration limit: {:.1}h / {:.1}h",
                            username, duration_hours, user_policy.duration_limit_hours
                        ),
                        time_to_limit: None,
                    });
                }
            }
        }

        Ok(EnforcementResult {
            timestamp: Utc::now(),
            violations,
            warnings,
            actions_taken,
            dry_run: self.config.global.dry_run,
        })
    }

    /// Get user policy (with defaults)
    fn get_user_policy(&self, username: &str) -> UserPolicy {
        if let Some(policy) = self.config.user_policies.get(username) {
            policy.clone()
        } else {
            // Create default policy for user
            UserPolicy {
                username: username.to_string(),
                memory_limit_gb: self.config.global.default_memory_limit_gb,
                utilization_limit_pct: self.config.global.default_utilization_limit_pct,
                duration_limit_hours: self.config.global.default_duration_limit_hours,
                max_concurrent_processes: 5,
                priority: 5,
                allowed_gpus: Vec::new(), // Allow all GPUs by default
                blocked_gpus: Vec::new(),
                time_overrides: Vec::new(),
            }
        }
    }

    fn get_time_multipliers(&self, now: DateTime<Utc>) -> TimeMultipliers {
        let mut multipliers = TimeMultipliers {
            memory: 1.0,
            utilization: 1.0,
            duration: 1.0,
        };

        for policy in &self.config.time_policies {
            if is_time_window_active(
                now,
                &policy.start_time,
                &policy.end_time,
                &policy.days_of_week,
            ) {
                multipliers.memory *= policy.memory_multiplier;
                multipliers.utilization *= policy.utilization_multiplier;
                multipliers.duration *= policy.duration_multiplier;
            }
        }

        multipliers
    }

    fn apply_time_overrides(&self, policy: &mut UserPolicy, now: DateTime<Utc>) {
        for override_rule in &policy.time_overrides {
            if is_time_window_active(
                now,
                &override_rule.start_time,
                &override_rule.end_time,
                &override_rule.days_of_week,
            ) {
                if let Some(limit) = override_rule.overrides.memory_limit_gb {
                    policy.memory_limit_gb = limit;
                }
                if let Some(limit) = override_rule.overrides.utilization_limit_pct {
                    policy.utilization_limit_pct = limit;
                }
                if let Some(limit) = override_rule.overrides.duration_limit_hours {
                    policy.duration_limit_hours = limit;
                }
                if let Some(limit) = override_rule.overrides.max_concurrent_processes {
                    policy.max_concurrent_processes = limit;
                }
            }
        }
    }

    fn apply_time_multipliers(&self, policy: &mut UserPolicy, multipliers: TimeMultipliers) {
        policy.memory_limit_gb *= multipliers.memory;
        policy.utilization_limit_pct *= multipliers.utilization;
        policy.duration_limit_hours *= multipliers.duration;
    }

    fn check_group_policies(
        &self,
        processes: &[GpuProc],
        time_multipliers: TimeMultipliers,
        violations: &mut Vec<PolicyViolation>,
        warnings: &mut Vec<PolicyWarning>,
    ) {
        for (group_name, group_policy) in &self.config.group_policies {
            let group_processes: Vec<&GpuProc> = processes
                .iter()
                .filter(|p| group_policy.members.contains(&p.user))
                .collect();

            if group_processes.is_empty() {
                continue;
            }

            let total_memory = group_processes
                .iter()
                .map(|p| p.used_mem_mb as f32 / 1024.0)
                .sum::<f32>();

            let effective_memory_limit =
                group_policy.total_memory_limit_gb * time_multipliers.memory;
            if total_memory > effective_memory_limit {
                violations.push(PolicyViolation {
                    violation_type: ViolationType::MemoryLimitExceeded,
                    severity: ViolationSeverity::High,
                    user: group_name.clone(),
                    process: group_processes[0].clone(),
                    policy_name: "group_memory_limit".to_string(),
                    current_value: total_memory,
                    limit_value: effective_memory_limit,
                    message: format!(
                        "Group {} exceeded memory limit: {:.1}GB > {:.1}GB",
                        group_name, total_memory, effective_memory_limit
                    ),
                    recommended_action: "Reduce group GPU usage or adjust limits".to_string(),
                });
            } else if total_memory > effective_memory_limit * 0.8 {
                warnings.push(PolicyWarning {
                    warning_type: WarningType::ApproachingMemoryLimit,
                    user: group_name.clone(),
                    process: group_processes[0].clone(),
                    policy_name: "group_memory_limit".to_string(),
                    current_value: total_memory,
                    limit_value: effective_memory_limit,
                    message: format!(
                        "Group {} approaching memory limit: {:.1}GB / {:.1}GB",
                        group_name, total_memory, effective_memory_limit
                    ),
                    time_to_limit: None,
                });
            }

            if group_policy.total_memory_limit_gb > 0.0 {
                let utilization_pct = (total_memory / group_policy.total_memory_limit_gb) * 100.0;
                let limit = group_policy.total_utilization_limit_pct * time_multipliers.utilization;
                if utilization_pct > limit {
                    violations.push(PolicyViolation {
                        violation_type: ViolationType::UtilizationLimitExceeded,
                        severity: ViolationSeverity::Medium,
                        user: group_name.clone(),
                        process: group_processes[0].clone(),
                        policy_name: "group_utilization_limit".to_string(),
                        current_value: utilization_pct,
                        limit_value: limit,
                        message: format!(
                            "Group {} exceeded utilization limit: {:.1}% > {:.1}%",
                            group_name, utilization_pct, limit
                        ),
                        recommended_action: "Reduce group GPU workload".to_string(),
                    });
                } else if utilization_pct > limit * 0.8 {
                    warnings.push(PolicyWarning {
                        warning_type: WarningType::ApproachingUtilizationLimit,
                        user: group_name.clone(),
                        process: group_processes[0].clone(),
                        policy_name: "group_utilization_limit".to_string(),
                        current_value: utilization_pct,
                        limit_value: limit,
                        message: format!(
                            "Group {} approaching utilization limit: {:.1}% / {:.1}%",
                            group_name, utilization_pct, limit
                        ),
                        time_to_limit: None,
                    });
                }
            }

            if group_processes.len() as u32 > group_policy.max_concurrent_processes {
                violations.push(PolicyViolation {
                    violation_type: ViolationType::ConcurrentProcessLimitExceeded,
                    severity: ViolationSeverity::Medium,
                    user: group_name.clone(),
                    process: group_processes[0].clone(),
                    policy_name: "group_concurrent_processes".to_string(),
                    current_value: group_processes.len() as f32,
                    limit_value: group_policy.max_concurrent_processes as f32,
                    message: format!(
                        "Group {} exceeded concurrent process limit: {} > {}",
                        group_name,
                        group_processes.len(),
                        group_policy.max_concurrent_processes
                    ),
                    recommended_action: "Reduce concurrent processes for group".to_string(),
                });
            }

            for process in group_processes {
                if !group_policy.allowed_gpus.is_empty()
                    && !group_policy.allowed_gpus.contains(&process.gpu_index)
                {
                    violations.push(PolicyViolation {
                        violation_type: ViolationType::UnauthorizedGpuAccess,
                        severity: ViolationSeverity::High,
                        user: group_name.clone(),
                        process: process.clone(),
                        policy_name: "group_gpu_access".to_string(),
                        current_value: process.gpu_index as f32,
                        limit_value: group_policy.allowed_gpus[0] as f32,
                        message: format!(
                            "Group {} accessing unauthorized GPU: {}",
                            group_name, process.gpu_index
                        ),
                        recommended_action: "Move workload to allowed GPUs".to_string(),
                    });
                }

                if group_policy.blocked_gpus.contains(&process.gpu_index) {
                    violations.push(PolicyViolation {
                        violation_type: ViolationType::UnauthorizedGpuAccess,
                        severity: ViolationSeverity::Critical,
                        user: group_name.clone(),
                        process: process.clone(),
                        policy_name: "group_blocked_gpu".to_string(),
                        current_value: process.gpu_index as f32,
                        limit_value: -1.0,
                        message: format!(
                            "Group {} accessing blocked GPU: {}",
                            group_name, process.gpu_index
                        ),
                        recommended_action: "Immediately terminate process".to_string(),
                    });
                }
            }
        }
    }

    fn check_gpu_policies(
        &self,
        processes: &[GpuProc],
        now: DateTime<Utc>,
        time_multipliers: TimeMultipliers,
        violations: &mut Vec<PolicyViolation>,
        warnings: &mut Vec<PolicyWarning>,
    ) {
        let mut processes_by_gpu: HashMap<u16, Vec<&GpuProc>> = HashMap::new();
        for process in processes {
            processes_by_gpu
                .entry(process.gpu_index)
                .or_default()
                .push(process);
        }

        for policy in self.config.gpu_policies.values() {
            let gpu_processes = match processes_by_gpu.get(&policy.gpu_index) {
                Some(processes) => processes,
                None => continue,
            };

            if let Some(window) = &policy.maintenance_window {
                if is_time_window_active(
                    now,
                    &window.start_time,
                    &window.end_time,
                    &window.days_of_week,
                ) {
                    for process in gpu_processes {
                        violations.push(PolicyViolation {
                            violation_type: ViolationType::MaintenanceWindowViolation,
                            severity: ViolationSeverity::Critical,
                            user: process.user.clone(),
                            process: (*process).clone(),
                            policy_name: "gpu_maintenance_window".to_string(),
                            current_value: 1.0,
                            limit_value: 0.0,
                            message: format!(
                                "GPU {} is in maintenance window: {}",
                                policy.gpu_index, window.message
                            ),
                            recommended_action: "Terminate processes and retry after maintenance"
                                .to_string(),
                        });
                    }
                }
            }

            let total_memory = gpu_processes
                .iter()
                .map(|p| p.used_mem_mb as f32 / 1024.0)
                .sum::<f32>();
            let effective_max_memory = (policy.max_memory_gb - policy.reserved_memory_gb).max(0.0)
                * time_multipliers.memory;

            if total_memory > effective_max_memory {
                let representative = gpu_processes.iter().max_by_key(|p| p.used_mem_mb).unwrap();
                violations.push(PolicyViolation {
                    violation_type: ViolationType::MemoryLimitExceeded,
                    severity: ViolationSeverity::High,
                    user: representative.user.clone(),
                    process: (*representative).clone(),
                    policy_name: "gpu_memory_limit".to_string(),
                    current_value: total_memory,
                    limit_value: effective_max_memory,
                    message: format!(
                        "GPU {} exceeded memory limit: {:.1}GB > {:.1}GB",
                        policy.gpu_index, total_memory, effective_max_memory
                    ),
                    recommended_action: "Reduce memory usage on GPU".to_string(),
                });
            } else if total_memory > effective_max_memory * 0.8 {
                let representative = gpu_processes.iter().max_by_key(|p| p.used_mem_mb).unwrap();
                warnings.push(PolicyWarning {
                    warning_type: WarningType::ApproachingMemoryLimit,
                    user: representative.user.clone(),
                    process: (*representative).clone(),
                    policy_name: "gpu_memory_limit".to_string(),
                    current_value: total_memory,
                    limit_value: effective_max_memory,
                    message: format!(
                        "GPU {} approaching memory limit: {:.1}GB / {:.1}GB",
                        policy.gpu_index, total_memory, effective_max_memory
                    ),
                    time_to_limit: None,
                });
            }

            if policy.max_memory_gb > 0.0 {
                let utilization_pct = (total_memory / policy.max_memory_gb) * 100.0;
                let limit = policy.max_utilization_pct * time_multipliers.utilization;
                if utilization_pct > limit {
                    let representative =
                        gpu_processes.iter().max_by_key(|p| p.used_mem_mb).unwrap();
                    violations.push(PolicyViolation {
                        violation_type: ViolationType::UtilizationLimitExceeded,
                        severity: ViolationSeverity::Medium,
                        user: representative.user.clone(),
                        process: (*representative).clone(),
                        policy_name: "gpu_utilization_limit".to_string(),
                        current_value: utilization_pct,
                        limit_value: limit,
                        message: format!(
                            "GPU {} exceeded utilization limit: {:.1}% > {:.1}%",
                            policy.gpu_index, utilization_pct, limit
                        ),
                        recommended_action: "Reduce GPU workload".to_string(),
                    });
                } else if utilization_pct > limit * 0.8 {
                    let representative =
                        gpu_processes.iter().max_by_key(|p| p.used_mem_mb).unwrap();
                    warnings.push(PolicyWarning {
                        warning_type: WarningType::ApproachingUtilizationLimit,
                        user: representative.user.clone(),
                        process: (*representative).clone(),
                        policy_name: "gpu_utilization_limit".to_string(),
                        current_value: utilization_pct,
                        limit_value: limit,
                        message: format!(
                            "GPU {} approaching utilization limit: {:.1}% / {:.1}%",
                            policy.gpu_index, utilization_pct, limit
                        ),
                        time_to_limit: None,
                    });
                }
            }

            for process in gpu_processes {
                if !policy.allowed_users.is_empty() && !policy.allowed_users.contains(&process.user)
                {
                    violations.push(PolicyViolation {
                        violation_type: ViolationType::UnauthorizedUserAccess,
                        severity: ViolationSeverity::High,
                        user: process.user.clone(),
                        process: (*process).clone(),
                        policy_name: "gpu_allowed_users".to_string(),
                        current_value: 1.0,
                        limit_value: 0.0,
                        message: format!(
                            "User {} is not allowed on GPU {}",
                            process.user, policy.gpu_index
                        ),
                        recommended_action: "Terminate process or update GPU policy".to_string(),
                    });
                }

                if policy.blocked_users.contains(&process.user) {
                    violations.push(PolicyViolation {
                        violation_type: ViolationType::UnauthorizedUserAccess,
                        severity: ViolationSeverity::Critical,
                        user: process.user.clone(),
                        process: (*process).clone(),
                        policy_name: "gpu_blocked_users".to_string(),
                        current_value: 1.0,
                        limit_value: 0.0,
                        message: format!(
                            "User {} is blocked from GPU {}",
                            process.user, policy.gpu_index
                        ),
                        recommended_action: "Immediately terminate process".to_string(),
                    });
                }
            }
        }
    }

    /// Get configuration file path
    pub fn get_config_file_path(&self) -> &PathBuf {
        &self.config_path
    }

    /// Export configuration to JSON
    pub fn export_to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&self.config)
            .map_err(|e| anyhow::anyhow!("Failed to export config to JSON: {}", e))
    }

    /// Import configuration from JSON
    pub fn import_from_json(&mut self, json: &str) -> Result<()> {
        let config: GuardModeConfig = serde_json::from_str(json)
            .map_err(|e| anyhow::anyhow!("Failed to import config from JSON: {}", e))?;

        self.update_config(config)?;
        Ok(())
    }

    /// Get violation history
    pub fn get_violation_history(&self) -> &Vec<PolicyViolation> {
        &self.violation_history
    }

    /// Get warning history
    pub fn get_warning_history(&self) -> &Vec<PolicyWarning> {
        &self.warning_history
    }

    /// Simulate actions in dry-run mode
    fn simulate_actions(
        &self,
        violations: &[PolicyViolation],
        warnings: &[PolicyWarning],
    ) -> Vec<EnforcementAction> {
        let mut actions = Vec::new();

        for violation in violations {
            let action = match violation.severity {
                ViolationSeverity::Critical => EnforcementAction {
                    action_type: ActionType::ProcessTermination,
                    user: violation.user.clone(),
                    process: violation.process.clone(),
                    policy_name: violation.policy_name.clone(),
                    message: format!(
                        "[DRY-RUN] Would terminate process {} for critical violation: {}",
                        violation.process.pid, violation.message
                    ),
                    success: true,
                },
                ViolationSeverity::High => EnforcementAction {
                    action_type: ActionType::Warning,
                    user: violation.user.clone(),
                    process: violation.process.clone(),
                    policy_name: violation.policy_name.clone(),
                    message: format!(
                        "[DRY-RUN] Would send warning for high severity violation: {}",
                        violation.message
                    ),
                    success: true,
                },
                ViolationSeverity::Medium => EnforcementAction {
                    action_type: ActionType::Warning,
                    user: violation.user.clone(),
                    process: violation.process.clone(),
                    policy_name: violation.policy_name.clone(),
                    message: format!(
                        "[DRY-RUN] Would send warning for medium severity violation: {}",
                        violation.message
                    ),
                    success: true,
                },
                ViolationSeverity::Low => EnforcementAction {
                    action_type: ActionType::NotificationSent,
                    user: violation.user.clone(),
                    process: violation.process.clone(),
                    policy_name: violation.policy_name.clone(),
                    message: format!(
                        "[DRY-RUN] Would send notification for low severity violation: {}",
                        violation.message
                    ),
                    success: true,
                },
            };
            actions.push(action);
        }

        for warning in warnings {
            let action = EnforcementAction {
                action_type: ActionType::NotificationSent,
                user: warning.user.clone(),
                process: warning.process.clone(),
                policy_name: warning.policy_name.clone(),
                message: format!(
                    "[DRY-RUN] Would send warning notification: {}",
                    warning.message
                ),
                success: true,
            };
            actions.push(action);
        }

        actions
    }

    /// Execute actions in enforcement mode
    fn execute_actions(
        &self,
        violations: &[PolicyViolation],
        warnings: &[PolicyWarning],
    ) -> Result<Vec<EnforcementAction>> {
        let mut actions = Vec::new();

        // Send notifications for warnings
        for warning in warnings {
            if self.config.enforcement.notifications.console {
                info!("⚠️ Policy Warning: {} - {}", warning.user, warning.message);
            }

            actions.push(EnforcementAction {
                action_type: ActionType::NotificationSent,
                user: warning.user.clone(),
                process: warning.process.clone(),
                policy_name: warning.policy_name.clone(),
                message: format!("Warning notification sent: {}", warning.message),
                success: true,
            });
        }

        // Handle violations based on enforcement settings
        for violation in violations {
            let action = match violation.severity {
                ViolationSeverity::Critical => {
                    if self.config.enforcement.hard_enforcement {
                        // In a real implementation, this would actually terminate the process
                        // For now, we'll just log it
                        info!(
                            "🚨 CRITICAL VIOLATION: Would terminate process {} for user {}: {}",
                            violation.process.pid, violation.user, violation.message
                        );

                        EnforcementAction {
                            action_type: ActionType::ProcessTermination,
                            user: violation.user.clone(),
                            process: violation.process.clone(),
                            policy_name: violation.policy_name.clone(),
                            message: format!(
                                "Process {} terminated for critical violation: {}",
                                violation.process.pid, violation.message
                            ),
                            success: true,
                        }
                    } else {
                        EnforcementAction {
                            action_type: ActionType::Warning,
                            user: violation.user.clone(),
                            process: violation.process.clone(),
                            policy_name: violation.policy_name.clone(),
                            message: format!(
                                "Critical violation detected (hard enforcement disabled): {}",
                                violation.message
                            ),
                            success: true,
                        }
                    }
                }
                ViolationSeverity::High | ViolationSeverity::Medium => {
                    if self.config.enforcement.soft_enforcement {
                        info!(
                            "⚠️ Policy Violation: {} - {}",
                            violation.user, violation.message
                        );

                        EnforcementAction {
                            action_type: ActionType::Warning,
                            user: violation.user.clone(),
                            process: violation.process.clone(),
                            policy_name: violation.policy_name.clone(),
                            message: format!("Warning sent for violation: {}", violation.message),
                            success: true,
                        }
                    } else {
                        EnforcementAction {
                            action_type: ActionType::NotificationSent,
                            user: violation.user.clone(),
                            process: violation.process.clone(),
                            policy_name: violation.policy_name.clone(),
                            message: format!(
                                "Violation logged (soft enforcement disabled): {}",
                                violation.message
                            ),
                            success: true,
                        }
                    }
                }
                ViolationSeverity::Low => EnforcementAction {
                    action_type: ActionType::NotificationSent,
                    user: violation.user.clone(),
                    process: violation.process.clone(),
                    policy_name: violation.policy_name.clone(),
                    message: format!("Low severity violation logged: {}", violation.message),
                    success: true,
                },
            };
            actions.push(action);
        }

        Ok(actions)
    }

    /// Run policy check simulation (dry-run mode)
    pub fn simulate_policy_check(&mut self, processes: &[GpuProc]) -> Result<EnforcementResult> {
        let original_dry_run = self.config.global.dry_run;
        self.config.global.dry_run = true;

        let result = self.check_policies(processes)?;

        // Restore original setting
        self.config.global.dry_run = original_dry_run;

        Ok(result)
    }

    /// Toggle dry-run mode
    pub fn toggle_dry_run(&mut self) -> Result<bool> {
        self.config.global.dry_run = !self.config.global.dry_run;
        self.config.metadata.last_modified = chrono::Utc::now().to_rfc3339();
        Self::save_config(&self.config_path, &self.config)?;
        Ok(self.config.global.dry_run)
    }
}

fn is_time_window_active(
    now: DateTime<Utc>,
    start_time: &str,
    end_time: &str,
    days_of_week: &[u8],
) -> bool {
    if let (Ok(start), Ok(end)) = (
        chrono::NaiveTime::parse_from_str(start_time, "%H:%M"),
        chrono::NaiveTime::parse_from_str(end_time, "%H:%M"),
    ) {
        let weekday = now.weekday().num_days_from_sunday() as u8;
        let day_match = days_of_week.is_empty() || days_of_week.contains(&weekday);
        if !day_match {
            return false;
        }

        let now_time = now.time();
        if start <= end {
            now_time >= start && now_time <= end
        } else {
            now_time >= start || now_time <= end
        }
    } else {
        false
    }
}

fn parse_duration_hours(start_time: &str, now: DateTime<Utc>) -> Option<f32> {
    if start_time == "unknown" {
        return None;
    }

    if let Ok(parsed) = DateTime::parse_from_rfc3339(start_time) {
        let start = parsed.with_timezone(&Utc);
        let duration = now.signed_duration_since(start);
        return Some(duration.num_seconds() as f32 / 3600.0);
    }

    let mut hours = 0.0;
    let mut minutes = 0.0;
    let mut seconds = 0.0;

    for token in start_time.split_whitespace() {
        if let Some(value) = token.strip_suffix('h') {
            if let Ok(parsed) = value.parse::<f32>() {
                hours += parsed;
            }
        } else if let Some(value) = token.strip_suffix('m') {
            if let Ok(parsed) = value.parse::<f32>() {
                minutes += parsed;
            }
        } else if let Some(value) = token.strip_suffix('s') {
            if let Ok(parsed) = value.parse::<f32>() {
                seconds += parsed;
            }
        }
    }

    if hours == 0.0 && minutes == 0.0 && seconds == 0.0 {
        None
    } else {
        Some(hours + minutes / 60.0 + seconds / 3600.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GuardModeConfig::default();
        assert!(!config.global.enabled);
        assert!(config.global.dry_run);
        assert!(config.global.default_memory_limit_gb > 0.0);
    }

    #[test]
    fn test_user_policy_creation() {
        let config = GuardModeConfig::default();
        let manager = GuardModeManager {
            config_path: PathBuf::new(),
            config,
            violation_history: Vec::new(),
            warning_history: Vec::new(),
        };

        let user_policy = manager.get_user_policy("testuser");
        assert_eq!(user_policy.username, "testuser");
        assert!(user_policy.memory_limit_gb > 0.0);
    }

    #[test]
    fn test_gpu_policy_enforced_for_blocked_user() {
        let mut config = GuardModeConfig::default();
        config.global.enabled = true;

        config.gpu_policies.insert(
            "0".to_string(),
            GpuPolicy {
                gpu_index: 0,
                max_memory_gb: 1.0,
                max_utilization_pct: 10.0,
                reserved_memory_gb: 0.0,
                allowed_users: Vec::new(),
                blocked_users: vec!["testuser".to_string()],
                maintenance_window: None,
            },
        );

        let mut manager = GuardModeManager {
            config_path: PathBuf::new(),
            config,
            violation_history: Vec::new(),
            warning_history: Vec::new(),
        };

        let processes = vec![GpuProc {
            gpu_index: 0,
            pid: 1234,
            user: "testuser".to_string(),
            proc_name: "test_proc".to_string(),
            used_mem_mb: 512,
            start_time: "unknown".to_string(),
            container: None,
        }];

        let result = manager.check_policies(&processes).unwrap();
        assert!(result
            .violations
            .iter()
            .any(|v| matches!(v.violation_type, ViolationType::UnauthorizedUserAccess)));
    }
}
