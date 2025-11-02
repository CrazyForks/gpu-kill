# CI/CD and Testing Infrastructure

This tests GPU Kill on actual GPU hardware to ensure reliability across all supported vendors.

## Overview

Our CI/CD system provides:
- **✅ Conditional GPU testing** - Automatically runs when GPU hardware is available
- **✅ Multi-vendor GPU testing** on real hardware (NVIDIA, AMD, Intel, Apple Silicon)
- **✅ Cross-platform compatibility** testing
- **✅ Performance benchmarking** and profiling
- **✅ Security auditing** and compliance checks
- **✅ Stress testing** for reliability validation
- **✅ Automated releases** with proper versioning
- **✅ Cloud GPU support** - Easy setup on AWS, GCP, Azure

## CI/CD Pipeline

### 1. Standard CI Tests (`ci.yml`)

**Triggers**: Push to main/develop, pull requests, manual dispatch

**Matrix Testing**:
- **Ubuntu 22.04**: Full feature testing
- **macOS 13**: Apple Silicon compatibility
- **Windows 2022**: Windows compatibility

**Test Coverage**:
- Unit tests with mock NVML
- Integration tests
- **GPU hardware tests** (runs when hardware available, skips gracefully otherwise)
- Code formatting (rustfmt)
- Linting (clippy)
- Security auditing (cargo-audit)
- Multi-platform testing (Ubuntu, macOS, Windows)
- Cross-compilation testing (x86_64, Windows)
- Native macOS compilation

### 2. Conditional GPU Testing

**How It Works**:
- **Automatic Detection**: GPU tests run automatically when hardware is available
- **Graceful Skipping**: Tests skip gracefully when no GPU hardware is found
- **Universal Compatibility**: Works on any runner (hosted, self-hosted, cloud, local)

**Supported Environments**:
- **GitHub Hosted Runners**: Tests skip (no GPU hardware available)
- **Self-Hosted Runners**: Tests run when GPU hardware is detected
- **Cloud GPU Instances**: Tests run automatically (AWS, GCP, Azure)
- **Developer Machines**: Tests run when GPU hardware is available

**Test Coverage**:
- GPU detection and enumeration
- Performance benchmarking
- Memory usage profiling
- Stress testing
- Concurrent access testing
- Long-running stability tests

### 3. GPU Hardware Tests (`gpu-testing.yml`)

**Triggers**: Push to main/develop, pull requests, manual dispatch

**Hardware Matrix**:
- **NVIDIA**: RTX/GTX series with CUDA/NVML
- **AMD**: RX/MI series with ROCm
- **Intel**: Arc/Iris Xe with intel-gpu-tools
- **Apple**: M1/M2/M3/M4 with Metal

**Test Coverage**:
- GPU detection and enumeration
- Performance benchmarking
- Memory usage profiling
- Stress testing
- Concurrent access testing
- Long-running stability tests

### 4. Release Pipeline (`release.yml`)

**Triggers**: Git tags (v*), manual dispatch

**Features**:
- Automated binary builds for all platforms
- Crate publishing to crates.io
- GitHub releases with changelog
- Asset distribution via cargo-dist

## Quick Setup Options

### Option 1: Test Locally (Already Working)
```bash
# Your GPU hardware is automatically detected and tested
cargo test --test gpu_hardware_tests
```

### Option 2: Cloud GPU Setup (5 minutes)
```bash
# On any cloud GPU instance (AWS, GCP, Azure):
curl -sSL https://raw.githubusercontent.com/treadiehq/gpu-kill/main/scripts/setup-gpu-runner.sh | bash
```

See **[docs/CLOUD_GPU_SETUP.md](docs/CLOUD_GPU_SETUP.md)** for detailed cloud provider setup instructions.

### Option 3: Self-Hosted Runners

### Hardware Requirements

#### NVIDIA Runner
- **GPU**: Any NVIDIA GPU with CUDA support
- **OS**: Ubuntu 22.04 LTS
- **RAM**: 16GB+ recommended
- **Storage**: 100GB+ SSD
- **CPU**: 4+ cores

#### AMD Runner
- **GPU**: AMD GPU with ROCm support (RX 5000/6000, MI series)
- **OS**: Ubuntu 22.04 LTS
- **RAM**: 16GB+ recommended
- **Storage**: 100GB+ SSD
- **CPU**: 4+ cores

#### Intel Runner
- **GPU**: Intel Arc, Iris Xe, or integrated GPU
- **OS**: Ubuntu 22.04 LTS
- **RAM**: 8GB+ recommended
- **Storage**: 50GB+ SSD
- **CPU**: 4+ cores

#### Apple Silicon Runner
- **Hardware**: Mac Studio, MacBook Pro, or Mac mini with M1/M2/M3/M4
- **OS**: macOS 13+ (Ventura)
- **RAM**: 16GB+ recommended
- **Storage**: 100GB+ SSD

### Setup Instructions

#### Automated Setup (Recommended)
```bash
# Run the automated setup script
curl -sSL https://raw.githubusercontent.com/treadiehq/gpu-kill/main/scripts/setup-gpu-runner.sh | bash
```

#### Manual Setup
See [`.github/workflows/self-hosted-setup.md`](.github/workflows/self-hosted-setup.md) for detailed manual setup instructions.

### Runner Labels

Each runner is configured with specific labels:
- `self-hosted` - Required for self-hosted runners
- `gpu` - Indicates GPU hardware availability
- `nvidia`/`amd`/`intel`/`apple` - GPU vendor
- `ubuntu-22.04`/`macos-13` - Operating system
- `stress-test` - For runners capable of stress testing

## Test Categories

### 1. Unit Tests
- **Location**: `src/` modules
- **Coverage**: Individual function testing
- **Mocking**: Uses `mock_nvml` feature for GPU-independent testing
- **Execution**: `cargo test --features mock_nvml`

### 2. Integration Tests
- **Location**: `tests/integration_tests.rs`
- **Coverage**: End-to-end CLI functionality
- **Mocking**: Uses mock NVML for consistent testing
- **Execution**: `cargo test --test integration_tests`

### 3. GPU Hardware Tests
- **Location**: `tests/gpu_hardware_tests.rs`
- **Coverage**: Real GPU hardware testing
- **Requirements**: Actual GPU hardware
- **Execution**: `cargo test --test gpu_hardware_tests`

### 4. Performance Tests
- **Coverage**: Benchmarking and profiling
- **Tools**: `time`, `valgrind`, `massif`
- **Metrics**: Execution time, memory usage, throughput

### 5. Stress Tests
- **Coverage**: Long-running stability
- **Scenarios**: 100+ iterations, concurrent access, extended monitoring
- **Duration**: 30+ seconds continuous operation

## Test Execution

### Local Testing

```bash
# Run all tests (GPU tests run automatically if hardware available)
cargo test

# Run with mock NVML (no GPU required)
cargo test --features mock_nvml

# Run integration tests
cargo test --test integration_tests

# Run GPU hardware tests (runs automatically if GPU available, skips gracefully otherwise)
cargo test --test gpu_hardware_tests

# Run specific vendor tests
cargo test --test gpu_hardware_tests nvidia_hardware_tests
cargo test --test gpu_hardware_tests amd_hardware_tests
```

### Conditional GPU Testing

The GPU tests are designed to work automatically:

- **✅ With GPU Hardware**: Tests run automatically and test actual GPU functionality
- **✅ Without GPU Hardware**: Tests skip gracefully with informative messages
- **✅ On Any System**: Works on GitHub runners, self-hosted runners, cloud instances, and developer machines

**Example Output**:
```bash
# On system with GPU (like your Apple M3 Max):
cargo test --test gpu_hardware_tests
# ✅ test_gpu_detection ... ok
# ✅ test_gpu_performance ... ok
# ✅ test_gpu_stress ... ok

# On system without GPU (like GitHub hosted runners):
cargo test --test gpu_hardware_tests
# ⏭️  test_gpu_detection ... ignored (no GPU hardware detected)
# ⏭️  test_gpu_performance ... ignored (no GPU hardware detected)
# ⏭️  test_gpu_stress ... ignored (no GPU hardware detected)
```

### CI Testing

```bash
# Trigger manual workflow
gh workflow run ci.yml
gh workflow run gpu-testing.yml

# Trigger with specific vendor
gh workflow run gpu-testing.yml -f gpu_vendor=nvidia
gh workflow run gpu-testing.yml -f gpu_vendor=amd
```

## Performance Benchmarks

### GPU Listing Performance
- **Target**: < 5 seconds for basic listing
- **Target**: < 10 seconds for detailed listing
- **Measurement**: End-to-end execution time

### Memory Usage
- **Target**: < 100MB peak memory usage
- **Measurement**: Valgrind massif profiling
- **Validation**: No memory leaks detected

### Concurrent Access
- **Target**: Support 10+ concurrent operations
- **Measurement**: Multiple simultaneous GPU queries
- **Validation**: No deadlocks or race conditions

## Security and Compliance

### Security Auditing
- **Tool**: `cargo-audit`
- **Coverage**: Dependency vulnerability scanning
- **Frequency**: Every CI run

### Code Quality
- **Tool**: `cargo-clippy`
- **Coverage**: Rust best practices and warnings
- **Policy**: Zero warnings allowed

### Formatting
- **Tool**: `cargo-fmt`
- **Coverage**: Consistent code formatting
- **Policy**: All code must be formatted

## Monitoring and Alerting

### Test Results
- **Dashboard**: GitHub Actions summary
- **Artifacts**: Test results and profiling data
- **Retention**: 7 days (30 days for profiling)

### Failure Handling
- **Notification**: GitHub notifications
- **Retry**: Automatic retry for transient failures
- **Escalation**: Manual intervention for persistent failures

### Performance Tracking
- **Metrics**: Execution time trends
- **Alerts**: Performance regression detection
- **Reporting**: Weekly performance summaries

## Contributing to CI/CD

### Adding New Tests

1. **Unit Tests**: Add to appropriate module in `src/`
2. **Integration Tests**: Add to `tests/integration_tests.rs`
3. **Hardware Tests**: Add to `tests/gpu_hardware_tests.rs`

### Modifying CI Pipeline

1. **Edit Workflows**: Modify `.github/workflows/*.yml` files
2. **Test Locally**: Use `act` or similar tools
3. **Validate**: Ensure all tests pass
4. **Document**: Update this file with changes

### Adding New GPU Vendors

1. **Hardware Setup**: Follow self-hosted runner setup guide
2. **Test Matrix**: Add vendor to CI matrix
3. **Hardware Tests**: Add vendor-specific tests
4. **Documentation**: Update setup instructions

## Troubleshooting

### Common Issues

1. **GPU Not Detected**
   ```bash
   # Check GPU status
   nvidia-smi  # NVIDIA
   rocm-smi    # AMD
   intel_gpu_top --help  # Intel
   system_profiler SPDisplaysDataType  # Apple
   ```

2. **Runner Connection Issues**
   ```bash
   # Check runner status
   sudo systemctl status actions.runner.*
   
   # Restart runner
   sudo systemctl restart actions.runner.*
   ```

3. **Test Failures**
   ```bash
   # Run with verbose output
   cargo test --test gpu_hardware_tests -- --nocapture
   
   # Check logs
   tail -f /home/runner/_diag/Runner_*.log
   ```

### Performance Issues

1. **Slow Tests**
   - Check GPU driver status
   - Verify system resources
   - Review test timeout settings

2. **Memory Issues**
   - Monitor system memory usage
   - Check for memory leaks
   - Review test data sizes

3. **Concurrency Issues**
   - Check GPU access permissions
   - Verify thread safety
   - Review locking mechanisms

## Cloud GPU Integration

### Supported Cloud Providers

#### AWS EC2 with GPU
- **Instance Types**: g4dn.xlarge, p3.2xlarge, p4d.xlarge
- **Setup**: Automated with setup script
- **Cost**: ~$0.50-3.00/hour (spot instances: 80-90% savings)

#### Google Cloud with GPU
- **Instance Types**: n1-standard-4 with T4, V100, A100
- **Setup**: Automated with setup script
- **Cost**: ~$0.35-2.50/hour (preemptible: 80% savings)

#### Azure with GPU
- **Instance Types**: Standard_NC6s_v3, Standard_NC12s_v3
- **Setup**: Automated with setup script
- **Cost**: ~$0.90-3.50/hour (spot instances: 90% savings)

### Quick Cloud Setup
```bash
# 1. Launch GPU instance on your preferred cloud provider
# 2. Connect via SSH
# 3. Run automated setup:
curl -sSL https://raw.githubusercontent.com/treadiehq/gpu-kill/main/scripts/setup-gpu-runner.sh | bash
```

### Cost Optimization
- **Spot/Preemptible Instances**: 80-90% cost savings
- **Auto-shutdown**: Prevent runaway costs
- **Scheduled Testing**: Only run during business hours
- **Docker Containers**: Efficient resource usage

See **[docs/CLOUD_GPU_SETUP.md](docs/CLOUD_GPU_SETUP.md)** for detailed setup instructions.

## Future Enhancements

### Planned Features
- **✅ GPU Cloud Integration**: AWS/GCP/Azure GPU instances (Implemented)
- **Distributed Testing**: Multi-node GPU clusters
- **Advanced Profiling**: GPU utilization monitoring
- **Automated Benchmarking**: Performance regression detection
- **Load Testing**: High-throughput scenarios
