#!/bin/bash
# ZisK Build Script for ZisK-SVM
# Following patterns from: https://0xpolygonhermez.github.io/zisk/getting_started/writing_programs.html

set -e

echo "🚀 Building ZisK-SVM for ZisK zkVM..."

# Check if cargo-zisk is installed
if ! command -v cargo-zisk &> /dev/null; then
    echo "❌ cargo-zisk not found. Please install it first:"
    echo "   cargo install cargo-zisk"
    exit 1
fi

# Clean previous builds
echo "🧹 Cleaning previous builds..."
cargo clean

# Build for ZisK (RISC-V target)
echo "🔨 Building for ZisK RISC-V target..."
cargo-zisk build --release

# Check if build was successful
if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    
    # Show the generated ELF file
    ELF_PATH="./target/riscv64ima-zisk-zkvm-elf/release/solana_test"
    if [ -f "$ELF_PATH" ]; then
        echo "📁 Generated ELF file: $ELF_PATH"
        echo "📊 File size: $(ls -lh "$ELF_PATH" | awk '{print $5}')"
    else
        echo "❌ ELF file not found at expected location"
        exit 1
    fi
else
    echo "❌ Build failed!"
    exit 1
fi

# Generate program setup files (first time only)
echo "⚙️  Generating program setup files..."
cargo-zisk rom-setup -e "$ELF_PATH" -k "$HOME/.zisk/provingKey"

# Create input file for testing
echo "📝 Creating test input file..."
mkdir -p build
cat > build/input.bin << 'EOF'
# This is a test input file for ZisK-SVM
# Format: [4 bytes: program size] + [program data]
# For testing, we'll use a minimal BPF program
EOF

# Add a minimal test program (4 bytes for size + program data)
echo -n -e "\x04\x00\x00\x00" > build/input.bin  # 4 bytes size
echo -n -e "\x95\x00\x00\x00" >> build/input.bin  # BPF EXIT instruction

echo "✅ ZisK build completed successfully!"
echo ""
echo "📋 Next steps:"
echo "   1. Test with ZisK emulator:"
echo "      ziskemu -e $ELF_PATH -i build/input.bin"
echo ""
echo "   2. Generate proof:"
echo "      cargo-zisk prove -e $ELF_PATH -i build/input.bin -o proof -a -y"
echo ""
echo "   3. Verify proof:"
echo "      cargo-zisk verify -p ./proof/vadcop_final_proof.bin"
