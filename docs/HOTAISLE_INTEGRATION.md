# Hot Aisle Integration for GPU Testing

This document describes the **optional** integration between GPU Kill and Hot Aisle's infrastructure for automated GPU testing in CI/CD pipelines.

> **Note**: Hot Aisle integration is an optional feature that must be enabled with the `hotaisle` feature flag.

## Overview

The Hot Aisle integration enables GPU Kill to run comprehensive tests on real GPU hardware by:

1. **Provisioning GPU instances** on-demand via Hot Aisle's API
2. **Running GPU tests** on actual hardware (NVIDIA, AMD, Intel, Apple Silicon)
3. **Automated cleanup** to minimize costs
4. **Comprehensive reporting** of test results

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   GitHub        │    │   Hot Aisle     │    │   GPU Hardware  │
│   Actions       │◄──►│   API           │◄──►│   (NVIDIA/AMD)  │
│   (CI/CD)       │    │   (Backend)     │    │   (Intel/Apple) │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Components

### 1. Hot Aisle API Client (`src/hotaisle_client.rs`)

Rust client for interacting with Hot Aisle's API:

```rust
use gpukill::hotaisle_client::{HotAisleClient, GpuInstanceConfig};

let client = HotAisleClient::new(api_key, None);

let config = GpuInstanceConfig {
    gpu_type: "nvidia".to_string(),
    duration_minutes: 30,
    instance_type: Some("g4dn.xlarge".to_string()),
    labels: Some(vec!["ci-test".to_string()]),
};

let instance = client.provision_gpu_instance(config).await?;
```

### 2. GPU Test Script (`scripts/run-gpu-tests.sh`)

Comprehensive test script that runs on provisioned instances:

- **GPU Detection Tests**: Verify GPU enumeration and information retrieval
- **Vendor-Specific Tests**: NVIDIA (nvidia-smi), AMD (rocm-smi, amd-smi), Intel (intel_gpu_top)
- **Performance Tests**: Run GPU hardware tests and benchmarks
- **Stress Tests**: Multiple iterations to ensure reliability
- **Report Generation**: Detailed test reports with system information

### 3. GitHub Actions Workflow (`.github/workflows/hotaisle-gpu-testing.yml`)

Automated CI/CD pipeline that:

- **Provisions GPU instances** based on matrix strategy
- **Deploys GPU Kill** to instances
- **Runs comprehensive tests** on real hardware
- **Collects results** and uploads artifacts
- **Cleans up instances** automatically

## Setup Instructions

### 1. Enable Hot Aisle Feature

Build GPU Kill with the Hot Aisle feature enabled:

```bash
# Build with Hot Aisle integration
cargo build --release --features hotaisle

# Or install with Hot Aisle integration
cargo install --path . --features hotaisle
```

### 2. Hot Aisle API Key

Add your Hot Aisle API key to GitHub Secrets:

```bash
# In your GitHub repository settings:
# Settings → Secrets and variables → Actions → New repository secret
# Name: HOTAISLE_API_KEY
# Value: your-hotaisle-api-key
```

### 3. Configure GPU Types

The workflow supports testing multiple GPU types:

```yaml
# Default configuration
matrix:
  gpu_type: [nvidia, amd, intel]

# Manual dispatch with custom GPU types
# Use workflow_dispatch with inputs:
# gpu_types: "nvidia,amd,intel,apple-silicon"
```

### 4. Test Duration

Configure test duration to balance thoroughness with cost:

```yaml
# Default: 30 minutes
# Can be overridden via workflow_dispatch
test_duration: "30"  # minutes
```

## Usage

### Automatic Testing

GPU tests run automatically on:
- **Main branch** pushes
- **Develop branch** pushes
- **Pull requests** to main branch

### Manual Testing

Trigger tests manually via GitHub Actions:

1. Go to **Actions** tab in your repository
2. Select **Hot Aisle GPU Testing** workflow
3. Click **Run workflow**
4. Configure parameters:
   - **GPU types**: Comma-separated list (e.g., `nvidia,amd,intel`)
   - **Test duration**: Minutes (e.g., `30`)

### Local Testing

Test the integration locally:

```bash
# Build GPU Kill
cargo build --release

# Run GPU tests (requires GPU hardware)
./scripts/run-gpu-tests.sh nvidia
```

## Supported GPU Types

| GPU Type | Tools Used | Tests |
|----------|------------|-------|
| **NVIDIA** | nvidia-smi, NVML | GPU enumeration, memory, utilization, temperature, power |
| **AMD** | rocm-smi, amd-smi | GPU enumeration, memory, utilization, temperature, power |
| **Intel** | intel_gpu_top | GPU enumeration, utilization, memory estimation |
| **Apple Silicon** | system_profiler | GPU enumeration, memory usage, Metal processes |

## Cost Optimization

### 1. Instance Lifecycle Management

- **Automatic provisioning** only when needed
- **Immediate cleanup** after tests complete
- **Timeout protection** to prevent runaway costs

### 2. Test Duration Control

- **Configurable duration** (default: 30 minutes)
- **Fast failure** for quick feedback
- **Comprehensive testing** when needed

### 3. Resource Efficiency

- **Parallel testing** across GPU types
- **Shared infrastructure** via Hot Aisle
- **No always-on runners** required

## Test Results

### Artifacts

Each test run produces:

- **Test Output Log**: Detailed execution logs
- **Test Report**: Comprehensive system and GPU information
- **Retention**: 30 days for debugging

### Metrics

Tests measure:

- **GPU Detection**: Number of GPUs found
- **Information Retrieval**: JSON validity and completeness
- **Performance**: Test execution time
- **Reliability**: Stress test success rate

## Troubleshooting

### Common Issues

1. **Instance Provisioning Fails**
   - Check Hot Aisle API key validity
   - Verify GPU type availability
   - Check Hot Aisle service status

2. **SSH Connection Issues**
   - Verify instance IP address
   - Check SSH key generation
   - Ensure instance is ready

3. **Test Failures**
   - Review test output logs
   - Check GPU driver installation
   - Verify tool availability (nvidia-smi, rocm-smi, etc.)

### Debug Mode

Enable debug logging:

```bash
export RUST_LOG=debug
export RUST_BACKTRACE=1
```

## API Reference

### HotAisleClient

```rust
impl HotAisleClient {
    pub fn new(api_key: String, base_url: Option<String>) -> Self
    pub async fn provision_gpu_instance(&self, config: GpuInstanceConfig) -> Result<GpuInstance>
    pub async fn wait_for_instance_ready(&self, instance_id: &str, timeout_minutes: u32) -> Result<GpuInstance>
    pub async fn get_instance(&self, instance_id: &str) -> Result<GpuInstance>
    pub async fn run_gpu_tests(&self, instance: &GpuInstance, test_config: &GpuTestConfig) -> Result<GpuTestResults>
    pub async fn terminate_instance(&self, instance_id: &str) -> Result<()>
    pub async fn list_available_gpu_types(&self) -> Result<Vec<String>>
}
```

### Configuration Types

```rust
pub struct GpuInstanceConfig {
    pub gpu_type: String,           // nvidia, amd, intel, apple-silicon
    pub duration_minutes: u32,      // Instance lifetime
    pub instance_type: Option<String>, // Auto-selected if None
    pub labels: Option<Vec<String>>,   // Custom labels
}

pub struct GpuTestConfig {
    pub test_command: String,       // Command to execute
    pub timeout_minutes: u32,       // Test timeout
    pub env_vars: Option<HashMap<String, String>>, // Environment variables
    pub working_dir: Option<String>, // Working directory
}
```

## Future Enhancements

### Planned Features

1. **Advanced GPU Testing**
   - CUDA/ROCm kernel testing
   - Memory bandwidth benchmarks
   - Multi-GPU coordination tests

2. **Cost Analytics**
   - Test cost tracking
   - Optimization recommendations
   - Budget alerts

3. **Integration Improvements**
   - Webhook notifications
   - Slack/Teams integration
   - Custom test configurations

### Contributing

To contribute to the Hot Aisle integration:

1. **Fork the repository**
2. **Create a feature branch**
3. **Add tests** for new functionality
4. **Update documentation**
5. **Submit a pull request**

## Support

For issues related to:

- **GPU Kill**: Create an issue in this repository
- **Hot Aisle API**: Contact Hot Aisle support
- **Integration**: Check the troubleshooting section above

## License

This integration is part of GPU Kill and follows the same license terms.
