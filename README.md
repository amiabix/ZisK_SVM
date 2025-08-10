# Solana Transaction Validator for ZisK zkVM

This project demonstrates how to validate Solana transaction simulation results using zero-knowledge proofs with the ZisK zkVM. The build system can fetch live Solana network data or generate realistic test data for development and testing.

## Features

- **Live Solana Data Fetching**: Fetches real-time slot information from Solana mainnet
- **Multi-Block Processing**: Process multiple Solana blocks simultaneously for comprehensive validation
- **Comprehensive Transaction Validation**: Validates compute units, fees, account changes, and state consistency
- **Fallback Test Data**: Generates realistic test data when live data is unavailable
- **ZisK Integration**: Uses the actual ZisK API (`ziskos::read_input()` and `ziskos::set_output()`)
- **Production-Ready**: Clean, formal code structure suitable for production environments

## Project Structure

```
solana_Test/
├── build.rs              # Enhanced build script with live data fetching
├── Cargo.toml            # Project dependencies and configuration
├── src/
│   └── main.rs          # Main ZK program for transaction validation
├── build/                # Generated output directory
│   ├── input.bin         # Binary input data for ZisK (single block)
│   ├── proof_request.json # Human-readable JSON data (single block)
│   ├── block_001_XXXXX.json # Individual block data files
│   ├── block_001_XXXXX.bin  # Individual block binary files
│   ├── multi_block_summary.json # Summary of all blocks
│   ├── multi_block_input.bin    # Binary input with all blocks
│   ├── multi_block_summary.txt  # Human-readable summary
│   ├── zk_program.rs     # Generated ZK program source
│   └── Cargo.toml        # ZK program configuration
├── test_multi_block.sh   # Test script for multi-block functionality
└── README.md             # This file
```

## Prerequisites

- Rust toolchain (latest stable version)
- `curl` command-line tool (for live data fetching)
- Internet connection (for live Solana data)

## Usage

### 1. Single Block Processing (Default)

```bash
# Build with fallback test data
cargo build

# Build with live Solana data (single block)
USE_REAL_SOLANA_DATA=true cargo build
```

### 2. Multi-Block Processing

```bash
# Process 5 Solana blocks
USE_REAL_SOLANA_DATA=true BLOCK_COUNT=5 cargo build

# Process 10 Solana blocks
USE_REAL_SOLANA_DATA=true BLOCK_COUNT=10 cargo build

# Process 20 Solana blocks
USE_REAL_SOLANA_DATA=true BLOCK_COUNT=20 cargo build
```

### 3. Run the ZK Program

```bash
cargo run
```

The program will read the input data and perform transaction validation, outputting results through the ZisK API.

### 4. Test Multi-Block Functionality

```bash
# Run comprehensive multi-block tests
./test_multi_block.sh
```

## Multi-Block Processing

The enhanced build system now supports processing multiple Solana blocks simultaneously:

### Environment Variables

- `USE_REAL_SOLANA_DATA`: Set to "true" to fetch live Solana data
- `BLOCK_COUNT`: Number of blocks to process (default: 1)

### Output Files

When processing multiple blocks, the system generates:

1. **Individual Block Files**: 
   - `block_001_XXXXX.json` - JSON data for each block
   - `block_001_XXXXX.bin` - Binary data for ZisK processing

2. **Summary Files**:
   - `multi_block_summary.json` - JSON summary of all blocks
   - `multi_block_input.bin` - Binary input containing all blocks
   - `multi_block_summary.txt` - Human-readable summary

### Block Data Structure

Each block contains:
- **Slot**: Solana slot number
- **Blockhash**: Unique block identifier
- **Block Time**: Unix timestamp
- **Transaction Count**: Number of transactions in the block
- **Total Fees**: Sum of all transaction fees
- **Total Compute Units**: Sum of all compute units used
- **Success/Failure Counts**: Transaction success statistics
- **Proof Request**: Complete transaction validation data

## Data Structures

### MultiBlockProofRequest
The main container for multi-block data:
- **blocks**: Vector of individual block data
- **proof_id**: Unique identifier for the proof
- **total_blocks**: Number of blocks processed
- **start_slot**: First block slot number
- **end_slot**: Last block slot number

### ProofRequest (Single Block)
The main data structure containing:
- **TransactionIntent**: Transaction details, fees, and account requirements
- **SimulationResult**: Execution results, account changes, and state snapshots
- **ProofID**: Unique identifier for the proof

### Validation Rules
The system validates:
1. Compute units within reasonable bounds
2. Fee calculations and limits
3. Account changes consistency
4. Lamports conservation
5. Success/error consistency
6. Merkle proof structure
7. State consistency across execution
8. Slot consistency validation

## Output Format

The ZK program outputs validation results as 16 u32 values:
- **Output 0**: Overall validation success (0 or 1)
- **Output 1-2**: Compute units used (split u64)
- **Output 3-4**: Fee paid (split u64)
- **Output 5**: Account changes count
- **Output 6**: Program invocations count
- **Output 7**: Error code (0 if no errors)
- **Output 8-15**: Merkle root (32 bytes as 8 u32s)

## Error Codes

| Code | Description |
|------|-------------|
| 0    | No errors (validation passed) |
| 1    | No compute units used |
| 2    | Exceeded compute budget |
| 3    | Exceeded Solana maximum compute units |
| 4    | Fee too low |
| 5    | Fee exceeds maximum |
| 6    | Too many account changes |
| 7    | Lamports conservation violated |
| 8    | Success but has error |
| 9    | Failed but no error |
| 10   | Invalid merkle root length |
| 11   | Slot mismatch between states |
| 12   | Intent slot mismatch |

## Configuration

### Environment Variables

- `USE_REAL_SOLANA_DATA`: Set to "true" to fetch live Solana data
- `BLOCK_COUNT`: Number of blocks to process (1-100 recommended)

### Build Script Features

- **Automatic Directory Creation**: Creates `build/` directory if it doesn't exist
- **Error Handling**: Graceful fallback to test data if live fetch fails
- **Multi-Format Output**: Generates JSON, binary, and text files
- **ZK Program Generation**: Automatically creates ZisK-compatible program files
- **Block Consistency**: Ensures slot and blockhash consistency across blocks

## Development

### Adding New Validation Rules

To add new validation rules, modify the `validate_solana_transaction` function in `src/main.rs`:

```rust
// Add new validation logic
if some_condition {
    return ValidationResult { valid: false, error_code: NEW_ERROR_CODE };
}
```

### Extending Data Structures

To add new fields to the data structures:

1. Update the struct definitions in both `build.rs` and `src/main.rs`
2. Modify the data generation functions in `build.rs`
3. Update the validation logic in `src/main.rs` if needed

### Processing Multiple Blocks

The system automatically handles multiple blocks by:
1. Fetching current Solana slot
2. Calculating slot range based on `BLOCK_COUNT`
3. Processing each block sequentially with API rate limiting
4. Generating individual and summary files
5. Providing comprehensive validation data

## Troubleshooting

### Build Failures

- Ensure all dependencies are properly installed
- Check that `curl` is available for live data fetching
- Verify internet connectivity for Solana API calls
- Check `BLOCK_COUNT` is reasonable (1-100 recommended)

### Runtime Errors

- Check that input files are properly generated in the `build/` directory
- Verify the input data format matches the expected structures
- Review error codes in the output for specific validation failures
- For multi-block processing, ensure all block files are present

### Performance Considerations

- **API Rate Limiting**: Built-in delays between API calls
- **Memory Usage**: Large block counts may increase memory usage
- **Processing Time**: More blocks = longer build time
- **Network Latency**: Consider network conditions when fetching live data

## Contributing

This project follows production-ready coding standards:
- Clean, readable code structure
- Comprehensive error handling
- Clear documentation
- Consistent formatting
- Proper dependency management
- Multi-block processing capabilities

## License

This project is provided as-is for educational and development purposes.

## Future Enhancements

### Planned Features
1. **Parallel Block Fetching**: Concurrent API calls for faster processing
2. **Block Range Selection**: Specify custom slot ranges
3. **Incremental Updates**: Process only new blocks since last run
4. **Block Validation**: Verify block finality and confirmation status
5. **Performance Metrics**: Processing time and throughput statistics

### Integration Opportunities
1. **Solana Programs**: Direct integration with Solana program execution
2. **External APIs**: Integration with additional blockchain data sources
3. **ZK Proof Systems**: Enhanced zero-knowledge proof generation
4. **Monitoring Tools**: Integration with observability platforms
5. **Batch Processing**: Process blocks in configurable batches

## ZisK zkVM Integration

This project now includes comprehensive ZisK zkVM integration for zero-knowledge proof generation and verification of Solana transaction validation.

### ZisK Features

#### **1. Build System Adaptations**
- **RISC-V Target Compilation**: Automatic compilation for `riscv64ima-zisk-zkvm-elf` target
- **Memory Layout Management**: Custom memory layout file (`zisk-memory.x`) for ZisK constraints
- **Build Flags Injection**: Automatic injection of ZisK-specific linker flags
- **Target Management**: Automatic installation and management of ZisK target

#### **2. BPF Interpreter Optimizations**
- **Static Opcode Table**: Replaced dynamic dispatch with static opcode table for ZisK efficiency
- **Cycle Accounting**: Precise cycle counting for each BPF instruction
- **Constraint Verification**: ZisK-specific constraint verification using `zk_assert!`
- **Memory Optimization**: Optimized memory layout for ZisK's constraints

#### **3. Solana Executor Integration**
- **State Serialization**: Efficient serialization of Solana account states for ZisK
- **Proof Generation Triggers**: Automatic triggering of ZisK proof generation
- **Account Model Preservation**: Maintains Solana account ownership and validation rules
- **Transaction Flow Integration**: Seamless integration with existing transaction processing

### ZisK Usage

#### **Building with ZisK Features**

```bash
# Standard build (without ZisK)
cargo build --release

# Build with ZisK features enabled
cargo build --release --features zk

# Build for ZisK target specifically
cargo build --release --target riscv64ima-zisk-zkvm-elf --features zk
```

#### **Running ZisK-Enabled Programs**

```bash
# Run with ZisK features
cargo run --release --features zk -- --slot 12345

# Run with custom slot
cargo run --release --features zk -- --slot 54321
```

#### **Proof Generation and Verification**

```bash
# Generate ZisK proof
cargo zisk prove \
    -e target/release/solana_test \
    -i zk_input.bin \
    -o zk_proof.bin

# Verify ZisK proof
cargo zisk verify --proof zk_proof.bin
```

### ZisK Demo Mode

The enhanced demo script now supports ZisK mode:

```bash
# Standard demo
./demo.sh

# ZisK-enabled demo
./demo.sh --zk

# ZisK demo with custom slot
./demo.sh --zk --slot 12345

# Show help
./demo.sh --help
```

### ZisK Testing

The test script includes comprehensive ZisK testing:

```bash
# Run all tests including ZisK
./test_bpf_interpreter.sh

# The script automatically:
# 1. Builds with ZisK features
# 2. Generates ZisK input data
# 3. Attempts proof generation
# 4. Verifies proofs
# 5. Reports results
```

### ZisK Architecture

#### **Memory Layout**
```
ZisK Memory Layout (64KB total)
├── Text Section: Program code
├── RO Data: Read-only constants
├── Data Section: Initialized variables
├── BSS Section: Uninitialized variables
├── Stack: 8KB growing downward
└── Heap: Remaining space growing upward
```

#### **Cycle Accounting**
Each BPF instruction has a predefined cycle cost:
- **Load/Store**: 2-4 cycles depending on size
- **Arithmetic**: 1-4 cycles depending on operation
- **Control Flow**: 1-2 cycles
- **Solana Ops**: 2-5 cycles for specialized operations

#### **State Serialization**
Solana accounts are serialized to fixed-size buffers:
- **Account Header**: 89 bytes (pubkey, lamports, owner, flags, metadata)
- **Data Section**: Up to 47 bytes of account data
- **Total Size**: 128 bytes per account

### ZisK Configuration

#### **Environment Variables**
```bash
# Enable ZisK features
export ZISK_FEATURES=1

# Set ZisK target directory
export CARGO_TARGET_DIR=target/zisk

# Custom memory layout
export ZISK_MEMORY_LAYOUT=zisk-memory.x
```

#### **Build Configuration**
The `Cargo.toml` includes ZisK-specific configurations:
- **Feature flags**: `zk` feature enables ZisK integration
- **Target-specific settings**: RISC-V target with custom linker flags
- **Dependency management**: Optional ZisK dependencies

### ZisK Performance

#### **Optimization Features**
- **Static Dispatch**: Eliminates runtime overhead of dynamic dispatch
- **Cycle Bounds**: Precise cycle counting for constraint generation
- **Memory Layout**: Optimized memory layout for ZisK constraints
- **Serialization**: Efficient state serialization for proof generation

#### **Benchmarking**
```bash
# Measure ZisK build time
time cargo build --release --features zk

# Measure proof generation time
time cargo zisk prove -e target/release/solana_test -i zk_input.bin -o zk_proof.bin

# Measure proof verification time
time cargo zisk verify --proof zk_proof.bin
```

### ZisK Troubleshooting

#### **Common Issues**

1. **Target Not Found**
   ```bash
   # Install ZisK target
   rustup target add riscv64ima-zisk-zkvm-elf
   ```

2. **Memory Layout Missing**
   ```bash
   # Regenerate memory layout
   cargo build --release --features zk
   ```

3. **Proof Generation Fails**
   ```bash
   # Check input file
   ls -la zk_input.bin
   
   # Verify program binary
   file target/release/solana_test
   ```

4. **Build Flags Issues**
   ```bash
   # Clean and rebuild
   cargo clean
   cargo build --release --features zk
   ```

#### **Debug Mode**
```bash
# Enable debug output
RUST_LOG=debug cargo run --release --features zk

# Verbose build
cargo build --release --features zk -vv
```

### ZisK Production Deployment

#### **Deployment Checklist**
- [ ] ZisK features enabled in production build
- [ ] Memory layout optimized for production constraints
- [ ] Cycle bounds validated against production limits
- [ ] Proof generation integrated with proving service
- [ ] Monitoring and alerting configured
- [ ] Performance benchmarks established

#### **Scaling Considerations**
- **Batch Processing**: Multiple transactions per proof
- **Parallel Proofs**: Concurrent proof generation
- **Proof Aggregation**: Combining multiple proofs
- **Resource Management**: Efficient memory and compute usage

### Future ZisK Enhancements

#### **Planned Features**
1. **Advanced Constraints**: More sophisticated constraint generation
2. **Proof Aggregation**: Combining multiple proofs efficiently
3. **Custom Circuits**: Specialized circuits for common operations
4. **Performance Optimization**: Further cycle and memory optimization
5. **Integration APIs**: REST APIs for proof generation and verification

#### **Research Areas**
- **Constraint Minimization**: Reducing circuit complexity
- **Memory Optimization**: Advanced memory layout strategies
- **Proof Compression**: Efficient proof representation
- **Verification Optimization**: Faster proof verification

---

*ZisK integration provides a production-ready foundation for zero-knowledge proof generation in Solana transaction validation systems.*
