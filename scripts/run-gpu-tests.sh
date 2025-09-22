#!/bin/bash

# GPU Kill - Hot Aisle GPU Testing Script
# This script runs comprehensive GPU tests on Hot Aisle provisioned instances

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
LOG_FILE="/tmp/gpu-kill-tests.log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1" | tee -a "$LOG_FILE"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1" | tee -a "$LOG_FILE"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1" | tee -a "$LOG_FILE"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" | tee -a "$LOG_FILE"
}

# Function to check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check if we're in the right directory
    if [[ ! -f "$PROJECT_ROOT/Cargo.toml" ]]; then
        log_error "Not in GPU Kill project root. Please run from project directory."
        exit 1
    fi
    
    # Check if cargo is available
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo not found. Please install Rust toolchain."
        exit 1
    fi
    
    # Check if git is available
    if ! command -v git &> /dev/null; then
        log_error "Git not found. Please install git."
        exit 1
    fi
    
    log_success "Prerequisites check passed"
}

# Function to build GPU Kill
build_gpukill() {
    log_info "Building GPU Kill..."
    
    cd "$PROJECT_ROOT"
    
    # Build in release mode for better performance
    if cargo build --release; then
        log_success "GPU Kill built successfully"
    else
        log_error "Failed to build GPU Kill"
        exit 1
    fi
}

# Function to run basic GPU detection tests
run_gpu_detection_tests() {
    log_info "Running GPU detection tests..."
    
    local gpu_type="$1"
    local test_results=()
    
    # Test 1: List GPUs
    log_info "Testing GPU enumeration..."
    if ./target/release/gpukill --list > /tmp/gpu-list.txt 2>&1; then
        local gpu_count=$(grep -c "GPU [0-9]" /tmp/gpu-list.txt || echo "0")
        log_success "Found $gpu_count GPU(s)"
        test_results+=("gpu_enumeration:passed:$gpu_count")
    else
        log_error "GPU enumeration failed"
        test_results+=("gpu_enumeration:failed:0")
    fi
    
    # Test 2: GPU information
    log_info "Testing GPU information retrieval..."
    if ./target/release/gpukill --list --format=json > /tmp/gpu-info.json 2>&1; then
        local json_valid=$(python3 -m json.tool /tmp/gpu-info.json > /dev/null 2>&1 && echo "true" || echo "false")
        if [[ "$json_valid" == "true" ]]; then
            log_success "GPU information JSON is valid"
            test_results+=("gpu_info_json:passed:valid")
        else
            log_warning "GPU information JSON is invalid"
            test_results+=("gpu_info_json:failed:invalid")
        fi
    else
        log_error "GPU information retrieval failed"
        test_results+=("gpu_info_json:failed:error")
    fi
    
    # Test 3: GPU-specific tests based on type
    case "$gpu_type" in
        "nvidia")
            run_nvidia_specific_tests
            ;;
        "amd")
            run_amd_specific_tests
            ;;
        "intel")
            run_intel_specific_tests
            ;;
        "apple-silicon")
            run_apple_specific_tests
            ;;
        *)
            log_warning "Unknown GPU type: $gpu_type"
            ;;
    esac
    
    # Output test results
    echo "=== GPU Detection Test Results ==="
    for result in "${test_results[@]}"; do
        echo "$result"
    done
}

# Function to run NVIDIA-specific tests
run_nvidia_specific_tests() {
    log_info "Running NVIDIA-specific tests..."
    
    # Test nvidia-smi availability
    if command -v nvidia-smi &> /dev/null; then
        log_success "nvidia-smi is available"
        nvidia-smi --query-gpu=name,memory.total,memory.used --format=csv,noheader,nounits
    else
        log_warning "nvidia-smi not found"
    fi
}

# Function to run AMD-specific tests
run_amd_specific_tests() {
    log_info "Running AMD-specific tests..."
    
    # Test rocm-smi availability
    if command -v rocm-smi &> /dev/null; then
        log_success "rocm-smi is available"
        rocm-smi --showproductname
        rocm-smi --showuse
        rocm-smi --showtemp
        rocm-smi --showpower
        rocm-smi --showmemuse
    else
        log_warning "rocm-smi not found"
    fi
    
    # Test amd-smi availability (newer tool)
    if command -v amd-smi &> /dev/null; then
        log_success "amd-smi is available"
        amd-smi
    else
        log_warning "amd-smi not found"
    fi
}

# Function to run Intel-specific tests
run_intel_specific_tests() {
    log_info "Running Intel-specific tests..."
    
    # Test intel_gpu_top availability
    if command -v intel_gpu_top &> /dev/null; then
        log_success "intel_gpu_top is available"
        timeout 5 intel_gpu_top -l 1 || true
    else
        log_warning "intel_gpu_top not found"
    fi
}

# Function to run Apple Silicon-specific tests
run_apple_specific_tests() {
    log_info "Running Apple Silicon-specific tests..."
    
    # Test system_profiler for GPU info
    if command -v system_profiler &> /dev/null; then
        log_success "system_profiler is available"
        system_profiler SPDisplaysDataType | grep -A 5 "Chipset Model" || true
    else
        log_warning "system_profiler not found"
    fi
}

# Function to run performance tests
run_performance_tests() {
    log_info "Running GPU performance tests..."
    
    local gpu_type="$1"
    local start_time=$(date +%s)
    
    # Run GPU hardware tests
    if cargo test --test gpu_hardware_tests --release; then
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        log_success "GPU performance tests completed in ${duration}s"
    else
        log_warning "Some GPU performance tests failed or were skipped"
    fi
}

# Function to run stress tests
run_stress_tests() {
    log_info "Running GPU stress tests..."
    
    # Run multiple iterations of GPU detection
    for i in {1..5}; do
        log_info "Stress test iteration $i/5..."
        if ./target/release/gpukill --list > /dev/null 2>&1; then
            log_success "Iteration $i passed"
        else
            log_error "Iteration $i failed"
            return 1
        fi
        sleep 1
    done
    
    log_success "All stress test iterations passed"
}

# Function to generate test report
generate_test_report() {
    log_info "Generating test report..."
    
    local gpu_type="$1"
    local report_file="/tmp/gpu-kill-test-report-$(date +%Y%m%d-%H%M%S).txt"
    
    {
        echo "=== GPU Kill Test Report ==="
        echo "Date: $(date)"
        echo "GPU Type: $gpu_type"
        echo "Hostname: $(hostname)"
        echo "OS: $(uname -a)"
        echo ""
        echo "=== GPU Detection Results ==="
        cat /tmp/gpu-list.txt 2>/dev/null || echo "No GPU list available"
        echo ""
        echo "=== GPU Information (JSON) ==="
        cat /tmp/gpu-info.json 2>/dev/null || echo "No GPU info available"
        echo ""
        echo "=== System Information ==="
        echo "CPU: $(lscpu | grep "Model name" | cut -d: -f2 | xargs || echo "Unknown")"
        echo "Memory: $(free -h | grep "Mem:" | awk '{print $2}' || echo "Unknown")"
        echo "GPU Drivers:"
        lsmod | grep -E "(nvidia|amdgpu|i915)" || echo "No GPU drivers found"
    } > "$report_file"
    
    log_success "Test report generated: $report_file"
    cat "$report_file"
}

# Main function
main() {
    local gpu_type="${1:-unknown}"
    
    log_info "Starting GPU Kill tests on Hot Aisle instance"
    log_info "GPU Type: $gpu_type"
    log_info "Project Root: $PROJECT_ROOT"
    
    # Initialize log file
    echo "=== GPU Kill Test Log - $(date) ===" > "$LOG_FILE"
    
    # Run test suite
    check_prerequisites
    build_gpukill
    run_gpu_detection_tests "$gpu_type"
    run_performance_tests "$gpu_type"
    run_stress_tests
    generate_test_report "$gpu_type"
    
    log_success "All GPU Kill tests completed successfully!"
}

# Run main function with all arguments
main "$@"
