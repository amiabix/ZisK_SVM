# BPF to RISC-V Transpiler for ZisK

A transpiler that converts Berkeley Packet Filter (BPF) bytecode to RISC-V assembly for native execution in ZisK zkVM with cryptographic proof generation.

## Overview

This project transpiles BPF programs to RISC-V assembly, enabling native execution in ZisK rather than interpretation. This approach eliminates interpretation overhead and generates cryptographic proofs of the original BPF program execution.

**Architecture:**
```
BPF Bytecode → Parser → RISC-V Generator → ZisK Execution → Proof Generation
```

## Key Features

- **Native Execution**: Converts BPF bytecode to RISC-V assembly for direct execution
- **Full BPF Support**: Handles 50+ BPF opcodes including ALU, memory, branch, and system operations
- **ZisK Integration**: Native execution in ZisK zkVM with proof generation
- **Performance**: Eliminates interpretation overhead compared to BPF interpreters

## Performance Comparison

| Approach | Performance | Memory Usage | Proof Generation |
|----------|-------------|--------------|------------------|
| BPF Interpreter | Slow (interpretation overhead) | High (interpreter + program state) | Complex (interpreter state) |
| **BPF Transpiler** | **Fast (native RISC-V)** | **Low (just RISC-V code)** | **Simple (execution trace)** |

## Installation

```bash
git clone [repository-url]
cd bpf-riscv-transpiler
cargo build --release
```

**Prerequisites:**
- Rust 1.70+
- ZisK toolchain (for full execution)

## Usage

### Transpile BPF to RISC-V
```bash
cargo run -- transpile input.bpf output.riscv
```

### Execute BPF in ZisK
```bash
cargo run -- execute input.bpf
```

### Run Tests
```bash
cargo test
```

### Demo
```bash
cargo run -- demo
```

## Technical Details

### Supported BPF Opcodes
- **ALU Operations**: `ADD64_IMM`, `ADD64_REG`, `MUL64_REG`, `DIV64_REG`, `MOV64_IMM`
- **Memory Operations**: `LD_IMM64`, `LDX64`, `ST64`, `STX64`
- **Branch Operations**: `JA`, `JEQ_IMM`, `JGT_REG`, `CALL`, `EXIT`
- **System Calls**: Standard BPF system call interface

### RISC-V Target
- **Architecture**: `riscv64ima-zisk-zkvm-elf`
- **Register Mapping**: BPF registers (R0-R10) → RISC-V registers (x10-x20)
- **Memory**: 4-byte aligned instructions with ZisK memory model

### Implementation Components

1. **BPF Parser** (`src/bpf_parser.rs`): Parses raw BPF bytecode into structured instructions
2. **RISC-V Generator** (`src/riscv_generator.rs`): Converts BPF instructions to RISC-V assembly
3. **ZisK Integration** (`src/zisk_integration.rs`): Executes RISC-V code natively in ZisK environment

## Project Structure

```
src/
├── lib.rs              # Main library and transpiler logic
├── main.rs             # Binary entry point and CLI
├── error.rs            # Error types and handling
├── types.rs            # Core data structures
├── bpf_parser.rs       # BPF bytecode parser
├── riscv_generator.rs  # RISC-V code generator
└── zisk_integration.rs # ZisK execution integration
```

## Example

```rust
use bpf_riscv_transpiler::Transpiler;

let transpiler = Transpiler::new();
let bpf_bytecode = std::fs::read("program.bpf")?;
let riscv_code = transpiler.transpile(&bpf_bytecode)?;
let result = transpiler.execute_in_zisk(&riscv_code)?;
```

## Development Status

- ✅ **Core Transpiler**: BPF parser, RISC-V generator, basic ZisK integration
- 🚧 **Advanced Features**: Branch optimization, memory access optimization
- 📋 **Production**: Full compatibility testing, performance tuning

## Testing

```bash
# Unit tests
cargo test

# Integration tests
cargo test --features test-utils

# Benchmarks
cargo bench --features benchmarks
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Submit a pull request

## License

MIT License - see LICENSE file for details.
