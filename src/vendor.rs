use crate::nvml_api::{GpuInfo, GpuProc, GpuSnapshot};
use anyhow::Result;
use nvml_wrapper::enums::device::UsedGpuMemory;
use nvml_wrapper::struct_wrappers::device::ProcessInfo;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// GPU vendor types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Apple,
    Unknown,
}

impl std::fmt::Display for GpuVendor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuVendor::Nvidia => write!(f, "NVIDIA"),
            GpuVendor::Amd => write!(f, "AMD"),
            GpuVendor::Intel => write!(f, "Intel"),
            GpuVendor::Apple => write!(f, "Apple"),
            GpuVendor::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Trait for GPU vendor implementations
pub trait GpuVendorInterface {
    /// Initialize the vendor interface
    fn initialize() -> Result<Self>
    where
        Self: Sized;

    /// Get the vendor type
    fn vendor_type(&self) -> GpuVendor;

    /// Get the number of available GPUs
    fn device_count(&self) -> Result<u32>;

    /// Get basic information about a GPU
    fn get_gpu_info(&self, index: u32) -> Result<GpuInfo>;

    /// Get a snapshot of GPU state and processes
    fn get_gpu_snapshot(&self, index: u32) -> Result<GpuSnapshot>;

    /// Get all processes using a specific GPU
    fn get_gpu_processes(&self, index: u32) -> Result<Vec<GpuProc>>;

    /// Reset a specific GPU
    fn reset_gpu(&self, index: u32) -> Result<()>;

    /// Check if the vendor is available on this system
    fn is_available() -> bool
    where
        Self: Sized;

    /// Get vendor-specific error message for common issues
    fn get_availability_error() -> String
    where
        Self: Sized;
}

/// NVIDIA GPU vendor implementation
pub struct NvidiaVendor {
    nvml: nvml_wrapper::Nvml,
}

fn merge_nvml_processes(
    compute_processes: Vec<ProcessInfo>,
    graphics_processes: Vec<ProcessInfo>,
) -> Vec<ProcessInfo> {
    let mut seen = HashSet::new();
    let mut processes = Vec::new();

    for process in compute_processes.into_iter().chain(graphics_processes) {
        if seen.insert(process.pid) {
            processes.push(process);
        }
    }

    processes
}

impl GpuVendorInterface for NvidiaVendor {
    fn initialize() -> Result<Self> {
        let nvml = nvml_wrapper::Nvml::init()
            .map_err(|e| anyhow::anyhow!("Failed to initialize NVML: {:?}", e))?;
        Ok(Self { nvml })
    }

    fn vendor_type(&self) -> GpuVendor {
        GpuVendor::Nvidia
    }

    fn device_count(&self) -> Result<u32> {
        self.nvml
            .device_count()
            .map_err(|e| anyhow::anyhow!("Failed to get device count: {:?}", e))
    }

    fn get_gpu_info(&self, index: u32) -> Result<GpuInfo> {
        let device = self
            .nvml
            .device_by_index(index)
            .map_err(|e| anyhow::anyhow!("Failed to get device at index {}: {:?}", index, e))?;

        let name = device
            .name()
            .map_err(|e| anyhow::anyhow!("Failed to get device name: {:?}", e))?;

        let mem_info = device
            .memory_info()
            .map_err(|e| anyhow::anyhow!("Failed to get memory info: {:?}", e))?;

        Ok(GpuInfo {
            index: index as u16,
            name,
            mem_total_mb: (mem_info.total / 1024 / 1024) as u32,
        })
    }

    fn get_gpu_snapshot(&self, index: u32) -> Result<GpuSnapshot> {
        let device = self
            .nvml
            .device_by_index(index)
            .map_err(|e| anyhow::anyhow!("Failed to get device at index {}: {:?}", index, e))?;

        let name = device
            .name()
            .map_err(|e| anyhow::anyhow!("Failed to get device name: {:?}", e))?;

        let mem_info = device
            .memory_info()
            .map_err(|e| anyhow::anyhow!("Failed to get memory info: {:?}", e))?;

        let util = device
            .utilization_rates()
            .map_err(|e| anyhow::anyhow!("Failed to get utilization rates: {:?}", e))?;

        let temp = device
            .temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
            .map_err(|e| anyhow::anyhow!("Failed to get temperature: {:?}", e))?;

        let power_usage = device
            .power_usage()
            .map_err(|e| anyhow::anyhow!("Failed to get power usage: {:?}", e))?;

        let compute_processes = device
            .running_compute_processes()
            .map_err(|e| anyhow::anyhow!("Failed to get running compute processes: {:?}", e))?;
        let graphics_processes = match device.running_graphics_processes() {
            Ok(processes) => processes,
            Err(nvml_wrapper::error::NvmlError::NotSupported) => Vec::new(),
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Failed to get running graphics processes: {:?}",
                    e
                ))
            }
        };
        let processes = merge_nvml_processes(compute_processes, graphics_processes);

        let pids: Vec<u32> = processes.iter().map(|p| p.pid).collect();
        let top_proc = processes.first().map(|p| GpuProc {
            gpu_index: index as u16,
            pid: p.pid,
            user: "unknown".to_string(),
            proc_name: "unknown".to_string(),
            used_mem_mb: match p.used_gpu_memory {
                UsedGpuMemory::Used(bytes) => (bytes / 1024 / 1024) as u32,
                UsedGpuMemory::Unavailable => 0,
            },
            start_time: "unknown".to_string(),
            container: None,
        });

        Ok(GpuSnapshot {
            gpu_index: index as u16,
            name,
            vendor: GpuVendor::Nvidia,
            mem_used_mb: (mem_info.used / 1024 / 1024) as u32,
            mem_total_mb: (mem_info.total / 1024 / 1024) as u32,
            util_pct: util.gpu as f32,
            temp_c: temp as i32,
            power_w: power_usage as f32 / 1000.0,
            ecc_volatile: None,
            pids: pids.len(),
            top_proc,
        })
    }

    fn get_gpu_processes(&self, index: u32) -> Result<Vec<GpuProc>> {
        let device = self
            .nvml
            .device_by_index(index)
            .map_err(|e| anyhow::anyhow!("Failed to get device at index {}: {:?}", index, e))?;

        let compute_processes = device
            .running_compute_processes()
            .map_err(|e| anyhow::anyhow!("Failed to get running compute processes: {:?}", e))?;
        let graphics_processes = match device.running_graphics_processes() {
            Ok(processes) => processes,
            Err(nvml_wrapper::error::NvmlError::NotSupported) => Vec::new(),
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Failed to get running graphics processes: {:?}",
                    e
                ))
            }
        };
        let processes = merge_nvml_processes(compute_processes, graphics_processes);

        let mut gpu_procs = Vec::new();
        for p in processes {
            gpu_procs.push(GpuProc {
                gpu_index: index as u16,
                pid: p.pid,
                user: "unknown".to_string(),
                proc_name: "unknown".to_string(),
                used_mem_mb: match p.used_gpu_memory {
                    UsedGpuMemory::Used(bytes) => (bytes / 1024 / 1024) as u32,
                    UsedGpuMemory::Unavailable => 0,
                },
                start_time: "unknown".to_string(),
                container: None,
            });
        }

        Ok(gpu_procs)
    }

    fn reset_gpu(&self, index: u32) -> Result<()> {
        let _device = self
            .nvml
            .device_by_index(index)
            .map_err(|e| anyhow::anyhow!("Failed to get device at index {}: {:?}", index, e))?;

        // Note: GPU reset is not directly supported by NVML in this version
        // This would require additional system-level operations
        Err(anyhow::anyhow!("GPU reset not supported via NVML"))
    }

    fn is_available() -> bool {
        nvml_wrapper::Nvml::init().is_ok()
    }

    fn get_availability_error() -> String {
        "NVIDIA drivers not installed or NVML not available. Please install NVIDIA drivers."
            .to_string()
    }
}

/// AMD GPU vendor implementation using rocm-smi
pub struct AmdVendor {
    // We'll use rocm-smi command-line tool for now
    // In the future, this could use direct kernel interfaces
}

impl GpuVendorInterface for AmdVendor {
    fn initialize() -> Result<Self> {
        // Check if AMD GPU is available (either via rocm-smi or lspci/sysfs)
        if !Self::is_available() {
            return Err(anyhow::anyhow!("{}", Self::get_availability_error()));
        }
        Ok(Self {})
    }

    fn vendor_type(&self) -> GpuVendor {
        GpuVendor::Amd
    }

    fn device_count(&self) -> Result<u32> {
        // Try rocm-smi first (most accurate)
        let rocm_result = std::process::Command::new("rocm-smi")
            .args(["--showid"])
            .output();

        if let Ok(output) = rocm_result {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);

                // Parse the output to count actual devices
                // For MI300X VF, we need to filter out virtual functions
                // rocm-smi --showid outputs device IDs, one per line
                let device_count = stdout
                    .lines()
                    .filter(|line| {
                        let line = line.trim();
                        !line.is_empty() && !line.starts_with("GPU") && !line.starts_with("#")
                        // Skip comments
                    })
                    .count();

                // For MI300X VF, if we detect multiple devices but they're all virtual functions,
                // we should only count the physical device
                if device_count > 1 {
                    // Try to get more detailed info to distinguish physical vs virtual
                    let detailed_output = std::process::Command::new("rocm-smi")
                        .args(["--showproductname"])
                        .output();

                    if let Ok(detailed) = detailed_output {
                        if detailed.status.success() {
                            let detailed_stdout = String::from_utf8_lossy(&detailed.stdout);
                            // If we see "MI300X VF" in the output, it's likely virtual functions
                            if detailed_stdout.contains("MI300X VF") {
                                return Ok(1); // Only count the physical device
                            }
                        }
                    }
                }

                return Ok(device_count as u32);
            }
        }

        // Fallback: Count AMD GPUs via lspci
        #[cfg(target_os = "linux")]
        {
            let lspci_result = std::process::Command::new("lspci").output();
            if let Ok(output) = lspci_result {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let count = stdout
                        .lines()
                        .filter(|line| {
                            let lower = line.to_lowercase();
                            lower.contains("amd")
                                && (lower.contains("vga")
                                    || lower.contains("display")
                                    || lower.contains("3d"))
                        })
                        .count();
                    if count > 0 {
                        return Ok(count as u32);
                    }
                }
            }
        }

        // If all else fails, return 1 (we detected AMD is available in is_available())
        Ok(1)
    }

    fn get_gpu_info(&self, index: u32) -> Result<GpuInfo> {
        // Try rocm-smi first
        let rocm_result = std::process::Command::new("rocm-smi")
            .args(["--showproductname", "-d", &index.to_string()])
            .output();

        let (name, mem_total_mb) = if let Ok(output) = rocm_result {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let gpu_name = stdout
                    .lines()
                    .find(|line| line.contains("Card series"))
                    .and_then(|line| line.split(':').nth(1))
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|| format!("AMD GPU {}", index));

                // Get memory info
                let mem_output = std::process::Command::new("rocm-smi")
                    .args(["--showmeminfo", "vram", "-d", &index.to_string()])
                    .output();

                let mem = if let Ok(mem_output) = mem_output {
                    if mem_output.status.success() {
                        let mem_stdout = String::from_utf8_lossy(&mem_output.stdout);
                        mem_stdout
                            .lines()
                            .find(|line| line.contains("Total"))
                            .and_then(|line| {
                                line.split_whitespace()
                                    .find(|s| s.ends_with("MB"))
                                    .and_then(|s| s.replace("MB", "").parse::<u32>().ok())
                            })
                            .unwrap_or(8192)
                    } else {
                        8192
                    }
                } else {
                    8192
                };

                (gpu_name, mem)
            } else {
                (format!("AMD GPU {}", index), 8192)
            }
        } else {
            // Fallback: Use lspci to get GPU name
            #[cfg(target_os = "linux")]
            {
                let lspci_result = std::process::Command::new("lspci").output();
                if let Ok(output) = lspci_result {
                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let amd_gpus: Vec<&str> = stdout
                            .lines()
                            .filter(|line| {
                                let lower = line.to_lowercase();
                                lower.contains("amd")
                                    && (lower.contains("vga")
                                        || lower.contains("display")
                                        || lower.contains("3d"))
                            })
                            .collect();

                        if let Some(gpu_line) = amd_gpus.get(index as usize) {
                            // Extract GPU name from lspci output
                            // Format: "00:00.0 VGA compatible controller: AMD/ATI [...]"
                            let gpu_name = gpu_line
                                .split(':')
                                .skip(2)
                                .collect::<Vec<_>>()
                                .join(":")
                                .trim()
                                .to_string();
                            return Ok(GpuInfo {
                                index: index as u16,
                                name: if gpu_name.is_empty() {
                                    format!("AMD GPU {}", index)
                                } else {
                                    gpu_name
                                },
                                mem_total_mb: 4096, // Default for integrated GPUs
                            });
                        }
                    }
                }
            }

            (format!("AMD GPU {}", index), 4096)
        };

        Ok(GpuInfo {
            index: index as u16,
            name,
            mem_total_mb,
        })
    }

    fn get_gpu_snapshot(&self, index: u32) -> Result<GpuSnapshot> {
        // Get basic info first
        let gpu_info = self.get_gpu_info(index)?;

        // Get utilization (don't fail if rocm-smi is unavailable)
        let util_output = std::process::Command::new("rocm-smi")
            .args(["--showuse", "-d", &index.to_string()])
            .output();

        let util_pct = if let Ok(output) = util_output {
            if output.status.success() {
                let util_stdout = String::from_utf8_lossy(&output.stdout);
                util_stdout
                    .lines()
                    .find(|line| line.contains("GPU use") || line.contains("GFX-Uti"))
                    .and_then(|line| {
                        // Try different patterns for ROCm 7.0.0
                        line.split_whitespace()
                            .find(|s| s.ends_with("%"))
                            .and_then(|s| s.replace("%", "").parse::<f32>().ok())
                            .or_else(|| {
                                // Alternative parsing for different output formats
                                line.split_whitespace()
                                    .find(|s| s.chars().all(|c| c.is_numeric() || c == '.'))
                                    .and_then(|s| s.parse::<f32>().ok())
                            })
                    })
                    .unwrap_or(0.0)
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Get temperature (don't fail if rocm-smi is unavailable)
        let temp_output = std::process::Command::new("rocm-smi")
            .args(["--showtemp", "-d", &index.to_string()])
            .output();

        let temp_c = if let Ok(output) = temp_output {
            if output.status.success() {
                let temp_stdout = String::from_utf8_lossy(&output.stdout);
                temp_stdout
                    .lines()
                    .find(|line| line.contains("Temperature") || line.contains("Temp"))
                    .and_then(|line| {
                        // Try different patterns for ROCm 7.0.0
                        line.split_whitespace()
                            .find(|s| s.ends_with("C") || s.ends_with("°C"))
                            .and_then(|s| s.replace("C", "").replace("°", "").parse::<i32>().ok())
                            .or_else(|| {
                                // Alternative parsing for different output formats
                                line.split_whitespace()
                                    .find(|s| s.chars().all(|c| c.is_numeric()))
                                    .and_then(|s| s.parse::<i32>().ok())
                            })
                    })
                    .unwrap_or(0)
            } else {
                0
            }
        } else {
            0
        };

        // Get power usage (don't fail if rocm-smi is unavailable)
        let power_output = std::process::Command::new("rocm-smi")
            .args(["--showpower", "-d", &index.to_string()])
            .output();

        let power_w = if let Ok(output) = power_output {
            if output.status.success() {
                let power_stdout = String::from_utf8_lossy(&output.stdout);
                power_stdout
                    .lines()
                    .find(|line| {
                        line.contains("Average Graphics Package Power")
                            || line.contains("Power-Usage")
                    })
                    .and_then(|line| {
                        // Try different patterns for ROCm 7.0.0
                        line.split_whitespace()
                            .find(|s| s.ends_with("W"))
                            .and_then(|s| s.replace("W", "").parse::<f32>().ok())
                            .or_else(|| {
                                // Alternative parsing for "134/750 W" format
                                line.split_whitespace()
                                    .find(|s| s.contains("/") && s.ends_with("W"))
                                    .and_then(|s| {
                                        s.split("/")
                                            .next()
                                            .and_then(|part| part.parse::<f32>().ok())
                                    })
                            })
                    })
                    .unwrap_or(0.0)
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Get memory usage (don't fail if rocm-smi is unavailable)
        let mem_output = std::process::Command::new("rocm-smi")
            .args(["--showmemuse", "-d", &index.to_string()])
            .output();

        let mem_used_mb = if let Ok(output) = mem_output {
            if output.status.success() {
                let mem_stdout = String::from_utf8_lossy(&output.stdout);
                mem_stdout
                    .lines()
                    .find(|line| line.contains("GPU memory use") || line.contains("Mem-Usage"))
                    .and_then(|line| {
                        // Try different patterns for ROCm 7.0.0
                        line.split_whitespace()
                            .find(|s| s.ends_with("MB"))
                            .and_then(|s| s.replace("MB", "").parse::<u32>().ok())
                            .or_else(|| {
                                // Alternative parsing for "285/196288 MB" format
                                line.split_whitespace()
                                    .find(|s| s.contains("/") && s.ends_with("MB"))
                                    .and_then(|s| {
                                        s.split("/")
                                            .next()
                                            .and_then(|part| part.parse::<u32>().ok())
                                    })
                            })
                    })
                    .unwrap_or(0)
            } else {
                0
            }
        } else {
            0
        };

        // For now, we'll return empty process info for AMD
        // This could be enhanced with additional rocm-smi queries
        Ok(GpuSnapshot {
            gpu_index: index as u16,
            name: gpu_info.name,
            vendor: GpuVendor::Amd,
            mem_used_mb,
            mem_total_mb: gpu_info.mem_total_mb,
            util_pct,
            temp_c,
            power_w,
            ecc_volatile: None,
            pids: 0, // TODO: Implement process detection for AMD
            top_proc: None,
        })
    }

    fn get_gpu_processes(&self, _index: u32) -> Result<Vec<GpuProc>> {
        // TODO: Implement process detection for AMD GPUs
        // This would require parsing rocm-smi output or using other tools
        Ok(Vec::new())
    }

    fn reset_gpu(&self, index: u32) -> Result<()> {
        let output = std::process::Command::new("rocm-smi")
            .args(["--reset", "-d", &index.to_string()])
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to run rocm-smi: {}", e))?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "rocm-smi reset failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }

    fn is_available() -> bool {
        // First check for rocm-smi (ROCm drivers)
        if std::process::Command::new("rocm-smi")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
        {
            return true;
        }

        // Fallback: Check for AMD GPUs via lspci (works for integrated/consumer GPUs)
        #[cfg(target_os = "linux")]
        {
            let lspci_check = std::process::Command::new("lspci")
                .output()
                .map(|output| {
                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        stdout.to_lowercase().contains("amd")
                            && (stdout.to_lowercase().contains("vga")
                                || stdout.to_lowercase().contains("display")
                                || stdout.to_lowercase().contains("3d"))
                    } else {
                        false
                    }
                })
                .unwrap_or(false);

            if lspci_check {
                return true;
            }

            // Also check sysfs for amdgpu devices
            std::path::Path::new("/sys/class/drm/card0/device/vendor").exists()
                && std::fs::read_to_string("/sys/class/drm/card0/device/vendor")
                    .ok()
                    .map(|v| {
                        let vendor_id = v.trim();
                        // AMD vendor IDs: 0x1002, 0x1022
                        vendor_id == "0x1002" || vendor_id == "0x1022"
                    })
                    .unwrap_or(false)
        }

        #[cfg(not(target_os = "linux"))]
        false
    }

    fn get_availability_error() -> String {
        "AMD GPU not detected. For full support, please install ROCm drivers (rocm-smi)."
            .to_string()
    }
}

/// Intel GPU vendor implementation using intel_gpu_top and intel_gpu_time
pub struct IntelVendor {
    // Intel GPU management via command-line tools
    // Future: Could integrate with Intel oneAPI Level Zero
}

impl GpuVendorInterface for IntelVendor {
    fn initialize() -> Result<Self> {
        // Check if Intel GPU tools are available
        if !Self::is_available() {
            return Err(anyhow::anyhow!("{}", Self::get_availability_error()));
        }
        Ok(Self {})
    }

    fn vendor_type(&self) -> GpuVendor {
        GpuVendor::Intel
    }

    fn device_count(&self) -> Result<u32> {
        // Try to get GPU count from intel_gpu_top
        let output = std::process::Command::new("intel_gpu_top")
            .args(["-l", "1"])
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to run intel_gpu_top: {}", e))?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "intel_gpu_top failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Count GPU entries in the output
        let gpu_count = stdout
            .lines()
            .filter(|line| line.contains("GPU") || line.contains("Render"))
            .count() as u32;

        Ok(if gpu_count > 0 { gpu_count } else { 1 }) // At least one Intel GPU
    }

    fn get_gpu_info(&self, index: u32) -> Result<GpuInfo> {
        // Get GPU name from intel_gpu_top
        let output = std::process::Command::new("intel_gpu_top")
            .args(["-l", "1"])
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to run intel_gpu_top: {}", e))?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "intel_gpu_top failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let name = stdout
            .lines()
            .find(|line| line.contains("GPU") || line.contains("Render"))
            .map(|line| {
                // Extract GPU name from the line
                if line.contains("Intel") {
                    line.to_string()
                } else {
                    format!("Intel GPU {}", index)
                }
            })
            .unwrap_or_else(|| format!("Intel GPU {}", index));

        // Intel GPUs typically have varying memory sizes
        // We'll use a reasonable default and could enhance this later
        let mem_total_mb = match index {
            0 => 4096, // 4GB for integrated graphics
            _ => 8192, // 8GB for discrete GPUs
        };

        Ok(GpuInfo {
            index: index as u16,
            name,
            mem_total_mb,
        })
    }

    fn get_gpu_snapshot(&self, index: u32) -> Result<GpuSnapshot> {
        // Get basic info first
        let gpu_info = self.get_gpu_info(index)?;

        // Get utilization from intel_gpu_top
        let output = std::process::Command::new("intel_gpu_top")
            .args(["-l", "1"])
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to run intel_gpu_top: {}", e))?;

        let (util_pct, mem_used_mb) = if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let util = stdout
                .lines()
                .find(|line| line.contains("Render/3D"))
                .and_then(|line| {
                    line.split_whitespace()
                        .find(|s| s.ends_with('%'))
                        .and_then(|s| s.replace('%', "").parse::<f32>().ok())
                })
                .unwrap_or(0.0);

            // Estimate memory usage (Intel tools don't provide exact memory info)
            let mem_used = (util / 100.0 * gpu_info.mem_total_mb as f32) as u32;
            (util, mem_used)
        } else {
            (0.0, 0)
        };

        // Intel GPUs don't typically provide temperature/power info via command line
        // We'll use reasonable defaults
        Ok(GpuSnapshot {
            gpu_index: index as u16,
            name: gpu_info.name,
            vendor: GpuVendor::Intel,
            mem_used_mb,
            mem_total_mb: gpu_info.mem_total_mb,
            util_pct,
            temp_c: 0,    // Not available via intel_gpu_top
            power_w: 0.0, // Not available via intel_gpu_top
            ecc_volatile: None,
            pids: 0, // Process detection would require additional parsing
            top_proc: None,
        })
    }

    fn get_gpu_processes(&self, _index: u32) -> Result<Vec<GpuProc>> {
        // Intel GPU process detection is more complex
        // For now, we'll return an empty vector
        // Future enhancement: Parse /proc/driver/i915/gt/gt*/exec_queues
        Ok(Vec::new())
    }

    fn reset_gpu(&self, _index: u32) -> Result<()> {
        // Intel GPU reset is not directly supported via command line
        // This would require kernel-level operations or driver-specific tools
        Err(anyhow::anyhow!(
            "Intel GPU reset not supported via command line tools"
        ))
    }

    fn is_available() -> bool {
        // Check if intel_gpu_top is available
        std::process::Command::new("intel_gpu_top")
            .arg("-h")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn get_availability_error() -> String {
        "Intel GPU tools not available. Please install intel-gpu-tools package.".to_string()
    }
}

/// Apple Silicon GPU vendor implementation using system_profiler and IOKit
#[cfg(target_os = "macos")]
pub struct AppleVendor {
    // Apple Silicon GPU management via system APIs
    gpu_info: Option<GpuInfo>,
}

#[cfg(target_os = "macos")]
impl GpuVendorInterface for AppleVendor {
    fn initialize() -> Result<Self> {
        // Check if we're on Apple Silicon
        if !Self::is_available() {
            return Err(anyhow::anyhow!("{}", Self::get_availability_error()));
        }

        // Get initial GPU info
        let gpu_info = Self::get_system_gpu_info()?;

        Ok(Self {
            gpu_info: Some(gpu_info),
        })
    }

    fn vendor_type(&self) -> GpuVendor {
        GpuVendor::Apple
    }

    fn device_count(&self) -> Result<u32> {
        // Apple Silicon typically has one unified GPU
        Ok(1)
    }

    fn get_gpu_info(&self, index: u32) -> Result<GpuInfo> {
        if index > 0 {
            return Err(anyhow::anyhow!("Apple Silicon only supports GPU index 0"));
        }

        if let Some(ref info) = self.gpu_info {
            Ok(info.clone())
        } else {
            Self::get_system_gpu_info()
        }
    }

    fn get_gpu_snapshot(&self, index: u32) -> Result<GpuSnapshot> {
        if index > 0 {
            return Err(anyhow::anyhow!("Apple Silicon only supports GPU index 0"));
        }

        let gpu_info = self.get_gpu_info(index)?;

        // Get memory usage from vm_stat
        let mem_used_mb = Self::get_gpu_memory_usage()?;

        // Get processes using Metal/GPU
        let processes = self.get_gpu_processes(index)?;
        let pids = processes.len();
        let top_proc = processes.into_iter().max_by_key(|p| p.used_mem_mb);

        Ok(GpuSnapshot {
            gpu_index: index as u16,
            name: gpu_info.name,
            vendor: GpuVendor::Apple,
            mem_used_mb,
            mem_total_mb: gpu_info.mem_total_mb,
            util_pct: 0.0,      // Not easily available on Apple Silicon
            temp_c: 0,          // Not available via system APIs
            power_w: 0.0,       // Not available via system APIs
            ecc_volatile: None, // Not applicable to Apple Silicon
            pids,
            top_proc,
        })
    }

    fn get_gpu_processes(&self, _index: u32) -> Result<Vec<GpuProc>> {
        // Find processes that might be using Metal/GPU
        let output = std::process::Command::new("ps")
            .args(["-axo", "pid,user,comm,%mem"])
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to run ps: {}", e))?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut processes = Vec::new();

        for line in stdout.lines().skip(1) {
            // Skip header
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                if let (Ok(pid), user, comm, mem_str) =
                    (parts[0].parse::<u32>(), parts[1], parts[2], parts[3])
                {
                    // Check if this process might be using GPU
                    if Self::is_gpu_process(comm) {
                        let mem_mb = Self::parse_memory_usage(mem_str, user)?;

                        processes.push(GpuProc {
                            gpu_index: 0,
                            pid,
                            user: user.to_string(),
                            proc_name: comm.to_string(),
                            used_mem_mb: mem_mb,
                            start_time: "unknown".to_string(), // Would need more complex parsing
                            container: None,
                        });
                    }
                }
            }
        }

        Ok(processes)
    }

    fn reset_gpu(&self, _index: u32) -> Result<()> {
        // Apple Silicon GPU reset is not directly supported via system APIs
        // This would require kernel-level operations
        Err(anyhow::anyhow!(
            "Apple Silicon GPU reset not supported via system APIs"
        ))
    }

    fn is_available() -> bool {
        // Check if we're on macOS and have Apple Silicon
        if !cfg!(target_os = "macos") {
            return false;
        }

        // Check for Apple Silicon by looking for Apple chip in system profiler
        let output = std::process::Command::new("system_profiler")
            .args(["SPHardwareDataType"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                return stdout.contains("Apple")
                    && (stdout.contains("M1")
                        || stdout.contains("M2")
                        || stdout.contains("M3")
                        || stdout.contains("M4"));
            }
        }

        false
    }

    fn get_availability_error() -> String {
        "Apple Silicon GPU not available. This feature requires macOS with Apple Silicon (M1/M2/M3/M4).".to_string()
    }
}

#[cfg(target_os = "macos")]
impl AppleVendor {
    /// Get GPU information from system_profiler
    fn get_system_gpu_info() -> Result<GpuInfo> {
        let output = std::process::Command::new("system_profiler")
            .args(["SPDisplaysDataType"])
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to run system_profiler: {}", e))?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("system_profiler failed"));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse GPU name and memory
        let mut name = "Apple Silicon GPU".to_string();
        let mut mem_total_mb = 8192; // Default fallback

        for line in stdout.lines() {
            if line.contains("Chipset Model:") {
                if let Some(chipset) = line.split("Chipset Model:").nth(1) {
                    name = chipset.trim().to_string();
                }
            }
        }

        // Get total memory from system
        let mem_output = std::process::Command::new("system_profiler")
            .args(["SPHardwareDataType"])
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to get memory info: {}", e))?;

        if mem_output.status.success() {
            let mem_stdout = String::from_utf8_lossy(&mem_output.stdout);
            for line in mem_stdout.lines() {
                if line.contains("Memory:") {
                    if let Some(mem_str) = line.split("Memory:").nth(1) {
                        let mem_str = mem_str.trim();
                        if let Some(gb_str) = mem_str.split_whitespace().next() {
                            if let Ok(gb) = gb_str.parse::<u32>() {
                                mem_total_mb = gb * 1024; // Convert GB to MB
                                break;
                            }
                        }
                    }
                }
            }
        }

        Ok(GpuInfo {
            index: 0,
            name,
            mem_total_mb,
        })
    }

    /// Get GPU memory usage from vm_stat
    fn get_gpu_memory_usage() -> Result<u32> {
        let output = std::process::Command::new("vm_stat")
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to run vm_stat: {}", e))?;

        if !output.status.success() {
            return Ok(0);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let page_size = 16384; // Apple Silicon uses 16KB pages

        // Parse active memory (which includes GPU memory on unified architecture)
        for line in stdout.lines() {
            if line.contains("Pages active:") {
                if let Some(pages_str) = line.split("Pages active:").nth(1) {
                    if let Some(pages) = pages_str.trim().split('.').next() {
                        if let Ok(pages) = pages.parse::<u64>() {
                            // Estimate GPU memory usage as a portion of active memory
                            // This is a rough approximation since Apple Silicon uses unified memory
                            let total_active_mb = (pages * page_size) / (1024 * 1024);
                            return Ok((total_active_mb / 4) as u32); // Assume 25% is GPU-related
                        }
                    }
                }
            }
        }

        Ok(0)
    }

    /// Check if a process is likely using GPU
    fn is_gpu_process(comm: &str) -> bool {
        let gpu_keywords = [
            "Metal",
            "OpenGL",
            "CoreAnimation",
            "Quartz",
            "WindowServer",
            "python",
            "tensorflow",
            "pytorch",
            "jupyter",
            "matplotlib",
            "ffmpeg",
            "blender",
            "unity",
            "unreal",
            "xcode",
            "simulator",
        ];

        gpu_keywords
            .iter()
            .any(|&keyword| comm.to_lowercase().contains(keyword))
    }

    /// Parse memory usage from ps output
    fn parse_memory_usage(mem_str: &str, _user: &str) -> Result<u32> {
        // Parse percentage and convert to MB (rough approximation)
        if let Ok(percent) = mem_str.parse::<f32>() {
            // Rough conversion: assume system has 32GB, so 1% = ~320MB
            Ok((percent * 320.0) as u32)
        } else {
            Ok(0)
        }
    }
}

/// Multi-vendor GPU manager
pub struct GpuManager {
    vendors: Vec<Box<dyn GpuVendorInterface + Send + Sync>>,
}

#[allow(dead_code)]
impl GpuManager {
    /// Initialize the GPU manager with all available vendors
    pub fn initialize() -> Result<Self> {
        let mut vendors: Vec<Box<dyn GpuVendorInterface + Send + Sync>> = Vec::new();

        // Try to initialize NVIDIA
        if NvidiaVendor::is_available() {
            match NvidiaVendor::initialize() {
                Ok(nvidia) => {
                    tracing::info!("NVIDIA GPU support initialized");
                    vendors.push(Box::new(nvidia));
                }
                Err(e) => {
                    tracing::warn!("Failed to initialize NVIDIA support: {}", e);
                }
            }
        }

        // Try to initialize AMD
        if AmdVendor::is_available() {
            match AmdVendor::initialize() {
                Ok(amd) => {
                    tracing::info!("AMD GPU support initialized");
                    vendors.push(Box::new(amd));
                }
                Err(e) => {
                    tracing::warn!("Failed to initialize AMD support: {}", e);
                }
            }
        }

        // Try to initialize Intel
        if IntelVendor::is_available() {
            match IntelVendor::initialize() {
                Ok(intel) => {
                    tracing::info!("Intel GPU support initialized");
                    vendors.push(Box::new(intel));
                }
                Err(e) => {
                    tracing::warn!("Failed to initialize Intel support: {}", e);
                }
            }
        }

        // Try to initialize Apple Silicon (macOS only)
        #[cfg(target_os = "macos")]
        if AppleVendor::is_available() {
            match AppleVendor::initialize() {
                Ok(apple) => {
                    tracing::info!("Apple Silicon GPU support initialized");
                    vendors.push(Box::new(apple));
                }
                Err(e) => {
                    tracing::warn!("Failed to initialize Apple Silicon support: {}", e);
                }
            }
        }

        if vendors.is_empty() {
            return Err(anyhow::anyhow!(
                "No GPU vendors available. Please install NVIDIA, AMD, Intel, or Apple Silicon GPU drivers."
            ));
        }

        Ok(Self { vendors })
    }

    /// Get total device count across all vendors
    pub fn total_device_count(&self) -> Result<u32> {
        let mut total = 0;
        for vendor in &self.vendors {
            total += vendor.device_count()?;
        }
        Ok(total)
    }

    /// Get all GPU snapshots from all vendors
    pub fn get_all_snapshots(&self) -> Result<Vec<GpuSnapshot>> {
        let mut snapshots = Vec::new();
        for vendor in &self.vendors {
            let count = vendor.device_count()?;
            for i in 0..count {
                match vendor.get_gpu_snapshot(i) {
                    Ok(snapshot) => snapshots.push(snapshot),
                    Err(e) => {
                        tracing::warn!("Failed to get snapshot for GPU {}: {}", i, e);
                    }
                }
            }
        }
        Ok(snapshots)
    }

    /// Get all processes from all vendors
    pub fn get_all_processes(&self) -> Result<Vec<GpuProc>> {
        let mut processes = Vec::new();
        for vendor in &self.vendors {
            let count = vendor.device_count()?;
            for i in 0..count {
                match vendor.get_gpu_processes(i) {
                    Ok(mut vendor_procs) => processes.append(&mut vendor_procs),
                    Err(e) => {
                        tracing::warn!("Failed to get processes for GPU {}: {}", i, e);
                    }
                }
            }
        }
        Ok(processes)
    }

    /// Reset a specific GPU by global index
    pub fn reset_gpu(&self, global_index: u32) -> Result<()> {
        let mut current_index = 0;
        for vendor in &self.vendors {
            let count = vendor.device_count()?;
            if global_index < current_index + count {
                let local_index = global_index - current_index;
                return vendor.reset_gpu(local_index);
            }
            current_index += count;
        }
        Err(anyhow::anyhow!("GPU index {} not found", global_index))
    }

    /// Get available vendors
    pub fn get_vendors(&self) -> Vec<GpuVendor> {
        self.vendors.iter().map(|v| v.vendor_type()).collect()
    }
}
