# GPU Kill - Detailed Documentation

## Table of Contents

- [Architecture](#architecture)
- [Installation](#installation)
- [Command Reference](#command-reference)
- [Enhanced Features](#enhanced-features)
- [Audit System](#audit-system)
- [Output Formats](#output-formats)
- [Configuration](#configuration)
- [Safety Features](#safety-features)
- [Exit Codes](#exit-codes)
- [Troubleshooting](#troubleshooting)
- [Development](#development)

## Architecture

GPU Kill is built with Rust and supports NVIDIA, AMD, Intel, and Apple Silicon GPUs through vendor-specific interfaces. The tool is designed with safety, usability, and multi-vendor support in mind.

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
- `--output <FORMAT>`: Output format (`table` or `json`)

**Examples:**
```bash
# Show last 24 hours of GPU usage
gpukill --audit

# Show last 6 hours
gpukill --audit --audit-hours 6

# Show usage summary
gpukill --audit --audit-summary

# Filter by user
gpukill --audit --audit-user john

# Filter by process
gpukill --audit --audit-process python

# Combine filters
gpukill --audit --audit-user alice --audit-process tensorflow --audit-hours 12

# JSON output
gpukill --audit --output json

# Export filtered data
gpukill --audit --audit-user john --output json > john_usage.json
```

## Enhanced Features

### Multi-Vendor Support

`gpukill` automatically detects and utilizes available GPU vendors (NVIDIA, AMD, Intel). You can also filter the displayed information by vendor.

**Options:**
- `--vendor <VENDOR>`: Filter by GPU vendor.
  - `nvidia`: Show only NVIDIA GPUs.
  - `amd`: Show only AMD GPUs.
  - `intel`: Show only Intel GPUs.
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

**JSON Output:**
```bash
# Export audit data as JSON for external processing
gpukill --audit --output json

# Export filtered data
gpukill --audit --audit-user john --output json > john_gpu_usage.json
```

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
