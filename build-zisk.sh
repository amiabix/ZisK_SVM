#!/bin/bash

# ZisK Build and Execution Script for Solana Test
# This script builds the program for ZisK zkVM and executes it

set -e

echo "🚀 Building Solana Test for ZisK zkVM..."
echo "=========================================="

# Clean previous builds
echo "🧹 Cleaning previous builds..."
cargo clean

# Build with ZisK target
echo "🔨 Building with cargo for ZisK target..."
cargo build --release --target riscv64ima-zisk-zkvm-elf

# Check if build was successful
if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    echo ""
    echo "🔧 Setting up ROM for ZisK..."
    echo "=========================================="
    
    # Set up ROM for ZisK
    cargo-zisk rom-setup -e target/riscv64ima-zisk-zkvm-elf/release/solana_test -k $HOME/.zisk/provingKey
    
    if [ $? -eq 0 ]; then
        echo "✅ ROM setup successful!"
        echo ""
        echo "🎯 Executing with ziskemu..."
        echo "=========================================="
        
        # Execute with ziskemu
        ziskemu -e target/riscv64ima-zisk-zkvm-elf/release/solana_test -i build/input.bin
        
        echo ""
        echo "✅ ZisK execution completed!"
    else
        echo "❌ ROM setup failed!"
        exit 1
    fi
else
    echo "❌ Build failed!"
    exit 1
fi
