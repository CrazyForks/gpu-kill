# GPU Kill - Detailed Documentation

## Table of Contents

- [Architecture](#architecture)
- [Installation](#installation)
- [Command Reference](#command-reference)
- [Enhanced Features](#enhanced-features)
- [Output Formats](#output-formats)
- [Configuration](#configuration)
- [Safety Features](#safety-features)
- [Exit Codes](#exit-codes)
- [API Reference](#api-reference)
- [Troubleshooting](#troubleshooting)
- [Development](#development)
- [Contributing](#contributing)

## Architecture

GPU Kill is built with Rust and supports both NVIDIA and AMD GPUs through vendor-specific interfaces. The tool is designed with safety, usability, and multi-vendor support in mind.

### Core Components

- **CLI Parser**: Uses `clap` for robust argument parsing and validation
- **Vendor Abstraction**: Multi-vendor GPU support (NVIDIA, AMD)
- **NVML Wrapper**: Interfaces with NVIDIA's management library
- **ROCm Interface**: AMD GPU management via rocm-smi
- **Enhanced Process Manager**: Advanced process filtering and batch operations
- **Container Detection**: Container-aware process management
- **Renderer**: Formats output as tables or JSON
- **Configuration**: Supports file and environment-based configuration

### Dependencies

- `clap`: Command-line argument parsing
- `nvml-wrapper`: NVIDIA GPU management
- `sysinfo`: System process information
- `regex`: Process filtering with regex patterns
- `tabled`: Table formatting
- `serde`: JSON serialization
- `tracing`: Structured logging
- `color-eyre`: Error handling

## Installation

### Prerequisites

- **NVIDIA GPU Support**:
  - NVIDIA GPU with supported drivers
  - NVIDIA Management Library (NVML) - included with NVIDIA drivers
- **AMD GPU Support**:
  - AMD GPU with ROCm drivers installed
  - rocm-smi command-line tool available
- **General Requirements**:
  - Rust 1.70+ (for building from source)
  - Linux, macOS, or Windows

### Install from Cargo

```bash
cargo install gpukill
```

### Build from Source

```bash
git clone https://github.com/kagehq/gpu-kill.git
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
```

### Kill Operation

```bash
gpukill --kill --pid <PID> [OPTIONS]
```

**Required:**
- `--pid <PID>`: Process ID to terminate

**Options:**
- `--timeout-secs <SECONDS>`: Timeout before escalation (default: 5)
- `--force`: Escalate to SIGKILL after timeout

**Examples:**
```bash
# Graceful termination
gpukill --kill --pid 12345

# Custom timeout
gpukill --kill --pid 12345 --timeout-secs 10

# Force escalation
gpukill --kill --pid 12345 --force
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

## Enhanced Features

### Multi-Vendor GPU Support

GPU Kill automatically detects and supports both NVIDIA and AMD GPUs:

```bash
# List all GPUs (NVIDIA and AMD)
gpukill --list

# Filter by vendor
gpukill --list --vendor nvidia
gpukill --list --vendor amd
gpukill --list --vendor all

# Check available vendors
gpukill --list --vendor all --details
```

**Vendor Detection:**
- **NVIDIA**: Automatically detected if NVML is available
- **AMD**: Automatically detected if rocm-smi is available
- **Mixed Systems**: Supports systems with both NVIDIA and AMD GPUs

### Advanced Process Management

#### Process Filtering

Filter processes by name using regex patterns:

```bash
# Kill processes by pattern
gpukill --kill --filter "python.*"
gpukill --kill --filter "tensorflow.*"
gpukill --kill --filter "pytorch.*"

# Batch operations
gpukill --kill --filter "python.*" --batch
gpukill --kill --filter "python.*" --batch --force
```

**Filter Examples:**
- `"python.*"` - All Python processes
- `"tensorflow.*"` - All TensorFlow processes
- `"pytorch.*"` - All PyTorch processes
- `".*train.*"` - Processes with "train" in the name
- `"^python$"` - Exact match for "python"

#### Container Detection

Detect and display container information for processes:

```bash
# Show container information
gpukill --list --containers
gpukill --list --containers --details

# Monitor containerized processes
gpukill --list --containers --watch
```

**Supported Containers:**
- Docker
- Podman
- Kubernetes
- LXC
- Generic container environments

#### Batch Process Operations

Efficiently manage multiple processes:

```bash
# Preview processes matching filter
gpukill --kill --filter "python.*"

# Batch kill with confirmation
gpukill --kill --filter "python.*" --batch

# Force batch kill
gpukill --kill --filter "python.*" --batch --force
```

### Enhanced Kill Operations

#### Process Tree Killing

Kill parent processes and all their children:

```bash
# Kill process tree (parent + children)
gpukill --kill --pid 12345 --tree

# Force kill process tree
gpukill --kill --pid 12345 --tree --force
```

#### Memory-Based Filtering

Filter processes by memory usage:

```bash
# Kill processes using more than 1GB
gpukill --kill --filter ".*" --memory-min 1024 --batch

# Kill processes using more than 2GB
gpukill --kill --filter ".*" --memory-min 2048 --batch --force
```

### Advanced Monitoring

#### Vendor-Specific Monitoring

```bash
# Monitor only NVIDIA GPUs
gpukill --list --vendor nvidia --watch

# Monitor only AMD GPUs
gpukill --list --vendor amd --watch

# Compare vendors
gpukill --list --vendor all --details
```

#### Container-Aware Monitoring

```bash
# Monitor with container information
gpukill --list --containers --watch

# Export container information
gpukill --list --containers --output json > containers.json
```

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

## API Reference

### Data Structures

#### GpuInfo
```rust
pub struct GpuInfo {
    pub index: u16,
    pub name: String,
    pub mem_total_mb: u32,
}
```

#### GpuProc
```rust
pub struct GpuProc {
    pub gpu_index: u16,
    pub pid: u32,
    pub user: String,
    pub proc_name: String,
    pub used_mem_mb: u32,
    pub start_time: String,
    pub container: Option<String>,
}
```

#### GpuSnapshot
```rust
pub struct GpuSnapshot {
    pub gpu_index: u16,
    pub name: String,
    pub mem_used_mb: u32,
    pub mem_total_mb: u32,
    pub util_pct: f32,
    pub temp_c: i32,
    pub power_w: f32,
    pub ecc_volatile: Option<u64>,
    pub pids: usize,
    pub top_proc: Option<GpuProc>,
}
```

#### Snapshot
```rust
pub struct Snapshot {
    pub host: String,
    pub ts: String,
    pub gpus: Vec<GpuSnapshot>,
    pub procs: Vec<GpuProc>,
}
```

### Core Functions

#### NvmlApi
```rust
impl NvmlApi {
    pub fn new() -> Result<Self>;
    pub fn device_count(&self) -> Result<u32>;
    pub fn create_snapshot(&self) -> Result<Snapshot>;
    pub fn reset_gpu(&self, index: u32) -> Result<()>;
}
```

#### ProcessManager
```rust
impl ProcessManager {
    pub fn new(nvml_api: NvmlApi) -> Self;
    pub fn get_process_info(&mut self, pid: u32) -> Result<ProcessInfo>;
    pub fn is_process_using_gpu(&self, pid: u32) -> Result<bool>;
    pub fn graceful_kill(&self, pid: u32, timeout_secs: u16, force: bool) -> Result<()>;
}
```

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
    - Verify that your NVIDIA/AMD drivers are correctly installed and recognize the GPU.

#### No GPU vendors available
- **Cause**: Neither NVIDIA NVML nor AMD ROCm could be initialized or found on the system.
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

## Development

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- --list
```

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

### Code Structure

```
src/
├── main.rs          # Entry point and operation dispatch
├── args.rs          # CLI argument parsing and validation
├── nvml_api.rs      # NVML wrapper and GPU data structures
├── vendor.rs        # Multi-vendor GPU support
├── process_mgmt.rs  # Enhanced process management
├── proc.rs          # Process management and killing
├── render.rs        # Table and JSON rendering
├── config.rs        # Configuration management
├── util.rs          # Utility functions
├── version.rs       # Version information
└── lib.rs           # Library exports for testing

tests/
└── integration_tests.rs  # CLI integration tests
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
- Uses [Clap](https://github.com/clap-rs/clap) for command-line argument parsing
- Table rendering powered by [Tabled](https://github.com/zhiburt/tabled)
- Error handling with [Color Eyre](https://github.com/yaahc/color-eyre)
- Process management via [Sysinfo](https://github.com/GuillaumeGomez/sysinfo)
- Regex processing with [Regex](https://github.com/rust-lang/regex)
- Signal handling with [Nix](https://github.com/nix-rust/nix)
