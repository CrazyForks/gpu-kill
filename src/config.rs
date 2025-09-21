use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Configuration structure for gpukill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Default log level
    pub log_level: String,
    
    /// Default output format
    pub output_format: String,
    
    /// Default timeout for process termination
    pub default_timeout_secs: u16,
    
    /// Whether to show detailed process information by default
    pub show_details: bool,
    
    /// Watch mode refresh interval in seconds
    pub watch_interval_secs: u64,
    
    /// Maximum number of processes to show in summary
    pub max_processes_summary: usize,
    
    /// Table width limit
    pub table_width: usize,
    
    /// Whether to use colors in output
    pub use_colors: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            output_format: "table".to_string(),
            default_timeout_secs: 5,
            show_details: false,
            watch_interval_secs: 2,
            max_processes_summary: 10,
            table_width: 120,
            use_colors: true,
        }
    }
}

/// Configuration manager
pub struct ConfigManager {
    config: Config,
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }

    /// Load configuration from file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config_path = path.as_ref();
        
        if !config_path.exists() {
            tracing::debug!("Config file not found at {:?}, using defaults", config_path);
            return Ok(Self::new());
        }

        let content = fs::read_to_string(config_path)
            .with_context(|| format!("Failed to read config file: {:?}", config_path))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", config_path))?;

        tracing::info!("Loaded configuration from {:?}", config_path);
        Ok(Self { config })
    }

    /// Load configuration from environment variables
    pub fn load_from_env() -> Self {
        let mut config = Config::default();

        // Override with environment variables if present
        if let Ok(log_level) = std::env::var("GPUKILL_LOG_LEVEL") {
            config.log_level = log_level;
        }

        if let Ok(output_format) = std::env::var("GPUKILL_OUTPUT_FORMAT") {
            config.output_format = output_format;
        }

        if let Ok(timeout) = std::env::var("GPUKILL_DEFAULT_TIMEOUT") {
            if let Ok(timeout_secs) = timeout.parse::<u16>() {
                config.default_timeout_secs = timeout_secs;
            }
        }

        if let Ok(show_details) = std::env::var("GPUKILL_SHOW_DETAILS") {
            config.show_details = show_details.parse().unwrap_or(false);
        }

        if let Ok(watch_interval) = std::env::var("GPUKILL_WATCH_INTERVAL") {
            if let Ok(interval_secs) = watch_interval.parse::<u64>() {
                config.watch_interval_secs = interval_secs;
            }
        }

        if let Ok(table_width) = std::env::var("GPUKILL_TABLE_WIDTH") {
            if let Ok(width) = table_width.parse::<usize>() {
                config.table_width = width;
            }
        }

        if let Ok(use_colors) = std::env::var("GPUKILL_USE_COLORS") {
            config.use_colors = use_colors.parse().unwrap_or(true);
        }

        Self { config }
    }

    /// Get the current configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get a mutable reference to the configuration
    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    /// Save configuration to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let config_path = path.as_ref();
        let content = toml::to_string_pretty(&self.config)
            .context("Failed to serialize configuration")?;

        fs::write(config_path, content)
            .with_context(|| format!("Failed to write config file: {:?}", config_path))?;

        tracing::info!("Saved configuration to {:?}", config_path);
        Ok(())
    }

    /// Get default configuration file path
    pub fn default_config_path() -> Result<std::path::PathBuf> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
        
        Ok(home_dir.join(".config").join("gpukill").join("config.toml"))
    }

    /// Load configuration from default location
    pub fn load_default() -> Result<Self> {
        let config_path = Self::default_config_path()?;
        Self::load_from_file(config_path)
    }

    /// Create default configuration file
    pub fn create_default_config() -> Result<()> {
        let config_path = Self::default_config_path()?;
        
        // Create directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        let config_manager = Self::new();
        config_manager.save_to_file(config_path)?;
        
        Ok(())
    }
}

/// Get configuration with fallback chain
pub fn get_config(config_path: Option<String>) -> Result<ConfigManager> {
    // 1. Try to load from specified path
    if let Some(path) = config_path {
        return ConfigManager::load_from_file(path);
    }

    // 2. Try to load from default location
    if let Ok(config_manager) = ConfigManager::load_default() {
        return Ok(config_manager);
    }

    // 3. Load from environment variables
    Ok(ConfigManager::load_from_env())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.log_level, "info");
        assert_eq!(config.output_format, "table");
        assert_eq!(config.default_timeout_secs, 5);
        assert!(!config.show_details);
        assert_eq!(config.watch_interval_secs, 2);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();
        
        assert_eq!(config.log_level, deserialized.log_level);
        assert_eq!(config.output_format, deserialized.output_format);
    }

    #[test]
    fn test_config_file_loading() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        
        let temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), toml_str).unwrap();
        
        let loaded_config = ConfigManager::load_from_file(temp_file.path()).unwrap();
        assert_eq!(loaded_config.config().log_level, config.log_level);
    }

    #[test]
    fn test_config_manager_creation() {
        let manager = ConfigManager::new();
        assert_eq!(manager.config().log_level, "info");
    }
}
