# GPU Kill MCP Server

A MCP server for GPU Kill, enabling AI assistants and other tools to interact with GPU management functionality through a standardized interface.

## Features

### Resources (Read-only data)
- **gpu://list** - Current GPU status and utilization
- **gpu://processes** - Currently running GPU processes  
- **gpu://audit** - Historical GPU usage data
- **gpu://policies** - Current Guard Mode policies
- **gpu://rogue-detection** - Security scan results and threats

### Tools (Actions)
- **kill_gpu_process** - Kill a GPU process by PID
- **reset_gpu** - Reset a GPU by ID
- **scan_rogue_activity** - Scan for suspicious GPU activity
- **create_user_policy** - Create a user policy for Guard Mode
- **get_gpu_status** - Get detailed status of a specific GPU
- **kill_processes_by_name** - Kill all processes matching a name pattern

## Quick Start

### Build and Run

```bash
# Build the MCP server
cargo build --release -p gpukill-mcp

# Run the MCP server
cargo run --release -p gpukill-mcp

# Or run with custom port
MCP_PORT=3001 cargo run --release -p gpukill-mcp
```

### Using with AI Assistants

The MCP server exposes GPU management capabilities through a JSON-RPC interface that AI assistants can use to:

- Monitor GPU usage and performance
- Kill stuck or problematic processes
- Reset crashed GPUs
- Scan for security threats
- Manage resource policies
- Automate GPU operations

### Example Usage

```bash
# Start the MCP server
cargo run --release -p gpukill-mcp

# The server will be available at http://localhost:3001/mcp
# AI assistants can connect and use the available tools and resources
```

### Natural Language Examples

Ask your AI assistant to use the MCP tools with natural language:

```text
What GPUs do I have and what's their current usage?
```

```text
Kill the Python process that's stuck on GPU 0
```

```text
Kill all training processes that are using too much GPU memory
```

```text
Show me GPU usage and kill any stuck processes
```

```text
Scan for crypto miners and suspicious activity
```

```text
Create a policy to limit user memory usage to 8GB
```

```text
Reset GPU 1 because it's not responding
```

```text
What processes are currently using my GPUs?
```

## API Endpoints

### HTTP Interface

- **POST /mcp** - Main MCP JSON-RPC endpoint
- **GET /health** - Health check endpoint

### MCP Methods

- **initialize** - Initialize the MCP connection
- **resources/list** - List available resources
- **resources/read** - Read resource contents
- **tools/list** - List available tools
- **tools/call** - Execute a tool

## Configuration

The MCP server can be configured using environment variables:

- **MCP_PORT** - Port to listen on (default: 3001)
- **RUST_LOG** - Logging level (default: info)

## Integration

This MCP server enables AI assistants to:

1. **Monitor GPU Health**: Check GPU status, utilization, and memory usage
2. **Manage Processes**: Kill problematic processes or reset GPUs
3. **Security Monitoring**: Scan for crypto miners and suspicious activity
4. **Policy Management**: Create and manage resource policies
5. **Automation**: Automate routine GPU management tasks

## Development

```bash
# Run in development mode
cargo run -p gpukill-mcp

# Run with debug logging
RUST_LOG=debug cargo run -p gpukill-mcp

# Test the server
curl -X POST http://localhost:3001/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":"1","method":"tools/list","params":{}}'
```
