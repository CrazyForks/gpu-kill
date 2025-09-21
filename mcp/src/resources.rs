//! MCP Resources for GPU Kill

use crate::types::*;
use gpukill::audit::AuditManager;
use gpukill::guard_mode::GuardModeManager;
use gpukill::rogue_detection::RogueDetector;
use gpukill::vendor::GpuManager;
use serde_json::json;
use std::collections::HashMap;

/// Resource handler for GPU Kill MCP server
pub struct ResourceHandler {
    gpu_manager: GpuManager,
    guard_mode: Option<GuardModeManager>,
    rogue_detector: Option<RogueDetector>,
    audit_manager: Option<AuditManager>,
}

impl ResourceHandler {
    pub async fn new() -> anyhow::Result<Self> {
        let gpu_manager = GpuManager::initialize()?;

        // Initialize optional components
        let guard_mode = GuardModeManager::new().ok();
        let audit_manager = AuditManager::new().await.ok();
        let rogue_detector = if let Some(am) = audit_manager {
            Some(RogueDetector::new(am))
        } else {
            None
        };

        Ok(Self {
            gpu_manager,
            guard_mode,
            rogue_detector,
            audit_manager: None, // We moved it to rogue_detector
        })
    }

    /// List all available resources
    pub fn list_resources(&self) -> Vec<Resource> {
        vec![
            Resource {
                uri: "gpu://list".to_string(),
                name: "GPU List".to_string(),
                description: Some("Current GPU status and utilization".to_string()),
                mime_type: Some("application/json".to_string()),
            },
            Resource {
                uri: "gpu://processes".to_string(),
                name: "GPU Processes".to_string(),
                description: Some("Currently running GPU processes".to_string()),
                mime_type: Some("application/json".to_string()),
            },
            Resource {
                uri: "gpu://audit".to_string(),
                name: "GPU Audit".to_string(),
                description: Some("Historical GPU usage data".to_string()),
                mime_type: Some("application/json".to_string()),
            },
            Resource {
                uri: "gpu://policies".to_string(),
                name: "Guard Mode Policies".to_string(),
                description: Some("Current Guard Mode policies".to_string()),
                mime_type: Some("application/json".to_string()),
            },
            Resource {
                uri: "gpu://rogue-detection".to_string(),
                name: "Rogue Detection".to_string(),
                description: Some("Security scan results and threats".to_string()),
                mime_type: Some("application/json".to_string()),
            },
        ]
    }

    /// Get resource contents by URI
    pub async fn get_resource(&self, uri: &str) -> anyhow::Result<ResourceContents> {
        match uri {
            "gpu://list" => self.get_gpu_list().await,
            "gpu://processes" => self.get_gpu_processes().await,
            "gpu://audit" => self.get_audit_data().await,
            "gpu://policies" => self.get_policies().await,
            "gpu://rogue-detection" => self.get_rogue_detection().await,
            _ => Err(anyhow::anyhow!("Unknown resource URI: {}", uri)),
        }
    }

    async fn get_gpu_list(&self) -> anyhow::Result<ResourceContents> {
        let gpus = self.gpu_manager.get_all_snapshots()?;
        let gpu_info: Vec<GpuInfo> = gpus
            .into_iter()
            .map(|gpu| GpuInfo {
                id: gpu.gpu_index as u32,
                name: gpu.name,
                vendor: gpu.vendor.to_string(),
                memory_used: gpu.mem_used_mb as f64,
                memory_total: gpu.mem_total_mb as f64,
                utilization: gpu.util_pct as f64,
                temperature: Some(gpu.temp_c as f64),
                power_usage: Some(gpu.power_w as f64),
                processes: gpu
                    .top_proc
                    .map(|proc| GpuProcess {
                        pid: proc.pid,
                        name: proc.proc_name,
                        memory_usage: proc.used_mem_mb as f64,
                        user: Some(proc.user),
                    })
                    .into_iter()
                    .collect(),
            })
            .collect();

        let json_text = serde_json::to_string_pretty(&gpu_info)?;

        Ok(ResourceContents {
            uri: "gpu://list".to_string(),
            mime_type: Some("application/json".to_string()),
            text: Some(json_text),
            blob: None,
        })
    }

    async fn get_gpu_processes(&self) -> anyhow::Result<ResourceContents> {
        let gpus = self.gpu_manager.get_all_snapshots()?;
        let mut all_processes = Vec::new();

        for gpu in gpus {
            if let Some(proc) = gpu.top_proc {
                all_processes.push(GpuProcess {
                    pid: proc.pid,
                    name: proc.proc_name,
                    memory_usage: proc.used_mem_mb as f64,
                    user: Some(proc.user),
                });
            }
        }

        let json_text = serde_json::to_string_pretty(&all_processes)?;

        Ok(ResourceContents {
            uri: "gpu://processes".to_string(),
            mime_type: Some("application/json".to_string()),
            text: Some(json_text),
            blob: None,
        })
    }

    async fn get_audit_data(&self) -> anyhow::Result<ResourceContents> {
        // For now, return empty audit data since we don't have access to audit_manager
        // In a full implementation, we would need to restructure to share the audit_manager
        Ok(ResourceContents {
            uri: "gpu://audit".to_string(),
            mime_type: Some("application/json".to_string()),
            text: Some("[]".to_string()),
            blob: None,
        })
    }

    async fn get_policies(&self) -> anyhow::Result<ResourceContents> {
        if let Some(guard_mode) = &self.guard_mode {
            let config = guard_mode.get_config();
            let policies: Vec<PolicyInfo> = config
                .user_policies
                .iter()
                .map(|(name, policy)| {
                    let mut limits = HashMap::new();
                    limits.insert("memory_limit_gb".to_string(), json!(policy.memory_limit_gb));
                    limits.insert(
                        "utilization_limit_pct".to_string(),
                        json!(policy.utilization_limit_pct),
                    );
                    limits.insert(
                        "process_limit".to_string(),
                        json!(policy.max_concurrent_processes),
                    );

                    PolicyInfo {
                        policy_type: "user".to_string(),
                        name: name.clone(),
                        enabled: true,
                        limits,
                    }
                })
                .collect();

            let json_text = serde_json::to_string_pretty(&policies)?;

            Ok(ResourceContents {
                uri: "gpu://policies".to_string(),
                mime_type: Some("application/json".to_string()),
                text: Some(json_text),
                blob: None,
            })
        } else {
            Ok(ResourceContents {
                uri: "gpu://policies".to_string(),
                mime_type: Some("application/json".to_string()),
                text: Some("[]".to_string()),
                blob: None,
            })
        }
    }

    async fn get_rogue_detection(&self) -> anyhow::Result<ResourceContents> {
        if let Some(rogue_detector) = &self.rogue_detector {
            let result = rogue_detector.detect_rogue_activity(24).await?;

            // Combine all threat types into a single list
            let mut all_threats = Vec::new();

            // Add suspicious processes
            for threat in result.suspicious_processes {
                all_threats.push(ThreatInfo {
                    id: format!("suspicious_{}", threat.process.pid),
                    threat_type: "suspicious_process".to_string(),
                    severity: "medium".to_string(),
                    confidence: threat.confidence as f64,
                    description: format!("Suspicious process: {}", threat.process.proc_name),
                    process_info: Some(GpuProcess {
                        pid: threat.process.pid,
                        name: threat.process.proc_name,
                        memory_usage: threat.process.used_mem_mb as f64,
                        user: Some(threat.process.user),
                    }),
                });
            }

            // Add crypto miners
            for threat in result.crypto_miners {
                all_threats.push(ThreatInfo {
                    id: format!("crypto_{}", threat.process.pid),
                    threat_type: "crypto_miner".to_string(),
                    severity: "high".to_string(),
                    confidence: threat.confidence as f64,
                    description: format!("Crypto miner detected: {}", threat.process.proc_name),
                    process_info: Some(GpuProcess {
                        pid: threat.process.pid,
                        name: threat.process.proc_name,
                        memory_usage: threat.process.used_mem_mb as f64,
                        user: Some(threat.process.user),
                    }),
                });
            }

            // Add resource abusers
            for threat in result.resource_abusers {
                all_threats.push(ThreatInfo {
                    id: format!("abuser_{}", threat.process.pid),
                    threat_type: "resource_abuser".to_string(),
                    severity: "medium".to_string(),
                    confidence: threat.severity as f64,
                    description: format!("Resource abuser: {}", threat.process.proc_name),
                    process_info: Some(GpuProcess {
                        pid: threat.process.pid,
                        name: threat.process.proc_name,
                        memory_usage: threat.process.used_mem_mb as f64,
                        user: Some(threat.process.user),
                    }),
                });
            }

            // Add data exfiltrators
            for threat in result.data_exfiltrators {
                all_threats.push(ThreatInfo {
                    id: format!("exfil_{}", threat.process.pid),
                    threat_type: "data_exfiltrator".to_string(),
                    severity: "high".to_string(),
                    confidence: threat.confidence as f64,
                    description: format!("Data exfiltrator: {}", threat.process.proc_name),
                    process_info: Some(GpuProcess {
                        pid: threat.process.pid,
                        name: threat.process.proc_name,
                        memory_usage: threat.process.used_mem_mb as f64,
                        user: Some(threat.process.user),
                    }),
                });
            }

            let threat_info = all_threats;

            let json_text = serde_json::to_string_pretty(&threat_info)?;

            Ok(ResourceContents {
                uri: "gpu://rogue-detection".to_string(),
                mime_type: Some("application/json".to_string()),
                text: Some(json_text),
                blob: None,
            })
        } else {
            Ok(ResourceContents {
                uri: "gpu://rogue-detection".to_string(),
                mime_type: Some("application/json".to_string()),
                text: Some("[]".to_string()),
                blob: None,
            })
        }
    }
}
