use crate::args::OutputFormat;
use crate::nvml_api::Snapshot;
use crate::util::{format_memory_mb_to_gib, truncate_string};
// serde_json is used via serde_json::to_string_pretty
use std::io::{self, Write};
use tabled::{
    settings::{
        object::Rows,
        style::Style,
        Alignment, Modify, Padding, Width,
    },
    Table, Tabled,
};

/// Render GPU information to various output formats
#[derive(Clone)]
pub struct Renderer {
    output_format: OutputFormat,
}

#[allow(dead_code)]
impl Renderer {
    /// Create a new renderer
    pub fn new(output_format: OutputFormat) -> Self {
        Self { output_format }
    }

    /// Render a complete snapshot
    pub fn render_snapshot(&self, snapshot: &Snapshot, details: bool) -> Result<(), Box<dyn std::error::Error>> {
        match self.output_format {
            OutputFormat::Table => self.render_table(snapshot, details),
            OutputFormat::Json => self.render_json(snapshot),
        }
    }

    /// Render as a table
    fn render_table(&self, snapshot: &Snapshot, details: bool) -> Result<(), Box<dyn std::error::Error>> {
        if details {
            self.render_detailed_table(snapshot)
        } else {
            self.render_summary_table(snapshot)
        }
    }

    /// Render summary table (one row per GPU)
    fn render_summary_table(&self, snapshot: &Snapshot) -> Result<(), Box<dyn std::error::Error>> {
        let mut table_data = Vec::new();

        for gpu in &snapshot.gpus {
            let mem_used_gib = format_memory_mb_to_gib(gpu.mem_used_mb);
            let mem_total_gib = format_memory_mb_to_gib(gpu.mem_total_mb);
            let mem_usage = format!("{}/{} GiB", mem_used_gib, mem_total_gib);

            let top_proc_info = if let Some(ref top_proc) = gpu.top_proc {
                format!("{}:{}:{}MB", 
                    truncate_string(&top_proc.proc_name, 15),
                    top_proc.pid,
                    top_proc.used_mem_mb
                )
            } else {
                "-".to_string()
            };

            let ecc_info = gpu.ecc_volatile
                .map(|e| e.to_string())
                .unwrap_or_else(|| "-".to_string());

            table_data.push(SummaryRow {
                gpu: gpu.gpu_index.to_string(),
                name: truncate_string(&gpu.name, 20),
                memory: mem_usage,
                utilization: format!("{:.1}%", gpu.util_pct),
                temperature: format!("{}°C", gpu.temp_c),
                power: format!("{:.1}W", gpu.power_w),
                ecc_volatile: ecc_info,
                pids: gpu.pids.to_string(),
                top_process: top_proc_info,
            });
        }

        let table = Table::new(&table_data)
            .with(Style::modern())
            .with(Modify::new(Rows::new(1..)).with(Alignment::left()))
            .with(Modify::new(Rows::new(1..)).with(Padding::new(1, 1, 0, 0)))
            .with(Width::wrap(120))
            .to_string();

        println!("{}", table);
        Ok(())
    }

    /// Render detailed table (one row per process)
    fn render_detailed_table(&self, snapshot: &Snapshot) -> Result<(), Box<dyn std::error::Error>> {
        // First render summary
        self.render_summary_table(snapshot)?;
        println!();

        // Then render process details
        if !snapshot.procs.is_empty() {
            let mut table_data = Vec::new();

            for proc in &snapshot.procs {
                let container_info = proc.container
                    .as_ref()
                    .map(|c| truncate_string(c, 15))
                    .unwrap_or_else(|| "-".to_string());

                table_data.push(ProcessRow {
                    gpu: proc.gpu_index.to_string(),
                    pid: proc.pid.to_string(),
                    user: truncate_string(&proc.user, 12),
                    process: truncate_string(&proc.proc_name, 20),
                    vram_mb: format!("{}MB", proc.used_mem_mb),
                    start_time: truncate_string(&proc.start_time, 10),
                    container: container_info,
                });
            }

            let table = Table::new(&table_data)
                .with(Style::modern())
                .with(Modify::new(Rows::new(1..)).with(Alignment::left()))
                .with(Modify::new(Rows::new(1..)).with(Padding::new(1, 1, 0, 0)))
                .with(Width::wrap(120))
                .to_string();

            println!("Process Details:");
            println!("{}", table);
        }

        Ok(())
    }

    /// Render as JSON
    fn render_json(&self, snapshot: &Snapshot) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(snapshot)?;
        println!("{}", json);
        Ok(())
    }

    /// Render JSON snapshot for watch mode (newline-delimited)
    pub fn render_json_snapshot(&self, snapshot: &Snapshot) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string(snapshot)?;
        println!("{}", json);
        io::stdout().flush()?;
        Ok(())
    }

    /// Clear screen for watch mode
    pub fn clear_screen(&self) {
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap_or_default();
    }

    /// Get output format
    pub fn get_output_format(&self) -> OutputFormat {
        self.output_format.clone()
    }
}

/// Summary table row structure
#[derive(Tabled)]
struct SummaryRow {
    #[tabled(rename = "GPU")]
    gpu: String,
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "MEM_USED/TOTAL")]
    memory: String,
    #[tabled(rename = "UTIL(%)")]
    utilization: String,
    #[tabled(rename = "TEMP(°C)")]
    temperature: String,
    #[tabled(rename = "POWER(W)")]
    power: String,
    #[tabled(rename = "ECC(volatile)")]
    ecc_volatile: String,
    #[tabled(rename = "PIDS")]
    pids: String,
    #[tabled(rename = "TOP_PROC")]
    top_process: String,
}

/// Process table row structure
#[derive(Tabled)]
struct ProcessRow {
    #[tabled(rename = "GPU")]
    gpu: String,
    #[tabled(rename = "PID")]
    pid: String,
    #[tabled(rename = "USER")]
    user: String,
    #[tabled(rename = "PROC")]
    process: String,
    #[tabled(rename = "VRAM_MB")]
    vram_mb: String,
    #[tabled(rename = "START_TIME")]
    start_time: String,
    #[tabled(rename = "CONTAINER?")]
    container: String,
}

/// Render error messages
pub fn render_error(message: &str) {
    eprintln!("Error: {}", message);
}

/// Render warning messages
pub fn render_warning(message: &str) {
    eprintln!("Warning: {}", message);
}

/// Render info messages
pub fn render_info(message: &str) {
    println!("Info: {}", message);
}

/// Render success messages
pub fn render_success(message: &str) {
    println!("Success: {}", message);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nvml_api::{GpuProc, GpuSnapshot, Snapshot};

    fn create_test_snapshot() -> Snapshot {
        Snapshot {
            host: "test-host".to_string(),
            ts: "2024-01-01T00:00:00Z".to_string(),
            gpus: vec![GpuSnapshot {
                gpu_index: 0,
                name: "Test GPU".to_string(),
                vendor: crate::vendor::GpuVendor::Unknown,
                mem_used_mb: 2048,
                mem_total_mb: 8192,
                util_pct: 50.0,
                temp_c: 75,
                power_w: 150.0,
                ecc_volatile: Some(0),
                pids: 2,
                top_proc: Some(GpuProc {
                    gpu_index: 0,
                    pid: 12345,
                    user: "testuser".to_string(),
                    proc_name: "test_process".to_string(),
                    used_mem_mb: 1024,
                    start_time: "1h 30m".to_string(),
                    container: None,
                }),
            }],
            procs: vec![GpuProc {
                gpu_index: 0,
                pid: 12345,
                user: "testuser".to_string(),
                proc_name: "test_process".to_string(),
                used_mem_mb: 1024,
                start_time: "1h 30m".to_string(),
                container: None,
            }],
        }
    }

    #[test]
    fn test_renderer_creation() {
        let renderer = Renderer::new(OutputFormat::Table);
        assert!(matches!(renderer.output_format, OutputFormat::Table));
    }

    #[test]
    fn test_json_rendering() {
        let renderer = Renderer::new(OutputFormat::Json);
        let snapshot = create_test_snapshot();
        
        // This should not panic
        let result = renderer.render_json(&snapshot);
        assert!(result.is_ok());
    }

    #[test]
    fn test_table_rendering() {
        let renderer = Renderer::new(OutputFormat::Table);
        let snapshot = create_test_snapshot();
        
        // This should not panic
        let result = renderer.render_table(&snapshot, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_detailed_table_rendering() {
        let renderer = Renderer::new(OutputFormat::Table);
        let snapshot = create_test_snapshot();
        
        // This should not panic
        let result = renderer.render_table(&snapshot, true);
        assert!(result.is_ok());
    }
}
