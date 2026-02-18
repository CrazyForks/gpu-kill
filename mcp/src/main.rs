//! GPU Kill MCP Server - Main entry point

use gpukill_mcp::GpuKillMCPServer;
use std::env;
use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting GPU Kill MCP Server");

    // Bind to 127.0.0.1 by default so the server is not reachable from the network.
    // Set MCP_HOST=0.0.0.0 only if you need remote access and have other protections.
    let host = env::var("MCP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

    let port = env::var("MCP_PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse::<u16>()
        .unwrap_or(3001);

    // Create and start the MCP server
    let server = GpuKillMCPServer::new().await?;

    info!("GPU Kill MCP Server initialized successfully");
    info!("Available resources:");
    info!("  - gpu://list - Current GPU status and utilization");
    info!("  - gpu://processes - Currently running GPU processes");
    info!("  - gpu://audit - Historical GPU usage data");
    info!("  - gpu://policies - Current Guard Mode policies");
    info!("  - gpu://rogue-detection - Security scan results");

    info!("Available tools:");
    info!("  - kill_gpu_process - Kill a GPU process by PID");
    info!("  - reset_gpu - Reset a GPU by ID");
    info!("  - scan_rogue_activity - Scan for suspicious GPU activity");
    info!("  - create_user_policy - Create a user policy for Guard Mode");
    info!("  - get_gpu_status - Get detailed status of a specific GPU");
    info!("  - kill_processes_by_name - Kill all processes matching a name pattern");

    // Start the server
    if let Err(e) = server.start(host.as_str(), port).await {
        error!("Failed to start MCP server: {}", e);
        return Err(e);
    }

    Ok(())
}
