use chrono::{DateTime, Local, Utc};
use std::time::{Duration, SystemTime};

/// Get the current hostname
pub fn get_hostname() -> String {
    hostname::get()
        .unwrap_or_else(|_| std::ffi::OsString::from("unknown"))
        .to_string_lossy()
        .to_string()
}

/// Format a timestamp as a human-readable string
pub fn format_timestamp(timestamp: SystemTime) -> String {
    let datetime: DateTime<Local> = timestamp.into();
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Format a timestamp as ISO 8601 string
pub fn format_timestamp_iso(timestamp: SystemTime) -> String {
    let datetime: DateTime<Utc> = timestamp.into();
    datetime.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

/// Get current timestamp as ISO 8601 string
pub fn get_current_timestamp_iso() -> String {
    format_timestamp_iso(SystemTime::now())
}

/// Format duration as human-readable string
pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

/// Format memory size in bytes to human-readable format
pub fn format_memory_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    const THRESHOLD: u64 = 1024;

    if bytes == 0 {
        return "0 B".to_string();
    }

    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= THRESHOLD as f64 && unit_index < UNITS.len() - 1 {
        size /= THRESHOLD as f64;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Format memory size in MB to GiB
pub fn format_memory_mb_to_gib(mb: u32) -> String {
    let gib = mb as f64 / 1024.0;
    format!("{:.1}", gib)
}

/// Check if running on Linux
pub fn is_linux() -> bool {
    cfg!(target_os = "linux")
}

/// Check if running on macOS
pub fn is_macos() -> bool {
    cfg!(target_os = "macos")
}

/// Check if running on Windows
pub fn is_windows() -> bool {
    cfg!(target_os = "windows")
}

/// Get operating system name
pub fn get_os_name() -> &'static str {
    if is_linux() {
        "Linux"
    } else if is_macos() {
        "macOS"
    } else if is_windows() {
        "Windows"
    } else {
        "Unknown"
    }
}

/// Truncate string to specified length with ellipsis
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Parse process start time from system time
pub fn parse_process_start_time(start_time: SystemTime) -> String {
    let now = SystemTime::now();
    let duration = now.duration_since(start_time).unwrap_or_default();
    format_duration(duration)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(30)), "30s");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m 30s");
        assert_eq!(format_duration(Duration::from_secs(3661)), "1h 1m 1s");
    }

    #[test]
    fn test_format_memory_size() {
        assert_eq!(format_memory_size(0), "0 B");
        assert_eq!(format_memory_size(1024), "1.0 KB");
        assert_eq!(format_memory_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_memory_size(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn test_format_memory_mb_to_gib() {
        assert_eq!(format_memory_mb_to_gib(0), "0.0");
        assert_eq!(format_memory_mb_to_gib(1024), "1.0");
        assert_eq!(format_memory_mb_to_gib(2048), "2.0");
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("short", 10), "short");
        assert_eq!(truncate_string("very long string", 10), "very lo...");
        assert_eq!(truncate_string("abc", 3), "abc");
    }

    #[test]
    fn test_os_detection() {
        // These tests will pass on the respective platforms
        assert!(get_os_name() != "Unknown");
    }
}
