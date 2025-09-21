use crate::nvml_api::NvmlApi;
use crate::util::parse_process_start_time;
use anyhow::{Context, Result};
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
// use std::process::Command; // Used conditionally below
use std::time::{Duration, SystemTime};
use sysinfo::{Pid as SysPid, System};

/// Process information for a running process
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    #[allow(dead_code)]
    pub pid: u32,
    pub user: String,
    pub name: String,
    #[allow(dead_code)]
    pub start_time: SystemTime,
    #[allow(dead_code)]
    pub cmdline: String,
}

/// Process management utilities
pub struct ProcessManager {
    nvml_api: NvmlApi,
    system: System,
}

#[allow(dead_code)]
impl ProcessManager {
    /// Create a new process manager
    pub fn new(nvml_api: NvmlApi) -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        Self {
            nvml_api,
            system,
        }
    }

    /// Get process information by PID
    pub fn get_process_info(&mut self, pid: u32) -> Result<ProcessInfo> {
        self.system.refresh_processes();

        let sys_pid = SysPid::from_u32(pid);
        let process = self.system.process(sys_pid)
            .ok_or_else(|| anyhow::anyhow!("Process with PID {} not found", pid))?;

        let user = get_process_user(pid)
            .unwrap_or_else(|_| "unknown".to_string());

        let start_time = process.start_time();
        let start_time_system = SystemTime::UNIX_EPOCH + Duration::from_secs(start_time);

        Ok(ProcessInfo {
            pid,
            user,
            name: process.name().to_string(),
            start_time: start_time_system,
            cmdline: process.cmd().join(" "),
        })
    }

    /// Check if a process is using any GPU
    pub fn is_process_using_gpu(&self, pid: u32) -> Result<bool> {
        self.nvml_api.is_process_using_gpu(pid)
    }

    /// Gracefully terminate a process with timeout and escalation
    pub fn graceful_kill(
        &self,
        pid: u32,
        timeout_secs: u16,
        force: bool,
    ) -> Result<()> {
        let pid = Pid::from_raw(pid as i32);

        // First, try SIGTERM
        tracing::info!("Sending SIGTERM to process {}", pid);
        kill(pid, Signal::SIGTERM)
            .map_err(|e| anyhow::anyhow!("Failed to send SIGTERM: {}", e))?;

        // Wait for the process to terminate
        let timeout = Duration::from_secs(timeout_secs as u64);
        let start = SystemTime::now();

        while SystemTime::now().duration_since(start).unwrap_or_default() < timeout {
            // Check if process still exists
            if !self.is_process_running(pid.as_raw() as u32)? {
                tracing::info!("Process {} terminated gracefully", pid);
                return Ok(());
            }

            std::thread::sleep(Duration::from_millis(100));
        }

        // Process didn't terminate, escalate if force is enabled
        if force {
            tracing::warn!("Process {} did not terminate, escalating to SIGKILL", pid);
            kill(pid, Signal::SIGKILL)
                .map_err(|e| anyhow::anyhow!("Failed to send SIGKILL: {}", e))?;

            // Wait a bit more for SIGKILL to take effect
            std::thread::sleep(Duration::from_millis(500));

            if !self.is_process_running(pid.as_raw() as u32)? {
                tracing::info!("Process {} terminated with SIGKILL", pid);
                Ok(())
            } else {
                Err(anyhow::anyhow!("Process {} still running after SIGKILL", pid))
            }
        } else {
            Err(anyhow::anyhow!(
                "Process {} did not terminate within {} seconds. Use --force to escalate to SIGKILL",
                pid,
                timeout_secs
            ))
        }
    }

    /// Check if a process is still running
    fn is_process_running(&self, pid: u32) -> Result<bool> {
        let sys_pid = SysPid::from_u32(pid);
        Ok(self.system.process(sys_pid).is_some())
    }

    /// Enrich GPU processes with system information
    pub fn enrich_gpu_processes(&mut self, mut processes: Vec<crate::nvml_api::GpuProc>) -> Result<Vec<crate::nvml_api::GpuProc>> {
        self.system.refresh_processes();

        for process in &mut processes {
            if let Ok(process_info) = self.get_process_info(process.pid) {
                process.user = process_info.user;
                process.proc_name = process_info.name;
                process.start_time = parse_process_start_time(process_info.start_time);
            }
        }

        Ok(processes)
    }

    /// Get all processes using GPUs with enriched information
    pub fn get_enriched_gpu_processes(&mut self) -> Result<Vec<crate::nvml_api::GpuProc>> {
        let processes = self.nvml_api.get_gpu_processes()?;
        self.enrich_gpu_processes(processes)
    }

    /// Validate that a process exists and optionally check GPU usage
    pub fn validate_process(&self, pid: u32, check_gpu_usage: bool) -> Result<()> {
        // Check if process exists
        let sys_pid = SysPid::from_u32(pid);
        if self.system.process(sys_pid).is_none() {
            return Err(anyhow::anyhow!("Process with PID {} not found", pid));
        }

        // Check GPU usage if requested
        if check_gpu_usage {
            let is_using_gpu = self.is_process_using_gpu(pid)?;
            if !is_using_gpu {
                return Err(anyhow::anyhow!(
                    "Process {} is not using any GPU. Use --force to kill anyway.",
                    pid
                ));
            }
        }

        Ok(())
    }

    /// Get device count
    pub fn device_count(&self) -> Result<u32> {
        self.nvml_api.device_count()
    }

    /// Create snapshot
    pub fn create_snapshot(&self) -> Result<crate::nvml_api::Snapshot> {
        self.nvml_api.create_snapshot()
    }

    /// Reset GPU
    pub fn reset_gpu(&self, index: u32) -> Result<()> {
        self.nvml_api.reset_gpu(index)
    }
}

/// Get the username for a process (cross-platform)
fn get_process_user(pid: u32) -> Result<String> {
    #[cfg(target_os = "linux")]
    {
        // On Linux, read from /proc/<pid>/status
        let status_path = format!("/proc/{}/status", pid);
        let status = std::fs::read_to_string(&status_path)
            .with_context(|| format!("Failed to read process status from {}", status_path))?;

        for line in status.lines() {
            if line.starts_with("Uid:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let uid = parts[1].parse::<u32>()
                        .with_context(|| format!("Failed to parse UID: {}", parts[1]))?;
                    
                    // Get username from UID
                    return get_username_from_uid(uid);
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        // On macOS, use ps command
        let output = Command::new("ps")
            .args(["-o", "user=", "-p", &pid.to_string()])
            .output()
            .context("Failed to execute ps command")?;

        if output.status.success() {
            let user = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !user.is_empty() {
                return Ok(user);
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        // On Windows, use wmic command
        let output = Command::new("wmic")
            .args(&["process", "where", &format!("ProcessId={}", pid), "get", "ExecutablePath", "/format:value"])
            .output()
            .context("Failed to execute wmic command")?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.starts_with("ExecutablePath=") {
                    let path = line.strip_prefix("ExecutablePath=").unwrap_or("");
                    if !path.is_empty() {
                        // Extract username from path or use a default
                        return Ok("windows_user".to_string());
                    }
                }
            }
        }
    }

    Ok("unknown".to_string())
}

#[cfg(target_os = "linux")]
fn get_username_from_uid(uid: u32) -> Result<String> {
    use std::ffi::CString;
    // use std::os::unix::ffi::OsStringExt; // Unused for now

    unsafe {
        let passwd = libc::getpwuid(uid as libc::uid_t);
        if passwd.is_null() {
            return Ok(format!("uid_{}", uid));
        }

        let username = CString::from_raw((*passwd).pw_name);
        let username_str = username.to_string_lossy().to_string();
        std::mem::forget(username); // Don't free the passwd struct
        Ok(username_str)
    }
}

#[cfg(not(target_os = "linux"))]
#[allow(dead_code)]
fn get_username_from_uid(_uid: u32) -> Result<String> {
    Ok("unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nvml_api::NvmlApi;

    #[test]
    fn test_process_info_creation() {
        // Skip this test if NVML is not available
        let nvml_api = match NvmlApi::new() {
            Ok(api) => api,
            Err(_) => {
                // Skip test if NVML is not available
                return;
            }
        };
        
        let mut proc_mgr = ProcessManager::new(nvml_api);
        
        // Test with a known process (init/systemd)
        if let Ok(info) = proc_mgr.get_process_info(1) {
            assert_eq!(info.pid, 1);
            assert!(!info.name.is_empty());
        }
    }

    #[test]
    fn test_process_validation() {
        // Skip this test if NVML is not available
        let nvml_api = match NvmlApi::new() {
            Ok(api) => api,
            Err(_) => {
                // Skip test if NVML is not available
                return;
            }
        };
        
        let proc_mgr = ProcessManager::new(nvml_api);
        
        // Test validation of non-existent process
        let result = proc_mgr.validate_process(999999, false);
        assert!(result.is_err());
    }
}
