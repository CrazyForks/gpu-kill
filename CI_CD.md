# CI/CD and Testing Infrastructure

This tests GPU Kill on actual GPU hardware to ensure reliability across all supported vendors.

## Overview

Our CI/CD system provides:
- **Multi-vendor GPU testing** on real hardware (NVIDIA, AMD, Intel, Apple Silicon)
- **Cross-platform compatibility** testing
- **Performance benchmarking** and profiling
- **Security auditing** and compliance checks
- **Stress testing** for reliability validation
- **Automated releases** with proper versioning

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
- Code formatting (rustfmt)
- Linting (clippy)
- Security auditing (cargo-audit)
- Multi-platform testing (Ubuntu, macOS, Windows)
- Cross-compilation testing (x86_64, ARM64, Windows)
- Native macOS compilation

### 2. GPU Hardware Tests (`gpu-testing.yml`)

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

### 3. Release Pipeline (`release.yml`)

**Triggers**: Git tags (v*), manual dispatch

**Features**:
- Automated binary builds for all platforms
- Crate publishing to crates.io
- GitHub releases with changelog
- Asset distribution via cargo-dist

## Self-Hosted Runners

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

See [`.github/workflows/self-hosted-setup.md`](.github/workflows/self-hosted-setup.md) for detailed setup instructions.

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
# Run all tests
cargo test

# Run with mock NVML (no GPU required)
cargo test --features mock_nvml

# Run integration tests
cargo test --test integration_tests

# Run GPU hardware tests (requires GPU)
cargo test --test gpu_hardware_tests

# Run specific vendor tests
cargo test --test gpu_hardware_tests nvidia_hardware_tests
cargo test --test gpu_hardware_tests amd_hardware_tests
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

## Future Enhancements

### Planned Features
- **GPU Cloud Integration**: AWS/GCP/Azure GPU instances
- **Distributed Testing**: Multi-node GPU clusters
- **Advanced Profiling**: GPU utilization monitoring
- **Automated Benchmarking**: Performance regression detection
- **Load Testing**: High-throughput scenarios
