#!/bin/bash

# Debug script to identify CI issues
set -euo pipefail

echo "=== CI Debug Test ==="
echo "CI environment: ${CI:-not set}"
echo "Current directory: $(pwd)"
echo "Files in current directory:"
ls -la

echo ""
echo "=== Testing individual commands ==="

echo "1. Testing Cargo.toml existence:"
if [[ -f "Cargo.toml" ]]; then
    echo "✅ Cargo.toml exists"
else
    echo "❌ Cargo.toml missing"
    exit 1
fi

echo "2. Testing cargo availability:"
if command -v cargo &> /dev/null; then
    echo "✅ cargo available"
else
    echo "❌ cargo not available"
    exit 1
fi

echo "3. Testing git availability:"
if command -v git &> /dev/null; then
    echo "✅ git available"
else
    echo "❌ git not available"
    exit 1
fi

echo "4. Testing build without hotaisle:"
if cargo build --release; then
    echo "✅ build without hotaisle successful"
else
    echo "❌ build without hotaisle failed"
    exit 1
fi

echo "5. Testing build with hotaisle:"
if cargo build --release --features hotaisle; then
    echo "✅ build with hotaisle successful"
else
    echo "❌ build with hotaisle failed"
    exit 1
fi

echo "6. Testing cargo check:"
if cargo check --features hotaisle; then
    echo "✅ cargo check successful"
else
    echo "❌ cargo check failed"
    exit 1
fi

echo "7. Testing script syntax:"
if bash -n scripts/run-gpu-tests.sh; then
    echo "✅ script syntax valid"
else
    echo "❌ script syntax invalid"
    exit 1
fi

echo "8. Testing workflow file:"
if [[ -f ".github/workflows/hotaisle-gpu-testing.yml" ]]; then
    echo "✅ workflow file exists"
else
    echo "❌ workflow file missing"
    exit 1
fi

echo "9. Testing documentation:"
if [[ -f "docs/HOTAISLE_INTEGRATION.md" ]]; then
    echo "✅ documentation exists"
else
    echo "❌ documentation missing"
    exit 1
fi

echo "10. Testing Cargo.toml grep:"
if grep -q "hotaisle = \\[\\]" Cargo.toml; then
    echo "✅ hotaisle feature found in Cargo.toml"
else
    echo "❌ hotaisle feature not found in Cargo.toml"
    exit 1
fi

echo "11. Testing lib.rs grep:"
if grep -q "#\\[cfg(feature = \"hotaisle\")\\]" src/lib.rs; then
    echo "✅ conditional compilation found in lib.rs"
else
    echo "❌ conditional compilation not found in lib.rs"
    exit 1
fi

echo ""
echo "🎉 All tests passed!"
