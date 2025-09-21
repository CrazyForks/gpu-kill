//! GPU Kill MCP Server
//!
//! This module provides a Model Context Protocol (MCP) server for GPU Kill,
//! enabling AI assistants and other tools to interact with GPU management
//! functionality through a standardized interface.

pub mod resources;
pub mod server;
pub mod tools;
pub mod types;

pub use server::GpuKillMCPServer;
pub use types::*;

/// MCP Server version
pub const MCP_VERSION: &str = "2024-11-05";

/// GPU Kill MCP Server capabilities
pub const CAPABILITIES: &[&str] = &["resources", "tools", "logging"];
