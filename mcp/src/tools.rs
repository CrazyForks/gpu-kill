//! MCP Tools for GPU Kill

use crate::types::*;
use gpukill::vendor::GpuManager;
use gpukill::guard_mode::GuardModeManager;
use gpukill::rogue_detection::RogueDetector;
use gpukill::process_mgmt::EnhancedProcessManager;
use gpukill::proc::ProcessManager;
use gpukill::nvml_api::NvmlApi;
use gpukill::audit::AuditManager;
use serde_json::json;
use std::collections::HashMap;

/// Tool handler for GPU Kill MCP server
pub struct ToolHandler {
    gpu_manager: GpuManager,
    process_manager: Option<EnhancedProcessManager>,
    guard_mode: Option<GuardModeManager>,
    rogue_detector: Option<RogueDetector>,
}

impl ToolHandler {
    pub async fn new() -> anyhow::Result<Self> {
        let gpu_manager = GpuManager::initialize()?;
        // Try to initialize NVML API, but don't panic if it fails
        let process_manager = if let Ok(nvml_api) = NvmlApi::new() {
            Some(EnhancedProcessManager::new(ProcessManager::new(nvml_api)))
        } else {
            tracing::warn!("NVML API not available, process management will be limited");
            None
        };
        
        // Initialize optional components
        let guard_mode = GuardModeManager::new().ok();
        let audit_manager = AuditManager::new().await.ok();
        let rogue_detector = audit_manager.map(|am| RogueDetector::new(am));

        Ok(Self {
            gpu_manager,
            process_manager,
            guard_mode,
            rogue_detector,
        })
    }

    /// List all available tools
    pub fn list_tools(&self) -> Vec<Tool> {
        vec![
            Tool {
                name: "kill_gpu_process".to_string(),
                description: Some("Kill a GPU process by PID".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "pid": {
                            "type": "integer",
                            "description": "Process ID to kill"
                        },
                        "force": {
                            "type": "boolean",
                            "description": "Force kill if graceful termination fails",
                            "default": false
                        }
                    },
                    "required": ["pid"]
                }),
            },
            Tool {
                name: "reset_gpu".to_string(),
                description: Some("Reset a GPU by ID".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "gpu_id": {
                            "type": "integer",
                            "description": "GPU ID to reset"
                        },
                        "force": {
                            "type": "boolean",
                            "description": "Force reset even if processes are running",
                            "default": false
                        }
                    },
                    "required": ["gpu_id"]
                }),
            },
            Tool {
                name: "scan_rogue_activity".to_string(),
                description: Some("Scan for suspicious GPU activity".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "hours": {
                            "type": "integer",
                            "description": "Hours of history to scan",
                            "default": 24
                        }
                    }
                }),
            },
            Tool {
                name: "create_user_policy".to_string(),
                description: Some("Create a user policy for Guard Mode".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "username": {
                            "type": "string",
                            "description": "Username for the policy"
                        },
                        "memory_limit_gb": {
                            "type": "number",
                            "description": "Memory limit in GB"
                        },
                        "utilization_limit_pct": {
                            "type": "number",
                            "description": "Utilization limit percentage"
                        },
                        "process_limit": {
                            "type": "integer",
                            "description": "Maximum number of processes"
                        }
                    },
                    "required": ["username"]
                }),
            },
            Tool {
                name: "get_gpu_status".to_string(),
                description: Some("Get detailed status of a specific GPU".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "gpu_id": {
                            "type": "integer",
                            "description": "GPU ID to get status for"
                        }
                    },
                    "required": ["gpu_id"]
                }),
            },
            Tool {
                name: "kill_processes_by_name".to_string(),
                description: Some("Kill all processes matching a name pattern".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "pattern": {
                            "type": "string",
                            "description": "Process name pattern (supports regex)"
                        },
                        "force": {
                            "type": "boolean",
                            "description": "Force kill if graceful termination fails",
                            "default": false
                        }
                    },
                    "required": ["pattern"]
                }),
            },
        ]
    }

    /// Execute a tool by name with arguments
    pub async fn execute_tool(&mut self, name: &str, arguments: Option<HashMap<String, serde_json::Value>>) -> anyhow::Result<ToolResult> {
        match name {
            "kill_gpu_process" => self.kill_gpu_process(arguments).await,
            "reset_gpu" => self.reset_gpu(arguments).await,
            "scan_rogue_activity" => self.scan_rogue_activity(arguments).await,
            "create_user_policy" => self.create_user_policy(arguments).await,
            "get_gpu_status" => self.get_gpu_status(arguments).await,
            "kill_processes_by_name" => self.kill_processes_by_name(arguments).await,
            _ => Err(anyhow::anyhow!("Unknown tool: {}", name)),
        }
    }

    async fn kill_gpu_process(&mut self, arguments: Option<HashMap<String, serde_json::Value>>) -> anyhow::Result<ToolResult> {
        let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
        let pid = args.get("pid")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid pid"))? as u32;
        
        let _force = args.get("force")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if let Some(ref mut pm) = self.process_manager {
            match pm.process_manager.graceful_kill(pid, 10, _force) {
                Ok(_) => Ok(ToolResult {
                    content: vec![ToolContent {
                        content_type: "text".to_string(),
                        text: Some(format!("Successfully killed process {}", pid)),
                        data: None,
                    }],
                    is_error: Some(false),
                }),
                Err(e) => Ok(ToolResult {
                    content: vec![ToolContent {
                        content_type: "text".to_string(),
                        text: Some(format!("Failed to kill process {}: {}", pid, e)),
                        data: None,
                    }],
                    is_error: Some(true),
                }),
            }
        } else {
            Ok(ToolResult {
                content: vec![ToolContent {
                    content_type: "text".to_string(),
                    text: Some("Process management not available on this system".to_string()),
                    data: None,
                }],
                is_error: Some(true),
            })
        }
    }

    async fn reset_gpu(&self, arguments: Option<HashMap<String, serde_json::Value>>) -> anyhow::Result<ToolResult> {
        let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
        let gpu_id = args.get("gpu_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid gpu_id"))? as u32;
        
        let _force = args.get("force")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        match self.gpu_manager.reset_gpu(gpu_id) {
            Ok(_) => Ok(ToolResult {
                content: vec![ToolContent {
                    content_type: "text".to_string(),
                    text: Some(format!("Successfully reset GPU {}", gpu_id)),
                    data: None,
                }],
                is_error: Some(false),
            }),
            Err(e) => Ok(ToolResult {
                content: vec![ToolContent {
                    content_type: "text".to_string(),
                    text: Some(format!("Failed to reset GPU {}: {}", gpu_id, e)),
                    data: None,
                }],
                is_error: Some(true),
            }),
        }
    }

    async fn scan_rogue_activity(&self, arguments: Option<HashMap<String, serde_json::Value>>) -> anyhow::Result<ToolResult> {
        let args = arguments.unwrap_or_default();
        let hours = args.get("hours")
            .and_then(|v| v.as_u64())
            .unwrap_or(24) as u32;

        if let Some(rogue_detector) = &self.rogue_detector {
            match rogue_detector.detect_rogue_activity(hours).await {
                Ok(result) => {
                    let total_threats = result.suspicious_processes.len() + 
                                       result.crypto_miners.len() + 
                                       result.resource_abusers.len() + 
                                       result.data_exfiltrators.len();
                    let high_severity = result.crypto_miners.len() + result.data_exfiltrators.len();
                    
                    Ok(ToolResult {
                        content: vec![ToolContent {
                            content_type: "text".to_string(),
                            text: Some(format!(
                                "Rogue activity scan completed. Found {} threats ({} high severity) in the last {} hours.",
                                total_threats, high_severity, hours
                            )),
                            data: Some(json!(result)),
                        }],
                        is_error: Some(false),
                    })
                },
                Err(e) => Ok(ToolResult {
                    content: vec![ToolContent {
                        content_type: "text".to_string(),
                        text: Some(format!("Failed to scan for rogue activity: {}", e)),
                        data: None,
                    }],
                    is_error: Some(true),
                }),
            }
        } else {
            Ok(ToolResult {
                content: vec![ToolContent {
                    content_type: "text".to_string(),
                    text: Some("Rogue detection not available".to_string()),
                    data: None,
                }],
                is_error: Some(true),
            })
        }
    }

    async fn create_user_policy(&mut self, arguments: Option<HashMap<String, serde_json::Value>>) -> anyhow::Result<ToolResult> {
        let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
        let username = args.get("username")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing username"))?;

        if let Some(guard_mode) = &mut self.guard_mode {
            let memory_limit = args.get("memory_limit_gb")
                .and_then(|v| v.as_f64())
                .unwrap_or(8.0);
            let utilization_limit = args.get("utilization_limit_pct")
                .and_then(|v| v.as_f64())
                .unwrap_or(70.0);
            let process_limit = args.get("process_limit")
                .and_then(|v| v.as_u64())
                .unwrap_or(3) as u32;

            let policy = gpukill::guard_mode::UserPolicy {
                username: username.to_string(),
                memory_limit_gb: memory_limit as f32,
                utilization_limit_pct: utilization_limit as f32,
                duration_limit_hours: 24.0, // Default 24 hours
                max_concurrent_processes: process_limit,
                allowed_gpus: Vec::new(), // Allow all GPUs by default
                blocked_gpus: Vec::new(), // Block none by default
                priority: 0, // Normal priority
                time_overrides: Vec::new(), // No time overrides
            };
            
            match guard_mode.add_user_policy(policy) {
                Ok(_) => Ok(ToolResult {
                    content: vec![ToolContent {
                        content_type: "text".to_string(),
                        text: Some(format!(
                            "Successfully created user policy for {}: {}GB memory, {}% utilization, {} processes",
                            username, memory_limit, utilization_limit, process_limit
                        )),
                        data: None,
                    }],
                    is_error: Some(false),
                }),
                Err(e) => Ok(ToolResult {
                    content: vec![ToolContent {
                        content_type: "text".to_string(),
                        text: Some(format!("Failed to create user policy: {}", e)),
                        data: None,
                    }],
                    is_error: Some(true),
                }),
            }
        } else {
            Ok(ToolResult {
                content: vec![ToolContent {
                    content_type: "text".to_string(),
                    text: Some("Guard Mode not available".to_string()),
                    data: None,
                }],
                is_error: Some(true),
            })
        }
    }

    async fn get_gpu_status(&self, arguments: Option<HashMap<String, serde_json::Value>>) -> anyhow::Result<ToolResult> {
        let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
        let gpu_id = args.get("gpu_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid gpu_id"))? as u32;

        match self.gpu_manager.get_all_snapshots() {
            Ok(gpus) => {
                if let Some(gpu) = gpus.into_iter().find(|g| g.gpu_index as u32 == gpu_id) {
                    Ok(ToolResult {
                        content: vec![ToolContent {
                            content_type: "text".to_string(),
                            text: Some(format!(
                                "GPU {}: {} ({}%) - {:.1}GB/{:.1}GB memory - {} processes",
                                gpu.gpu_index, gpu.name, gpu.util_pct, gpu.mem_used_mb, gpu.mem_total_mb, gpu.pids
                            )),
                            data: Some(json!(gpu)),
                        }],
                        is_error: Some(false),
                    })
                } else {
                    Ok(ToolResult {
                        content: vec![ToolContent {
                            content_type: "text".to_string(),
                            text: Some(format!("GPU {} not found", gpu_id)),
                            data: None,
                        }],
                        is_error: Some(true),
                    })
                }
            },
            Err(e) => Ok(ToolResult {
                content: vec![ToolContent {
                    content_type: "text".to_string(),
                    text: Some(format!("Failed to get GPU status: {}", e)),
                    data: None,
                }],
                is_error: Some(true),
            }),
        }
    }

    async fn kill_processes_by_name(&mut self, arguments: Option<HashMap<String, serde_json::Value>>) -> anyhow::Result<ToolResult> {
        let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
        let pattern = args.get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing pattern"))?;
        
        let _force = args.get("force")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Get all GPU processes first
        match self.gpu_manager.get_all_processes() {
            Ok(all_processes) => {
                // Filter processes by name pattern
                if let Some(ref mut pm) = self.process_manager {
                    match pm.filter_processes_by_name(&all_processes, pattern) {
                    Ok(filtered_processes) => {
                        if filtered_processes.is_empty() {
                            Ok(ToolResult {
                                content: vec![ToolContent {
                                    content_type: "text".to_string(),
                                    text: Some(format!("No processes found matching pattern '{}'", pattern)),
                                    data: None,
                                }],
                                is_error: Some(false),
                            })
                        } else {
                            // Kill the filtered processes
                            match pm.batch_kill_processes(&filtered_processes, 10, _force) {
                                Ok(killed_pids) => Ok(ToolResult {
                                    content: vec![ToolContent {
                                        content_type: "text".to_string(),
                                        text: Some(format!("Successfully killed {} processes matching pattern '{}'", killed_pids.len(), pattern)),
                                        data: Some(json!(killed_pids)),
                                    }],
                                    is_error: Some(false),
                                }),
                                Err(e) => Ok(ToolResult {
                                    content: vec![ToolContent {
                                        content_type: "text".to_string(),
                                        text: Some(format!("Failed to kill processes: {}", e)),
                                        data: None,
                                    }],
                                    is_error: Some(true),
                                }),
                            }
                        }
                    },
                    Err(e) => Ok(ToolResult {
                        content: vec![ToolContent {
                            content_type: "text".to_string(),
                            text: Some(format!("Failed to filter processes: {}", e)),
                            data: None,
                        }],
                        is_error: Some(true),
                    }),
                }
                } else {
                    Ok(ToolResult {
                        content: vec![ToolContent {
                            content_type: "text".to_string(),
                            text: Some("Process management not available on this system".to_string()),
                            data: None,
                        }],
                        is_error: Some(true),
                    })
                }
            },
            Err(e) => Ok(ToolResult {
                content: vec![ToolContent {
                    content_type: "text".to_string(),
                    text: Some(format!("Failed to get GPU processes: {}", e)),
                    data: None,
                }],
                is_error: Some(true),
            }),
        }
    }
}
