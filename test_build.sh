#!/bin/bash

# Solana Transaction Validator Build Test Script
# This script demonstrates the different build modes and generated outputs

echo "=== Solana Transaction Validator Build Test ==="
echo

# Clean previous build
echo "1. Cleaning previous build..."
rm -rf build/
echo "   ✓ Build directory cleaned"
echo

# Test 1: Build with fallback test data
echo "2. Building with fallback test data..."
touch build.rs  # Force build script to run
cargo build
echo "   ✓ Build completed with test data"
echo

# Show generated files
echo "3. Generated files:"
ls -la build/
echo

# Show test data slot
echo "4. Test data slot number:"
grep '"slot"' build/proof_request.json | head -1
echo

# Test 2: Build with live Solana data
echo "5. Building with live Solana data..."
touch build.rs  # Force build script to run
USE_REAL_SOLANA_DATA=true cargo build
echo "   ✓ Build completed with live data"
echo

# Show live data slot
echo "6. Live data slot number:"
grep '"slot"' build/proof_request.json | head -1
echo

# Show file sizes
echo "7. File sizes:"
ls -lh build/
echo

# Test 3: Validate the generated data
echo "8. Validating generated data structure..."
if [ -f "build/input.bin" ] && [ -f "build/proof_request.json" ]; then
    echo "   ✓ Input files generated successfully"
    echo "   ✓ Binary input size: $(wc -c < build/input.bin) bytes"
    echo "   ✓ JSON input size: $(wc -c < build/proof_request.json) bytes"
else
    echo "   ✗ Input files missing"
fi
echo

echo "=== Build Test Completed ==="
echo
echo "To run the ZK program:"
echo "  cargo run"
echo
echo "To rebuild with live data:"
echo "  USE_REAL_SOLANA_DATA=true cargo build"
echo
echo "To rebuild with test data:"
echo "  cargo build"
