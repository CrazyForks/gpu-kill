/// Version information for the gpukill CLI tool
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build information
pub const BUILD_DATE: &str = "unknown";
pub const BUILD_TARGET: &str = "unknown";
#[allow(dead_code)]
pub const GIT_COMMIT: &str = "unknown";

/// Get formatted version string
pub fn get_version_string() -> String {
    format!("gpukill {} ({} {})", VERSION, BUILD_TARGET, BUILD_DATE)
}

/// Get detailed version information
#[allow(dead_code)]
pub fn get_detailed_version() -> String {
    format!(
        "gpukill version {}\n\
         Build target: {}\n\
         Build date: {}\n\
         Git commit: {}",
        VERSION, BUILD_TARGET, BUILD_DATE, GIT_COMMIT
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_string_format() {
        let version = get_version_string();
        assert!(version.contains("gpukill"));
        assert!(version.contains(VERSION));
    }

    #[test]
    fn test_detailed_version_format() {
        let detailed = get_detailed_version();
        assert!(detailed.contains("gpukill version"));
        assert!(detailed.contains(VERSION));
        assert!(detailed.contains("Build target:"));
        assert!(detailed.contains("Build date:"));
        assert!(detailed.contains("Git commit:"));
    }
}
