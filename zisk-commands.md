# ZisK Build and Execution Commands

## Quick Build and Run

Use the automated script:
```bash
./build-zisk.sh
```

## Manual Commands

### 1. Build for ZisK zkVM
```bash
cargo build --release --target riscv64ima-zisk-zkvm-elf
```

### 2. Set up ROM for ZisK
```bash
cargo-zisk rom-setup -e target/riscv64ima-zisk-zkvm-elf/release/solana_test -k $HOME/.zisk/provingKey
```

### 3. Execute with ziskemu
```bash
ziskemu -e target/riscv64ima-zisk-zkvm-elf/release/solana_test -i build/input.bin
```

## Alternative Build Methods

### Using cargo with ZisK target
```bash
cargo build --release --target riscv64ima-zisk-zkvm-elf
```

### Using cargo-zisk for other operations
```bash
# Check ZisK setup
cargo-zisk check-setup

# Set up ROM for proof generation
cargo-zisk rom-setup -e <elf_file> -k <proving_key_path>

# Execute ZisK program
cargo-zisk execute

# Generate proofs
cargo-zisk prove
```

## Input File

The build script automatically creates `build/input.bin` with test data:
- Test transaction signature
- Test account key
- Test instruction data
- Test BPF program (LdReg + Exit)

## Output

- **Binary**: `target/riscv64ima-zisk-zkvm-elf/release/solana_test`
- **Input**: `build/input.bin`
- **Memory Layout**: `zisk-memory.x`
- **ROM Setup**: Stored in `$HOME/.zisk/cache/` and `$HOME/.zisk/provingKey/`

## Verified Working Commands

✅ **Build Command**:
```bash
cargo build --release --target riscv64ima-zisk-zkvm-elf
```

✅ **ROM Setup Command**:
```bash
cargo-zisk rom-setup -e target/riscv64ima-zisk-zkvm-elf/release/solana_test -k $HOME/.zisk/provingKey
```

✅ **Execution Command**:
```bash
ziskemu -e target/riscv64ima-zisk-zkvm-elf/release/solana_test -i build/input.bin
```

## Complete Workflow

1. **Build**: `cargo build --release --target riscv64ima-zisk-zkvm-elf`
2. **ROM Setup**: `cargo-zisk rom-setup -e target/riscv64ima-zisk-zkvm-elf/release/solana_test -k $HOME/.zisk/provingKey`
3. **Execute**: `ziskemu -e target/riscv64ima-zisk-zkvm-elf/release/solana_test -i build/input.bin`

## Troubleshooting

If you get build errors:
1. Ensure ZisK toolchain is installed: `which cargo-zisk`
2. Check target is available: `rustup target list | grep zisk`
3. Verify memory layout file exists: `ls zisk-memory.x`
4. Clean and rebuild: `cargo clean && cargo build --release --target riscv64ima-zisk-zkvm-elf`

If ROM setup fails:
1. Ensure proving key directory exists: `ls $HOME/.zisk/provingKey/`
2. Check ELF file exists: `ls target/riscv64ima-zisk-zkvm-elf/release/solana_test`
3. Verify ZisK cache directory permissions: `ls -la $HOME/.zisk/cache/`

## Notes

- `cargo-zisk build` doesn't support `--target` flag
- Use regular `cargo build` with the ZisK target
- ROM setup is required before proof generation
- The program executes successfully with `ziskemu`
- Input file is automatically generated during build
- ROM setup creates assembly files and binary traces in cache
