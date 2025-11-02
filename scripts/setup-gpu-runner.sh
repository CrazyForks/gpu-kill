#!/bin/bash
# GPU Runner Setup Script
# This script helps set up a self-hosted GitHub Actions runner with GPU support

set -e

echo "🚀 GPU Kill - Self-Hosted Runner Setup"
echo "======================================"

# Check if running as root
if [[ $EUID -eq 0 ]]; then
   echo "❌ This script should not be run as root"
   exit 1
fi

# Detect OS
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    OS="linux"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    OS="macos"
else
    echo "❌ Unsupported OS: $OSTYPE"
    exit 1
fi

echo "📋 Detected OS: $OS"

# Function to install dependencies
install_deps() {
    echo "📦 Installing system dependencies..."
    
    if [[ "$OS" == "linux" ]]; then
        sudo apt-get update
        sudo apt-get install -y build-essential libssl-dev pkg-config curl tar
        
        # Install GPU-specific tools
        echo "🔧 Installing GPU tools..."
        
        # NVIDIA
        if command -v nvidia-smi &> /dev/null; then
            echo "✅ NVIDIA GPU detected"
            sudo apt-get install -y nvidia-utils-* || echo "⚠️  NVIDIA utils installation failed"
        else
            echo "ℹ️  No NVIDIA GPU detected"
        fi
        
        # AMD
        if command -v rocm-smi &> /dev/null; then
            echo "✅ AMD GPU with ROCm detected"
        else
            echo "ℹ️  Installing ROCm tools..."
            sudo apt-get install -y rocm-smi || echo "⚠️  ROCm installation failed"
        fi
        
        # Intel
        echo "ℹ️  Installing Intel GPU tools..."
        sudo apt-get install -y intel-gpu-tools || echo "⚠️  Intel GPU tools installation failed"
        
    elif [[ "$OS" == "macos" ]]; then
        # Check for Xcode command line tools
        if ! command -v xcode-select &> /dev/null; then
            echo "📱 Installing Xcode command line tools..."
            xcode-select --install || echo "⚠️  Xcode tools installation failed"
        else
            echo "✅ Xcode command line tools already installed"
        fi
    fi
}

# Function to install Rust
install_rust() {
    echo "🦀 Installing Rust..."
    
    if command -v rustc &> /dev/null; then
        echo "✅ Rust already installed: $(rustc --version)"
    else
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source ~/.cargo/env
        echo "✅ Rust installed: $(rustc --version)"
    fi
}

# Function to setup GitHub Actions runner
setup_runner() {
    echo "🏃 Setting up GitHub Actions runner..."
    
    # Get repository URL and token from user
    read -p "📝 Enter your GitHub repository URL (e.g., https://github.com/username/gpu-kill): " REPO_URL
    read -p "🔑 Enter your GitHub Personal Access Token (with repo and admin:org permissions): " GITHUB_TOKEN
    
    # Create runner directory
    RUNNER_DIR="$HOME/actions-runner"
    mkdir -p "$RUNNER_DIR"
    cd "$RUNNER_DIR"
    
    # Download runner
    if [[ "$OS" == "linux" ]]; then
        RUNNER_FILE="actions-runner-linux-x64-2.311.0.tar.gz"
    elif [[ "$OS" == "macos" ]]; then
        RUNNER_FILE="actions-runner-osx-x64-2.311.0.tar.gz"
    fi
    
    echo "📥 Downloading GitHub Actions runner..."
    curl -o "$RUNNER_FILE" -L "https://github.com/actions/runner/releases/download/v2.311.0/$RUNNER_FILE"
    tar xzf "$RUNNER_FILE"
    
    # Configure runner
    echo "⚙️  Configuring runner..."
    ./config.sh --url "$REPO_URL" --token "$GITHUB_TOKEN" --labels "gpu,$OS" --name "gpu-runner-$(hostname)"
    
    echo "✅ Runner configured successfully!"
    echo ""
    echo "🎯 To start the runner:"
    echo "   cd $RUNNER_DIR"
    echo "   ./run.sh"
    echo ""
    echo "🎯 To run as a service:"
    echo "   sudo ./svc.sh install"
    echo "   sudo ./svc.sh start"
}

# Function to test GPU detection
test_gpu() {
    echo "🧪 Testing GPU detection..."
    
    # Clone and build GPU Kill
    if [[ ! -d "gpu-kill" ]]; then
        git clone https://github.com/treadiehq/gpu-kill.git
    fi
    
    cd gpu-kill
    cargo build --release
    
    echo "🔍 GPU Detection Results:"
    ./target/release/gpukill --list || echo "No GPUs detected"
    
    echo "🧪 Running GPU hardware tests..."
    cargo test --test gpu_hardware_tests || echo "GPU tests completed (some may have been skipped)"
}

# Main execution
main() {
    echo "🎯 What would you like to do?"
    echo "1) Install dependencies only"
    echo "2) Setup GitHub Actions runner"
    echo "3) Test GPU detection"
    echo "4) Full setup (dependencies + runner + test)"
    echo "5) Exit"
    
    read -p "Choose an option (1-5): " choice
    
    case $choice in
        1)
            install_deps
            install_rust
            ;;
        2)
            install_deps
            install_rust
            setup_runner
            ;;
        3)
            install_deps
            install_rust
            test_gpu
            ;;
        4)
            install_deps
            install_rust
            setup_runner
            test_gpu
            ;;
        5)
            echo "👋 Goodbye!"
            exit 0
            ;;
        *)
            echo "❌ Invalid option"
            exit 1
            ;;
    esac
    
    echo "✅ Setup completed!"
}

# Run main function
main "$@"
