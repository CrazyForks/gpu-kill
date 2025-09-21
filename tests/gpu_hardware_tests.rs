use gpukill::vendor::{GpuManager, GpuVendorInterface, NvidiaVendor, AmdVendor, IntelVendor};
// GPU hardware tests - imports are used in specific test modules
use std::process::Command;

/// Integration tests that require actual GPU hardware
/// These tests will be skipped if the required hardware is not available

#[cfg(test)]
mod nvidia_hardware_tests {
    use super::*;

    #[test]
    fn test_nvidia_gpu_detection() {
        if !NvidiaVendor::is_available() {
            println!("Skipping NVIDIA test - no NVIDIA GPU available");
            return;
        }

        let vendor = NvidiaVendor::initialize().expect("Failed to initialize NVIDIA vendor");
        let count = vendor.device_count().expect("Failed to get device count");
        
        assert!(count > 0, "No NVIDIA GPUs detected");
        println!("Detected {} NVIDIA GPU(s)", count);
    }

    #[test]
    fn test_nvidia_gpu_info() {
        if !NvidiaVendor::is_available() {
            println!("Skipping NVIDIA test - no NVIDIA GPU available");
            return;
        }

        let vendor = NvidiaVendor::initialize().expect("Failed to initialize NVIDIA vendor");
        let count = vendor.device_count().expect("Failed to get device count");
        
        for i in 0..count {
            let info = vendor.get_gpu_info(i).expect(&format!("Failed to get info for GPU {}", i));
            assert!(!info.name.is_empty(), "GPU name should not be empty");
            assert!(info.mem_total_mb > 0, "GPU memory should be greater than 0");
            println!("GPU {}: {} ({} MB)", i, info.name, info.mem_total_mb);
        }
    }

    #[test]
    fn test_nvidia_gpu_snapshot() {
        if !NvidiaVendor::is_available() {
            println!("Skipping NVIDIA test - no NVIDIA GPU available");
            return;
        }

        let vendor = NvidiaVendor::initialize().expect("Failed to initialize NVIDIA vendor");
        let count = vendor.device_count().expect("Failed to get device count");
        
        for i in 0..count {
            let snapshot = vendor.get_gpu_snapshot(i).expect(&format!("Failed to get snapshot for GPU {}", i));
            assert_eq!(snapshot.gpu_index, i as u16);
            assert!(!snapshot.name.is_empty(), "GPU name should not be empty");
            assert!(snapshot.mem_total_mb > 0, "GPU memory should be greater than 0");
            assert!(snapshot.mem_used_mb <= snapshot.mem_total_mb, "Used memory should not exceed total");
            assert!(snapshot.util_pct >= 0.0 && snapshot.util_pct <= 100.0, "Utilization should be between 0-100%");
            assert!(snapshot.temp_c >= 0, "Temperature should be non-negative");
            assert!(snapshot.power_w >= 0.0, "Power should be non-negative");
            println!("GPU {} snapshot: {}% util, {}°C, {}W", i, snapshot.util_pct, snapshot.temp_c, snapshot.power_w);
        }
    }

    #[test]
    fn test_nvidia_gpu_processes() {
        if !NvidiaVendor::is_available() {
            println!("Skipping NVIDIA test - no NVIDIA GPU available");
            return;
        }

        let vendor = NvidiaVendor::initialize().expect("Failed to initialize NVIDIA vendor");
        let count = vendor.device_count().expect("Failed to get device count");
        
        for i in 0..count {
            let processes = vendor.get_gpu_processes(i).expect(&format!("Failed to get processes for GPU {}", i));
            println!("GPU {} has {} processes", i, processes.len());
            
            for proc in &processes {
                assert!(proc.pid > 0, "Process ID should be positive");
                assert!(!proc.user.is_empty(), "User should not be empty");
                assert!(!proc.proc_name.is_empty(), "Process name should not be empty");
                assert!(proc.used_mem_mb > 0, "Used memory should be positive");
            }
        }
    }
}

#[cfg(test)]
mod amd_hardware_tests {
    use super::*;

    #[test]
    fn test_amd_gpu_detection() {
        if !AmdVendor::is_available() {
            println!("Skipping AMD test - no AMD GPU or ROCm available");
            return;
        }

        let vendor = AmdVendor::initialize().expect("Failed to initialize AMD vendor");
        let count = vendor.device_count().expect("Failed to get device count");
        
        assert!(count > 0, "No AMD GPUs detected");
        println!("Detected {} AMD GPU(s)", count);
    }

    #[test]
    fn test_amd_gpu_info() {
        if !AmdVendor::is_available() {
            println!("Skipping AMD test - no AMD GPU or ROCm available");
            return;
        }

        let vendor = AmdVendor::initialize().expect("Failed to initialize AMD vendor");
        let count = vendor.device_count().expect("Failed to get device count");
        
        for i in 0..count {
            let info = vendor.get_gpu_info(i).expect(&format!("Failed to get info for GPU {}", i));
            assert!(!info.name.is_empty(), "GPU name should not be empty");
            assert!(info.mem_total_mb > 0, "GPU memory should be greater than 0");
            println!("GPU {}: {} ({} MB)", i, info.name, info.mem_total_mb);
        }
    }

    #[test]
    fn test_amd_gpu_snapshot() {
        if !AmdVendor::is_available() {
            println!("Skipping AMD test - no AMD GPU or ROCm available");
            return;
        }

        let vendor = AmdVendor::initialize().expect("Failed to initialize AMD vendor");
        let count = vendor.device_count().expect("Failed to get device count");
        
        for i in 0..count {
            let snapshot = vendor.get_gpu_snapshot(i).expect(&format!("Failed to get snapshot for GPU {}", i));
            assert_eq!(snapshot.gpu_index, i as u16);
            assert!(!snapshot.name.is_empty(), "GPU name should not be empty");
            assert!(snapshot.mem_total_mb > 0, "GPU memory should be greater than 0");
            assert!(snapshot.mem_used_mb <= snapshot.mem_total_mb, "Used memory should not exceed total");
            assert!(snapshot.util_pct >= 0.0 && snapshot.util_pct <= 100.0, "Utilization should be between 0-100%");
            assert!(snapshot.temp_c >= 0, "Temperature should be non-negative");
            assert!(snapshot.power_w >= 0.0, "Power should be non-negative");
            println!("GPU {} snapshot: {}% util, {}°C, {}W", i, snapshot.util_pct, snapshot.temp_c, snapshot.power_w);
        }
    }

    #[test]
    fn test_rocm_smi_availability() {
        let output = Command::new("rocm-smi")
            .arg("--version")
            .output();
            
        match output {
            Ok(result) => {
                if result.status.success() {
                    let version = String::from_utf8_lossy(&result.stdout);
                    println!("ROCm version: {}", version);
                } else {
                    println!("ROCm not available or not working");
                }
            }
            Err(_) => {
                println!("rocm-smi command not found");
            }
        }
    }
}

#[cfg(test)]
mod intel_hardware_tests {
    use super::*;

    #[test]
    fn test_intel_gpu_detection() {
        if !IntelVendor::is_available() {
            println!("Skipping Intel test - no Intel GPU or intel-gpu-tools available");
            return;
        }

        let vendor = IntelVendor::initialize().expect("Failed to initialize Intel vendor");
        let count = vendor.device_count().expect("Failed to get device count");
        
        assert!(count > 0, "No Intel GPUs detected");
        println!("Detected {} Intel GPU(s)", count);
    }

    #[test]
    fn test_intel_gpu_info() {
        if !IntelVendor::is_available() {
            println!("Skipping Intel test - no Intel GPU or intel-gpu-tools available");
            return;
        }

        let vendor = IntelVendor::initialize().expect("Failed to initialize Intel vendor");
        let count = vendor.device_count().expect("Failed to get device count");
        
        for i in 0..count {
            let info = vendor.get_gpu_info(i).expect(&format!("Failed to get info for GPU {}", i));
            assert!(!info.name.is_empty(), "GPU name should not be empty");
            assert!(info.mem_total_mb > 0, "GPU memory should be greater than 0");
            println!("GPU {}: {} ({} MB)", i, info.name, info.mem_total_mb);
        }
    }

    #[test]
    fn test_intel_gpu_tools_availability() {
        let output = Command::new("intel_gpu_top")
            .arg("--help")
            .output();
            
        match output {
            Ok(result) => {
                if result.status.success() {
                    println!("intel_gpu_top is available");
                } else {
                    println!("intel_gpu_top not working");
                }
            }
            Err(_) => {
                println!("intel_gpu_top command not found");
            }
        }
    }
}

#[cfg(test)]
mod multi_vendor_tests {
    use super::*;

    #[test]
    fn test_gpu_manager_initialization() {
        let manager = GpuManager::initialize();
        
        match manager {
            Ok(manager) => {
                let count = manager.total_device_count().expect("Failed to get total device count");
                println!("GPU Manager initialized with {} total devices", count);
                assert!(count > 0, "Should have at least one GPU");
            }
            Err(e) => {
                println!("GPU Manager initialization failed: {}", e);
                // This is acceptable if no GPUs are available
            }
        }
    }

    #[test]
    fn test_gpu_manager_snapshots() {
        let manager = match GpuManager::initialize() {
            Ok(manager) => manager,
            Err(_) => {
                println!("Skipping test - no GPUs available");
                return;
            }
        };

        let snapshots = manager.get_all_snapshots().expect("Failed to get snapshots");
        assert!(!snapshots.is_empty(), "Should have at least one GPU snapshot");
        
        for snapshot in &snapshots {
            assert!(!snapshot.name.is_empty(), "GPU name should not be empty");
            assert!(snapshot.mem_total_mb > 0, "GPU memory should be greater than 0");
            println!("GPU {}: {} ({} MB total, {} MB used)", 
                snapshot.gpu_index, snapshot.name, snapshot.mem_total_mb, snapshot.mem_used_mb);
        }
    }

    #[test]
    fn test_gpu_manager_processes() {
        let manager = match GpuManager::initialize() {
            Ok(manager) => manager,
            Err(_) => {
                println!("Skipping test - no GPUs available");
                return;
            }
        };

        let processes = manager.get_all_processes().expect("Failed to get processes");
        println!("Found {} total GPU processes", processes.len());
        
        for proc in &processes {
            assert!(proc.pid > 0, "Process ID should be positive");
            assert!(!proc.user.is_empty(), "User should not be empty");
            assert!(!proc.proc_name.is_empty(), "Process name should not be empty");
        }
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_gpu_listing_performance() {
        let manager = match GpuManager::initialize() {
            Ok(manager) => manager,
            Err(_) => {
                println!("Skipping performance test - no GPUs available");
                return;
            }
        };

        // Test basic listing performance
        let start = Instant::now();
        let snapshots = manager.get_all_snapshots().expect("Failed to get snapshots");
        let duration = start.elapsed();
        
        println!("GPU listing took {:?} for {} GPUs", duration, snapshots.len());
        assert!(duration.as_millis() < 5000, "GPU listing should complete within 5 seconds");
    }

    #[test]
    fn test_gpu_process_enumeration_performance() {
        let manager = match GpuManager::initialize() {
            Ok(manager) => manager,
            Err(_) => {
                println!("Skipping performance test - no GPUs available");
                return;
            }
        };

        // Test process enumeration performance
        let start = Instant::now();
        let processes = manager.get_all_processes().expect("Failed to get processes");
        let duration = start.elapsed();
        
        println!("Process enumeration took {:?} for {} processes", duration, processes.len());
        assert!(duration.as_millis() < 10000, "Process enumeration should complete within 10 seconds");
    }

    #[test]
    fn test_repeated_gpu_queries() {
        let manager = match GpuManager::initialize() {
            Ok(manager) => manager,
            Err(_) => {
                println!("Skipping performance test - no GPUs available");
                return;
            }
        };

        // Test repeated queries for stability
        for i in 0..10 {
            let start = Instant::now();
            let snapshots = manager.get_all_snapshots().expect("Failed to get snapshots");
            let duration = start.elapsed();
            
            println!("Query {}: {:?} for {} GPUs", i + 1, duration, snapshots.len());
            assert!(!snapshots.is_empty(), "Should always return snapshots");
        }
    }
}

#[cfg(test)]
mod stress_tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_concurrent_gpu_access() {
        let manager = match GpuManager::initialize() {
            Ok(manager) => manager,
            Err(_) => {
                println!("Skipping stress test - no GPUs available");
                return;
            }
        };

        // Test concurrent access from multiple threads
        let handles: Vec<_> = (0..5).map(|i| {
            thread::spawn(move || {
                // Create a new manager instance for each thread
                let thread_manager = match GpuManager::initialize() {
                    Ok(manager) => manager,
                    Err(_) => {
                        println!("Thread {}: Failed to initialize GPU manager", i);
                        return;
                    }
                };
                
                for j in 0..10 {
                    let snapshots = thread_manager.get_all_snapshots().expect("Failed to get snapshots");
                    println!("Thread {} iteration {}: {} GPUs", i, j, snapshots.len());
                    thread::sleep(Duration::from_millis(100));
                }
            })
        }).collect();

        // Wait for all threads to complete
        for handle in handles {
            handle.join().expect("Thread panicked");
        }
    }

    #[test]
    fn test_long_running_gpu_monitoring() {
        let manager = match GpuManager::initialize() {
            Ok(manager) => manager,
            Err(_) => {
                println!("Skipping stress test - no GPUs available");
                return;
            }
        };

        // Test long-running monitoring
        let start = std::time::Instant::now();
        let mut iteration = 0;
        
        while start.elapsed().as_secs() < 30 { // Run for 30 seconds
            let snapshots = manager.get_all_snapshots().expect("Failed to get snapshots");
            println!("Long-running test iteration {}: {} GPUs", iteration, snapshots.len());
            
            iteration += 1;
            thread::sleep(Duration::from_secs(2));
        }
        
        println!("Completed {} iterations in 30 seconds", iteration);
        assert!(iteration > 10, "Should complete at least 10 iterations");
    }
}
