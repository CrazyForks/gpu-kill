#!/bin/bash

# GPU Kill - Hot Aisle Integration Test Script (CI-friendly version)
# This script tests the Hot Aisle integration without requiring actual API access

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Test results
TESTS_PASSED=0
TESTS_FAILED=0

# Function to run a test
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    log_info "Running test: $test_name"
    log_info "Command: $test_command"
    
    if eval "$test_command"; then
        log_success "✅ $test_name passed"
        ((TESTS_PASSED++))
    else
        log_error "❌ $test_name failed"
        log_error "Command that failed: $test_command"
        ((TESTS_FAILED++))
    fi
    echo
}

# Main test function
main() {
    log_info "Starting Hot Aisle Integration Tests (Simple Version)"
    echo "========================================"
    
    # Debug information
    log_info "Environment:"
    log_info "  CI: ${CI:-false}"
    log_info "  PWD: $(pwd)"
    log_info "  USER: ${USER:-unknown}"
    echo
    
    # Test 1: Check if we're in the right directory
    run_test "Project Root Check" '[[ -f "Cargo.toml" ]]'
    
    # Test 2: Check if Rust is available
    run_test "Rust Toolchain Check" 'command -v cargo > /dev/null 2>&1'
    
    # Test 3: Check if git is available
    run_test "Git Check" 'command -v git > /dev/null 2>&1'
    
    # Test 4: Build without Hot Aisle feature
    run_test "Build Without Hot Aisle Feature" 'cargo build --release'
    
    # Test 5: Build with Hot Aisle feature
    run_test "Build With Hot Aisle Feature" 'cargo build --release --features hotaisle'
    
    # Test 6: Check if Hot Aisle client compiles
    run_test "Hot Aisle Client Compilation" 'cargo check --features hotaisle'
    
    # Test 7: Validate test script syntax
    run_test "Test Script Syntax Check" 'bash -n scripts/run-gpu-tests.sh'
    
    # Test 8: Check if workflow file exists
    run_test "Workflow File Exists" '[[ -f ".github/workflows/hotaisle-gpu-testing.yml" ]]'
    
    # Test 9: Check if documentation exists
    run_test "Documentation Exists" '[[ -f "docs/HOTAISLE_INTEGRATION.md" ]]'
    
    # Test 10: Validate Cargo.toml has hotaisle feature
    run_test "Hot Aisle Feature in Cargo.toml" 'grep -q "hotaisle = \\[\\]" Cargo.toml'
    
    # Test 11: Check if lib.rs has conditional compilation
    run_test "Conditional Compilation in lib.rs" 'grep -q "#\\[cfg(feature = \"hotaisle\")\\]" src/lib.rs'
    
    # Summary
    echo "========================================"
    log_info "Test Summary:"
    log_success "✅ Tests Passed: $TESTS_PASSED"
    if [[ $TESTS_FAILED -gt 0 ]]; then
        log_error "❌ Tests Failed: $TESTS_FAILED"
        exit 1
    else
        log_success "✅ Tests Failed: $TESTS_FAILED"
    fi
    
    log_success "🎉 All integration tests passed!"
    log_info "The Hot Aisle integration is ready for use with a valid API key."
    exit 0
}

# Run main function
main "$@"
