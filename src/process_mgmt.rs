use crate::nvml_api::GpuProc;
use crate::proc::ProcessManager;
use anyhow::Result;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use sysinfo::{Pid as SysPid, System};

/// Enhanced process management with filtering and batch operations
pub struct EnhancedProcessManager {
    pub process_manager: ProcessManager,
    system: System,
}

#[allow(dead_code)]
impl EnhancedProcessManager {
    pub fn new(process_manager: ProcessManager) -> Self {
        Self {
            process_manager,
            system: System::new_all(),
        }
    }

    /// Filter processes by name pattern (supports regex)
    pub fn filter_processes_by_name(
        &mut self,
        processes: &[GpuProc],
        pattern: &str,
    ) -> Result<Vec<GpuProc>> {
        let regex = Regex::new(pattern)
            .map_err(|e| anyhow::anyhow!("Invalid regex pattern '{}': {}", pattern, e))?;

        let mut filtered = Vec::new();
        for proc in processes {
            if regex.is_match(&proc.proc_name) {
                filtered.push(proc.clone());
            }
        }

        Ok(filtered)
    }

    /// Filter processes by user
    pub fn filter_processes_by_user(
        &mut self,
        processes: &[GpuProc],
        user: &str,
    ) -> Result<Vec<GpuProc>> {
        let regex = Regex::new(user)
            .map_err(|e| anyhow::anyhow!("Invalid regex pattern '{}': {}", user, e))?;

        let mut filtered = Vec::new();
        for proc in processes {
            if regex.is_match(&proc.user) {
                filtered.push(proc.clone());
            }
        }

        Ok(filtered)
    }

    /// Filter processes by memory usage threshold
    pub fn filter_processes_by_memory(
        &mut self,
        processes: &[GpuProc],
        min_mb: u32,
    ) -> Vec<GpuProc> {
        processes
            .iter()
            .filter(|proc| proc.used_mem_mb >= min_mb)
            .cloned()
            .collect()
    }

    /// Get process tree for a given PID
    pub fn get_process_tree(&mut self, root_pid: u32) -> Result<Vec<u32>> {
        self.system.refresh_processes();

        let mut pids = Vec::new();
        let mut to_process = vec![root_pid];

        while let Some(pid) = to_process.pop() {
            pids.push(pid);

            // Find child processes
            for process in self.system.processes().values() {
                if let Some(parent) = process.parent() {
                    if parent.as_u32() == pid {
                        to_process.push(process.pid().as_u32());
                    }
                }
            }
        }

        Ok(pids)
    }

    /// Kill a process and its children
    pub fn kill_process_tree(
        &mut self,
        root_pid: u32,
        timeout_secs: u16,
        force: bool,
    ) -> Result<()> {
        let pids = self.get_process_tree(root_pid)?;

        tracing::info!("Killing process tree: {:?}", pids);

        // Kill children first, then parent
        for pid in pids.iter().rev() {
            if let Err(e) = self
                .process_manager
                .graceful_kill(*pid, timeout_secs, force)
            {
                tracing::warn!("Failed to kill process {}: {}", pid, e);
            }
        }

        Ok(())
    }

    /// Batch kill processes. Deduplicates by PID so a process using multiple GPUs
    /// is only killed once (otherwise the first kill succeeds and later attempts fail with ESRCH).
    pub fn batch_kill_processes(
        &mut self,
        processes: &[GpuProc],
        timeout_secs: u16,
        force: bool,
    ) -> Result<Vec<u32>> {
        let mut killed_pids = Vec::new();
        let mut failed_pids = Vec::new();
        let mut seen_pids = HashSet::new();
        for proc in processes {
            if !seen_pids.insert(proc.pid) {
                continue;
            }
            match self
                .process_manager
                .graceful_kill(proc.pid, timeout_secs, force)
            {
                Ok(()) => {
                    killed_pids.push(proc.pid);
                    tracing::info!(
                        "Successfully killed process {} ({})",
                        proc.pid,
                        proc.proc_name
                    );
                }
                Err(e) => {
                    failed_pids.push(proc.pid);
                    tracing::warn!(
                        "Failed to kill process {} ({}): {}",
                        proc.pid,
                        proc.proc_name,
                        e
                    );
                }
            }
        }

        if !failed_pids.is_empty() {
            return Err(anyhow::anyhow!(
                "Failed to kill {} processes: {:?}",
                failed_pids.len(),
                failed_pids
            ));
        }

        Ok(killed_pids)
    }

    /// Detect if a process is running in a container
    pub fn detect_container(&mut self, pid: u32) -> Result<Option<String>> {
        self.system.refresh_processes();

        let sys_pid = SysPid::from_u32(pid);
        let process = self
            .system
            .process(sys_pid)
            .ok_or_else(|| anyhow::anyhow!("Process {} not found", pid))?;

        // Check for common container indicators
        let cmdline = process.cmd().join(" ");

        // Docker
        if cmdline.contains("docker") || cmdline.contains("containerd") {
            return Ok(Some("docker".to_string()));
        }

        // Podman
        if cmdline.contains("podman") {
            return Ok(Some("podman".to_string()));
        }

        // Kubernetes
        if cmdline.contains("kubelet") || cmdline.contains("k8s") {
            return Ok(Some("kubernetes".to_string()));
        }

        // LXC
        if cmdline.contains("lxc") {
            return Ok(Some("lxc".to_string()));
        }

        // Check environment variables for container indicators
        let env = process.environ();
        for env_var in env {
            if env_var.starts_with("CONTAINER")
                || env_var.starts_with("DOCKER")
                || env_var.starts_with("KUBERNETES")
            {
                return Ok(Some("container".to_string()));
            }
        }

        Ok(None)
    }

    /// Enrich GPU processes with container information
    pub fn enrich_with_containers(&mut self, mut processes: Vec<GpuProc>) -> Result<Vec<GpuProc>> {
        for proc in &mut processes {
            match self.detect_container(proc.pid) {
                Ok(container) => proc.container = container,
                Err(e) => {
                    tracing::warn!("Failed to detect container for PID {}: {}", proc.pid, e);
                    proc.container = None;
                }
            }
        }

        Ok(processes)
    }

    /// Get process statistics. Counts unique PIDs so multi-GPU processes are not double-counted.
    pub fn get_process_stats(&mut self, processes: &[GpuProc]) -> ProcessStats {
        let mut stats = ProcessStats::default();
        let mut seen_pids = HashSet::new();

        for proc in processes {
            seen_pids.insert(proc.pid);
            stats.total_memory_mb += proc.used_mem_mb;

            // Count by user
            *stats.users.entry(proc.user.clone()).or_insert(0) += 1;

            // Count by process name
            *stats
                .process_names
                .entry(proc.proc_name.clone())
                .or_insert(0) += 1;

            // Count containers
            if let Some(container) = &proc.container {
                *stats.containers.entry(container.clone()).or_insert(0) += 1;
            } else {
                stats.non_container_processes += 1;
            }
        }

        stats.total_processes = seen_pids.len();
        stats
    }
}

/// Process statistics
#[derive(Debug, Default)]
pub struct ProcessStats {
    pub total_processes: usize,
    pub total_memory_mb: u32,
    pub non_container_processes: usize,
    pub users: HashMap<String, usize>,
    pub process_names: HashMap<String, usize>,
    pub containers: HashMap<String, usize>,
}

impl std::fmt::Display for ProcessStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Process Statistics:")?;
        writeln!(f, "  Total processes: {}", self.total_processes)?;
        writeln!(f, "  Total memory: {} MB", self.total_memory_mb)?;
        writeln!(
            f,
            "  Non-container processes: {}",
            self.non_container_processes
        )?;

        if !self.users.is_empty() {
            writeln!(f, "  Users:")?;
            for (user, count) in &self.users {
                writeln!(f, "    {}: {}", user, count)?;
            }
        }

        if !self.process_names.is_empty() {
            writeln!(f, "  Process names:")?;
            for (name, count) in &self.process_names {
                writeln!(f, "    {}: {}", name, count)?;
            }
        }

        if !self.containers.is_empty() {
            writeln!(f, "  Containers:")?;
            for (container, count) in &self.containers {
                writeln!(f, "    {}: {}", container, count)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nvml_api::GpuProc;

    fn create_test_process(pid: u32, name: &str, user: &str, memory: u32) -> GpuProc {
        GpuProc {
            gpu_index: 0,
            pid,
            user: user.to_string(),
            proc_name: name.to_string(),
            used_mem_mb: memory,
            start_time: "1h".to_string(),
            container: None,
            node_id: None,
        }
    }

    #[test]
    fn test_filter_processes_by_name() {
        let processes = vec![
            create_test_process(1, "python", "user1", 100),
            create_test_process(2, "python3", "user1", 200),
            create_test_process(3, "java", "user2", 300),
        ];

        // Skip test if NVML is not available
        if let Ok(nvml_api) = crate::nvml_api::NvmlApi::new() {
            let mut manager = EnhancedProcessManager {
                process_manager: ProcessManager::new(nvml_api),
                system: System::new_all(),
            };

            let filtered = manager
                .filter_processes_by_name(&processes, "python")
                .unwrap();
            assert_eq!(filtered.len(), 2);
            assert_eq!(filtered[0].proc_name, "python");
            assert_eq!(filtered[1].proc_name, "python3");
        }
    }

    #[test]
    fn test_filter_processes_by_memory() {
        let processes = vec![
            create_test_process(1, "python", "user1", 100),
            create_test_process(2, "python3", "user1", 200),
            create_test_process(3, "java", "user2", 300),
        ];

        // Skip test if NVML is not available
        if let Ok(nvml_api) = crate::nvml_api::NvmlApi::new() {
            let mut manager = EnhancedProcessManager {
                process_manager: ProcessManager::new(nvml_api),
                system: System::new_all(),
            };

            let filtered = manager.filter_processes_by_memory(&processes, 200);
            assert_eq!(filtered.len(), 2);
            assert!(filtered.iter().all(|p| p.used_mem_mb >= 200));
        }
    }

    #[test]
    fn test_process_stats() {
        let processes = vec![
            create_test_process(1, "python", "user1", 100),
            create_test_process(2, "python", "user1", 200),
            create_test_process(3, "java", "user2", 300),
        ];

        // Skip test if NVML is not available
        if let Ok(nvml_api) = crate::nvml_api::NvmlApi::new() {
            let mut manager = EnhancedProcessManager {
                process_manager: ProcessManager::new(nvml_api),
                system: System::new_all(),
            };

            let stats = manager.get_process_stats(&processes);
            assert_eq!(stats.total_processes, 3);
            assert_eq!(stats.total_memory_mb, 600);
            assert_eq!(stats.users.len(), 2);
            assert_eq!(stats.process_names.len(), 2);
        }
    }
}
