# GPU Kill

A lightweight CLI tool for managing NVIDIA and AMD GPUs. List, monitor, and control GPU processes with ease.

## What it does

- **📊 Monitor**: See GPU usage, memory, temperature, and running processes
- **⚡ Kill**: Gracefully terminate GPU processes (with force option)
- **🔄 Reset**: Reset individual GPUs or all GPUs
- **👀 Watch**: Real-time monitoring with auto-refresh
- **📋 Export**: JSON output for scripting and automation
- **🎯 Filter**: Advanced process filtering and batch operations
- **🐳 Containers**: Container-aware process detection
- **🔧 Multi-Vendor**: Support for both NVIDIA and AMD GPUs

## Requirements

- NVIDIA GPU with drivers installed (for NVIDIA support)
- AMD GPU with ROCm installed (for AMD support)
- Linux, macOS, or Windows

## Quick Start

```bash
# Install
cargo install gpukill

# List your GPUs
gpukill --list

# Watch GPU usage in real-time
gpukill --list --watch

# Kill a stuck process
gpukill --kill --pid 12345

# Reset a GPU
gpukill --reset --gpu 0
```

## Basic Usage

### List GPUs
```bash
gpukill --list                    # Basic info
gpukill --list --details          # Show all processes
gpukill --list --watch            # Auto-refresh every 2s
gpukill --list --output json      # JSON format
gpukill --list --vendor nvidia    # Filter by vendor
gpukill --list --containers       # Show container info
```

### Kill Processes
```bash
gpukill --kill --pid 12345        # Graceful kill (5s timeout)
gpukill --kill --pid 12345 --force # Force kill after timeout
gpukill --kill --pid 12345 --timeout-secs 10  # Custom timeout
gpukill --kill --filter "python.*" # Kill by pattern
gpukill --kill --filter "python.*" --batch # Batch kill
```

### Reset GPUs
```bash
gpukill --reset --gpu 0           # Reset GPU 0
gpukill --reset --all             # Reset all GPUs
gpukill --reset --gpu 0 --force   # Force reset (ignores active processes)
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

**Monitor only NVIDIA GPUs:**
```bash
gpukill --list --vendor nvidia --watch
```

**Check containerized processes:**
```bash
gpukill --list --containers --details
```

**Reset crashed GPU:**
```bash
gpukill --reset --gpu 0 --force
```

**Export stats for logging:**
```bash
gpukill --list --output json > gpu_stats.json
```


## Troubleshooting

**"NVML initialization failed"**
- Make sure NVIDIA drivers are installed
- Try running with `sudo` if needed
- Check `nvidia-smi` works

**"AMD ROCm not installed"**
- Install ROCm drivers for AMD GPU support
- Check `rocm-smi` works

**"Permission denied"**
- Some operations need elevated privileges
- Try `sudo gpukill --list` first

**"GPU not found"**
- Check GPU index with `gpukill --list`
- Verify GPU is properly connected

**"No GPU vendors available"**
- Ensure at least one GPU vendor (NVIDIA or AMD) is properly installed
- Check driver installation and GPU connectivity

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
