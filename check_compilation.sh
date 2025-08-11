#!/bin/bash
# check_compilation.sh - ZisK Solana Integration Compilation Progress Tracker

echo "🔍 Checking ZisK Solana Integration compilation status..."
echo "=================================================="

# Clean build
echo "🧹 Cleaning previous build..."
cargo clean

# Check for errors
echo "📋 Running cargo check..."
cargo check 2>&1 | tee compilation_report.txt

# Count errors and warnings
error_count=$(grep -c "error\[" compilation_report.txt || echo "0")
warning_count=$(grep -c "warning:" compilation_report.txt || echo "0")

echo ""
echo "📊 Compilation Report:"
echo "   Errors: $error_count"
echo "   Warnings: $warning_count"
echo ""

if [ "$error_count" -eq "0" ]; then
    echo "✅ Compilation successful!"
    
    # Try to build
    echo "🔨 Running cargo build..."
    cargo build
    
    if [ $? -eq 0 ]; then
        echo "🎉 Build successful! Ready for testing."
    else
        echo "❌ Build failed. Check the output above."
    fi
else
    echo "❌ $error_count compilation errors remaining."
    echo ""
    echo "🔍 Top 10 errors:"
    grep -E "error\[" compilation_report.txt | head -10
    echo ""
    echo "📝 Check compilation_report.txt for full details."
    
    # Show error categories
    echo ""
    echo "📊 Error Categories:"
    echo "   Type mismatches: $(grep -c "mismatched types" compilation_report.txt || echo "0")"
    echo "   Unresolved imports: $(grep -c "unresolved import" compilation_report.txt || echo "0")"
    echo "   Missing types: $(grep -c "cannot find type" compilation_report.txt || echo "0")"
    echo "   Generic arguments: $(grep -c "takes .* generic arguments" compilation_report.txt || echo "0")"
    echo "   Method not found: $(grep -c "no method named" compilation_report.txt || echo "0")"
fi

echo ""
echo "📁 Full report saved to: compilation_report.txt"
