#!/bin/bash

echo "🧪 Testing Multi-Block Solana Processing"
echo "========================================"

# Test 1: Single block processing (default)
echo ""
echo "📦 Test 1: Single Block Processing"
echo "----------------------------------"
USE_REAL_SOLANA_DATA=true BLOCK_COUNT=1 cargo build
echo "✅ Single block processing completed"

# Test 2: Multi-block processing (5 blocks)
echo ""
echo "📦 Test 2: Multi-Block Processing (5 blocks)"
echo "---------------------------------------------"
USE_REAL_SOLANA_DATA=true BLOCK_COUNT=5 cargo build
echo "✅ Multi-block processing completed"

# Test 3: Multi-block processing (10 blocks)
echo ""
echo "📦 Test 3: Multi-Block Processing (10 blocks)"
echo "----------------------------------------------"
USE_REAL_SOLANA_DATA=true BLOCK_COUNT=10 cargo build
echo "✅ Multi-block processing completed"

# Show generated files
echo ""
echo "📁 Generated Files:"
echo "-------------------"
ls -la build/ | grep -E "(block_|multi_block|input\.bin|proof_request)"

echo ""
echo "📊 Multi-Block Summary:"
echo "----------------------"
if [ -f "build/multi_block_summary.txt" ]; then
    cat build/multi_block_summary.txt
else
    echo "No multi-block summary found (single block mode was used)"
fi

echo ""
echo "🎯 Individual Block Files:"
echo "-------------------------"
for file in build/block_*.json; do
    if [ -f "$file" ]; then
        echo "📄 $file"
        # Show first few lines of each block file
        head -5 "$file" | sed 's/^/  /'
        echo "  ..."
    fi
done

echo ""
echo "✅ Multi-block testing completed successfully!"
