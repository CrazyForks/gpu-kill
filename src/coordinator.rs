use crate::nvml_api::{GpuProc, GpuSnapshot};
use anyhow::Result;
use axum::{
    extract::{Path, State, WebSocketUpgrade},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

/// Node information for cluster management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: String,
    pub hostname: String,
    pub ip_address: String,
    pub last_seen: DateTime<Utc>,
    pub status: NodeStatus,
    pub gpu_count: u32,
    pub total_memory_gb: f32,
    pub tags: HashMap<String, String>,
}

/// Node status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeStatus {
    Online,
    Offline,
    Degraded,
}

/// Cluster snapshot combining all nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterSnapshot {
    pub timestamp: DateTime<Utc>,
    pub nodes: Vec<NodeSnapshot>,
    pub total_gpus: u32,
    pub total_memory_gb: f32,
    pub active_processes: u32,
    pub utilization_avg: f32,
}

/// Node snapshot with GPU and process data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSnapshot {
    pub node_id: String,
    pub hostname: String,
    pub timestamp: DateTime<Utc>,
    pub gpus: Vec<GpuSnapshot>,
    pub processes: Vec<GpuProc>,
    pub status: NodeStatus,
}

/// Contention analysis for Magic Moment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentionAnalysis {
    pub blocked_gpus: Vec<BlockedGpu>,
    pub top_users: Vec<UserUsage>,
    pub recommendations: Vec<String>,
}

/// Information about a blocked GPU
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedGpu {
    pub node_id: String,
    pub gpu_index: u16,
    pub gpu_name: String,
    pub blocking_processes: Vec<GpuProc>,
    pub utilization_pct: f32,
    pub memory_used_mb: u32,
    pub memory_total_mb: u32,
}

/// User usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserUsage {
    pub user: String,
    pub gpu_count: u32,
    pub total_memory_mb: u32,
    pub avg_utilization: f32,
    pub process_count: u32,
}

/// Coordinator state
#[derive(Debug, Clone)]
pub struct CoordinatorState {
    pub nodes: Arc<RwLock<HashMap<String, NodeInfo>>>,
    pub snapshots: Arc<RwLock<HashMap<String, NodeSnapshot>>>,
    pub last_cluster_snapshot: Arc<RwLock<Option<ClusterSnapshot>>>,
}

impl Default for CoordinatorState {
    fn default() -> Self {
        Self::new()
    }
}

impl CoordinatorState {
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            snapshots: Arc::new(RwLock::new(HashMap::new())),
            last_cluster_snapshot: Arc::new(RwLock::new(None)),
        }
    }

    /// Start background tasks for cluster management
    pub fn start_background_tasks(&self) {
        let state = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;

                // Clean up stale nodes
                if let Err(e) = state.cleanup_stale_nodes().await {
                    tracing::warn!("Failed to cleanup stale nodes: {}", e);
                }

                // Update cluster snapshot
                if let Err(e) = state.update_cluster_snapshot().await {
                    tracing::warn!("Failed to update cluster snapshot: {}", e);
                }
            }
        });
    }

    /// Register or update a node
    pub async fn register_node(&self, node_info: NodeInfo) -> Result<()> {
        let mut nodes = self.nodes.write().await;
        nodes.insert(node_info.id.clone(), node_info);
        Ok(())
    }

    /// Update node snapshot
    pub async fn update_snapshot(&self, node_id: String, snapshot: NodeSnapshot) -> Result<()> {
        // Update node last seen
        {
            let mut nodes = self.nodes.write().await;
            if let Some(node) = nodes.get_mut(&node_id) {
                node.last_seen = Utc::now();
                node.status = NodeStatus::Online;
            }
        }

        // Store snapshot
        {
            let mut snapshots = self.snapshots.write().await;
            snapshots.insert(node_id.clone(), snapshot);
        }

        // Update cluster snapshot
        self.update_cluster_snapshot().await?;
        Ok(())
    }

    /// Get all nodes
    pub async fn get_nodes(&self) -> Vec<NodeInfo> {
        let nodes = self.nodes.read().await;
        nodes.values().cloned().collect()
    }

    /// Get cluster snapshot
    pub async fn get_cluster_snapshot(&self) -> Option<ClusterSnapshot> {
        let snapshot = self.last_cluster_snapshot.read().await;
        snapshot.clone()
    }

    /// Build cluster snapshot from current node data
    pub async fn build_cluster_snapshot(&self) -> Result<ClusterSnapshot> {
        let nodes = self.nodes.read().await;
        let snapshots = self.snapshots.read().await;

        let mut node_snapshots = Vec::new();
        let mut total_gpus = 0;
        let mut total_memory_gb = 0.0;
        let mut active_processes = 0;
        let mut total_utilization = 0.0;
        let mut gpu_count = 0;

        for (node_id, node_info) in nodes.iter() {
            if let Some(snapshot) = snapshots.get(node_id) {
                let node_snapshot = NodeSnapshot {
                    node_id: node_id.clone(),
                    hostname: node_info.hostname.clone(),
                    timestamp: snapshot.timestamp,
                    gpus: snapshot.gpus.clone(),
                    processes: snapshot.processes.clone(),
                    status: node_info.status.clone(),
                };

                node_snapshots.push(node_snapshot);

                total_gpus += snapshot.gpus.len() as u32;
                for gpu in &snapshot.gpus {
                    total_memory_gb += gpu.mem_total_mb as f32 / 1024.0;
                    total_utilization += gpu.util_pct;
                    gpu_count += 1;
                }
                active_processes += snapshot.processes.len() as u32;
            }
        }

        let utilization_avg = if gpu_count > 0 {
            total_utilization / gpu_count as f32
        } else {
            0.0
        };

        Ok(ClusterSnapshot {
            timestamp: Utc::now(),
            nodes: node_snapshots,
            total_gpus,
            total_memory_gb,
            active_processes,
            utilization_avg,
        })
    }

    /// Update cluster snapshot and cache it
    pub async fn update_cluster_snapshot(&self) -> Result<()> {
        let snapshot = self.build_cluster_snapshot().await?;
        let mut cached = self.last_cluster_snapshot.write().await;
        *cached = Some(snapshot);
        Ok(())
    }

    /// Get contention analysis (Magic Moment)
    pub async fn get_contention_analysis(&self) -> Result<ContentionAnalysis> {
        let snapshots = self.snapshots.read().await;
        let mut blocked_gpus = Vec::new();
        let mut user_stats: HashMap<String, (u32, u32, f32, u32)> = HashMap::new(); // (gpu_count, memory, util, processes)

        for (node_id, snapshot) in snapshots.iter() {
            for gpu in &snapshot.gpus {
                // Find processes using this GPU
                let gpu_processes: Vec<GpuProc> = snapshot
                    .processes
                    .iter()
                    .filter(|p| p.gpu_index == gpu.gpu_index)
                    .cloned()
                    .collect();

                // Check if GPU is blocked (high utilization or memory usage)
                let is_blocked =
                    gpu.util_pct > 80.0 || (gpu.mem_used_mb as f32 / gpu.mem_total_mb as f32) > 0.8;

                if is_blocked && !gpu_processes.is_empty() {
                    blocked_gpus.push(BlockedGpu {
                        node_id: node_id.clone(),
                        gpu_index: gpu.gpu_index,
                        gpu_name: gpu.name.clone(),
                        blocking_processes: gpu_processes.clone(),
                        utilization_pct: gpu.util_pct,
                        memory_used_mb: gpu.mem_used_mb,
                        memory_total_mb: gpu.mem_total_mb,
                    });
                }

                // Aggregate user statistics
                for process in &gpu_processes {
                    let entry = user_stats
                        .entry(process.user.clone())
                        .or_insert((0, 0, 0.0, 0));
                    entry.0 += 1; // gpu_count
                    entry.1 += process.used_mem_mb; // memory
                    entry.2 += gpu.util_pct; // utilization (will average later)
                    entry.3 += 1; // process_count
                }
            }
        }

        // Convert user stats to UserUsage
        let mut top_users: Vec<UserUsage> = user_stats
            .into_iter()
            .map(
                |(user, (gpu_count, total_memory_mb, total_util, process_count))| UserUsage {
                    user,
                    gpu_count,
                    total_memory_mb,
                    avg_utilization: total_util / process_count as f32,
                    process_count,
                },
            )
            .collect();

        // Sort by memory usage
        top_users.sort_by(|a, b| b.total_memory_mb.cmp(&a.total_memory_mb));
        top_users.truncate(10);

        // Generate recommendations
        let mut recommendations = Vec::new();
        if !blocked_gpus.is_empty() {
            recommendations.push(format!(
                "{} GPUs are currently blocked by high utilization",
                blocked_gpus.len()
            ));
        }
        if let Some(top_user) = top_users.first() {
            recommendations.push(format!(
                "User '{}' is using the most GPU memory ({} MB)",
                top_user.user, top_user.total_memory_mb
            ));
        }

        Ok(ContentionAnalysis {
            blocked_gpus,
            top_users,
            recommendations,
        })
    }

    /// Clean up stale nodes (offline for more than 5 minutes)
    pub async fn cleanup_stale_nodes(&self) -> Result<()> {
        let cutoff = Utc::now() - chrono::Duration::minutes(5);
        let mut nodes = self.nodes.write().await;
        let mut snapshots = self.snapshots.write().await;

        let stale_nodes: Vec<String> = nodes
            .iter()
            .filter(|(_, node)| node.last_seen < cutoff)
            .map(|(id, _)| id.clone())
            .collect();

        for node_id in stale_nodes {
            nodes.remove(&node_id);
            snapshots.remove(&node_id);
        }

        Ok(())
    }
}

/// Create the coordinator API router
pub fn create_router(state: CoordinatorState) -> Router {
    Router::new()
        .route("/api/nodes", get(get_nodes))
        .route("/api/nodes/:node_id/register", post(register_node))
        .route("/api/nodes/:node_id/snapshot", post(update_snapshot))
        .route("/api/cluster/snapshot", get(get_cluster_snapshot))
        .route("/api/cluster/contention", get(get_contention_analysis))
        .route("/api/cluster/rogue", get(get_rogue_analysis))
        .route("/api/cluster/rogue/test", get(get_rogue_analysis_test))
        .route("/api/guard/config", get(get_guard_config))
        .route("/api/guard/config", post(update_guard_config))
        .route("/api/guard/policies", get(get_guard_policies))
        .route("/api/guard/policies", post(update_guard_policies))
        .route("/api/guard/status", get(get_guard_status))
        .route("/api/guard/toggle-dry-run", post(toggle_guard_dry_run))
        .route("/api/guard/test-policies", post(test_guard_policies))
        .route("/ws", get(websocket_handler))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// Get all nodes
async fn get_nodes(State(state): State<CoordinatorState>) -> Json<Vec<NodeInfo>> {
    let nodes = state.get_nodes().await;
    Json(nodes)
}

/// Register a new node
async fn register_node(
    State(state): State<CoordinatorState>,
    Path(_node_id): Path<String>,
    Json(node_info): Json<NodeInfo>,
) -> Result<Json<()>, StatusCode> {
    state
        .register_node(node_info)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(()))
}

/// Update node snapshot
async fn update_snapshot(
    State(state): State<CoordinatorState>,
    Path(node_id): Path<String>,
    Json(snapshot): Json<NodeSnapshot>,
) -> Result<Json<()>, StatusCode> {
    state
        .update_snapshot(node_id, snapshot)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(()))
}

/// Get cluster snapshot
async fn get_cluster_snapshot(
    State(state): State<CoordinatorState>,
) -> Json<Option<ClusterSnapshot>> {
    let snapshot = state.get_cluster_snapshot().await;
    Json(snapshot)
}

/// Get contention analysis (Magic Moment)
async fn get_contention_analysis(
    State(state): State<CoordinatorState>,
) -> Result<Json<ContentionAnalysis>, StatusCode> {
    let analysis = state
        .get_contention_analysis()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(analysis))
}

/// Get rogue activity analysis
async fn get_rogue_analysis(
    State(_state): State<CoordinatorState>,
) -> Result<Json<crate::rogue_detection::RogueDetectionResult>, StatusCode> {
    use crate::audit::AuditManager;
    use crate::rogue_detection::RogueDetector;

    // Initialize audit manager and rogue detector
    let audit_manager = AuditManager::new()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let detector = RogueDetector::new(audit_manager);
    let result = detector
        .detect_rogue_activity(24)
        .await // Default to 24 hours
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(result))
}

/// Get test rogue activity analysis with sample data
async fn get_rogue_analysis_test(
) -> Result<Json<crate::rogue_detection::RogueDetectionResult>, StatusCode> {
    use crate::nvml_api::GpuProc;
    use crate::rogue_detection::{
        AbuseType, CryptoMiner, ResourceAbuser, RiskLevel, RogueDetectionResult, SuspiciousProcess,
    };
    use chrono::Utc;

    let test_result = RogueDetectionResult {
        timestamp: Utc::now(),
        suspicious_processes: vec![SuspiciousProcess {
            process: GpuProc {
                gpu_index: 0,
                pid: 12345,
                user: "hacker".to_string(),
                proc_name: "suspicious_miner".to_string(),
                used_mem_mb: 2048,
                start_time: "2025-09-20T01:00:00Z".to_string(),
                container: None,
            },
            reasons: vec![
                "High GPU utilization with low CPU usage".to_string(),
                "Process name contains mining keywords".to_string(),
                "Unusual memory allocation patterns".to_string(),
            ],
            confidence: 0.85,
            risk_level: RiskLevel::High,
        }],
        crypto_miners: vec![CryptoMiner {
            process: GpuProc {
                gpu_index: 1,
                pid: 67890,
                user: "miner".to_string(),
                proc_name: "xmrig".to_string(),
                used_mem_mb: 1024,
                start_time: "2025-09-20T00:30:00Z".to_string(),
                container: None,
            },
            mining_indicators: vec![
                "Known cryptocurrency mining software".to_string(),
                "Extremely high GPU utilization".to_string(),
                "Long-running process with consistent resource usage".to_string(),
            ],
            confidence: 0.92,
            estimated_hashrate: Some(150.5),
        }],
        resource_abusers: vec![ResourceAbuser {
            process: GpuProc {
                gpu_index: 2,
                pid: 11111,
                user: "abuser".to_string(),
                proc_name: "gpu_hog".to_string(),
                used_mem_mb: 8192,
                start_time: "2025-09-19T20:00:00Z".to_string(),
                container: None,
            },
            abuse_type: AbuseType::MemoryHog,
            severity: 0.9,
            duration_hours: 8.5,
        }],
        data_exfiltrators: vec![],
        risk_score: 0.78,
        recommendations: vec![
            "🚨 Immediate action required: Terminate crypto mining processes".to_string(),
            "⚠️ Review user 'miner' and 'hacker' accounts for unauthorized access".to_string(),
            "🔍 Investigate process 'gpu_hog' for potential resource abuse".to_string(),
            "📊 Consider implementing GPU usage quotas per user".to_string(),
        ],
    };

    Ok(Json(test_result))
}

/// WebSocket handler for real-time updates
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<CoordinatorState>,
) -> axum::response::Response {
    ws.on_upgrade(|socket| websocket_connection(socket, state))
}

/// Handle WebSocket connection
async fn websocket_connection(socket: axum::extract::ws::WebSocket, state: CoordinatorState) {
    use axum::extract::ws::Message;
    use futures_util::{sink::SinkExt, stream::StreamExt};

    let (mut sender, mut receiver) = socket.split();

    // Send initial cluster snapshot
    if let Some(snapshot) = state.get_cluster_snapshot().await {
        if let Ok(json) = serde_json::to_string(&snapshot) {
            let _ = sender.send(Message::Text(json)).await;
        }
    }

    // Handle incoming messages and send periodic updates
    let mut interval = tokio::time::interval(Duration::from_secs(5));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                // Send updated cluster snapshot
                if let Some(snapshot) = state.get_cluster_snapshot().await {
                    if let Ok(json) = serde_json::to_string(&snapshot) {
                        let _ = sender.send(Message::Text(json)).await;
                    }
                }
            }
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Close(_))) => break,
                    Some(Ok(Message::Ping(data))) => {
                        let _ = sender.send(Message::Pong(data)).await;
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Get Guard Mode configuration
async fn get_guard_config(
    State(_state): State<CoordinatorState>,
) -> Result<Json<crate::guard_mode::GuardModeConfig>, StatusCode> {
    use crate::guard_mode::GuardModeManager;

    let guard_manager = GuardModeManager::new().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let config = guard_manager.get_config();
    Ok(Json(config.clone()))
}

/// Update Guard Mode configuration
async fn update_guard_config(
    State(_state): State<CoordinatorState>,
    Json(config): Json<crate::guard_mode::GuardModeConfig>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    use crate::guard_mode::GuardModeManager;

    let mut guard_manager =
        GuardModeManager::new().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    guard_manager
        .update_config(config)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(
        serde_json::json!({"success": true, "message": "Guard Mode configuration updated"}),
    ))
}

/// Get Guard Mode policies
async fn get_guard_policies(
    State(_state): State<CoordinatorState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    use crate::guard_mode::GuardModeManager;

    let guard_manager = GuardModeManager::new().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let config = guard_manager.get_config();

    let policies = serde_json::json!({
        "user_policies": config.user_policies,
        "group_policies": config.group_policies,
        "gpu_policies": config.gpu_policies,
        "time_policies": config.time_policies,
        "enforcement": config.enforcement
    });

    Ok(Json(policies))
}

/// Update Guard Mode policies
async fn update_guard_policies(
    State(_state): State<CoordinatorState>,
    Json(policies): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    use crate::guard_mode::GuardModeManager;

    let mut guard_manager =
        GuardModeManager::new().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Parse and update policies
    if let Some(user_policies) = policies.get("user_policies") {
        if let Ok(user_policies_map) = serde_json::from_value::<
            std::collections::HashMap<String, crate::guard_mode::UserPolicy>,
        >(user_policies.clone())
        {
            for (_, policy) in user_policies_map {
                guard_manager
                    .add_user_policy(policy)
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            }
        }
    }

    Ok(Json(
        serde_json::json!({"success": true, "message": "Policies updated"}),
    ))
}

/// Get Guard Mode status
async fn get_guard_status(
    State(_state): State<CoordinatorState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    use crate::guard_mode::GuardModeManager;

    let guard_manager = GuardModeManager::new().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let config = guard_manager.get_config();
    let violation_history = guard_manager.get_violation_history();
    let warning_history = guard_manager.get_warning_history();

    let status = serde_json::json!({
        "enabled": config.global.enabled,
        "dry_run": config.global.dry_run,
        "soft_enforcement": config.enforcement.soft_enforcement,
        "hard_enforcement": config.enforcement.hard_enforcement,
        "total_violations": violation_history.len(),
        "total_warnings": warning_history.len(),
        "recent_violations": violation_history.iter().rev().take(10).collect::<Vec<_>>(),
        "recent_warnings": warning_history.iter().rev().take(10).collect::<Vec<_>>(),
        "user_policy_count": config.user_policies.len(),
        "group_policy_count": config.group_policies.len(),
        "gpu_policy_count": config.gpu_policies.len()
    });

    Ok(Json(status))
}

/// Toggle Guard Mode dry-run
async fn toggle_guard_dry_run(
    State(_state): State<CoordinatorState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    use crate::guard_mode::GuardModeManager;

    let mut guard_manager =
        GuardModeManager::new().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let new_dry_run = guard_manager
        .toggle_dry_run()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "dry_run": new_dry_run,
        "message": format!("Dry-run mode {}", if new_dry_run { "enabled" } else { "disabled" })
    })))
}

/// Test Guard Mode policies
async fn test_guard_policies(
    State(_state): State<CoordinatorState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    use crate::guard_mode::GuardModeManager;
    use crate::vendor::GpuManager;

    let mut guard_manager =
        GuardModeManager::new().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get current GPU processes for testing
    let gpu_manager = GpuManager::initialize().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let test_processes = gpu_manager
        .get_all_processes()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result = guard_manager
        .simulate_policy_check(&test_processes)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "simulation_result": {
            "violations": result.violations,
            "warnings": result.warnings,
            "actions_taken": result.actions_taken,
            "dry_run": result.dry_run,
            "timestamp": result.timestamp
        },
        "summary": {
            "violation_count": result.violations.len(),
            "warning_count": result.warnings.len(),
            "action_count": result.actions_taken.len()
        }
    })))
}
