# GPU Kill - Detailed Documentation

## Table of Contents

- [Architecture](#architecture)
- [Installation](#installation)
- [Command Reference](#command-reference)
- [Enhanced Features](#enhanced-features)
- [Audit System](#audit-system)
- [Suspicious Usage Detection](#suspicious-usage-detection)
- [Guard Mode](#guard-mode)
- [Cluster Management](#cluster-management)
- [Remote Operations](#remote-operations)
- [Dashboard](#dashboard)
- [MCP Server](#mcp-server)
- [Output Formats](#output-formats)
- [Configuration](#configuration)
- [Safety Features](#safety-features)
- [Exit Codes](#exit-codes)
- [Troubleshooting](#troubleshooting)
- [Development](#development)

## Architecture

GPU Kill is built with Rust and supports NVIDIA, AMD, Intel, and Apple Silicon GPUs through vendor-specific interfaces. The tool is designed with safety, usability, multi-vendor support, and distributed cluster management in mind.

### Core Components

- **CLI Parser**: Uses `clap` for robust argument parsing and validation
- **Vendor Abstraction**: Multi-vendor GPU support (NVIDIA, AMD, Intel, Apple Silicon)
- **NVML Wrapper**: Interfaces with NVIDIA's management library
- **ROCm Interface**: AMD GPU management via rocm-smi
- **Intel GPU Tools**: Intel GPU management via intel_gpu_top
- **Apple Silicon Interface**: Apple Silicon GPU management via system_profiler and system APIs
- **Enhanced Process Manager**: Advanced process filtering and batch operations
- **Container Detection**: Container-aware process management
- **Audit System**: Automatic GPU usage tracking and historical analysis
- **Renderer**: Formats output as tables or JSON
- **Configuration**: Supports file and environment-based configuration
- **Coordinator API**: RESTful API server for cluster management
- **SSH Remote Manager**: Secure remote GPU management via SSH
- **MCP Server**: Model Context Protocol server for AI assistant integration

### Dependencies

- `clap`: Command-line argument parsing
- `nvml-wrapper`: NVIDIA GPU management
- `sysinfo`: System process information
- `regex`: Process filtering with regex patterns
- `tabled`: Table formatting
- `serde`: JSON serialization
- `serde_json`: JSON parsing and generation
- `chrono`: Date and time handling for audit timestamps
- `dirs`: Cross-platform data directory management
- `tracing`: Structured logging
- `color-eyre`: Error handling
- `axum`: HTTP server framework for coordinator API
- `tower`: HTTP middleware and services
- `tower-http`: HTTP middleware (CORS, tracing)
- `uuid`: Unique identifier generation for nodes
- `jsonrpc-core`: JSON-RPC protocol implementation for MCP server
- `jsonrpc-ws-server`: WebSocket JSON-RPC server for MCP
- `futures-util`: Async utilities for WebSocket handling
- `tokio`: Async runtime for HTTP server and WebSocket

## Installation

### Prerequisites

- **NVIDIA GPU Support**:
  - NVIDIA GPU with supported drivers
  - NVIDIA Management Library (NVML) - included with NVIDIA drivers
- **AMD GPU Support**:
  - AMD GPU with ROCm drivers installed
  - rocm-smi command-line tool available
- **Intel GPU Support**:
  - Intel GPU with intel-gpu-tools package installed
  - intel_gpu_top command-line tool available
- **Apple Silicon GPU Support**:
  - macOS with Apple Silicon (M1, M2, M3, M4)
  - system_profiler command-line tool available
- **General Requirements**:
  - Rust 1.70+ (for building from source)
  - Linux, macOS, or Windows

### Install from Cargo

```bash
cargo install gpukill
```

### Build from Source

```bash
git clone https://github.com/treadiehq/gpu-kill.git
cd gpukill
cargo build --release
```

### Cross-compilation

The project supports cross-compilation for different platforms:

```bash
# For Linux from macOS
cargo build --release --target x86_64-unknown-linux-gnu

# For Windows from Linux
cargo build --release --target x86_64-pc-windows-gnu
```


## Command Reference

### Global Options

| Option | Description | Default |
|--------|-------------|---------|
| `--log-level <LEVEL>` | Set logging level | `info` |
| `--config <PATH>` | Configuration file path | None |
| `--remote <HOST>` | Remote host to connect to via SSH | None |
| `--ssh-user <USER>` | SSH username (requires --remote) | Current user |
| `--ssh-port <PORT>` | SSH port (requires --remote) | `22` |
| `--ssh-key <PATH>` | SSH private key path (requires --remote) | None |
| `--ssh-password <PASSWORD>` | SSH password (requires --remote) | Interactive prompt |
| `--ssh-timeout <SECONDS>` | SSH connection timeout (requires --remote) | `30` |
| `--register-node <URL>` | Register this node with a coordinator | None |
| `--help` | Show help information | - |
| `--version` | Show version information | - |

### List Operation

```bash
gpukill --list [OPTIONS]
```

**Options:**
- `--details`: Show detailed per-process information
- `--watch`: Refresh output every 2 seconds until Ctrl-C
- `--output <FORMAT>`: Output format (`table` or `json`)
- `--vendor <VENDOR>`: Filter by GPU vendor (`nvidia`, `amd`, `intel`, `apple`, `all`)

**Examples:**
```bash
# Basic listing
gpukill --list

# With process details
gpukill --list --details

# Watch mode
gpukill --list --watch

# JSON output
gpukill --list --output json

# Combined options
gpukill --list --details --watch --output json

# Filter by vendor
gpukill --list --vendor nvidia
gpukill --list --vendor amd --details
gpukill --list --vendor apple --watch
```

### Kill Operation

```bash
gpukill --kill (--pid <PID> | --filter <PATTERN>) [OPTIONS]
```

**Required (one of):**
- `--pid <PID>`: Process ID to terminate
- `--filter <PATTERN>`: Filter processes by name pattern (supports regex)

**Options:**
- `--timeout-secs <SECONDS>`: Timeout before escalation (default: 5)
- `--force`: Escalate to SIGKILL after timeout
- `--batch`: Kill multiple processes matching the filter (requires `--filter`)

**Examples:**
```bash
# Graceful termination of a single process
gpukill --kill --pid 12345

# Custom timeout for a single process
gpukill --kill --pid 12345 --timeout-secs 10

# Force escalation for a single process
gpukill --kill --pid 12345 --force

# Kill processes matching a pattern
gpukill --kill --filter "python.*"

# Batch kill all processes matching a pattern
gpukill --kill --filter "python.*" --batch --force
```

### Reset Operation

```bash
gpukill --reset [--gpu <ID> | --all] [OPTIONS]
```

**Required (one of):**
- `--gpu <ID>`: Specific GPU ID to reset
- `--all`: Reset all GPUs

**Options:**
- `--force`: Force reset even with active processes

**Examples:**
```bash
# Reset specific GPU
gpukill --reset --gpu 0

# Reset all GPUs
gpukill --reset --all

# Force reset
gpukill --reset --gpu 0 --force
```

### Audit Operation

```bash
gpukill --audit [OPTIONS]
```

**Options:**
- `--audit-user <USER>`: Filter by specific user
- `--audit-process <PATTERN>`: Filter by process name pattern
- `--audit-hours <HOURS>`: Show records from last N hours (default: 24)
- `--audit-summary`: Show summary statistics instead of detailed records

### Suspicious Usage Detection

```bash
gpukill --audit --rogue [OPTIONS]
```

**Detection Options:**
- `--rogue`: Perform rogue activity detection
- `--rogue-config`: Show current detection configuration
- `--rogue-memory-threshold <GB>`: Set memory usage threshold
- `--rogue-utilization-threshold <PERCENT>`: Set GPU utilization threshold
- `--rogue-duration-threshold <HOURS>`: Set process duration threshold
- `--rogue-confidence-threshold <CONFIDENCE>`: Set minimum confidence for detection

**Whitelist Management:**
- `--rogue-whitelist-process <NAME>`: Add process to whitelist
- `--rogue-unwhitelist-process <NAME>`: Remove process from whitelist
- `--rogue-whitelist-user <USERNAME>`: Add user to whitelist
- `--rogue-unwhitelist-user <USERNAME>`: Remove user from whitelist

**Configuration Management:**
- `--rogue-export-config`: Export configuration to JSON
- `--rogue-import-config <FILE>`: Import configuration from JSON file

**Examples:**
```bash
# Detect suspicious activity
gpukill --audit --rogue --audit-hours 48

# View current configuration
gpukill --audit --rogue-config

# Update detection thresholds
gpukill --audit --rogue-memory-threshold 15.0 --rogue-utilization-threshold 90.0

# Manage whitelists
gpukill --audit --rogue-whitelist-process "my-app"
gpukill --audit --rogue-whitelist-user "developer"

# Export/import configuration
gpukill --audit --rogue-export-config > config.json
gpukill --audit --rogue-import-config config.json
```

### Server Operation

```bash
gpukill --server [OPTIONS]
```

**Options:**
- `--server-port <PORT>`: Port for coordinator API (default: 8080)
- `--server-host <HOST>`: Host to bind coordinator API (default: 0.0.0.0)

**Description:**
Starts the GPU Kill coordinator server that provides:
- RESTful API for cluster management
- WebSocket server for real-time updates
- Node registration and heartbeat management
- Magic Moment contention analysis

**Examples:**
```bash
# Start coordinator on default port 8080
gpukill --server

# Start coordinator on custom port
gpukill --server --server-port 9000

# Start coordinator on all interfaces
gpukill --server --server-host 0.0.0.0
```

### Node Registration Operation

```bash
gpukill --register-node <COORDINATOR_URL>
```

**Description:**
Registers this node with a coordinator server for cluster management:
- Generates unique node ID
- Sends periodic GPU snapshots to coordinator
- Maintains heartbeat for health monitoring
- Enables cluster-wide monitoring and management

**Examples:**
```bash
# Register with default coordinator
gpukill --register-node http://coordinator:8080

# Register with custom coordinator
gpukill --register-node http://gpu-cluster:9000

# Register with HTTPS coordinator
gpukill --register-node https://secure-cluster:8443
```

## Enhanced Features

### Multi-Vendor Support

`gpukill` automatically detects and utilizes available GPU vendors (NVIDIA, AMD, Intel). You can also filter the displayed information by vendor.

**Options:**
- `--vendor <VENDOR>`: Filter by GPU vendor.
  - `nvidia`: Show only NVIDIA GPUs.
  - `amd`: Show only AMD GPUs.
  - `intel`: Show only Intel GPUs.
  - `apple`: Show only Apple Silicon GPUs.
  - `all`: Show all detected GPUs (default if `--vendor` is not specified).

**Examples:**
```bash
# List only NVIDIA GPUs
gpukill --list --vendor nvidia

# List only AMD GPUs with details
gpukill --list --vendor amd --details

# List only Intel GPUs
gpukill --list --vendor intel

# Monitor all GPUs in watch mode
gpukill --list --vendor all --watch
```

**Vendor Detection:**
- **NVIDIA**: Automatically detected if NVML is available
- **AMD**: Automatically detected if rocm-smi is available
- **Intel**: Automatically detected if intel_gpu_top is available
- **Apple Silicon**: Automatically detected if running on macOS with Apple Silicon
- **Mixed Systems**: Supports systems with multiple GPU vendors

### Advanced Process Filtering

The `--kill` command now supports filtering processes by name using regular expressions, enabling powerful batch operations.

**Options:**
- `--filter <PATTERN>`: A regular expression pattern to match against process names.
- `--batch`: When used with `--filter`, all matching processes will be targeted for termination. Without `--batch`, `gpukill` will list matching processes and warn you to use `--batch` to proceed with killing.

**Examples:**
```bash
# List processes matching "python" (case-sensitive)
gpukill --list --details --filter "python"

# Kill all processes whose names start with "tensor"
gpukill --kill --filter "^tensor" --batch --force

# Find and kill all processes related to "jupyter"
gpukill --kill --filter "jupyter" --batch
```

### Container-Aware Process Detection

`gpukill` can now attempt to identify if a process is running within a container environment.

**Options:**
- `--containers`: When used with `--list`, an additional column or field will indicate if a process is running in a container (e.g., Docker, LXC, Kubernetes).

**Examples:**
```bash
# List GPUs and show container info for processes
gpukill --list --details --containers

# Watch containerized processes on NVIDIA GPUs
gpukill --list --watch --containers --vendor nvidia
```

### Apple Silicon Specific Features

Apple Silicon GPUs use unified memory architecture, which provides unique capabilities:

```bash
# Monitor Apple Silicon GPU
gpukill --list --vendor apple

# Monitor with details
gpukill --list --vendor apple --details

# Watch Apple Silicon GPU usage
gpukill --list --vendor apple --watch
```

**Apple Silicon Characteristics:**
- **Unified Memory**: GPU and CPU share the same memory pool
- **Memory Estimation**: GPU memory usage is estimated from active system memory
- **Process Detection**: Identifies Metal, OpenGL, and ML framework processes
- **No Temperature/Power**: These metrics are not available via system APIs
- **No Reset Support**: GPU reset requires kernel-level operations

## Audit System

The audit system automatically tracks GPU usage history whenever you run `gpukill --list`. This provides valuable insights into GPU utilization patterns, resource planning, and troubleshooting.

### How It Works

**Automatic Data Collection:**
- Every `gpukill --list` command automatically logs GPU usage data
- Data is stored in JSON Lines format for easy processing
- No additional configuration required - works out of the box

**Data Storage:**
- **Linux**: `~/.local/share/gpukill/audit.jsonl`
- **macOS**: `~/Library/Application Support/gpukill/audit.jsonl`
- **Windows**: `%APPDATA%\gpukill\audit.jsonl`

**Data Captured:**
- Timestamp of each GPU check
- GPU information (index, name, memory usage, utilization, temperature, power)
- Process information (when processes are using GPU)
- Container information (when available)
- User information (when processes are detected)

### Audit Commands

**Basic Audit Queries:**
```bash
# Show last 24 hours of GPU usage
gpukill --audit

# Show last 6 hours
gpukill --audit --audit-hours 6

# Show last 3 days
gpukill --audit --audit-hours 72
```

**Filtered Queries:**
```bash
# Show only specific user's GPU usage
gpukill --audit --audit-user john

# Show only specific process types
gpukill --audit --audit-process python
gpukill --audit --audit-process tensorflow

# Combine filters
gpukill --audit --audit-user alice --audit-process pytorch --audit-hours 12
```

**Summary Reports:**
```bash
# Get usage summary for last 24 hours
gpukill --audit --audit-summary

# Get summary for last week
gpukill --audit --audit-summary --audit-hours 168
```

## Suspicious Usage Detection

The suspicious usage detection system provides comprehensive security monitoring for GPU resources, detecting crypto miners, suspicious processes, and resource abuse patterns.

### Detection Capabilities

**Crypto Miner Detection:**
- Identifies known mining software (xmrig, ccminer, ethminer, etc.)
- Detects mining patterns in process names and behavior
- Analyzes high GPU utilization and sustained usage
- Provides confidence-based scoring for mining activity

**Suspicious Process Detection:**
- Flags unusual process names and patterns
- Detects excessive resource usage
- Identifies processes from unusual users
- Analyzes process behavior over time

**Resource Abuse Detection:**
- Memory hogs consuming excessive GPU memory
- Long-running processes that may be stuck
- Excessive GPU utilization patterns
- Unauthorized access attempts

**Risk Assessment:**
- Confidence-based threat scoring (0.0 - 1.0)
- Risk level classification (Low, Medium, High, Critical)
- Weighted scoring for different threat types
- Actionable recommendations for each threat

### Configuration System

**Configuration File:**
- **Location**: `~/.config/gpukill/rogue_config.toml`
- **Format**: TOML with comprehensive detection rules
- **Auto-creation**: Default configuration created on first use
- **Version tracking**: Metadata includes version and modification timestamps

**Detection Thresholds:**
```toml
[detection]
max_memory_usage_gb = 20.0        # Maximum memory usage threshold
max_utilization_pct = 95.0        # Maximum GPU utilization threshold
max_duration_hours = 24.0         # Maximum process duration threshold
min_confidence_threshold = 0.7    # Minimum confidence for detection
```

**Pattern Matching:**
```toml
[patterns]
crypto_miner_patterns = ["cuda", "opencl", "miner", "hash"]
suspicious_process_names = ["xmrig", "ccminer", "ethminer"]
user_whitelist = ["root", "admin", "system"]
process_whitelist = ["python", "jupyter", "tensorflow"]
```

**Risk Scoring:**
```toml
[scoring.threat_weights]
crypto_miner = 0.8
suspicious_process = 0.6
resource_abuser = 0.3
data_exfiltrator = 0.9

[scoring.risk_thresholds]
critical = 0.9
high = 0.7
medium = 0.5
low = 0.3
```

### Detection Examples

**Basic Detection:**
```bash
# Scan for suspicious activity in last 24 hours
gpukill --audit --rogue

# Scan last 48 hours with JSON output
gpukill --audit --rogue --audit-hours 48 --output json
```

**Configuration Management:**
```bash
# View current configuration
gpukill --audit --rogue-config

# Update thresholds
gpukill --audit --rogue-memory-threshold 15.0
gpukill --audit --rogue-utilization-threshold 90.0

# Manage whitelists
gpukill --audit --rogue-whitelist-process "my-app"
gpukill --audit --rogue-whitelist-user "developer"
```

**Configuration Export/Import:**
```bash
# Export configuration
gpukill --audit --rogue-export-config > security-config.json

# Import configuration
gpukill --audit --rogue-import-config security-config.json
```

**JSON Output:**
```bash
# Export audit data as JSON for external processing
gpukill --audit --output json

# Export filtered data
gpukill --audit --audit-user john --output json > john_gpu_usage.json
```


### Dashboard

The suspicious usage detection is fully integrated with the dashboard:

Check the [Kill Suite](https://treadiehq.com) website



### Audit Data Structure

Each audit record contains:

```json
{
  "id": 1758236888745,
  "timestamp": "2025-09-18T23:08:08.745114Z",
  "gpu_index": 0,
  "gpu_name": "Apple M3 Max",
  "pid": null,
  "user": null,
  "process_name": null,
  "memory_used_mb": 3216,
  "utilization_pct": 0.0,
  "temperature_c": 0,
  "power_w": 0.0,
  "container": null
}
```

**Field Descriptions:**
- `id`: Unique identifier (timestamp + process ID)
- `timestamp`: ISO 8601 timestamp of the measurement
- `gpu_index`: GPU device index
- `gpu_name`: Human-readable GPU name
- `pid`: Process ID (null for GPU-level records)
- `user`: Username (null for GPU-level records)
- `process_name`: Process name (null for GPU-level records)
- `memory_used_mb`: Memory usage in megabytes
- `utilization_pct`: GPU utilization percentage
- `temperature_c`: GPU temperature in Celsius
- `power_w`: GPU power consumption in watts
- `container`: Container name (null if not in container)

### Use Cases

**Resource Planning:**
```bash
# Analyze peak usage patterns
gpukill --audit --audit-summary --audit-hours 168

# Find heavy users
gpukill --audit --audit-summary | grep "Top Users"
```

**Troubleshooting:**
```bash
# Check what was running when GPU crashed
gpukill --audit --audit-hours 1

# Find processes that used most memory
gpukill --audit --audit-summary | grep "Top Processes"
```

**Compliance and Billing:**
```bash
# Generate usage report for specific user
gpukill --audit --audit-user alice --output json > alice_usage.json

# Export all data for analysis
gpukill --audit --output json > gpu_usage_export.json
```

**Performance Analysis:**
```bash
# Check hourly usage patterns
gpukill --audit --audit-summary --audit-hours 24

# Monitor specific application usage
gpukill --audit --audit-process tensorflow --audit-hours 48
```

### Data Management

**File Size:**
- Each audit record is approximately 200-300 bytes
- 1000 records ≈ 250KB
- 10,000 records ≈ 2.5MB
- Automatic cleanup recommended for long-term usage

**Retention:**
- No automatic cleanup by default
- Manual cleanup: Delete old records from `audit.jsonl`
- Recommended: Keep 30-90 days of data depending on needs

**Backup:**
```bash
# Backup audit data
cp ~/.local/share/gpukill/audit.jsonl gpu_audit_backup.jsonl

# Restore audit data
cp gpu_audit_backup.jsonl ~/.local/share/gpukill/audit.jsonl
```

## Cluster Management

GPU Kill includes a powerful cluster management system that allows you to monitor and manage multiple GPU nodes from a central coordinator.

### Coordinator API

The coordinator is a RESTful API server that aggregates data from multiple GPU nodes and provides real-time cluster monitoring.

#### Starting the Coordinator

```bash
# Start coordinator on default port 8080
gpukill --server

# Start on custom port
gpukill --server --server-port 9000

# Start on all interfaces
gpukill --server --server-host 0.0.0.0
```

#### API Endpoints

- `GET /api/nodes` - List all registered nodes
- `POST /api/nodes/:id/register` - Register a new node
- `POST /api/nodes/:id/snapshot` - Update node snapshot
- `GET /api/cluster/snapshot` - Get cluster-wide snapshot
- `GET /api/cluster/contention` - Get GPU contention analysis
- `WS /ws` - WebSocket for real-time updates

#### Node Registration

Nodes automatically register themselves when they start the coordinator. Each node:
- Generates a unique UUID
- Reports hostname and IP address
- Sends periodic snapshots of GPU and process data
- Maintains heartbeat for health monitoring

### Magic Moment Analysis

The "Magic Moment" feature provides instant identification of GPU contention and resource blocking:

- **Blocked GPUs**: GPUs with high utilization that are blocking other users
- **Top Users**: Users ranked by GPU memory usage and utilization
- **Contention Recommendations**: Suggestions for optimizing GPU allocation
- **Real-time Updates**: Live updates via WebSocket connections

## Remote Operations

GPU Kill supports SSH-based remote management, allowing you to control GPUs across distributed systems.

### SSH Configuration

```bash
# Basic remote connection
gpukill --remote staging-server --list

# With custom SSH options
gpukill --remote server --ssh-user admin --ssh-port 2222 --list
gpukill --remote server --ssh-key ~/.ssh/id_rsa --list
gpukill --remote server --ssh-password mypassword --list
```

### Remote Authentication

- **SSH Keys**: Preferred method for automated operations
- **Password**: Interactive or provided via command line
- **SSH Agent**: Uses system SSH agent for key management
- **Custom Ports**: Support for non-standard SSH ports

### Remote Requirements

- SSH access to remote host
- `gpukill` installed on remote host
- Proper SSH key permissions (`chmod 600 ~/.ssh/id_rsa`)
- Network connectivity to remote host

### Remote Operations

All local operations work remotely:

```bash
# Remote monitoring
gpukill --remote server --list --details --watch

# Remote process management
gpukill --remote server --kill --pid 1234
gpukill --remote server --kill --filter "python.*" --batch

# Remote GPU control
gpukill --remote server --reset --gpu 0
gpukill --remote server --reset --all

# Remote auditing
gpukill --remote server --audit --audit-summary
```

## Dashboard

The GPU Kill dashboard is a modern web interface built with Nuxt.js and Tailwind CSS for real-time cluster monitoring.

Check the [Kill Suite](https://treadiehq.com) website.


## MCP Server

GPU Kill includes a MCP server that enables AI assistants and other tools to interact with GPU management functionality through a standardized interface.

### Overview

The MCP server provides a JSON-RPC interface that allows AI assistants to:
- Monitor GPU health and performance
- Kill problematic processes
- Reset crashed GPUs
- Scan for security threats
- Manage resource policies
- Automate GPU operations

### Architecture

The MCP server is built as a separate crate (`gpukill-mcp`) that integrates with the main GPU Kill functionality:

- **HTTP Server**: Runs on port 3001 (configurable via `MCP_PORT`)
- **JSON-RPC Protocol**: Standard MCP protocol for AI integration
- **Resource Handler**: Provides read-only access to GPU data
- **Tool Handler**: Executes GPU management actions
- **Cross-platform**: Works on macOS, Linux, and Windows

### Resources

The MCP server exposes the following resources for AI assistants to read:

#### gpu://list
Current GPU status and utilization data including:
- GPU ID, name, and vendor
- Memory usage and total capacity
- Utilization percentage
- Temperature and power usage
- Active processes

#### gpu://processes
Currently running GPU processes with:
- Process ID and name
- Memory usage
- User information
- GPU assignment

#### gpu://audit
Historical GPU usage data including:
- Usage patterns over time
- Process execution history
- Resource utilization trends
- User activity logs

#### gpu://policies
Current Guard Mode policies with:
- User-specific limits
- Group policies
- GPU-specific restrictions
- Time-based overrides

#### gpu://rogue-detection
Security scan results including:
- Suspicious processes
- Crypto miner detection
- Resource abuse patterns
- Data exfiltration attempts

### Tools

The MCP server provides the following tools for AI assistants to execute:

#### kill_gpu_process
Kill a specific GPU process by PID:
```json
{
  "name": "kill_gpu_process",
  "arguments": {
    "pid": 12345,
    "force": false
  }
}
```

#### reset_gpu
Reset a specific GPU:
```json
{
  "name": "reset_gpu",
  "arguments": {
    "gpu_id": 0,
    "force": false
  }
}
```

#### scan_rogue_activity
Scan for suspicious GPU activity:
```json
{
  "name": "scan_rogue_activity",
  "arguments": {
    "hours": 24
  }
}
```

#### create_user_policy
Create a user policy for Guard Mode:
```json
{
  "name": "create_user_policy",
  "arguments": {
    "username": "developer",
    "memory_limit_gb": 8.0,
    "utilization_limit_pct": 70.0,
    "process_limit": 3
  }
}
```

#### get_gpu_status
Get detailed status of a specific GPU:
```json
{
  "name": "get_gpu_status",
  "arguments": {
    "gpu_id": 0
  }
}
```

#### kill_processes_by_name
Kill all processes matching a name pattern:
```json
{
  "name": "kill_processes_by_name",
  "arguments": {
    "pattern": "python.*train",
    "force": false
  }
}
```

### API Endpoints

#### HTTP Interface
- **POST /mcp** - Main MCP JSON-RPC endpoint
- **GET /health** - Health check endpoint

#### MCP Methods
- **initialize** - Initialize the MCP connection
- **resources/list** - List available resources
- **resources/read** - Read resource contents
- **tools/list** - List available tools
- **tools/call** - Execute a tool

### Configuration

The MCP server can be configured using environment variables:

- **MCP_PORT** - Port to listen on (default: 3001)
- **RUST_LOG** - Logging level (default: info)

### Usage Examples

#### Starting the Server
```bash
# Start the MCP server
cargo run --release -p gpukill-mcp

# Or with custom port
MCP_PORT=3001 cargo run --release -p gpukill-mcp
```

#### Testing the Server
```bash
# Health check
curl -X GET http://localhost:3001/health

# List available tools
curl -X POST http://localhost:3001/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":"1","method":"tools/list","params":{}}'

# Get GPU list
curl -X POST http://localhost:3001/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":"2","method":"resources/read","params":{"uri":"gpu://list"}}'
```

### Development

```bash
# Run in development mode
cargo run -p gpukill-mcp

# Run with debug logging
RUST_LOG=debug cargo run -p gpukill-mcp

# Build release version
cargo build --release -p gpukill-mcp
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

For detailed MCP server documentation, see [mcp/README.md](mcp/README.md).


## Output Formats

### Table Format (Default)

The table format provides a clean, human-readable view of GPU information:

```
┌─────┬──────────────────────┬─────────────────┬──────────┬──────────┬─────────┬─────────────┬──────┬─────────────────────┐
│ GPU │ NAME                 │ MEM_USED/TOTAL  │ UTIL(%)  │ TEMP(°C) │ POWER(W)│ ECC(volatile)│ PIDS │ TOP_PROC            │
├─────┼──────────────────────┼─────────────────┼──────────┼──────────┼─────────┼─────────────┼──────┼─────────────────────┤
│ 0   │ NVIDIA GeForce RTX...│ 2.0/8.0 GiB     │ 45.2     │ 72       │ 150.3   │ 0           │ 2    │ python:12345:1024MB │
└─────┴──────────────────────┴─────────────────┴──────────┴──────────┴─────────┴─────────────┴──────┴─────────────────────┘
```

**Columns:**
- **GPU**: GPU index
- **NAME**: GPU model name
- **MEM_USED/TOTAL**: Memory usage in GiB
- **UTIL(%)**: GPU utilization percentage
- **TEMP(°C)**: Current temperature
- **POWER(W)**: Current power consumption
- **ECC(volatile)**: ECC error count (if available)
- **PIDS**: Number of processes using this GPU
- **TOP_PROC**: Highest memory-using process (format: name:pid:memory)

### Detailed Table Format

When using `--details`, additional process rows are shown:

```
┌─────┬──────┬─────────┬─────────┬─────────┬─────────────┬─────────────┬──────────┐
│ GPU │ PID  │ USER    │ PROC    │ VRAM_MB │ START_TIME  │ CONTAINER   │          │
├─────┼──────┼─────────┼─────────┼─────────┼─────────────┼─────────────┼──────────┤
│ 0   │ 12345│ developer│ python  │ 1024    │ 1h 30m      │ -           │          │
│ 0   │ 12346│ developer│ python  │ 512     │ 45m         │ -           │          │
└─────┴──────┴─────────┴─────────┴─────────┴─────────────┴─────────────┴──────────┘
```

### JSON Format

JSON output provides structured data for scripting and automation:

```json
{
  "host": "workstation",
  "ts": "2024-01-01T12:00:00.000Z",
  "gpus": [
    {
      "gpu_index": 0,
      "name": "NVIDIA GeForce RTX 4090",
      "mem_used_mb": 2048,
      "mem_total_mb": 8192,
      "util_pct": 45.2,
      "temp_c": 72,
      "power_w": 150.3,
      "ecc_volatile": 0,
      "pids": 2,
      "top_proc": {
        "gpu_index": 0,
        "pid": 12345,
        "user": "developer",
        "proc_name": "python",
        "used_mem_mb": 1024,
        "start_time": "1h 30m",
        "container": null
      }
    }
  ],
  "procs": [
    {
      "gpu_index": 0,
      "pid": 12345,
      "user": "developer",
      "proc_name": "python",
      "used_mem_mb": 1024,
      "start_time": "1h 30m",
      "container": null
    }
  ]
}
```

## Configuration

### Configuration File

Create a configuration file at `~/.config/gpukill/config.toml`:

```toml
# Logging
log_level = "info"

# Output
output_format = "table"
use_colors = true
table_width = 120

# Process management
default_timeout_secs = 5
max_processes_summary = 10

# Watch mode
watch_interval_secs = 2

# Display options
show_details = false
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `GPUKILL_LOG_LEVEL` | Log level (trace, debug, info, warn, error) | `info` |
| `GPUKILL_OUTPUT_FORMAT` | Output format (table, json) | `table` |
| `GPUKILL_DEFAULT_TIMEOUT` | Default timeout in seconds | `5` |
| `GPUKILL_SHOW_DETAILS` | Show detailed process information | `false` |
| `GPUKILL_WATCH_INTERVAL` | Watch mode refresh interval | `2` |
| `GPUKILL_TABLE_WIDTH` | Table width limit | `120` |
| `GPUKILL_USE_COLORS` | Enable/disable colored output | `true` |

### Configuration Precedence

1. Command-line arguments (highest priority)
2. Environment variables
3. Configuration file
4. Default values (lowest priority)

## Safety Features

### Process Termination Safety

- **Existence Validation**: Verifies the target process exists before attempting termination
- **GPU Usage Check**: Confirms the process is actually using a GPU (unless `--force` is used)
- **Graceful Shutdown**: Sends SIGTERM first for clean process termination
- **Escalation Control**: Only escalates to SIGKILL with explicit `--force` flag
- **Timeout Protection**: Prevents indefinite waiting with configurable timeouts

### GPU Reset Safety

- **Process Detection**: Lists all active processes before reset
- **Confirmation Required**: Requires `--force` flag if active processes are detected
- **Index Validation**: Verifies GPU index exists before reset attempt
- **Operation Support**: Checks if reset is supported on the target GPU
- **Clear Messaging**: Provides detailed error messages for unsupported operations

### Error Handling

- **Actionable Messages**: Clear, specific error messages with suggested solutions
- **Appropriate Exit Codes**: Different exit codes for different failure modes
- **Graceful Degradation**: Continues operation when non-critical components fail
- **NVML Fallback**: Handles cases where NVML is unavailable with helpful messages

## Exit Codes

| Code | Meaning | Description |
|------|---------|-------------|
| `0` | Success | Operation completed successfully |
| `1` | General Error | Unspecified error occurred |
| `2` | NVML Failure | NVML initialization failed |
| `3` | Invalid Arguments | Command-line argument validation failed |
| `4` | Permission Error | Insufficient permissions for operation |
| `5` | Unsupported Operation | Operation not supported on this system |

## Troubleshooting

### Common Issues

#### NVML Initialization Failed
- **Cause**: NVIDIA drivers are not installed, are outdated, or NVML library is not accessible.
- **Solution**:
    - Ensure NVIDIA drivers are properly installed and up to date.
    - Verify that your GPU is recognized by the system (e.g., run `nvidia-smi` on Linux/Windows).
    - Check if you have the necessary permissions to access NVML (you might need to run `gpukill` with `sudo` for some operations).

#### AMD ROCm not installed
- **Cause**: AMD ROCm drivers are not installed, or `rocm-smi` is not in your PATH.
- **Solution**:
    - Install ROCm drivers for your AMD GPU
    - Ensure `rocm-smi` is accessible from your terminal
    - Check if you have the necessary permissions to access AMD GPU information

#### Intel GPU tools not available
- **Cause**: Intel GPU tools are not installed, or `intel_gpu_top` is not in your PATH.
- **Solution**:
    - Install intel-gpu-tools package for Intel GPU support
    - Ensure `intel_gpu_top` is accessible from your terminal
    - Check if you have the necessary permissions to access Intel GPU information


#### Permission Denied
- **Cause**: The current user does not have the necessary privileges to perform the requested action (e.g., killing a process owned by another user, resetting a GPU).
- **Solution**:
    - For process management, ensure you have rights to manage the target PID.
    - For GPU reset or other system-level operations, try running `gpukill` with `sudo`.
    - Consult your system's documentation for managing user permissions for NVIDIA/AMD devices.

#### GPU Not Found
- **Cause**: The specified GPU index does not exist, or the GPU is not properly detected.
- **Solution**:
    - Use `gpukill --list` to see available GPU indices.
    - Ensure your GPU is physically connected and powered on.
    - Verify that your GPU drivers are correctly installed and recognize the GPU.

#### No GPU vendors available
- **Cause**: No supported GPU vendors (NVIDIA, AMD, Intel, or Apple Silicon) could be initialized or found on the system.
- **Solution**:
    - Ensure at least one supported GPU vendor's drivers and management tools are correctly installed.
    - Check system logs for driver-related errors.

#### Process filtering not working
- **Cause**: Invalid regex pattern or no processes match the filter.
- **Solution**:
    - Verify your regex pattern is correct
    - Use `gpukill --list --details` to see available processes
    - Test your pattern with a simple filter first

#### Container detection not working
- **Cause**: Container runtime not detected or process not in container.
- **Solution**:
    - Ensure container runtime (Docker, Podman, etc.) is running
    - Check if the process is actually running in a container
    - Container detection is best-effort and may not work in all environments

#### Batch operations failing
- **Cause**: Permission issues or processes not found.
- **Solution**:
    - Ensure you have permission to kill the target processes
    - Use `--force` flag if processes are unresponsive
    - Check that the filter pattern matches existing processes

## Guard Mode

Guard Mode provides soft policy enforcement to prevent GPU resource abuse with safe testing capabilities. It allows administrators to set policies for users, groups, and GPUs, with configurable enforcement modes and comprehensive monitoring.

### Overview

Guard Mode is designed to:
- **Prevent Resource Abuse**: Set limits on memory usage, GPU utilization, and concurrent processes
- **Safe Testing**: Dry-run mode allows testing policies without affecting running processes
- **Flexible Enforcement**: Choose between soft warnings and hard enforcement actions
- **Real-time Monitoring**: Live policy violation detection and alerting

### Configuration

Guard Mode configuration is stored in TOML format at:
- **Linux**: `~/.local/share/gpukill/guard_mode_config.toml`
- **macOS**: `~/Library/Application Support/gpukill/guard_mode_config.toml`
- **Windows**: `%APPDATA%\gpukill\guard_mode_config.toml`

### Policy Types

#### User Policies
Control resource usage per user:
```toml
[user_policies.developer]
username = "developer"
memory_limit_gb = 8.0
utilization_limit_pct = 70.0
duration_limit_hours = 12.0
max_concurrent_processes = 3
priority = 5
allowed_gpus = []
blocked_gpus = []
time_overrides = []
```

#### Group Policies
Control resource usage per group with member management:
```toml
[group_policies.researchers]
group_name = "researchers"
total_memory_limit_gb = 32.0
total_utilization_limit_pct = 80.0
max_concurrent_processes = 10
priority = 3
allowed_gpus = [0, 1]
blocked_gpus = []
members = ["alice", "bob", "charlie"]
```

**Key Features:**
- **Member Management**: Specify which users belong to the group
- **Total Resource Limits**: Set aggregate limits for all group members
- **CLI Support**: Add members via `--guard-group-members "user1,user2,user3"`

#### GPU Policies
Control access to specific GPUs with user restrictions:
```toml
[gpu_policies."0"]
gpu_index = 0
max_memory_gb = 24.0
max_utilization_pct = 90.0
reserved_memory_gb = 2.0
allowed_users = ["alice", "bob"]
blocked_users = []
maintenance_window = null
```

**Key Features:**
- **User Access Control**: Specify which users can access specific GPUs
- **Reserved Memory**: Set aside memory that cannot be used by processes
- **CLI Support**: Add allowed users via `--guard-gpu-allowed-users "user1,user2,user3"`
- **Flexible Access**: Leave `allowed_users` empty to allow all users

#### Time Policies
Control resource usage during specific time periods:
```toml
[[time_policies]]
name = "business_hours"
start_time = "09:00"
end_time = "17:00"
days_of_week = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday"]
memory_limit_gb = 16.0
utilization_limit_pct = 80.0
max_concurrent_processes = 5
```

### Enforcement Modes

#### Dry-Run Mode
Safe testing without affecting running processes:
- **Simulation Only**: All policy violations are simulated
- **No Actions Taken**: Processes continue running normally
- **Detailed Logging**: Shows exactly what would happen
- **Safe Testing**: Perfect for policy validation

#### Soft Enforcement
Warnings and notifications before hard actions:
- **Warning Notifications**: Send alerts for policy violations
- **Grace Period**: Allow time for users to adjust
- **Escalation**: Progress to hard enforcement if violations persist
- **Logging**: Record all violations and warnings

#### Hard Enforcement
Immediate action on policy violations:
- **Process Termination**: Kill processes that violate policies
- **Resource Limits**: Enforce memory and utilization limits
- **Access Control**: Block access to restricted GPUs
- **Immediate Action**: No grace period for critical violations

### CLI Commands

#### Basic Guard Mode Operations
```bash
# Enable Guard Mode
gpukill --guard --guard-enable

# Disable Guard Mode
gpukill --guard --guard-disable

# View current configuration
gpukill --guard --guard-config

# Set dry-run mode (safe testing)
gpukill --guard --guard-dry-run

# Set enforcement mode (live enforcement)
gpukill --guard --guard-enforce
```

#### Policy Management

**User Policies:**
```bash
# Add user policy
gpukill --guard --guard-add-user "developer" --guard-memory-limit 8.0 --guard-utilization-limit 70.0 --guard-process-limit 3

# Remove user policy
gpukill --guard --guard-remove-user "developer"

# Update policy limits
gpukill --guard --guard-memory-limit 16.0 --guard-utilization-limit 80.0 --guard-process-limit 5
```

**Group Policies:**
```bash
# Add group policy with members
gpukill --guard --guard-add-group "developers" --guard-group-members "alice,bob,charlie" --guard-group-memory-limit 32.0 --guard-group-utilization-limit 80.0 --guard-group-process-limit 15

# Add group policy without members (empty group)
gpukill --guard --guard-add-group "testers" --guard-group-memory-limit 16.0

# Remove group policy
gpukill --guard --guard-remove-group "developers"
```

**GPU Policies:**
```bash
# Add GPU policy with allowed users
gpukill --guard --guard-add-gpu 0 --guard-gpu-allowed-users "alice,bob" --guard-gpu-memory-limit 24.0 --guard-gpu-utilization-limit 90.0 --guard-gpu-reserved-memory 2.0

# Add GPU policy allowing all users
gpukill --guard --guard-add-gpu 1 --guard-gpu-memory-limit 16.0

# Remove GPU policy
gpukill --guard --guard-remove-gpu 0
```

**Additional CLI Options:**
- `--guard-group-members <MEMBERS>`: Comma-separated list of group members
- `--guard-gpu-allowed-users <USERS>`: Comma-separated list of allowed users for GPU
- `--guard-group-memory-limit <GB>`: Group memory limit in GB
- `--guard-group-utilization-limit <PERCENT>`: Group utilization limit percentage
- `--guard-group-process-limit <COUNT>`: Group process limit count
- `--guard-gpu-memory-limit <GB>`: GPU memory limit in GB
- `--guard-gpu-utilization-limit <PERCENT>`: GPU utilization limit percentage
- `--guard-gpu-reserved-memory <GB>`: GPU reserved memory in GB

#### Policy Testing
```bash
# Test policies in dry-run mode
gpukill --guard --guard-test-policies

# Toggle dry-run mode
gpukill --guard --guard-toggle-dry-run
```

#### Configuration Management
```bash
# Export configuration
gpukill --guard --guard-export-config > guard_config.json

# Import configuration
gpukill --guard --guard-import-config guard_config.json
```

### API Endpoints

#### Configuration Management
```bash
# Get Guard Mode configuration
GET /api/guard/config

# Update Guard Mode configuration
POST /api/guard/config
Content-Type: application/json
{
  "global": {
    "enabled": true,
    "dry_run": true,
    "default_memory_limit_gb": 16.0,
    "default_utilization_limit_pct": 80.0
  }
}
```

#### Policy Management
```bash
# Get policies
GET /api/guard/policies

# Update policies
POST /api/guard/policies
Content-Type: application/json
{
  "user_policies": {
    "developer": {
      "username": "developer",
      "memory_limit_gb": 8.0,
      "utilization_limit_pct": 70.0,
      "max_concurrent_processes": 3
    }
  }
}
```

#### Status and Testing
```bash
# Get Guard Mode status
GET /api/guard/status

# Toggle dry-run mode
POST /api/guard/toggle-dry-run

# Test policies
POST /api/guard/test-policies
```

### Violation Types

#### Memory Violations
- **Excessive Memory Usage**: Process exceeds memory limit
- **Memory Hoarding**: Long-running processes with high memory usage
- **Memory Leaks**: Processes with continuously increasing memory usage

#### Utilization Violations
- **High GPU Utilization**: Process exceeds utilization limit
- **Sustained High Usage**: Long periods of high GPU utilization
- **Resource Waste**: Processes with low efficiency

#### Process Violations
- **Too Many Processes**: User exceeds concurrent process limit
- **Long-running Processes**: Processes exceeding duration limits
- **Unauthorized Processes**: Processes not allowed by policy

#### Access Violations
- **GPU Access**: Attempting to use blocked GPUs
- **Time Restrictions**: Using GPUs during restricted hours
- **User Restrictions**: Unauthorized user access

### Enforcement Actions

#### Warning Actions
- **Console Notifications**: Display warnings in terminal
- **Log File Entries**: Record warnings in log files
- **Email Alerts**: Send email notifications (if configured)
- **Webhook Notifications**: Send alerts to external systems

#### Enforcement Actions
- **Process Termination**: Kill violating processes
- **Resource Limits**: Enforce memory and utilization limits
- **Access Blocking**: Prevent access to restricted resources
- **User Notifications**: Inform users of policy violations

### Best Practices

#### Policy Design
- **Start Conservative**: Begin with generous limits and tighten over time
- **Test Thoroughly**: Use dry-run mode extensively before enabling enforcement
- **Monitor Closely**: Watch for false positives and adjust policies accordingly
- **Document Policies**: Keep clear records of policy decisions and changes

#### Implementation
- **Gradual Rollout**: Enable policies for a subset of users first
- **User Communication**: Inform users about new policies and limits
- **Training**: Provide guidance on policy compliance
- **Feedback Loop**: Collect user feedback and adjust policies

#### Monitoring
- **Regular Reviews**: Periodically review policy effectiveness
- **Violation Analysis**: Analyze patterns in policy violations
- **Performance Impact**: Monitor system performance under policies
- **User Satisfaction**: Track user satisfaction with policy enforcement

### Troubleshooting

#### Common Issues
- **False Positives**: Adjust policy thresholds if legitimate processes are flagged
- **Performance Impact**: Monitor system performance under policy enforcement
- **User Complaints**: Address user concerns about policy restrictions
- **Configuration Errors**: Validate policy configuration syntax

#### Debugging
- **Enable Debug Logging**: Use `RUST_LOG=debug` for detailed logs
- **Test Policies**: Use dry-run mode to validate policy behavior
- **Check Configuration**: Verify policy configuration files
- **Monitor Violations**: Review violation logs for patterns

## Development

### Building

```bash
# Debug build (fastest, ~3 seconds)
cargo build

# Fast release build (recommended for development, ~28 seconds)
cargo build --profile release-fast

# Standard release build (production-ready, ~28 seconds)
cargo build --release

# Maximum optimization (slowest, best performance, ~60+ seconds)
cargo build --profile release-max

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- --list
```

### Build Performance Optimization

The project includes multiple build profiles optimized for different use cases:

- **`dev`**: Fast debug builds for development
- **`release-fast`**: Optimized for development with good performance
- **`release`**: Balanced optimization for production use
- **`release-max`**: Maximum optimization for final releases

**Performance improvements made:**
- Changed from fat LTO (`lto = true`) to thin LTO (`lto = "thin"`)
- Increased codegen units from 1 to 4 for parallel compilation
- Added fast release profile for development workflows

### Testing

The project includes comprehensive tests:

```bash
# Run all tests
cargo test

# Run specific test module
cargo test args::tests

# Run integration tests
cargo test --test integration_tests

# Run with mock NVML (for CI)
cargo test --features mock_nvml
```


### Adding New Features

1. **Define CLI arguments** in `src/args.rs`
2. **Implement core logic** in appropriate module
3. **Add tests** for new functionality
4. **Update documentation** in this file
5. **Test on multiple platforms**

### Performance Considerations

- **NVML Caching**: GPU information is cached during snapshot creation
- **Process Enumeration**: Uses efficient system calls for process information
- **Memory Management**: Minimizes allocations in hot paths
- **Error Handling**: Fast-path for common success cases

## Acknowledgments

- Built with [NVML Wrapper](https://github.com/Cldfire/nvml-wrapper) for NVIDIA GPU management
- AMD GPU support via [ROCm](https://rocm.docs.amd.com/) and rocm-smi
- Intel GPU support via [intel-gpu-tools](https://gitlab.freedesktop.org/drm/igt-gpu-tools) and intel_gpu_top
- Apple Silicon GPU support via macOS system_profiler and system APIs
- Uses [Clap](https://github.com/clap-rs/clap) for command-line argument parsing
- Table rendering powered by [Tabled](https://github.com/zhiburt/tabled)
- Error handling with [Color Eyre](https://github.com/yaahc/color-eyre)
- Process management via [Sysinfo](https://github.com/GuillaumeGomez/sysinfo)
- Regex processing with [Regex](https://github.com/rust-lang/regex)
- Signal handling with [Nix](https://github.com/nix-rust/nix)
