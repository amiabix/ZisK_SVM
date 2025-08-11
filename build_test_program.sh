#!/bin/bash
set -e

echo "🔨 Building test BPF program..."

# Create programs directory
mkdir -p programs/hello_world/src

# Build the BPF program
cd programs
if [ ! -f hello_world/Cargo.toml ]; then
    echo "Creating Cargo.toml..."
    # Create the files as shown above
fi

echo "📦 Installing Solana CLI tools..."
if ! command -v solana &> /dev/null; then
    curl -sSfL https://release.solana.com/v1.18.17/install | sh
    export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
fi

echo "🔧 Building BPF program..."
cd hello_world
cargo build-bpf

echo "📁 Copying to target directory..."
cp target/deploy/hello_world.so ../../target/

echo "✅ Test BPF program built successfully!"
