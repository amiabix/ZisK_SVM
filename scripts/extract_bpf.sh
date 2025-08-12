#!/bin/bash

set -e

echo "🚀 Starting End-to-End Test: Solana Program → BPF → ZisK → Proof"
echo "=================================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if Solana CLI is installed
if ! command -v solana &> /dev/null; then
    echo -e "${RED}❌ Solana CLI not found. Please install it first.${NC}"
    echo "Install: https://docs.solana.com/cli/install-solana-cli-tools"
    exit 1
fi

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Cargo not found. Please install Rust first.${NC}"
    echo "Install: https://rustup.rs/"
    exit 1
fi

echo -e "${BLUE}📦 Step 1: Compiling Solana Program...${NC}"

# Navigate to the program directory
cd programs/simple_calculator

# Clean previous builds
cargo clean

# Build the program
echo "Building with cargo build-bpf..."
cargo build-bpf

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Solana program compiled successfully${NC}"
else
    echo -e "${RED}❌ Failed to compile Solana program${NC}"
    exit 1
fi

# Find the compiled program
PROGRAM_PATH=$(find target/deploy -name "*.so" | head -n 1)

if [ -z "$PROGRAM_PATH" ]; then
    echo -e "${RED}❌ No compiled program found${NC}"
    exit 1
fi

echo -e "${GREEN}📁 Compiled program found: ${PROGRAM_PATH}${NC}"

# Go back to root
cd ../..

echo -e "${BLUE}🔍 Step 2: Extracting BPF Bytecode...${NC}"

# Create build directory if it doesn't exist
mkdir -p build

# Use objdump to extract the .text section (BPF bytecode)
echo "Extracting BPF bytecode using objdump..."
objdump -d -j .text "$PROGRAM_PATH" > build/bpf_disassembly.txt

# Extract just the hex bytes
echo "Extracting raw BPF bytes..."
objdump -d -j .text "$PROGRAM_PATH" | grep -E "^ [0-9a-f]+:" | cut -d: -f2 | tr -d ' ' | tr -d '\n' > build/bpf_raw_hex.txt

# Convert hex to binary
echo "Converting hex to binary..."
xxd -r -p build/bpf_raw_hex.txt > build/solana_program.bpf

# Get file size
BPF_SIZE=$(wc -c < build/solana_program.bpf)
echo -e "${GREEN}✅ BPF bytecode extracted: ${BPF_SIZE} bytes${NC}"

echo -e "${BLUE}📊 Step 3: Analyzing BPF Program...${NC}"

# Show first few bytes
echo "First 64 bytes of BPF program:"
hexdump -C build/solana_program.bpf | head -4

# Show disassembly
echo -e "\nBPF Disassembly (first 20 lines):"
head -20 build/bpf_disassembly.txt

echo -e "${BLUE}🔧 Step 4: Creating ZisK Input...${NC}"

# Create ZisK input format: [size:4 bytes][bpf_program]
python3 -c "
import struct

# Read the BPF program
with open('build/solana_program.bpf', 'rb') as f:
    bpf_data = f.read()

# Create ZisK input: [program_size:4 bytes][bpf_program]
program_size = len(bpf_data)
input_data = bytearray()

# First 4 bytes: program size (little-endian)
input_data.extend(struct.pack('<I', program_size))

# Rest: BPF program data
input_data.extend(bpf_data)

# Write to file
with open('build/solana_program_zisk_input.bin', 'wb') as f:
    f.write(input_data)

print(f'✅ Created ZisK input: {len(input_data)} bytes')
print(f'   Program size: {program_size} bytes')
print(f'   Total input: {len(input_data)} bytes')
"

echo -e "${GREEN}🎯 Ready for ZisK Execution!${NC}"
echo ""
echo "Next steps:"
echo "1. Test in ZisK emulator: ziskemu -e target/riscv64ima-zisk-zkvm-elf/release/solana_test -i build/solana_program_zisk_input.bin"
echo "2. Generate proof: cargo-zisk prove -e target/riscv64ima-zisk-zkvm-elf/release/solana_test -i build/solana_program_zisk_input.bin -o proof_solana_program -a -y"
echo ""
echo -e "${YELLOW}⚠️  Note: This will execute the actual Solana program BPF bytecode in ZisK!${NC}"
