# GPU Kill

A CLI tool for managing GPUs across NVIDIA, AMD, Intel, and Apple Silicon systems. Monitor, control, and secure your GPU infrastructure with ease.

## Community & Support

Join our Discord community for discussions, support, and updates:

[![Discord](https://img.shields.io/badge/Discord-Join%20our%20community-7289DA?style=for-the-badge&logo=discord&logoColor=white)](https://discord.gg/KqdBcqRk5E)


## Features

- **Monitor GPUs**: Real-time usage, memory, temperature, and processes
- **Kill Processes**: Gracefully terminate stuck GPU processes
- **Security**: Detect crypto miners and suspicious activity
- **Guard Mode**: Policy enforcement to prevent resource abuse
- **Remote**: Manage GPUs across multiple servers
- **Multi-Vendor**: Works with NVIDIA, AMD, Intel, and Apple Silicon
- **AI Integration**: MCP server for AI assistant integration

## Requirements

### Build Performance

**For faster development builds:**
```bash
# Fast release build (recommended for development)
cargo build --profile release-fast

# Standard release build (optimized for production)
cargo build --release

# Maximum optimization (slowest, best performance)
cargo build --profile release-max
```

**Build times on typical hardware:**
- Debug build: ~3 seconds
- Release-fast: ~28 seconds  
- Release: ~28 seconds (improved from 76 seconds)
- Release-max: ~60+ seconds (maximum optimization)

### System Dependencies

**Linux (Ubuntu/Debian):**
```bash
sudo apt install build-essential libssl-dev pkg-config
```

**Linux (Fedora/RHEL/CentOS):**
```bash
sudo dnf install gcc gcc-c++ pkg-config openssl-devel
# or for older systems:
# sudo yum install gcc gcc-c++ pkg-config openssl-devel
```

**macOS:**
```bash
# Install Xcode command line tools
xcode-select --install
# OpenSSL is included with macOS
```

**Windows:**
- Install Visual Studio Build Tools
- OpenSSL is handled automatically by vcpkg

### GPU Drivers

- **NVIDIA**: NVIDIA drivers installed
- **AMD**: ROCm drivers installed  
- **Intel**: intel-gpu-tools package installed
- **Apple Silicon**: macOS with Apple Silicon (M1/M2/M3/M4)

### Build Requirements

- **OS**: Linux, macOS, or Windows
- **Rust**: 1.70+ (for building from source)

## Quick Start

### Install & Run
```bash
# Build from source (first build may take 2-3 minutes)
git clone https://github.com/treadiehq/gpu-kill.git
cd gpu-kill
cargo build --release

# Or install via Cargo
cargo install gpukill

# Or one-liner installers (recommended)
# macOS/Linux
curl -fsSL https://raw.githubusercontent.com/treadiehq/gpu-kill/refs/heads/main/scripts/install.sh | sh
# Windows (PowerShell)
irm https://raw.githubusercontent.com/treadiehq/gpu-kill/refs/heads/main/scripts/install.ps1 | iex

# List your GPUs
gpukill --list

# Watch GPU usage in real-time
gpukill --list --watch
```

### Dead-simple cheatsheet
```bash
# Live watch (alias)
gpukill watch            # = gpukill --list --watch

# Kill job by PID (positional alias)
gpukill 12345            # = gpukill --kill --pid 12345

# Free a specific GPU index (kill all jobs on GPU 0)
gpukill --kill --gpu 0   # add --batch to actually kill; preview without it

# Force reset a GPU (shorthand)
gpukill --reset 0        # = gpukill --reset --gpu 0

# Safe mode: dry-run first (no changes)
gpukill 12345 --safe     # alias: --dry-run
```

## Dashboard (Local Development)

The GPU Kill dashboard provides a modern web interface for GPU cluster monitoring. The dashboard is included in the repository for local development but is **not required** for core GPU Kill functionality.

![GPU Kill Dashboard](dashboard/public/screenshot.png)

### Quick Start

```bash
# 1. Start the backend API server
gpukill --server --server-port 8080

# 2. In a new terminal, start the dashboard UI
cd dashboard
npm install  # First time only
npm run dev

# 3. Access the dashboard
open http://localhost:3000
```

**Requirements:**
- Node.js 18+ and npm
- GPU Kill backend server running (provides the API)

**Note**: You need both the backend server (port 8080) and frontend UI (port 3000) running for the dashboard to work.

### Dashboard Features

- **Real-time monitoring** of all GPUs across your cluster
- **Security detection** with threat analysis and risk scoring
- **Policy management** for resource control and enforcement
- **Cluster overview** with Magic Moment contention insights
- **Interactive controls** for process management and GPU operations

### Production Deployment

For production GPU monitoring solutions, check the [Kill Suite](https://treadie.com) website.

## MCP Server

GPU Kill includes a MCP server that enables AI assistants to interact with GPU management functionality:

- **Resources**: Read GPU status, processes, audit data, policies, and security scans
- **Tools**: Kill processes, reset GPUs, scan for threats, create policies

```bash
# Start the MCP server
cargo run --release -p gpukill-mcp

# Server runs on http://localhost:3001/mcp
```

## Usage

Ask your AI to use the tools.

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

See [mcp/README.md](mcp/README.md) for detailed MCP server documentation.


## Security & Policies

### Detect Threats
```bash
# Scan for crypto miners and suspicious activity
gpukill --audit --rogue

# Configure detection rules
gpukill --audit --rogue-config
```

### Policy Enforcement
```bash
# Enable Guard Mode
gpukill --guard --guard-enable

# Test policies safely
gpukill --guard --guard-test-policies
```

*For detailed security and policy documentation, see [DETAILED.md](DETAILED.md).*

## Remote Management

Manage GPUs across multiple servers via SSH:

```bash
# List GPUs on remote server
gpukill --remote staging-server --list

# Kill process on remote server
gpukill --remote prod-gpu-01 --kill --pid 1234

# Reset GPU on remote server
gpukill --remote gpu-cluster --reset --gpu 0
```

## Troubleshooting

### Build Issues

**OpenSSL not found:**
```bash
# Ubuntu/Debian
sudo apt install build-essential libssl-dev pkg-config

# Fedora/RHEL/CentOS
sudo dnf install gcc gcc-c++ pkg-config openssl-devel
```

**Other common build issues:**
- Ensure you have the latest Rust toolchain: `rustup update`
- Clean and rebuild: `cargo clean && cargo build --release`
- Check system dependencies are installed (see Requirements section)

## Need Help?

```bash
gpukill --help                    # Show all options
gpukill --version                 # Show version
```

## CI/CD and Testing

GPU Kill uses a CI/CD pipeline with **automatic GPU testing**:

- **✅ Conditional GPU testing** - Runs automatically when GPU hardware is available
- **✅ Multi-vendor GPU testing** on real hardware (NVIDIA, AMD, Intel, Apple Silicon)
- **✅ Hot Aisle integration** - Optional on-demand GPU instance provisioning for comprehensive testing
- **✅ Cross-platform compatibility** testing
- **✅ Performance benchmarking** and profiling
- **✅ Security auditing** and compliance checks
- **✅ Stress testing** for reliability validation

### How GPU Testing Works

- **On GitHub hosted runners**: GPU tests skip gracefully (no GPU hardware)
- **On self-hosted runners**: GPU tests run automatically when GPU hardware is detected
- **On cloud instances**: GPU tests run automatically when GPU hardware is available
- **On developer machines**: GPU tests run automatically when GPU hardware is detected
- **Via Hot Aisle**: On-demand GPU instance provisioning for comprehensive testing

### Quick Setup

**Option 1: Test Locally (Already Working)**
```bash
cargo test --test gpu_hardware_tests  # Runs on your GPU hardware
```

**Option 2: Set Up Cloud GPU (5 minutes)**
```bash
# On any cloud GPU instance:
curl -sSL https://raw.githubusercontent.com/treadiehq/gpu-kill/main/scripts/setup-gpu-runner.sh | bash
```

**Option 3: Self-Hosted Runner**
See **[CI_CD.md](CI_CD.md)** for detailed information about our testing infrastructure and how to set up self-hosted runners with GPU hardware.

**Option 4: Hot Aisle Integration (Optional)**
```bash
# Build with Hot Aisle feature
cargo build --release --features hotaisle

# Integration tests run automatically (no API key required)
# For actual GPU testing:
# 1. Set up HOTAISLE_API_KEY in GitHub Secrets
# 2. Manually trigger "Hot Aisle GPU Testing" workflow
# 3. Tests run on real GPU hardware with automatic cleanup
```

**Option 5: Cloud GPU Setup**
See **[docs/CLOUD_GPU_SETUP.md](docs/CLOUD_GPU_SETUP.md)** for AWS, GCP, and Azure GPU instance setup.

## Documentation

- **[DETAILED.md](DETAILED.md)** - Complete documentation, API reference, and advanced features
- **[CI_CD.md](CI_CD.md)** - CI/CD pipeline and testing infrastructure
- **[docs/HOTAISLE_INTEGRATION.md](docs/HOTAISLE_INTEGRATION.md)** - Hot Aisle integration guide
- **[docs/CLOUD_GPU_SETUP.md](docs/CLOUD_GPU_SETUP.md)** - Cloud GPU setup guide (AWS, GCP, Azure)

## License

This project is licensed under the FSL-1.1-MIT License. See the LICENSE file for details.