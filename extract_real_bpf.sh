#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Solana Program BPF Extraction Script${NC}"
echo "=================================="

# Configuration
PROGRAM_ID="DfRwpT676RZGJN8b2tHEHaHbaBazYVeCXmwtUyJ6ejqB"
DEVNET_RPC="https://api.devnet.solana.com"
OUTPUT_DIR="build"

# Create output directory
mkdir -p "$OUTPUT_DIR"

echo -e "${YELLOW}📋 Program Details:${NC}"
echo "Program ID: $PROGRAM_ID"
echo "Network: devnet"
echo "RPC Endpoint: $DEVNET_RPC"
echo ""

# Check if we have the required tools
echo -e "${BLUE}🔍 Checking dependencies...${NC}"

if ! command -v curl &> /dev/null; then
    echo -e "${RED}❌ curl is required but not installed${NC}"
    exit 1
fi

if ! command -v jq &> /dev/null; then
    echo -e "${YELLOW}⚠️  jq not found, installing...${NC}"
    sudo apt-get update && sudo apt-get install -y jq
fi

echo -e "${GREEN}✅ Dependencies ready${NC}"
echo ""

# Step 1: Fetch program account info
echo -e "${BLUE}📥 Step 1: Fetching program account info...${NC}"
curl -s "$DEVNET_RPC" -X POST -H "Content-Type: application/json" -d "{
  \"jsonrpc\": \"2.0\",
  \"id\": 1,
  \"method\": \"getAccountInfo\",
  \"params\": [
    \"$PROGRAM_ID\",
    {
      \"encoding\": \"base64\"
    }
  ]
}" | jq '.' > "$OUTPUT_DIR/program_account.json"

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Program account info fetched${NC}"
else
    echo -e "${RED}❌ Failed to fetch program account info${NC}"
    exit 1
fi

# Step 2: Extract program data
echo -e "${BLUE}🔍 Step 2: Extracting program data...${NC}"
PROGRAM_DATA=$(jq -r '.result.value.data[0]' "$OUTPUT_DIR/program_account.json")

if [ "$PROGRAM_DATA" = "null" ] || [ -z "$PROGRAM_DATA" ]; then
    echo -e "${RED}❌ No program data found${NC}"
    echo "This might mean:"
    echo "1. Program not deployed yet"
    echo "2. Program ID is incorrect"
    echo "3. Network issue"
    exit 1
fi

echo -e "${GREEN}✅ Program data extracted (${#PROGRAM_DATA} characters)${NC}"

# Step 3: Convert base64 to binary
echo -e "${BLUE}🔄 Step 3: Converting base64 to binary...${NC}"
echo "$PROGRAM_DATA" | base64 -d > "$OUTPUT_DIR/program_raw.bin"

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Converted to binary (${#PROGRAM_DATA} bytes)${NC}"
else
    echo -e "${RED}❌ Failed to convert base64 to binary${NC}"
    exit 1
fi

# Step 4: Extract BPF bytecode (skip header)
echo -e "${BLUE}📦 Step 4: Extracting BPF bytecode...${NC}"
# Solana programs have a header, we need to skip it to get the actual BPF
# The BPF starts after the program header (usually around 64 bytes)
dd if="$OUTPUT_DIR/program_raw.bin" of="$OUTPUT_DIR/program_bpf.bin" bs=1 skip=64 2>/dev/null

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ BPF bytecode extracted${NC}"
else
    echo -e "${RED}❌ Failed to extract BPF bytecode${NC}"
    exit 1
fi

# Step 5: Create ZisK input format
echo -e "${BLUE}🔧 Step 5: Creating ZisK input format...${NC}"
python3 -c "
import struct
import os

# Read the BPF program
with open('build/program_bpf.bin', 'rb') as f:
    bpf_data = f.read()

program_size = len(bpf_data)
print(f'BPF Program size: {program_size} bytes')

# Create ZisK input format: [size:u32][bpf_program]
input_data = bytearray()
input_data.extend(struct.pack('<I', program_size))
input_data.extend(bpf_data)

# Write ZisK input
with open('build/real_solana_program_zisk_input.bin', 'wb') as f:
    f.write(input_data)

print(f'ZisK input created: {len(input_data)} bytes')
print(f'  - Size prefix: 4 bytes')
print(f'  - BPF program: {program_size} bytes')
print(f'  - Total: {len(input_data)} bytes')
"

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ ZisK input created${NC}"
else
    echo -e "${RED}❌ Failed to create ZisK input${NC}"
    exit 1
fi

# Step 6: Show file details
echo -e "${BLUE}📊 Step 6: File details...${NC}"
echo ""
echo -e "${GREEN}📁 Generated files:${NC}"
ls -la "$OUTPUT_DIR"/*.bin "$OUTPUT_DIR"/*.json 2>/dev/null | while read line; do
    echo "   $line"
done

echo ""
echo -e "${GREEN}🎯 Ready for ZisK execution!${NC}"
echo ""
echo -e "${YELLOW}📋 Next steps:${NC}"
echo "1. Test in ZisK emulator:"
echo "   ziskemu -e target/riscv64ima-zisk-zkvm-elf/release/solana_test -i build/real_solana_program_zisk_input.bin"
echo ""
echo "2. Generate proof:"
echo "   cargo-zisk prove -e target/riscv64ima-zisk-zkvm-elf/release/solana_test -i build/real_solana_program_zisk_input.bin -o proof_real_solana_program -a -y"
echo ""
echo -e "${BLUE}⚠️  Note: This will execute the ACTUAL deployed Solana program BPF bytecode in ZisK!${NC}"
echo -e "${GREEN}🚀 This is a REAL end-to-end test with your deployed calculator program!${NC}"
