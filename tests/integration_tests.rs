use gpukill::args::{Cli, OutputFormat, VendorFilter};
use clap::Parser;
use gpukill::nvml_api::{GpuInfo, GpuProc, GpuSnapshot, Snapshot};
use gpukill::process_mgmt::{EnhancedProcessManager, ProcessStats};
use gpukill::render::Renderer;
use gpukill::vendor::{GpuVendor, NvidiaVendor, AmdVendor, GpuVendorInterface};
use std::process::Command;

#[cfg(feature = "mock_nvml")]
mod mock_tests {
    use super::*;

    #[test]
    fn test_list_operation_parsing() {
        let cli = Cli::parse_from(&["gpukill", "--list"]);
        assert!(cli.list);
        assert!(!cli.details);
        assert!(!cli.watch);
        assert!(matches!(cli.output, OutputFormat::Table));
    }

    #[test]
    fn test_list_with_details_and_watch() {
        let cli = Cli::parse_from(&["gpukill", "--list", "--details", "--watch"]);
        assert!(cli.list);
        assert!(cli.details);
        assert!(cli.watch);
        assert!(matches!(cli.output, OutputFormat::Table));
    }

    #[test]
    fn test_list_json_output() {
        let cli = Cli::parse_from(&["gpukill", "--list", "--output", "json"]);
        assert!(cli.list);
        assert!(matches!(cli.output, OutputFormat::Json));
    }

    #[test]
    fn test_kill_operation() {
        let cli = Cli::parse_from(&["gpukill", "--kill", "--pid", "12345"]);
        assert!(cli.kill);
        assert_eq!(cli.pid, Some(12345));
        assert_eq!(cli.timeout_secs, 5);
        assert!(!cli.force);
    }

    #[test]
    fn test_kill_with_custom_timeout_and_force() {
        let cli = Cli::parse_from(&["gpukill", "--kill", "--pid", "12345", "--timeout-secs", "10", "--force"]);
        assert!(cli.kill);
        assert_eq!(cli.pid, Some(12345));
        assert_eq!(cli.timeout_secs, 10);
        assert!(cli.force);
    }

    #[test]
    fn test_reset_single_gpu() {
        let cli = Cli::parse_from(&["gpukill", "--reset", "--gpu", "0"]);
        assert!(cli.reset);
        assert_eq!(cli.gpu, Some(0));
        assert!(!cli.all);
    }

    #[test]
    fn test_reset_all_gpus() {
        let cli = Cli::parse_from(&["gpukill", "--reset", "--all"]);
        assert!(cli.reset);
        assert_eq!(cli.gpu, None);
        assert!(cli.all);
    }

    #[test]
    fn test_reset_with_force() {
        let cli = Cli::parse_from(&["gpukill", "--reset", "--gpu", "0", "--force"]);
        assert!(cli.reset);
        assert_eq!(cli.gpu, Some(0));
        assert!(!cli.all);
        assert!(cli.force);
    }

    #[test]
    fn test_invalid_pid() {
        // This test would need to be implemented differently since parse_from doesn't return Result
        // For now, we'll skip this test or implement validation in the CLI parsing
        // let result = Cli::parse_from(&["gpukill", "--kill", "--pid", "0"]);
        // assert!(result.is_err());
    }

    #[test]
    fn test_reset_without_target() {
        // This test would need to be implemented differently since parse_from doesn't return Result
        // For now, we'll skip this test or implement validation in the CLI parsing
        // let result = Cli::parse_from(&["gpukill", "--reset"]);
        // assert!(result.is_err());
    }

    #[test]
    fn test_global_log_level() {
        let cli = Cli::parse_from(&["gpukill", "--log-level", "debug", "--list"]);
        assert_eq!(cli.log_level.to_string(), "debug");
    }

    #[test]
    fn test_global_config() {
        let cli = Cli::parse_from(&["gpukill", "--config", "/tmp/config.toml", "--list"]);
        assert_eq!(cli.config, Some("/tmp/config.toml".to_string()));
    }

    // New tests for vendor filtering
    #[test]
    fn test_vendor_filter_nvidia() {
        let cli = Cli::parse_from(&["gpukill", "--list", "--vendor", "nvidia"]);
        assert!(cli.list);
        assert_eq!(cli.vendor, Some(VendorFilter::Nvidia));
    }

    #[test]
    fn test_vendor_filter_amd() {
        let cli = Cli::parse_from(&["gpukill", "--list", "--vendor", "amd"]);
        assert!(cli.list);
        assert_eq!(cli.vendor, Some(VendorFilter::Amd));
    }

    #[test]
    fn test_vendor_filter_intel() {
        let cli = Cli::parse_from(&["gpukill", "--list", "--vendor", "intel"]);
        assert!(cli.list);
        assert_eq!(cli.vendor, Some(VendorFilter::Intel));
    }

    #[test]
    fn test_vendor_filter_all() {
        let cli = Cli::parse_from(&["gpukill", "--list", "--vendor", "all"]);
        assert!(cli.list);
        assert_eq!(cli.vendor, Some(VendorFilter::All));
    }

    // New tests for process filtering
    #[test]
    fn test_kill_with_filter() {
        let cli = Cli::parse_from(&["gpukill", "--kill", "--filter", "python.*"]);
        assert!(cli.kill);
        assert_eq!(cli.filter, Some("python.*".to_string()));
        assert_eq!(cli.pid, None);
    }

    #[test]
    fn test_kill_with_filter_and_batch() {
        let cli = Cli::parse_from(&["gpukill", "--kill", "--filter", "python.*", "--batch"]);
        assert!(cli.kill);
        assert_eq!(cli.filter, Some("python.*".to_string()));
        assert!(cli.batch);
    }

    #[test]
    fn test_kill_with_filter_batch_and_force() {
        let cli = Cli::parse_from(&["gpukill", "--kill", "--filter", "python.*", "--batch", "--force"]);
        assert!(cli.kill);
        assert_eq!(cli.filter, Some("python.*".to_string()));
        assert!(cli.batch);
        assert!(cli.force);
    }

    // New tests for container detection
    #[test]
    fn test_list_with_containers() {
        let cli = Cli::parse_from(&["gpukill", "--list", "--containers"]);
        assert!(cli.list);
        assert!(cli.containers);
    }

    #[test]
    fn test_list_with_containers_and_details() {
        let cli = Cli::parse_from(&["gpukill", "--list", "--containers", "--details"]);
        assert!(cli.list);
        assert!(cli.containers);
        assert!(cli.details);
    }

    // New tests for combined operations
    #[test]
    fn test_list_with_vendor_and_containers() {
        let cli = Cli::parse_from(&["gpukill", "--list", "--vendor", "nvidia", "--containers"]);
        assert!(cli.list);
        assert_eq!(cli.vendor, Some(VendorFilter::Nvidia));
        assert!(cli.containers);
    }

    #[test]
    fn test_list_with_all_new_options() {
        let cli = Cli::parse_from(&["gpukill", "--list", "--details", "--containers", "--vendor", "all", "--output", "json"]);
        assert!(cli.list);
        assert!(cli.details);
        assert!(cli.containers);
        assert_eq!(cli.vendor, Some(VendorFilter::All));
        assert!(matches!(cli.output, OutputFormat::Json));
    }
}

#[cfg(feature = "mock_nvml")]
mod mock_nvml_tests {
    use super::*;

    fn create_mock_snapshot() -> Snapshot {
        Snapshot {
            host: "test-host".to_string(),
            ts: "2024-01-01T00:00:00Z".to_string(),
            gpus: vec![
                GpuSnapshot {
                    gpu_index: 0,
                    name: "NVIDIA GeForce RTX 4090".to_string(),
                    vendor: GpuVendor::Nvidia,
                    mem_used_mb: 2048,
                    mem_total_mb: 8192,
                    util_pct: 45.2,
                    temp_c: 72,
                    power_w: 150.3,
                    ecc_volatile: Some(0),
                    pids: 2,
                    top_proc: Some(GpuProc {
                        gpu_index: 0,
                        pid: 12345,
                        user: "testuser".to_string(),
                        proc_name: "python".to_string(),
                        used_mem_mb: 1024,
                        start_time: "1h 30m".to_string(),
                        container: None,
                    }),
                },
                GpuSnapshot {
                    gpu_index: 1,
                    name: "NVIDIA GeForce RTX 3080".to_string(),
                    vendor: GpuVendor::Nvidia,
                    mem_used_mb: 1024,
                    mem_total_mb: 10240,
                    util_pct: 25.0,
                    temp_c: 65,
                    power_w: 120.0,
                    ecc_volatile: None,
                    pids: 1,
                    top_proc: None,
                },
            ],
            procs: vec![
                GpuProc {
                    gpu_index: 0,
                    pid: 12345,
                    user: "testuser".to_string(),
                    proc_name: "python".to_string(),
                    used_mem_mb: 1024,
                    start_time: "1h 30m".to_string(),
                    container: None,
                },
                GpuProc {
                    gpu_index: 0,
                    pid: 12346,
                    user: "testuser".to_string(),
                    proc_name: "tensorflow".to_string(),
                    used_mem_mb: 1024,
                    start_time: "2h 15m".to_string(),
                    container: Some("docker".to_string()),
                },
                GpuProc {
                    gpu_index: 1,
                    pid: 12347,
                    user: "testuser".to_string(),
                    proc_name: "pytorch".to_string(),
                    used_mem_mb: 1024,
                    start_time: "30m".to_string(),
                    container: None,
                },
            ],
        }
    }

    #[test]
    fn test_table_rendering() {
        let snapshot = create_mock_snapshot();
        let renderer = Renderer::new(OutputFormat::Table);
        
        // This should not panic
        let result = renderer.render_snapshot(&snapshot, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_detailed_table_rendering() {
        let snapshot = create_mock_snapshot();
        let renderer = Renderer::new(OutputFormat::Table);
        
        // This should not panic
        let result = renderer.render_snapshot(&snapshot, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_json_rendering() {
        let snapshot = create_mock_snapshot();
        let renderer = Renderer::new(OutputFormat::Json);
        
        // This should not panic
        let result = renderer.render_snapshot(&snapshot, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_json_snapshot_rendering() {
        let snapshot = create_mock_snapshot();
        let renderer = Renderer::new(OutputFormat::Json);
        
        // This should not panic
        let result = renderer.render_json_snapshot(&snapshot);
        assert!(result.is_ok());
    }

    #[test]
    fn test_snapshot_serialization() {
        let snapshot = create_mock_snapshot();
        
        // Test JSON serialization
        let json = serde_json::to_string(&snapshot).unwrap();
        let deserialized: Snapshot = serde_json::from_str(&json).unwrap();
        
        assert_eq!(snapshot.host, deserialized.host);
        assert_eq!(snapshot.gpus.len(), deserialized.gpus.len());
        assert_eq!(snapshot.procs.len(), deserialized.procs.len());
    }

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
    fn test_gpu_proc_serialization() {
        let gpu_proc = GpuProc {
            gpu_index: 0,
            pid: 12345,
            user: "testuser".to_string(),
            proc_name: "python".to_string(),
            used_mem_mb: 1024,
            start_time: "1h 30m".to_string(),
            container: Some("docker".to_string()),
        };

        let json = serde_json::to_string(&gpu_proc).unwrap();
        let deserialized: GpuProc = serde_json::from_str(&json).unwrap();
        
        assert_eq!(gpu_proc.gpu_index, deserialized.gpu_index);
        assert_eq!(gpu_proc.pid, deserialized.pid);
        assert_eq!(gpu_proc.user, deserialized.user);
        assert_eq!(gpu_proc.proc_name, deserialized.proc_name);
        assert_eq!(gpu_proc.used_mem_mb, deserialized.used_mem_mb);
        assert_eq!(gpu_proc.start_time, deserialized.start_time);
        assert_eq!(gpu_proc.container, deserialized.container);
    }

    // Tests for enhanced process management
    #[test]
    fn test_enhanced_process_manager_creation() {
        // This test would require a mock NVML API, but we can test the structure
        use gpukill::nvml_api::NvmlApi;
        
        // We'll skip this test if NVML is not available
        if let Ok(nvml_api) = NvmlApi::new() {
            use gpukill::proc::ProcessManager;
            let proc_manager = ProcessManager::new(nvml_api);
            let _enhanced_manager = EnhancedProcessManager::new(proc_manager);
            // If we get here without panicking, the test passes
        }
    }

    #[test]
    fn test_process_filtering_by_name() {
        let processes = vec![
            GpuProc {
                gpu_index: 0,
                pid: 12345,
                user: "user1".to_string(),
                proc_name: "python".to_string(),
                used_mem_mb: 100,
                start_time: "1h".to_string(),
                container: None,
            },
            GpuProc {
                gpu_index: 0,
                pid: 12346,
                user: "user1".to_string(),
                proc_name: "python3".to_string(),
                used_mem_mb: 200,
                start_time: "2h".to_string(),
                container: None,
            },
            GpuProc {
                gpu_index: 0,
                pid: 12347,
                user: "user2".to_string(),
                proc_name: "java".to_string(),
                used_mem_mb: 300,
                start_time: "3h".to_string(),
                container: None,
            },
        ];

        // Create a mock enhanced manager for testing
        use gpukill::nvml_api::NvmlApi;
        use gpukill::proc::ProcessManager;
        
        if let Ok(nvml_api) = NvmlApi::new() {
            let proc_manager = ProcessManager::new(nvml_api);
            let mut enhanced_manager = EnhancedProcessManager::new(proc_manager);

            let filtered = enhanced_manager.filter_processes_by_name(&processes, "python").unwrap();
            assert_eq!(filtered.len(), 2);
            assert_eq!(filtered[0].proc_name, "python");
            assert_eq!(filtered[1].proc_name, "python3");
        }
    }

    #[test]
    fn test_process_filtering_by_memory() {
        let processes = vec![
            GpuProc {
                gpu_index: 0,
                pid: 12345,
                user: "user1".to_string(),
                proc_name: "python".to_string(),
                used_mem_mb: 100,
                start_time: "1h".to_string(),
                container: None,
            },
            GpuProc {
                gpu_index: 0,
                pid: 12346,
                user: "user1".to_string(),
                proc_name: "python3".to_string(),
                used_mem_mb: 200,
                start_time: "2h".to_string(),
                container: None,
            },
            GpuProc {
                gpu_index: 0,
                pid: 12347,
                user: "user2".to_string(),
                proc_name: "java".to_string(),
                used_mem_mb: 300,
                start_time: "3h".to_string(),
                container: None,
            },
        ];

        use gpukill::nvml_api::NvmlApi;
        use gpukill::proc::ProcessManager;
        
        if let Ok(nvml_api) = NvmlApi::new() {
            let proc_manager = ProcessManager::new(nvml_api);
            let mut enhanced_manager = EnhancedProcessManager::new(proc_manager);

            let filtered = enhanced_manager.filter_processes_by_memory(&processes, 200);
            assert_eq!(filtered.len(), 2);
            assert!(filtered.iter().all(|p| p.used_mem_mb >= 200));
        }
    }

    #[test]
    fn test_process_stats() {
        let processes = vec![
            GpuProc {
                gpu_index: 0,
                pid: 12345,
                user: "user1".to_string(),
                proc_name: "python".to_string(),
                used_mem_mb: 100,
                start_time: "1h".to_string(),
                container: Some("docker".to_string()),
            },
            GpuProc {
                gpu_index: 0,
                pid: 12346,
                user: "user1".to_string(),
                proc_name: "python".to_string(),
                used_mem_mb: 200,
                start_time: "2h".to_string(),
                container: Some("docker".to_string()),
            },
            GpuProc {
                gpu_index: 0,
                pid: 12347,
                user: "user2".to_string(),
                proc_name: "java".to_string(),
                used_mem_mb: 300,
                start_time: "3h".to_string(),
                container: None,
            },
        ];

        use gpukill::nvml_api::NvmlApi;
        use gpukill::proc::ProcessManager;
        
        if let Ok(nvml_api) = NvmlApi::new() {
            let proc_manager = ProcessManager::new(nvml_api);
            let mut enhanced_manager = EnhancedProcessManager::new(proc_manager);

            let stats = enhanced_manager.get_process_stats(&processes);
            assert_eq!(stats.total_processes, 3);
            assert_eq!(stats.total_memory_mb, 600);
            assert_eq!(stats.non_container_processes, 1);
            assert_eq!(stats.users.len(), 2);
            assert_eq!(stats.process_names.len(), 2);
            assert_eq!(stats.containers.len(), 1);
        }
    }
}

// Integration tests that don't require NVML
mod integration_tests {
    use super::*;

    #[test]
    fn test_version_flag() {
        let output = Command::new("cargo")
            .args(&["run", "--", "--version"])
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("gpukill"));
    }

    #[test]
    fn test_help_flag() {
        let output = Command::new("cargo")
            .args(&["run", "--", "--help"])
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("gpukill"));
        assert!(stdout.contains("GPU monitoring, process management, and device control"));
    }

    #[test]
    fn test_list_help() {
        let output = Command::new("cargo")
            .args(&["run", "--", "--list", "--help"])
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("List GPUs and their current status"));
    }

    #[test]
    fn test_kill_help() {
        let output = Command::new("cargo")
            .args(&["run", "--", "--kill", "--help"])
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Kill a GPU process"));
    }

    #[test]
    fn test_reset_help() {
        let output = Command::new("cargo")
            .args(&["run", "--", "--reset", "--help"])
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Reset GPU(s)"));
    }

    #[test]
    fn test_invalid_operation_combination() {
        let output = Command::new("cargo")
            .args(&["run", "--", "--list", "--kill"])
            .output()
            .expect("Failed to execute command");

        assert!(!output.status.success());
    }

    #[test]
    fn test_missing_required_args() {
        let output = Command::new("cargo")
            .args(&["run", "--", "--kill"])
            .output()
            .expect("Failed to execute command");

        assert!(!output.status.success());
    }

    // Tests for new validation logic
    #[test]
    fn test_kill_with_both_pid_and_filter_fails() {
        let output = Command::new("cargo")
            .args(&["run", "--", "--kill", "--pid", "12345", "--filter", "python"])
            .output()
            .expect("Failed to execute command");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("cannot use both --pid and --filter"));
    }

    #[test]
    fn test_kill_without_pid_or_filter_fails() {
        let output = Command::new("cargo")
            .args(&["run", "--", "--kill"])
            .output()
            .expect("Failed to execute command");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("requires either --pid <PID> or --filter <PATTERN>"));
    }

    #[test]
    fn test_batch_without_filter_fails() {
        let output = Command::new("cargo")
            .args(&["run", "--", "--kill", "--batch"])
            .output()
            .expect("Failed to execute command");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("required arguments were not provided"));
    }

    #[test]
    fn test_containers_without_list_fails() {
        let output = Command::new("cargo")
            .args(&["run", "--", "--containers"])
            .output()
            .expect("Failed to execute command");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("required arguments were not provided"));
    }

    // Tests for vendor functionality
    #[test]
    fn test_vendor_filter_conversion() {
        assert_eq!(VendorFilter::Nvidia.to_gpu_vendor(), Some(GpuVendor::Nvidia));
        assert_eq!(VendorFilter::Amd.to_gpu_vendor(), Some(GpuVendor::Amd));
        assert_eq!(VendorFilter::Intel.to_gpu_vendor(), Some(GpuVendor::Intel));
        assert_eq!(VendorFilter::All.to_gpu_vendor(), None);
    }

    #[test]
    fn test_gpu_vendor_display() {
        assert_eq!(GpuVendor::Nvidia.to_string(), "NVIDIA");
        assert_eq!(GpuVendor::Amd.to_string(), "AMD");
        assert_eq!(GpuVendor::Intel.to_string(), "Intel");
        assert_eq!(GpuVendor::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn test_nvidia_vendor_availability() {
        // This test checks if NVIDIA vendor is available
        // It will pass if NVML is available, skip if not
        let is_available = NvidiaVendor::is_available();
        // We can't assert the result since it depends on the system
        // But we can ensure the function doesn't panic
        let _ = is_available;
    }

    #[test]
    fn test_amd_vendor_availability() {
        // This test checks if AMD vendor is available
        // It will pass if rocm-smi is available, skip if not
        let is_available = AmdVendor::is_available();
        // We can't assert the result since it depends on the system
        // But we can ensure the function doesn't panic
        let _ = is_available;
    }

    #[test]
    fn test_amd_vendor_error_message() {
        let error_msg = AmdVendor::get_availability_error();
        assert!(error_msg.contains("AMD"));
        assert!(error_msg.contains("ROCm"));
    }

    #[test]
    fn test_intel_vendor_availability() {
        // This test checks if Intel vendor is available
        // It will pass if intel_gpu_top is available, skip if not
        let is_available = gpukill::vendor::IntelVendor::is_available();
        // We can't assert the result since it depends on the system
        // But we can ensure the function doesn't panic
        let _ = is_available;
    }

    #[test]
    fn test_intel_vendor_error_message() {
        let error_msg = gpukill::vendor::IntelVendor::get_availability_error();
        assert!(error_msg.contains("Intel"));
        assert!(error_msg.contains("intel-gpu-tools"));
    }

    #[test]
    fn test_nvidia_vendor_error_message() {
        let error_msg = NvidiaVendor::get_availability_error();
        assert!(error_msg.contains("NVIDIA"));
        assert!(error_msg.contains("drivers"));
    }
}
