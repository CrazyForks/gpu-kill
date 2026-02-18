//! MCP Server implementation for GPU Kill

use crate::resources::ResourceHandler;
use crate::tools::ToolHandler;
use crate::types::*;
use crate::MCP_VERSION;
use anyhow::Result;
use axum::response::IntoResponse;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

/// GPU Kill MCP Server
pub struct GpuKillMCPServer {
    resource_handler: Arc<ResourceHandler>,
    tool_handler: Arc<RwLock<ToolHandler>>,
}

impl GpuKillMCPServer {
    /// Create a new MCP server instance
    pub async fn new() -> Result<Self> {
        let resource_handler = Arc::new(ResourceHandler::new().await?);
        let tool_handler = Arc::new(RwLock::new(ToolHandler::new().await?));

        Ok(Self {
            resource_handler,
            tool_handler,
        })
    }

    /// Handle an MCP request
    pub async fn handle_request(&self, request: JsonRpcRequest) -> Result<Option<JsonRpcResponse>> {
        debug!("Handling MCP request: {}", request.method);
        let request_id = request.id.clone();

        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params).await,
            "resources/list" => self.handle_resources_list().await,
            "resources/read" => self.handle_resources_read(request.params).await,
            "tools/list" => self.handle_tools_list().await,
            "tools/call" => self.handle_tools_call(request.params).await,
            _ => Err(anyhow::anyhow!("Unknown method: {}", request.method)),
        };

        match result {
            Ok(data) => Ok(request_id.map(|id| JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(data),
                error: None,
            })),
            Err(e) => {
                error!("Error handling request {}: {}", request.method, e);
                Ok(request_id.map(|id| JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32603,
                        message: "Internal error".to_string(),
                        data: Some(json!({ "details": e.to_string() })),
                    }),
                }))
            }
        }
    }

    async fn handle_initialize(
        &self,
        _params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        info!("MCP client initializing");

        let response = InitializeResponse {
            protocol_version: MCP_VERSION.to_string(),
            capabilities: ServerCapabilities {
                resources: Some(ResourcesCapability {
                    subscribe: Some(false),
                    list_changed: Some(false),
                }),
                tools: Some(ToolsCapability {
                    list_changed: Some(false),
                }),
                logging: Some(LoggingCapability {}),
            },
            server_info: ServerInfo {
                name: "GPU Kill MCP Server".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        Ok(serde_json::to_value(response)?)
    }

    async fn handle_resources_list(&self) -> Result<serde_json::Value> {
        let resources = self.resource_handler.list_resources();
        Ok(json!({ "resources": resources }))
    }

    async fn handle_resources_read(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let params = params.ok_or_else(|| anyhow::anyhow!("Missing parameters"))?;
        let uri = params
            .get("uri")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing uri parameter"))?;

        let contents = self.resource_handler.get_resource(uri).await?;
        Ok(json!({ "contents": [contents] }))
    }

    async fn handle_tools_list(&self) -> Result<serde_json::Value> {
        let tool_handler = self.tool_handler.read().await;
        let tools = tool_handler.list_tools();
        Ok(json!({ "tools": tools }))
    }

    async fn handle_tools_call(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let params = params.ok_or_else(|| anyhow::anyhow!("Missing parameters"))?;
        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing name parameter"))?;

        let arguments = params
            .get("arguments")
            .and_then(|v| v.as_object())
            .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect());

        let mut tool_handler = self.tool_handler.write().await;
        let result = tool_handler.execute_tool(name, arguments).await?;

        Ok(json!({ "content": result.content, "isError": result.is_error }))
    }

    /// Start the MCP server.
    ///
    /// Binds to `host:port`. Use `127.0.0.1` for local-only access (default).
    /// Use `0.0.0.0` only when you need remote access and have other protections (e.g. auth, firewall).
    pub async fn start(self, host: &str, port: u16) -> Result<()> {
        let bind_addr = format!("{}:{}", host, port);
        info!("Starting GPU Kill MCP Server on {}", bind_addr);

        let server = Arc::new(self);

        // For now, we'll implement a simple HTTP-based MCP server
        // In a full implementation, this would use stdio or WebSocket transport
        let app = axum::Router::new()
            .route(
                "/mcp",
                axum::routing::post({
                    let server = server.clone();
                    move |request: axum::extract::Json<JsonRpcRequest>| {
                        let server = server.clone();
                        async move {
                            match server.handle_request(request.0).await {
                                Ok(Some(response)) => {
                                    axum::response::Json(response).into_response()
                                }
                                Ok(None) => axum::http::StatusCode::NO_CONTENT.into_response(),
                                Err(e) => {
                                    error!("Failed to handle HTTP request: {}", e);
                                    axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
                                }
                            }
                        }
                    }
                }),
            )
            .route("/health", axum::routing::get(|| async { "OK" }));

        let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
        info!("MCP Server listening on http://{}", bind_addr);

        axum::serve(listener, app).await?;
        Ok(())
    }
}

// Remove Default implementation since new() is now async
