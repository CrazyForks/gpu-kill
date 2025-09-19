# GPU Kill

A CLI tool for managing NVIDIA, AMD, Intel, and Apple Silicon GPUs across local and remote systems. List, monitor, and control GPU processes with ease, plus cluster management and real-time dashboards.

## What it does

- **📊 Monitor**: See GPU usage, memory, temperature, and running processes
- **⚡ Kill**: Gracefully terminate GPU processes (with force option)
- **🔄 Reset**: Reset individual GPUs or all GPUs
- **👀 Watch**: Real-time monitoring with auto-refresh
- **🎯 Filter**: Advanced process filtering and batch operations
- **📈 Audit**: Track GPU usage history and generate usage reports
- **🕵️ Rogue Detection**: Detect crypto miners, suspicious processes, and resource abuse
- **⚙️ Configurable Rules**: Customize detection thresholds, whitelists, and patterns
- **🛡️ Guard Mode**: Soft policy enforcement to prevent GPU resource abuse
- **🧪 Dry-Run Mode**: Safe policy testing without affecting running processes
- **🔧 Multi-Vendor**: Support for NVIDIA, AMD, Intel, and Apple Silicon GPUs
- **🌐 Remote**: SSH-based remote GPU management across clusters
- **📊 Dashboard**: Real-time web dashboard for multi-node GPU monitoring
- **⚡ Magic Moment**: Instant identification of GPU contention and blocked resources
- **🖥️ Cluster**: Coordinator API for managing distributed GPU clusters

## Requirements

- **NVIDIA**: NVIDIA drivers installed
- **AMD**: ROCm drivers installed  
- **Intel**: intel-gpu-tools package installed
- **Apple Silicon**: macOS with Apple Silicon (M1/M2/M3/M4)
- **OS**: Linux, macOS, or Windows

## Quick Start

```bash
### Build from Source
git clone <repository-url>
cd gpu-kill
cargo build --release

# Install from Cargo
cargo install gpukill

# List your GPUs
gpukill --list

# Watch GPU usage in real-time
gpukill --list --watch

# Kill a stuck process
gpukill --kill --pid 12345

# Reset a GPU
gpukill --reset --gpu 0

# Start cluster coordinator and dashboard
gpukill --server

# Manage remote GPUs via SSH
gpukill --remote staging-server --list
gpukill --remote prod-gpu-01 --kill --pid 1234

# Suspicious usage detection
gpukill --audit --rogue --audit-hours 24
gpukill --audit --rogue-config
gpukill --audit --rogue-memory-threshold 15.0 --rogue-utilization-threshold 90.0

# Guard Mode policy enforcement
gpukill --guard --guard-enable
gpukill --guard --guard-test-policies
gpukill --guard --guard-toggle-dry-run
```

## Common Examples

**Monitor training job:**
```bash
gpukill --list --watch --details
```

**Kill stuck process:**
```bash
gpukill --kill --pid 12345 --force
```

**Kill all Python processes:**
```bash
gpukill --kill --filter "python.*" --batch --force
```

**Monitor specific GPU vendor:**
```bash
gpukill --list --vendor nvidia --watch
gpukill --list --vendor amd --watch
gpukill --list --vendor intel --watch
gpukill --list --vendor apple --watch
```

**Reset crashed GPU:**
```bash
gpukill --reset --gpu 0 --force
```

**Export stats for logging:**
```bash
gpukill --list --output json > gpu_stats.json
```

**View GPU usage history:**
```bash
gpukill --audit                           # Show last 24 hours
gpukill --audit --audit-hours 6          # Show last 6 hours
gpukill --audit --audit-summary          # Show usage summary
```

**Filter audit by user or process:**
```bash
gpukill --audit --audit-user john        # Show only john's usage
gpukill --audit --audit-process python   # Show only Python processes
```

## Suspicious Usage Detection

**Detect rogue activity:**
```bash
gpukill --audit --rogue                  # Scan for suspicious activity
gpukill --audit --rogue --audit-hours 48 # Scan last 48 hours
gpukill --audit --rogue --output json    # JSON output for automation
```

**Configure detection rules:**
```bash
# View current configuration
gpukill --audit --rogue-config

# Update detection thresholds
gpukill --audit --rogue-memory-threshold 15.0
gpukill --audit --rogue-utilization-threshold 90.0
gpukill --audit --rogue-duration-threshold 12.0
gpukill --audit --rogue-confidence-threshold 0.8

# Manage whitelists
gpukill --audit --rogue-whitelist-process "my-app"
gpukill --audit --rogue-whitelist-user "developer"
gpukill --audit --rogue-unwhitelist-process "suspicious-app"

# Export/import configuration
gpukill --audit --rogue-export-config > my-config.json
gpukill --audit --rogue-import-config my-config.json
```

**Detection capabilities:**
- **🚨 Crypto Miners**: Detects known mining software and patterns
- **⚠️ Suspicious Processes**: Identifies unusual process names and behavior
- **📊 Resource Abusers**: Flags excessive memory usage and long-running processes
- **🎯 Risk Scoring**: Provides confidence-based threat assessment
- **📋 Recommendations**: Generates actionable security advice

## Guard Mode - Policy Enforcement

Soft policy enforcement to prevent GPU resource abuse with safe testing capabilities:

**Enable and configure Guard Mode:**
```bash
# Enable/disable Guard Mode
gpukill --guard --guard-enable
gpukill --guard --guard-disable

# Configure enforcement modes
gpukill --guard --guard-dry-run    # Simulation only (safe testing)
gpukill --guard --guard-enforce    # Live enforcement

# View current configuration
gpukill --guard --guard-config
```

**Manage user policies:**
```bash
# Add user policy with limits
gpukill --guard --guard-add-user "developer" --guard-memory-limit 8.0 --guard-utilization-limit 70.0 --guard-process-limit 3

# Remove user policy
gpukill --guard --guard-remove-user "developer"

# Update policy limits
gpukill --guard --guard-memory-limit 16.0 --guard-utilization-limit 80.0 --guard-process-limit 5
```

**Safe policy testing:**
```bash
# Test policies in dry-run mode (no actual enforcement)
gpukill --guard --guard-test-policies

# Toggle between dry-run and enforcement modes
gpukill --guard --guard-toggle-dry-run

# Export/import configuration
gpukill --guard --guard-export-config > guard_config.json
gpukill --guard --guard-import-config guard_config.json
```

**Policy features:**
- **🛡️ User Policies**: Memory, utilization, and process limits per user
- **🧪 Dry-Run Mode**: Safe testing without affecting running processes
- **⚡ Soft Enforcement**: Warnings and notifications before hard actions
- **🚨 Hard Enforcement**: Process termination for critical violations
- **📊 Real-time Monitoring**: Live policy violation detection
- **🎯 Flexible Configuration**: Per-user, per-group, and per-GPU policies
- **📈 Dashboard Integration**: Visual policy management and monitoring

## Cluster Management

**Start the coordinator server:**
```bash
gpukill --server                          # Start on default port 8080
gpukill --server --server-port 9000      # Start on custom port
gpukill --server --server-host 0.0.0.0   # Listen on all interfaces
```

**Access the dashboard:**
- Open your browser to `http://localhost:8080` (or your custom port)
- View real-time cluster overview, GPU contention, and node details
- Magic Moment view shows which users/jobs are blocking GPU resources

## Remote Operations

**SSH-based remote management:**
```bash
# Basic remote operations
gpukill --remote staging-server --list
gpukill --remote prod-gpu-01 --kill --pid 1234
gpukill --remote gpu-cluster --reset --gpu 0

# With SSH authentication
gpukill --remote server --ssh-key ~/.ssh/id_rsa --list
gpukill --remote server --ssh-user admin --ssh-port 2222 --list
gpukill --remote server --ssh-password mypassword --list

# All operations work remotely
gpukill --remote server --list --details --watch
gpukill --remote server --kill --filter "python.*" --batch
gpukill --remote server --audit --audit-summary
```

**Remote requirements:**
- SSH access to remote host
- `gpukill` installed on remote host
- Proper SSH keys or password authentication

## Troubleshooting

**"No GPU vendors available"**
- Install drivers for your GPU vendor (NVIDIA, AMD, Intel, or Apple Silicon)
- Check `nvidia-smi`, `rocm-smi`, `intel_gpu_top`, or system_profiler works

**"Permission denied"**
- Try `sudo gpukill --list` first

**"GPU not found"**
- Check GPU index with `gpukill --list`

**"No audit records found"**
- Run `gpukill --list` first to generate audit data
- Check audit data directory: `~/.local/share/gpukill/` (Linux) or `~/Library/Application Support/gpukill/` (macOS)

**"SSH connection failed"**
- Check SSH connectivity: `ssh user@host`
- Verify SSH key permissions: `chmod 600 ~/.ssh/id_rsa`
- Test with password auth: `gpukill --remote host --ssh-password password --list`

**"gpukill not found on remote host"**
- Install gpukill on the remote host first
- Check PATH: `ssh user@host "which gpukill"`

**"Dashboard not loading"**
- Check if server is running: `gpukill --server`
- Verify port is not in use: `lsof -i :8080`
- Check firewall settings for the coordinator port

## Get Help

```bash
gpukill --help                    # Show all options
gpukill --version                 # Show version
```

## Need More Details?

For advanced configuration, detailed API documentation, and developer information, see [DETAILED.md](DETAILED.md).

## License

FSL-1.1-MIT License - see [LICENSE](LICENSE) for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request