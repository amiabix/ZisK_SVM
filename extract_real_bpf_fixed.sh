#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Solana Program BPF Extraction Script (FIXED)${NC}"
echo "================================================"

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

# Step 1: Find the program data account
echo -e "${BLUE}📥 Step 1: Finding program data account...${NC}"
echo "Note: Solana programs store their BPF bytecode in a separate data account"
echo ""

# The program data account is derived from the program ID
# We need to find it using getProgramAccounts or derive it
echo -e "${YELLOW}🔍 Searching for program data accounts...${NC}"
curl -s "$DEVNET_RPC" -X POST -H "Content-Type: application/json" -d "{
  \"jsonrpc\": \"2.0\",
  \"id\": 1,
  \"method\": \"getProgramAccounts\",
  \"params\": [
    \"$PROGRAM_ID\",
    {
      \"encoding\": \"base64\",
      \"filters\": [
        {
          \"dataSize\": 0
        }
      ]
    }
  ]
}" | jq '.' > "$OUTPUT_DIR/program_accounts.json"

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Program accounts fetched${NC}"
else
    echo -e "${RED}❌ Failed to fetch program accounts${NC}"
    exit 1
fi

# Step 2: Try to get the program data directly
echo -e "${BLUE}📥 Step 2: Trying direct program data fetch...${NC}"
curl -s "$DEVNET_RPC" -X POST -H "Content-Type: application/json" -d "{
  \"jsonrpc\": \"2.0\",
  \"id\": 1,
  \"method\": \"getAccountInfo\",
  \"params\": [
    \"$PROGRAM_ID\",
    {
      \"encoding\": \"base64\",
      \"commitment\": \"confirmed\"
    }
  ]
}" | jq '.' > "$OUTPUT_DIR/program_direct.json"

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Direct program fetch completed${NC}"
else
    echo -e "${RED}❌ Failed direct program fetch${NC}"
    exit 1
fi

# Step 3: Check what we got
echo -e "${BLUE}🔍 Step 3: Analyzing what we received...${NC}"
echo ""

echo -e "${YELLOW}📋 Program Direct Info:${NC}"
cat "$OUTPUT_DIR/program_direct.json" | jq '.result.value | {executable, owner, space, lamports}'

echo ""
echo -e "${YELLOW}📋 Program Accounts:${NC}"
cat "$OUTPUT_DIR/program_accounts.json" | jq '.result | length'

# Step 4: Try a different approach - get recent transactions
echo -e "${BLUE}📥 Step 4: Getting recent program transactions...${NC}"
curl -s "$DEVNET_RPC" -X POST -H "Content-Type: application/json" -d "{
  \"jsonrpc\": \"2.0\",
  \"id\": 1,
  \"method\": \"getSignaturesForAddress\",
  \"params\": [
    \"$PROGRAM_ID\",
    {
      \"limit\": 5
    }
  ]
}" | jq '.' > "$OUTPUT_DIR/recent_transactions.json"

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Recent transactions fetched${NC}"
else
    echo -e "${RED}❌ Failed to fetch recent transactions${NC}"
fi

# Step 5: Show all the data we collected
echo -e "${BLUE}📊 Step 5: Summary of collected data...${NC}"
echo ""
echo -e "${GREEN}📁 Generated files:${NC}"
ls -la "$OUTPUT_DIR"/*.json 2>/dev/null | while read line; do
    echo "   $line"
done

echo ""
echo -e "${YELLOW}🔍 Analysis:${NC}"
echo "The issue is that we're trying to fetch the program ID directly,"
echo "but Solana programs store their BPF bytecode in program data accounts."
echo ""
echo "We need to either:"
echo "1. Find the correct program data account"
echo "2. Use a different method to extract the BPF"
echo "3. Check if the program was actually deployed correctly"
echo ""
echo -e "${BLUE}📋 Next steps:${NC}"
echo "1. Check the recent transactions to see if deployment was successful"
echo "2. Look for the actual program data account"
echo "3. Verify the program deployment status"
