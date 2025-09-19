use crate::args::{Cli, OutputFormat, VendorFilter};
use crate::config::get_config;
use crate::coordinator::{CoordinatorState, create_router};
use crate::nvml_api::{NvmlApi, Snapshot};
use crate::proc::ProcessManager;
use crate::process_mgmt::EnhancedProcessManager;
use crate::render::{render_error, render_info, render_success, render_warning, Renderer};
use crate::vendor::{GpuManager, GpuVendor};
use crate::version::get_version_string;
use anyhow::{Context, Result};
use std::process;
use std::time::Duration;
use tracing::{error, info, warn};

mod args;
mod audit;
mod config;
mod coordinator;
mod nvml_api;
mod proc;
mod process_mgmt;
mod render;
mod util;
mod vendor;
mod version;

fn main() -> Result<()> {
    // Initialize error handling
    color_eyre::install().map_err(|e| anyhow::anyhow!("Failed to install error handler: {}", e))?;

    // Parse command line arguments
    let cli = Cli::parse();

    // Initialize logging
    init_logging(&cli.log_level.to_string())?;

    // Load configuration
    let config_manager = get_config(cli.config.clone())
        .context("Failed to load configuration")?;

    info!("Starting gpukill {}", get_version_string());

        // Execute the requested operation
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to create tokio runtime: {}", e))?;
        match rt.block_on(execute_operation(cli, config_manager)) {
        Ok(()) => {
            info!("Operation completed successfully");
            Ok(())
        }
        Err(e) => {
            error!("Operation failed: {}", e);
            render_error(&e.to_string());
            
            // Set appropriate exit codes
            let exit_code = if e.to_string().contains("NVML") {
                2 // NVML initialization failure
            } else if e.to_string().contains("Invalid argument") {
                3 // Invalid arguments
            } else if e.to_string().contains("permission") || e.to_string().contains("Permission") {
                4 // Permission errors
            } else if e.to_string().contains("not supported") || e.to_string().contains("unsupported") {
                5 // Operation not supported
            } else {
                1 // General error
            };

            process::exit(exit_code);
        }
    }
}

/// Initialize logging system
fn init_logging(log_level: &str) -> Result<()> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .init();

    Ok(())
}

/// Execute the requested operation
async fn execute_operation(cli: Cli, config_manager: crate::config::ConfigManager) -> Result<()> {
    // Initialize GPU manager
    let gpu_manager = GpuManager::initialize()
        .context("Failed to initialize GPU manager")?;

    if cli.list {
        execute_list_operation(
            cli.details, 
            cli.watch, 
            cli.output, 
            cli.vendor,
            cli.containers,
            gpu_manager,
            config_manager
        ).await
    } else if cli.kill {
        execute_kill_operation(
            cli.pid,
            cli.timeout_secs, 
            cli.force,
            cli.filter,
            cli.batch,
            gpu_manager,
            config_manager
        )
    } else if cli.reset {
        execute_reset_operation(cli.gpu, cli.all, cli.force, gpu_manager, config_manager)
    } else if cli.audit {
        execute_audit_operation(
            cli.audit_user,
            cli.audit_process,
            cli.audit_hours,
            cli.audit_summary,
            cli.output,
        ).await
    } else if cli.server {
        execute_server_operation(
            cli.server_host,
            cli.server_port,
            gpu_manager,
        ).await
    } else {
        Err(anyhow::anyhow!("No operation specified"))
    }
}

/// Execute list operation
async fn execute_list_operation(
    details: bool,
    watch: bool,
    output: OutputFormat,
    vendor_filter: Option<VendorFilter>,
    containers: bool,
    gpu_manager: GpuManager,
    config_manager: crate::config::ConfigManager,
) -> Result<()> {
    let renderer = Renderer::new(output);

    if watch {
        execute_watch_mode(details, containers, vendor_filter, renderer, gpu_manager, config_manager).await
    } else {
        execute_single_list(details, containers, &vendor_filter, &renderer, &gpu_manager).await
    }
}

/// Execute single list operation
async fn execute_single_list(
    details: bool,
    containers: bool,
    vendor_filter: &Option<VendorFilter>,
    renderer: &Renderer,
    gpu_manager: &GpuManager,
) -> Result<()> {
    // Get all GPU snapshots
    let mut gpus = gpu_manager.get_all_snapshots()?;
    
    // Filter by vendor if specified
    if let Some(filter) = vendor_filter {
        if let Some(target_vendor) = filter.to_gpu_vendor() {
            // For now, we'll filter by vendor name in the GPU name
            // This is a simplified approach - in a real implementation,
            // we'd track vendor per GPU more precisely
            gpus.retain(|gpu| {
                match target_vendor {
                    GpuVendor::Nvidia => gpu.name.contains("NVIDIA") || gpu.name.contains("GeForce") || gpu.name.contains("Tesla") || gpu.name.contains("Quadro"),
                    GpuVendor::Amd => gpu.name.contains("AMD") || gpu.name.contains("Radeon"),
                    GpuVendor::Intel => gpu.name.contains("Intel"),
                    GpuVendor::Apple => gpu.name.contains("Apple") || gpu.name.contains("M1") || gpu.name.contains("M2") || gpu.name.contains("M3") || gpu.name.contains("M4"),
                    _ => true,
                }
            });
        }
    }
    
    // Get all processes
    let mut procs = gpu_manager.get_all_processes()?;
    
    // Enrich with container information if requested
    if containers {
        // Create a temporary process manager for enrichment
        let nvml_api = NvmlApi::new().unwrap_or_else(|_| {
            // Create a dummy NVML API for container detection
            // This is a workaround since we need ProcessManager for enrichment
            NvmlApi::new().unwrap()
        });
        let proc_manager = ProcessManager::new(nvml_api);
        let mut enhanced_manager = EnhancedProcessManager::new(proc_manager);
        procs = enhanced_manager.enrich_with_containers(procs)?;
    }
    
    // Create snapshot for rendering
    let snapshot = Snapshot {
        host: crate::util::get_hostname(),
        ts: crate::util::get_current_timestamp_iso(),
        gpus: gpus.clone(),
        procs: procs.clone(),
    };

    // Log to audit database (async)
    // Now that execute_single_list is async, we can directly log to audit
    match crate::audit::AuditManager::new().await {
        Ok(audit_manager) => {
            match audit_manager.log_snapshot(&gpus, &procs).await {
                Ok(()) => {
                    tracing::debug!("Successfully logged audit snapshot with {} GPUs and {} processes", 
                        gpus.len(), procs.len());
                }
                Err(e) => {
                    tracing::warn!("Failed to log audit snapshot: {}", e);
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to initialize audit manager: {}", e);
        }
    }

    renderer.render_snapshot(&snapshot, details).map_err(|e| anyhow::anyhow!("Render error: {}", e))?;
    Ok(())
}

/// Execute watch mode
async fn execute_watch_mode(
    details: bool,
    containers: bool,
    vendor_filter: Option<VendorFilter>,
    renderer: Renderer,
    gpu_manager: GpuManager,
    config_manager: crate::config::ConfigManager,
) -> Result<()> {
    let _interval = Duration::from_secs(config_manager.config().watch_interval_secs);
    
    info!("Starting watch mode (refresh every {}s). Press Ctrl-C to stop.", 
          config_manager.config().watch_interval_secs);

    loop {
        match execute_single_list(details, containers, &vendor_filter, &renderer, &gpu_manager).await {
            Ok(()) => {
                if matches!(renderer.get_output_format(), OutputFormat::Table) {
                    renderer.clear_screen();
                }
            }
            Err(e) => {
                warn!("Failed to refresh data: {}", e);
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(config_manager.config().watch_interval_secs)).await;
    }
}

/// Execute kill operation
fn execute_kill_operation(
    pid: Option<u32>,
    timeout_secs: u16,
    force: bool,
    filter: Option<String>,
    batch: bool,
    gpu_manager: GpuManager,
    _config_manager: crate::config::ConfigManager,
) -> Result<()> {
    // Initialize process manager for enhanced operations
    let nvml_api = NvmlApi::new()
        .context("Failed to initialize NVML. Ensure NVIDIA drivers are installed and GPU is accessible.")?;
    let proc_manager = ProcessManager::new(nvml_api);
    let mut enhanced_manager = EnhancedProcessManager::new(proc_manager);

    if let Some(filter_pattern) = filter {
        // Batch kill based on filter
        let all_processes = gpu_manager.get_all_processes()?;
        let filtered_processes = enhanced_manager.filter_processes_by_name(&all_processes, &filter_pattern)?;
        
        if filtered_processes.is_empty() {
            render_warning(&format!("No processes found matching pattern: {}", filter_pattern));
            return Ok(());
        }

        render_info(&format!(
            "Found {} processes matching pattern '{}'",
            filtered_processes.len(),
            filter_pattern
        ));

        if batch {
            let killed_pids = enhanced_manager.batch_kill_processes(&filtered_processes, timeout_secs, force)?;
            render_success(&format!("Successfully killed {} processes: {:?}", killed_pids.len(), killed_pids));
        } else {
            // Show processes and ask for confirmation (for now, just show them)
            for proc in &filtered_processes {
                render_info(&format!("  PID {}: {} ({}) - {} MB", 
                    proc.pid, proc.proc_name, proc.user, proc.used_mem_mb));
            }
            render_warning("Use --batch flag to actually kill these processes");
        }
    } else if let Some(target_pid) = pid {
        // Single process kill
        let check_gpu_usage = !force;
        enhanced_manager.process_manager.validate_process(target_pid, check_gpu_usage)?;

        // Get process info for display
        let process_info = enhanced_manager.process_manager.get_process_info(target_pid)?;
        render_info(&format!(
            "Terminating process {} ({}: {})",
            target_pid, process_info.user, process_info.name
        ));

        // Perform graceful kill
        enhanced_manager.process_manager.graceful_kill(target_pid, timeout_secs, force)?;

        render_success(&format!("Process {} terminated successfully", target_pid));
    } else {
        return Err(anyhow::anyhow!("Either --pid or --filter must be specified"));
    }

    Ok(())
}

/// Execute reset operation
fn execute_reset_operation(
    gpu: Option<u16>,
    all: bool,
    force: bool,
    gpu_manager: GpuManager,
    _config_manager: crate::config::ConfigManager,
) -> Result<()> {
    if all {
        execute_reset_all_gpus(&gpu_manager, force)
    } else if let Some(gpu_id) = gpu {
        execute_reset_single_gpu(&gpu_manager, gpu_id, force)
    } else {
        Err(anyhow::anyhow!("No GPU specified for reset operation"))
    }
}

/// Execute reset for all GPUs
fn execute_reset_all_gpus(
    gpu_manager: &GpuManager,
    force: bool,
) -> Result<()> {
    let device_count = gpu_manager.total_device_count()?;
    
    if device_count == 0 {
        return Err(anyhow::anyhow!("No GPUs found"));
    }

    render_info(&format!("Resetting all {} GPUs", device_count));

    // Check for active processes if not forcing
    if !force {
        let active_processes = gpu_manager.get_all_processes()?;
        
        if !active_processes.is_empty() {
            render_warning("Active GPU processes found:");
            for proc in &active_processes {
                render_warning(&format!("  GPU {}: PID {} ({})", 
                    proc.gpu_index, proc.pid, proc.proc_name));
            }
            return Err(anyhow::anyhow!(
                "Cannot reset GPUs with active processes. Use --force to override."
            ));
        }
    }

    // Reset each GPU
    for i in 0..device_count {
        match gpu_manager.reset_gpu(i) {
            Ok(()) => {
                render_success(&format!("GPU {} reset successfully", i));
            }
            Err(e) => {
                render_error(&format!("Failed to reset GPU {}: {}", i, e));
            }
        }
    }

    Ok(())
}

/// Execute reset for a single GPU
fn execute_reset_single_gpu(
    gpu_manager: &GpuManager,
    gpu_id: u16,
    force: bool,
) -> Result<()> {
    let device_count = gpu_manager.total_device_count()?;
    
    if gpu_id as u32 >= device_count {
        return Err(anyhow::anyhow!(
            "GPU {} not found. Available GPUs: 0-{}",
            gpu_id,
            device_count - 1
        ));
    }

    render_info(&format!("Resetting GPU {}", gpu_id));

    // Check for active processes on this GPU if not forcing
    if !force {
        let all_processes = gpu_manager.get_all_processes()?;
        let gpu_processes: Vec<_> = all_processes
            .iter()
            .filter(|p| p.gpu_index == gpu_id)
            .collect();
        
        if !gpu_processes.is_empty() {
            render_warning(&format!("Active processes found on GPU {}:", gpu_id));
            for proc in &gpu_processes {
                render_warning(&format!("  PID {} ({})", proc.pid, proc.proc_name));
            }
            return Err(anyhow::anyhow!(
                "Cannot reset GPU {} with active processes. Use --force to override.",
                gpu_id
            ));
        }
    }

    // Reset the GPU
    gpu_manager.reset_gpu(gpu_id as u32)?;
    render_success(&format!("GPU {} reset successfully", gpu_id));

    Ok(())
}

/// Execute audit operation
async fn execute_audit_operation(
    user_filter: Option<String>,
    process_filter: Option<String>,
    hours: u32,
    summary: bool,
    output_format: crate::args::OutputFormat,
) -> Result<()> {
    use gpukill::audit::AuditManager;
    use crate::render::{render_info, render_warning};

    // Initialize audit manager
    let audit_manager = AuditManager::new().await
        .context("Failed to initialize audit manager")?;

    if summary {
        // Show audit summary
        let summary = audit_manager.get_summary(hours).await
            .context("Failed to get audit summary")?;

        render_info(&format!("GPU Usage Audit Summary (Last {} hours)", hours));
        render_info(&format!("Total records: {}", summary.total_records));

        if !summary.top_users.is_empty() {
            render_info("\nTop Users by Memory Usage:");
            for (i, (user, count, memory_mb)) in summary.top_users.iter().enumerate() {
                render_info(&format!("  {}. {}: {} records, {} MB total", 
                    i + 1, user, count, memory_mb));
            }
        }

        if !summary.top_processes.is_empty() {
            render_info("\nTop Processes by Memory Usage:");
            for (i, (process, count, memory_mb)) in summary.top_processes.iter().enumerate() {
                render_info(&format!("  {}. {}: {} records, {} MB total", 
                    i + 1, process, count, memory_mb));
            }
        }

        render_info("\nHourly GPU Memory Usage:");
        for (hour, avg_memory) in &summary.gpu_usage_by_hour {
            render_info(&format!("  Hour {}: {} MB average", hour, avg_memory));
        }

    } else {
        // Show detailed audit records
        let records = audit_manager.query_records(
            hours,
            user_filter.as_deref(),
            process_filter.as_deref(),
        ).await
        .context("Failed to query audit records")?;

        if records.is_empty() {
            render_warning(&format!("No audit records found for the last {} hours", hours));
            if user_filter.is_some() || process_filter.is_some() {
                render_info("Try removing filters to see all records");
            }
            return Ok(());
        }

        render_info(&format!("Found {} audit records (Last {} hours)", records.len(), hours));

        if output_format == crate::args::OutputFormat::Json {
            // JSON output
            let json = serde_json::to_string_pretty(&records)
                .context("Failed to serialize audit records to JSON")?;
            println!("{}", json);
        } else {
            // Table output
            use tabled::{Table, Tabled};

            #[derive(Tabled)]
            struct AuditTableRow {
                #[tabled(rename = "Time")]
                time: String,
                #[tabled(rename = "GPU")]
                gpu: String,
                #[tabled(rename = "PID")]
                pid: String,
                #[tabled(rename = "User")]
                user: String,
                #[tabled(rename = "Process")]
                process: String,
                #[tabled(rename = "Memory (MB)")]
                memory: u32,
                #[tabled(rename = "Container")]
                container: String,
            }

            let table_rows: Vec<AuditTableRow> = records.iter().map(|record| {
                AuditTableRow {
                    time: record.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
                    gpu: format!("{} ({})", record.gpu_index, record.gpu_name),
                    pid: record.pid.map(|p| p.to_string()).unwrap_or_else(|| "-".to_string()),
                    user: record.user.clone().unwrap_or_else(|| "-".to_string()),
                    process: record.process_name.clone().unwrap_or_else(|| "-".to_string()),
                    memory: record.memory_used_mb,
                    container: record.container.clone().unwrap_or_else(|| "-".to_string()),
                }
            }).collect();

            let table = Table::new(table_rows);
            println!("{}", table);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_initialization() {
        // This test just ensures the function doesn't panic
        let result = init_logging("info");
        assert!(result.is_ok());
    }

    #[test]
    fn test_version_string() {
        let version = get_version_string();
        assert!(version.contains("gpukill"));
    }
}

/// Execute server operation
async fn execute_server_operation(
    host: String,
    port: u16,
    gpu_manager: GpuManager,
) -> Result<()> {
    use axum::serve;
    use std::net::SocketAddr;

    info!("Starting GPU Kill Coordinator Server on {}:{}", host, port);

    // Initialize coordinator state
    let state = CoordinatorState::new();

    // Register this node as the coordinator
    let node_id = uuid::Uuid::new_v4().to_string();
    let hostname = crate::util::get_hostname();
    
    // Get initial GPU information
    let gpu_snapshots = gpu_manager.get_all_snapshots()?;
    let gpu_processes = gpu_manager.get_all_processes()?;
    let total_memory_gb = gpu_snapshots.iter()
        .map(|gpu| gpu.mem_total_mb as f32 / 1024.0)
        .sum();

    let node_info = crate::coordinator::NodeInfo {
        id: node_id.clone(),
        hostname: hostname.clone(),
        ip_address: "127.0.0.1".to_string(), // TODO: Get actual IP
        last_seen: chrono::Utc::now(),
        status: crate::coordinator::NodeStatus::Online,
        gpu_count: gpu_snapshots.len() as u32,
        total_memory_gb,
        tags: std::collections::HashMap::new(),
    };

    state.register_node(node_info).await?;

    // Create initial snapshot
    let initial_snapshot = crate::coordinator::NodeSnapshot {
        node_id: node_id.clone(),
        hostname,
        timestamp: chrono::Utc::now(),
        gpus: gpu_snapshots,
        processes: gpu_processes,
        status: crate::coordinator::NodeStatus::Online,
    };

    state.update_snapshot(node_id, initial_snapshot).await?;

    // Create router
    let app = create_router(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("GPU Kill Coordinator Server listening on http://{}", addr);
    info!("Dashboard will be available at http://{}:{}", host, port);
    info!("API endpoints:");
    info!("  GET  /api/nodes - List all nodes");
    info!("  GET  /api/cluster/snapshot - Get cluster snapshot");
    info!("  GET  /api/cluster/contention - Get contention analysis");
    info!("  WS   /ws - WebSocket for real-time updates");

    let listener = tokio::net::TcpListener::bind(&addr).await
        .context("Failed to bind to address")?;
    
    serve(listener, app)
        .await
        .context("Failed to start server")?;

    Ok(())
}
