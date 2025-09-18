# GPU Kill

A lightweight CLI tool for managing NVIDIA, AMD, Intel, and Apple Silicon GPUs. List, monitor, and control GPU processes with ease.

## What it does

- **📊 Monitor**: See GPU usage, memory, temperature, and running processes
- **⚡ Kill**: Gracefully terminate GPU processes (with force option)
- **🔄 Reset**: Reset individual GPUs or all GPUs
- **👀 Watch**: Real-time monitoring with auto-refresh
- **🎯 Filter**: Advanced process filtering and batch operations
- **📈 Audit**: Track GPU usage history and generate usage reports
- **🔧 Multi-Vendor**: Support for NVIDIA, AMD, Intel, and Apple Silicon GPUs

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