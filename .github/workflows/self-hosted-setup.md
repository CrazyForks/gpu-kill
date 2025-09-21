# Self-Hosted GPU Runner Setup Guide

This guide explains how to set up self-hosted GitHub Actions runners with GPU hardware for testing of GPU Kill.

## Overview

GPU Kill requires actual GPU hardware to test all functionality. This setup provides:
- **NVIDIA GPU testing** with CUDA/NVML
- **AMD GPU testing** with ROCm
- **Intel GPU testing** with intel-gpu-tools
- **Apple Silicon testing** on macOS
- **Cross-platform compatibility** testing

## Hardware Requirements

### NVIDIA Runner
- **GPU**: Any NVIDIA GPU with CUDA support
- **OS**: Ubuntu 22.04 LTS
- **RAM**: 16GB+ recommended
- **Storage**: 100GB+ SSD
- **CPU**: 4+ cores

### AMD Runner
- **GPU**: AMD GPU with ROCm support (RX 5000/6000 series, MI series)
- **OS**: Ubuntu 22.04 LTS
- **RAM**: 16GB+ recommended
- **Storage**: 100GB+ SSD
- **CPU**: 4+ cores

### Intel Runner
- **GPU**: Intel Arc, Iris Xe, or integrated GPU
- **OS**: Ubuntu 22.04 LTS
- **RAM**: 8GB+ recommended
- **Storage**: 50GB+ SSD
- **CPU**: 4+ cores

### Apple Silicon Runner
- **Hardware**: Mac Studio, MacBook Pro, or Mac mini with M1/M2/M3/M4
- **OS**: macOS 13+ (Ventura)
- **RAM**: 16GB+ recommended
- **Storage**: 100GB+ SSD

## Setup Instructions

### 1. NVIDIA Runner Setup

```bash
# Install Ubuntu 22.04 LTS
# Update system
sudo apt update && sudo apt upgrade -y

# Install NVIDIA drivers
sudo apt install -y nvidia-driver-535
sudo reboot

# Verify NVIDIA installation
nvidia-smi

# Install development tools
sudo apt install -y build-essential curl git libssl-dev pkg-config

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install GitHub Actions runner
mkdir actions-runner && cd actions-runner
curl -o actions-runner-linux-x64-2.311.0.tar.gz -L https://github.com/actions/runner/releases/download/v2.311.0/actions-runner-linux-x64-2.311.0.tar.gz
tar xzf ./actions-runner-linux-x64-2.311.0.tar.gz

# Configure runner (get token from GitHub repo settings)
./config.sh --url https://github.com/kagehq/gpu-kill --token <YOUR_TOKEN>
./config.sh --name "nvidia-gpu-runner" --labels "self-hosted,gpu,nvidia,ubuntu-22.04"

# Install as service
sudo ./svc.sh install
sudo ./svc.sh start
```

### 2. AMD Runner Setup

```bash
# Install Ubuntu 22.04 LTS
# Update system
sudo apt update && sudo apt upgrade -y

# Install ROCm
wget https://repo.radeon.com/amdgpu-install/5.7/ubuntu/jammy/amdgpu-install_5.7.50700-1_all.deb
sudo dpkg -i amdgpu-install_5.7.50700-1_all.deb
sudo apt-get update
sudo amdgpu-install --usecase=rocm

# Verify ROCm installation
rocm-smi
rocminfo

# Install development tools
sudo apt install -y build-essential curl git libssl-dev pkg-config

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install GitHub Actions runner (same as NVIDIA)
mkdir actions-runner && cd actions-runner
curl -o actions-runner-linux-x64-2.311.0.tar.gz -L https://github.com/actions/runner/releases/download/v2.311.0/actions-runner-linux-x64-2.311.0.tar.gz
tar xzf ./actions-runner-linux-x64-2.311.0.tar.gz

# Configure runner
./config.sh --url https://github.com/kagehq/gpu-kill --token <YOUR_TOKEN>
./config.sh --name "amd-gpu-runner" --labels "self-hosted,gpu,amd,ubuntu-22.04"

# Install as service
sudo ./svc.sh install
sudo ./svc.sh start
```

### 3. Intel Runner Setup

```bash
# Install Ubuntu 22.04 LTS
# Update system
sudo apt update && sudo apt upgrade -y

# Install Intel GPU tools
sudo apt install -y intel-gpu-tools

# Verify Intel GPU tools
intel_gpu_top --help

# Install development tools
sudo apt install -y build-essential curl git libssl-dev pkg-config

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install GitHub Actions runner
mkdir actions-runner && cd actions-runner
curl -o actions-runner-linux-x64-2.311.0.tar.gz -L https://github.com/actions/runner/releases/download/v2.311.0/actions-runner-linux-x64-2.311.0.tar.gz
tar xzf ./actions-runner-linux-x64-2.311.0.tar.gz

# Configure runner
./config.sh --url https://github.com/kagehq/gpu-kill --token <YOUR_TOKEN>
./config.sh --name "intel-gpu-runner" --labels "self-hosted,gpu,intel,ubuntu-22.04"

# Install as service
sudo ./svc.sh install
sudo ./svc.sh start
```

### 4. Apple Silicon Runner Setup

```bash
# Install macOS 13+ (Ventura)
# Install Xcode command line tools
xcode-select --install

# Install Homebrew
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify Apple Silicon GPU
system_profiler SPDisplaysDataType

# Install GitHub Actions runner
mkdir actions-runner && cd actions-runner
curl -o actions-runner-osx-arm64-2.311.0.tar.gz -L https://github.com/actions/runner/releases/download/v2.311.0/actions-runner-osx-arm64-2.311.0.tar.gz
tar xzf ./actions-runner-osx-arm64-2.311.0.tar.gz

# Configure runner
./config.sh --url https://github.com/kagehq/gpu-kill --token <YOUR_TOKEN>
./config.sh --name "apple-gpu-runner" --labels "self-hosted,gpu,apple,macos-13"

# Install as service
./svc.sh install
./svc.sh start
```

## Runner Labels

Each runner should be configured with these labels:
- `self-hosted` - Required for self-hosted runners
- `gpu` - Indicates GPU hardware availability
- `nvidia`/`amd`/`intel`/`apple` - GPU vendor
- `ubuntu-22.04`/`macos-13` - Operating system
- `stress-test` - For runners capable of stress testing

## Security Considerations

1. **Network Security**: Ensure runners are behind a firewall
2. **Access Control**: Limit who can access the runner machines
3. **Token Management**: Regularly rotate GitHub tokens
4. **Monitoring**: Monitor runner health and performance
5. **Updates**: Keep runners updated with security patches

## Monitoring and Maintenance

### Health Checks
```bash
# Check runner status
sudo systemctl status actions.runner.*

# Check GPU status
nvidia-smi  # NVIDIA
rocm-smi    # AMD
intel_gpu_top --help  # Intel
system_profiler SPDisplaysDataType  # Apple
```

### Logs
```bash
# View runner logs
sudo journalctl -u actions.runner.* -f

# View GitHub Actions logs
tail -f /home/runner/_diag/Runner_*.log
```

### Updates
```bash
# Update runner software
cd actions-runner
./config.sh remove --token <NEW_TOKEN>
# Download new version
./config.sh --url https://github.com/kagehq/gpu-kill --token <NEW_TOKEN>
```

## Cost Optimization

1. **Scheduled Testing**: Run tests during off-peak hours
2. **Resource Scaling**: Use smaller instances for basic tests
3. **Caching**: Implement aggressive caching for dependencies
4. **Parallel Testing**: Run multiple test suites in parallel

## Troubleshooting

### Common Issues

1. **GPU Not Detected**
   ```bash
   # Check GPU status
   lspci | grep -i vga
   nvidia-smi  # or rocm-smi, intel_gpu_top
   ```

2. **Permission Issues**
   ```bash
   # Add user to video group
   sudo usermod -a -G video $USER
   sudo usermod -a -G render $USER
   ```

3. **Driver Issues**
   ```bash
   # Reinstall drivers
   sudo apt purge nvidia-*  # NVIDIA
   sudo apt purge rocm-*    # AMD
   sudo apt install nvidia-driver-535  # Reinstall
   ```

4. **Runner Connection Issues**
   ```bash
   # Check network connectivity
   curl -I https://github.com
   # Restart runner service
   sudo systemctl restart actions.runner.*
   ```

## Integration with GPU Kill

The runners will automatically execute the GPU testing workflow when:
- Code is pushed to main/develop branches
- Pull requests are opened
- Manual workflow dispatch is triggered

Tests include:
- GPU detection and enumeration
- Performance benchmarking
- Memory usage testing
- Stress testing
- Cross-platform compatibility
- Security auditing
