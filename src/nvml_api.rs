use crate::util::{get_current_timestamp_iso, get_hostname};
use anyhow::{Context, Result};
use nvml_wrapper::error::NvmlError;
use nvml_wrapper::Nvml;
use serde::{Deserialize, Serialize};

/// GPU information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub index: u16,
    pub name: String,
    pub mem_total_mb: u32,
}

/// GPU process information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuProc {
    pub gpu_index: u16,
    pub pid: u32,
    pub user: String,
    pub proc_name: String,
    pub used_mem_mb: u32,
    pub start_time: String,
    pub container: Option<String>,
}

/// GPU snapshot with current status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuSnapshot {
    pub gpu_index: u16,
    pub name: String,
    pub mem_used_mb: u32,
    pub mem_total_mb: u32,
    pub util_pct: f32,
    pub temp_c: i32,
    pub power_w: f32,
    pub ecc_volatile: Option<u64>,
    pub pids: usize,
    pub top_proc: Option<GpuProc>,
}

/// Complete system snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub host: String,
    pub ts: String,
    pub gpus: Vec<GpuSnapshot>,
    pub procs: Vec<GpuProc>,
}

/// NVML API wrapper for GPU operations
pub struct NvmlApi {
    nvml: Nvml,
}

#[allow(dead_code)]
impl NvmlApi {
    /// Initialize NVML API
    pub fn new() -> Result<Self> {
        let nvml = Nvml::init()
            .map_err(|e| map_nvml_error(e))
            .context("Failed to initialize NVML. Ensure NVIDIA drivers are installed and GPU is accessible.")?;

        Ok(Self { nvml })
    }

    /// Get the number of available GPUs
    pub fn device_count(&self) -> Result<u32> {
        self.nvml
            .device_count()
            .map_err(|e| map_nvml_error(e))
            .context("Failed to get device count")
    }

    /// Get basic GPU information
    pub fn get_gpu_info(&self, index: u32) -> Result<GpuInfo> {
        let device = self.nvml.device_by_index(index)
            .map_err(|e| map_nvml_error(e))
            .with_context(|| format!("Failed to get device at index {}", index))?;

        let name = device.name()
            .map_err(|e| map_nvml_error(e))
            .context("Failed to get device name")?;

        let mem_info = device.memory_info()
            .map_err(|e| map_nvml_error(e))
            .context("Failed to get memory info")?;

        Ok(GpuInfo {
            index: index as u16,
            name,
            mem_total_mb: (mem_info.total / 1024 / 1024) as u32,
        })
    }

    /// Get detailed GPU snapshot
    pub fn get_gpu_snapshot(&self, index: u32) -> Result<GpuSnapshot> {
        let device = self.nvml.device_by_index(index)
            .map_err(|e| map_nvml_error(e))
            .with_context(|| format!("Failed to get device at index {}", index))?;

        let name = device.name()
            .map_err(|e| map_nvml_error(e))
            .context("Failed to get device name")?;

        let mem_info = device.memory_info()
            .map_err(|e| map_nvml_error(e))
            .context("Failed to get memory info")?;

        let utilization = device.utilization_rates()
            .map_err(|e| map_nvml_error(e))
            .context("Failed to get utilization rates")?;

        let temperature = device.temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
            .map_err(|e| map_nvml_error(e))
            .context("Failed to get temperature")?;

        let power_usage = device.power_usage()
            .map_err(|e| map_nvml_error(e))
            .context("Failed to get power usage")?;

        let ecc_volatile = None; // ECC errors not available in this version

        let processes = device.running_compute_processes()
            .map_err(|e| map_nvml_error(e))
            .context("Failed to get running processes")?;

        let pids: Vec<u32> = processes.iter().map(|p| p.pid).collect();
        let top_proc = processes.first()
            .map(|p| GpuProc {
                gpu_index: index as u16,
                pid: p.pid,
                user: "unknown".to_string(), // Will be filled by process info
                proc_name: "unknown".to_string(), // Will be filled by process info
                used_mem_mb: 0, // Will be filled by process info
                start_time: "unknown".to_string(), // Will be filled by process info
                container: None,
            });

        Ok(GpuSnapshot {
            gpu_index: index as u16,
            name,
            mem_used_mb: (mem_info.used / 1024 / 1024) as u32,
            mem_total_mb: (mem_info.total / 1024 / 1024) as u32,
            util_pct: utilization.gpu as f32,
            temp_c: temperature as i32,
            power_w: power_usage as f32 / 1000.0, // Convert mW to W
            ecc_volatile,
            pids: pids.len(),
            top_proc,
        })
    }

    /// Get all GPU snapshots
    pub fn get_all_snapshots(&self) -> Result<Vec<GpuSnapshot>> {
        let count = self.device_count()?;
        let mut snapshots = Vec::new();

        for i in 0..count {
            let snapshot = self.get_gpu_snapshot(i)?;
            snapshots.push(snapshot);
        }

        Ok(snapshots)
    }

    /// Get processes using GPUs
    pub fn get_gpu_processes(&self) -> Result<Vec<GpuProc>> {
        let count = self.device_count()?;
        let mut all_processes = Vec::new();

        for i in 0..count {
            let device = self.nvml.device_by_index(i)
                .map_err(|e| map_nvml_error(e))
                .with_context(|| format!("Failed to get device at index {}", i))?;

            let processes = device.running_compute_processes()
                .map_err(|e| map_nvml_error(e))
                .with_context(|| format!("Failed to get processes for GPU {}", i))?;

            for process in processes {
                all_processes.push(GpuProc {
                    gpu_index: i as u16,
                    pid: process.pid,
                    user: "unknown".to_string(), // Will be filled by process info
                    proc_name: "unknown".to_string(), // Will be filled by process info
                    used_mem_mb: 0, // Will be filled by process info
                    start_time: "unknown".to_string(), // Will be filled by process info
                    container: None,
                });
            }
        }

        Ok(all_processes)
    }

    /// Check if a process is using any GPU
    pub fn is_process_using_gpu(&self, pid: u32) -> Result<bool> {
        let count = self.device_count()?;

        for i in 0..count {
            let device = self.nvml.device_by_index(i)
                .map_err(|e| map_nvml_error(e))
                .with_context(|| format!("Failed to get device at index {}", i))?;

            let processes = device.running_compute_processes()
                .map_err(|e| map_nvml_error(e))
                .with_context(|| format!("Failed to get processes for GPU {}", i))?;

            if processes.iter().any(|p| p.pid == pid) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Reset a specific GPU
    pub fn reset_gpu(&self, index: u32) -> Result<()> {
        let _device = self.nvml.device_by_index(index)
            .map_err(|e| map_nvml_error(e))
            .with_context(|| format!("Failed to get device at index {}", index))?;

        // Check if reset is supported (placeholder check)
        // Note: NVML doesn't provide a direct way to check reset support

        // Note: NVML doesn't provide a direct reset function
        // This would typically require system-level operations
        Err(anyhow::anyhow!("GPU reset is not supported via NVML. Use system-level tools like nvidia-smi --gpu-reset"))
    }

    /// Create a complete system snapshot
    pub fn create_snapshot(&self) -> Result<Snapshot> {
        let gpus = self.get_all_snapshots()?;
        let procs = self.get_gpu_processes()?;

        Ok(Snapshot {
            host: get_hostname(),
            ts: get_current_timestamp_iso(),
            gpus,
            procs,
        })
    }
}

/// Map NVML errors to user-friendly messages
fn map_nvml_error(error: NvmlError) -> anyhow::Error {
    match error {
        NvmlError::Uninitialized => {
            anyhow::anyhow!("NVML not initialized. Ensure NVIDIA drivers are properly installed.")
        }
        NvmlError::InvalidArg => {
            anyhow::anyhow!("Invalid argument provided to NVML function.")
        }
        NvmlError::NotSupported => {
            anyhow::anyhow!("NVML operation not supported on this system.")
        }
        NvmlError::NoPermission => {
            anyhow::anyhow!("Insufficient permissions to access GPU. Try running with sudo or check user permissions.")
        }
        NvmlError::AlreadyInitialized => {
            anyhow::anyhow!("NVML already initialized.")
        }
        NvmlError::NotFound => {
            anyhow::anyhow!("GPU device not found.")
        }
        NvmlError::InsufficientSize(_) => {
            anyhow::anyhow!("Insufficient buffer size for NVML operation.")
        }
        NvmlError::DriverNotLoaded => {
            anyhow::anyhow!("NVIDIA driver not loaded. Please install and load the NVIDIA driver.")
        }
        NvmlError::Timeout => {
            anyhow::anyhow!("NVML operation timed out.")
        }
        NvmlError::IrqIssue => {
            anyhow::anyhow!("GPU interrupt issue detected.")
        }
        NvmlError::LibraryNotFound => {
            anyhow::anyhow!("NVML library not found. Please install NVIDIA drivers.")
        }
        NvmlError::FunctionNotFound => {
            anyhow::anyhow!("NVML function not found. Driver version may be incompatible.")
        }
        NvmlError::CorruptedInfoROM => {
            anyhow::anyhow!("GPU information ROM is corrupted.")
        }
        NvmlError::GpuLost => {
            anyhow::anyhow!("GPU has been lost and needs to be reset.")
        }
        NvmlError::ResetRequired => {
            anyhow::anyhow!("GPU reset is required.")
        }
        NvmlError::OperatingSystem => {
            anyhow::anyhow!("Operating system error occurred.")
        }
        NvmlError::LibRmVersionMismatch => {
            anyhow::anyhow!("NVML and driver version mismatch.")
        }
        NvmlError::InUse => {
            anyhow::anyhow!("GPU is currently in use.")
        }
        NvmlError::NoData => {
            anyhow::anyhow!("No data available.")
        }
        NvmlError::Unknown => {
            anyhow::anyhow!("Unknown NVML error occurred.")
        }
        _ => {
            anyhow::anyhow!("NVML error: {:?}", error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_info_serialization() {
        let gpu_info = GpuInfo {
            index: 0,
            name: "Test GPU".to_string(),
            mem_total_mb: 8192,
        };

        let json = serde_json::to_string(&gpu_info).unwrap();
        let deserialized: GpuInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(gpu_info.index, deserialized.index);
        assert_eq!(gpu_info.name, deserialized.name);
        assert_eq!(gpu_info.mem_total_mb, deserialized.mem_total_mb);
    }

    #[test]
    fn test_gpu_snapshot_serialization() {
        let snapshot = GpuSnapshot {
            gpu_index: 0,
            name: "Test GPU".to_string(),
            mem_used_mb: 4096,
            mem_total_mb: 8192,
            util_pct: 50.0,
            temp_c: 75,
            power_w: 150.0,
            ecc_volatile: Some(0),
            pids: 2,
            top_proc: None,
        };

        let json = serde_json::to_string(&snapshot).unwrap();
        let deserialized: GpuSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(snapshot.gpu_index, deserialized.gpu_index);
        assert_eq!(snapshot.util_pct, deserialized.util_pct);
    }
}
