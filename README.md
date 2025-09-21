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
- **Dashboard**: Web interface for cluster monitoring
- **Remote**: Manage GPUs across multiple servers
- **Multi-Vendor**: Works with NVIDIA, AMD, Intel, and Apple Silicon
- **AI Integration**: MCP server for AI assistant integration

## Requirements

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
git clone https://github.com/kagehq/gpu-kill.git
cd gpu-kill
cargo build --release

# Or install via Cargo
cargo install gpukill

# List your GPUs
gpukill --list

# Watch GPU usage in real-time
gpukill --list --watch
```

### Common Tasks
```bash
# Kill a stuck process
gpukill --kill --pid 12345 --force

# Reset a crashed GPU
gpukill --reset --gpu 0 --force

# Start the web dashboard (backend only)
gpukill --server --server-port 8080
```

## Dashboard

Start the web interface for cluster monitoring:

```bash
# 1. Start the backend API server
gpukill --server --server-port 8080

# 2. Start the dashboard UI (in a new terminal)
cd dashboard
npm install  # First time only
npm run dev

# 3. Access the dashboard
open http://localhost:3000
```

**Note**: You need both the backend server (port 8080) and frontend UI (port 3000) running for the dashboard to work.

![GPU Kill Dashboard](dashboard/public/screenshot.png)

The dashboard provides:
- **Real-time monitoring** of all GPUs
- **Security detection** with threat analysis
- **Policy management** for resource control
- **Cluster overview** with Magic Moment insights

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

GPU Kill uses a comprehensive CI/CD pipeline that tests on actual GPU hardware:

- **Multi-vendor GPU testing** on real hardware (NVIDIA, AMD, Intel, Apple Silicon)
- **Cross-platform compatibility** testing
- **Performance benchmarking** and profiling
- **Security auditing** and compliance checks
- **Stress testing** for reliability validation

See **[CI_CD.md](CI_CD.md)** for detailed information about our testing infrastructure and how to set up self-hosted runners with GPU hardware.

## Documentation

- **[DETAILED.md](DETAILED.md)** - Complete documentation, API reference, and advanced features
- **[Dashboard README](dashboard/README.md)** - Web interface documentation
- **[CI_CD.md](CI_CD.md)** - CI/CD pipeline and testing infrastructure

## License

This project is licensed under the FSL-1.1-MIT License. See the LICENSE file for details.