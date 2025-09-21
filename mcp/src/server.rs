//! MCP Server implementation for GPU Kill

use crate::resources::ResourceHandler;
use crate::tools::ToolHandler;
use crate::types::*;
use crate::MCP_VERSION;
use anyhow::Result;
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
    pub async fn handle_request(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        debug!("Handling MCP request: {}", request.method);

        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params).await,
            "resources/list" => self.handle_resources_list().await,
            "resources/read" => self.handle_resources_read(request.params).await,
            "tools/list" => self.handle_tools_list().await,
            "tools/call" => self.handle_tools_call(request.params).await,
            _ => Err(anyhow::anyhow!("Unknown method: {}", request.method)),
        };

        match result {
            Ok(data) => Ok(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(data),
                error: None,
            }),
            Err(e) => {
                error!("Error handling request {}: {}", request.method, e);
                Ok(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32603,
                        message: "Internal error".to_string(),
                        data: Some(json!({ "details": e.to_string() })),
                    }),
                })
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
        Ok(json!({ "contents": contents }))
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

    /// Start the MCP server
    pub async fn start(self, port: u16) -> Result<()> {
        info!("Starting GPU Kill MCP Server on port {}", port);

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
                                Ok(response) => axum::response::Json(response),
                                Err(e) => {
                                    error!("Failed to handle HTTP request: {}", e);
                                    axum::response::Json(JsonRpcResponse {
                                        jsonrpc: "2.0".to_string(),
                                        id: "error".to_string(),
                                        result: None,
                                        error: Some(JsonRpcError {
                                            code: -32603,
                                            message: "Internal error".to_string(),
                                            data: Some(json!({ "details": e.to_string() })),
                                        }),
                                    })
                                }
                            }
                        }
                    }
                }),
            )
            .route("/health", axum::routing::get(|| async { "OK" }));

        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
        info!("MCP Server listening on http://0.0.0.0:{}", port);

        axum::serve(listener, app).await?;
        Ok(())
    }
}

// Remove Default implementation since new() is now async
