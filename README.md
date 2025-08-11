# Solana Virtual Machine with ZisK Integration

An implementation of a Solana Virtual Machine (SVM) integrated with the ZisK zero-knowledge virtual machine for generating cryptographic proofs of transaction execution.

## Overview

This project implements a complete Solana transaction processing pipeline that executes within ZisK constraints, enabling zero-knowledge proof generation for Solana program execution. The system fetches real transaction data from Solana mainnet, executes transactions using the official Solana RBPF interpreter, and generates cryptographic proofs that can be verified without revealing transaction details.

## Architecture

The system is built around a layered architecture that separates concerns while maintaining tight integration between components:

- **Data Layer**: RPC integration for fetching live Solana blockchain data
- **Parsing Layer**: Transaction and account data parsing from multiple formats
- **Execution Layer**: BPF program execution using Solana RBPF
- **ZisK Integration Layer**: Memory management and proof generation within ZisK constraints
- **Output Layer**: Proof generation and verification data

## Core Components

### Main Entry Point (`src/main.rs`)

The ZisK entry point that orchestrates the entire execution pipeline. This file implements the `#![no_main]` attribute required by ZisK and provides the main execution function that reads input data, executes SVM logic, and generates proofs.

**Key Functions:**
- `main()`: ZisK entry point for program execution
- `read_zisk_input()`: Reads transaction data from ZisK input format
- `execute_svm_validation()`: Orchestrates SVM execution within ZisK context
- `generate_zisk_proof()`: Creates cryptographic proofs from execution results
- `parse_zisk_input()`: Converts binary input to structured transaction data

**Data Structures:**
- `ZiskInputData`: Complete transaction input structure
- `TransactionData`: Parsed transaction information
- `SvmExecutionResult`: SVM execution results with proof data
- `ZiskProof`: Complete proof structure with metadata

### ZisK-SVM Bridge (`src/zisk_svm_bridge.rs`)

The critical bridge module that manages execution context when running SVM within ZisK constraints. This module handles memory layout, cycle counting, proof generation state, and provides the interface between SVM execution and ZisK proof generation.

**Key Components:**
- `ZiskSvmContext`: Main execution context managing SVM within ZisK
- `ZiskMemoryLayout`: Memory organization optimized for ZisK constraints
- `ProofGenerationState`: State management for proof generation
- `ExecutionTraceEntry`: Detailed execution tracking for proofs

**Memory Layout:**
- Code Section: 0x1000 - 0x11000 (64KB)
- Data Section: 0x11000 - 0x111000 (1MB)
- Stack Section: 0x111000 - 0x211000 (1MB)
- Heap Section: 0x211000 - 0x411000 (32MB)
- Total Available: 64MB

**Core Functions:**
- `execute_transaction()`: Main execution function with proof generation
- `start_proof_generation()`: Initializes proof generation state
- `record_execution_result()`: Captures execution results for proofs
- `calculate_memory_hash()`: Creates memory state hashes for proofs

### Solana Execution Environment (`src/solana_executor.rs`)

The core SVM implementation that manages account state, executes BPF programs, and handles transaction processing. This module provides the execution environment for Solana programs with proper compute unit accounting and account state management.

**Key Features:**
- Account state management with real Solana account structures
- BPF program execution using Solana RBPF
- Transaction parsing from multiple formats (binary, base64, JSON)
- Ed25519 signature verification using ed25519-dalek
- Compute unit tracking and limits

**Core Methods:**
- `execute_transaction()`: Main transaction execution function
- `execute_instruction()`: Individual instruction execution
- `parse_binary_transaction()`: Binary transaction parsing
- `validate_signatures()`: Ed25519 signature validation
- `update_account()`: Account state updates

**Data Structures:**
- `SolanaExecutionEnvironment`: Main execution context
- `SolanaTransaction`: Transaction representation
- `TransactionResult`: Execution results with metadata
- `InstructionResult`: Individual instruction execution results

### BPF Interpreter (`src/bpf_interpreter.rs`)

Implements the Berkeley Packet Filter instruction set and execution context for Solana programs. This module provides the low-level execution environment for BPF programs with proper memory management and instruction execution.

**Key Components:**
- Complete BPF instruction set implementation
- Memory region management (heap, stack, account data)
- Solana account structure with all required fields
- Instruction execution context and state

**Account Structure:**
```rust
pub struct SolanaAccount {
    pub pubkey: [u8; 32],
    pub lamports: u64,
    pub owner: [u8; 32],
    pub executable: bool,
    pub rent_epoch: u64,
    pub data: Vec<u8>,
}
```

**Core Functions:**
- `execute_instruction()`: BPF instruction execution
- `allocate_memory()`: Dynamic memory allocation
- `serialize_account()`: Account data serialization
- `deserialize_account()`: Account data deserialization

### Real BPF Loader (`src/real_bpf_loader.rs`)

Manages loading and execution of real BPF programs from Solana mainnet. This module integrates with the Solana RBPF crate to provide program loading, execution, and account conversion capabilities.

**Key Features:**
- Real BPF program loading from binary data
- Program execution with proper memory context
- Account data conversion between formats
- Program information and metadata management

**Core Methods:**
- `load_program()`: Loads BPF program into execution environment
- `execute_program_simple()`: Executes program with simplified context
- `convert_account()`: Converts between account representations
- `get_program_info()`: Retrieves program metadata
- `list_programs()`: Lists loaded programs

### Real Solana Parser (`src/real_solana_parser.rs`)

Parses Solana transaction and account data from various formats including JSON, binary, and base64. This module provides comprehensive parsing capabilities for real Solana blockchain data.

**Supported Formats:**
- JSON transaction data from RPC
- Binary transaction data
- Base64 encoded transaction data
- Account data parsing and conversion

**Core Functions:**
- `parse_binary_transaction()`: Binary transaction parsing
- `parse_base64_transaction()`: Base64 transaction parsing
- `parse_raw_binary_transaction()`: Raw binary parsing
- `to_bpf_account()`: Account format conversion

### Real Account Loader (`src/real_account_loader.rs`)

Fetches real account data from Solana RPC endpoints and manages account state loading. This module ensures that all account data used in execution comes from live blockchain data rather than sample or placeholder data.

**Key Features:**
- RPC integration for live account data
- Account state caching and management
- Error handling and fallback mechanisms
- Real-time account synchronization

**Core Methods:**
- `load_account()`: Fetches account from RPC
- `load_multiple_accounts()`: Batch account loading
- `update_account_cache()`: Cache management
- `get_account_info()`: Account information retrieval

## Build System

### Build Script (`build.rs`)

The build script generates ZisK input files from real Solana transaction data. During the build process, it fetches live transaction data from Solana mainnet, converts it to ZisK-compatible binary format, and generates proof request metadata.

**Key Functions:**
- `generate_zisk_input_files()`: Main build function
- `get_latest_transaction_signature()`: Fetches recent transaction signatures
- `fetch_transaction_data()`: Retrieves full transaction data from RPC
- `fetch_account_data()`: Fetches real account state data
- `create_zisk_input_from_transaction()`: Converts transaction data to ZisK format

**Generated Files:**
- `build/input.bin`: Binary input data for ZisK execution
- `build/proof_request.json`: Proof request metadata

**Data Flow:**
1. Fetch latest transaction signature from Solana mainnet
2. Retrieve complete transaction data including account information
3. Fetch real account state data for all accounts involved
4. Serialize data into ZisK binary input format
5. Generate proof request metadata with transaction details

### Memory Configuration (`zisk-memory.x`)

Linker script that defines the memory layout for ZisK execution. This file configures memory regions, stack placement, and heap allocation to work within ZisK constraints while providing optimal performance for SVM execution.

**Memory Layout:**
- Code Section: Executable code and read-only data
- Data Section: Static variables and constants
- Stack Section: Function call stack and local variables
- Heap Section: Dynamic memory allocation

**Linker Configuration:**
- Memory region definitions
- Section placement rules
- Symbol definitions for Rust integration
- Alignment and optimization settings

## Dependencies

### Core Dependencies
- `solana-rbpf`: Official Solana BPF interpreter
- `solana-sdk`: Solana development kit
- `ed25519-dalek`: Ed25519 signature verification
- `sha2`: Cryptographic hashing
- `anyhow`: Error handling and context

### ZisK Dependencies
- `ziskos`: ZisK zero-knowledge virtual machine
- Target: `riscv64ima-zisk-zkvm-elf`

### Build Dependencies
- `reqwest`: HTTP client for RPC calls
- `bs58`: Base58 encoding/decoding
- `chrono`: Date and time handling
- `serde`: Serialization framework

## Usage

### Building for ZisK

```bash
# Build for ZisK execution target
cargo build --target riscv64ima-zisk-zkvm-elf --release

# Generate ZisK input files with real transaction data
cargo build
```

### ZisK Execution

```bash
# Execute with ZisK using generated input
zisk prove --input build/input.bin --output proof.bin

# Verify generated proof
zisk verify --input build/input.bin --proof proof.bin
```

### Development

```bash
# Run tests
cargo test

# Check compilation
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy
```

## Technical Specifications

### ZisK Integration
- **Entry Point**: `#![no_main]` with `ziskos::entrypoint!`
- **Memory Model**: 64MB total with optimized layout
- **Cycle Counting**: Real-time ZisK cycle consumption tracking
- **Proof Generation**: Complete execution proof data
- **Memory Access**: Tracked for proof inclusion

### Solana Compatibility
- **Transaction Formats**: JSON, Binary, Base64
- **Signature Verification**: Ed25519 with ed25519-dalek
- **Account Structure**: Complete Solana account representation
- **BPF Programs**: Full instruction set support
- **Compute Units**: Proper accounting and limits

### Performance Characteristics
- **Memory Usage**: 64MB total with 32MB heap
- **Stack Size**: 1MB for function calls
- **Code Space**: 64KB for program code
- **Data Space**: 1MB for static data
- **Execution**: Optimized for ZisK constraints

## Error Handling

The system implements comprehensive error handling throughout all components:

- **RPC Failures**: Graceful fallbacks and retry mechanisms
- **Data Parsing**: Detailed error messages with context
- **Execution Errors**: Proper error propagation and logging
- **Memory Errors**: Bounds checking and allocation validation
- **Proof Generation**: Error handling for proof creation failures

## Testing

The codebase includes comprehensive testing for all major components:

- **Unit Tests**: Individual function and method testing
- **Integration Tests**: Component interaction testing
- **Memory Tests**: ZisK memory layout validation
- **Execution Tests**: SVM execution verification
- **Proof Tests**: ZisK proof generation validation

## Production Considerations

### Security
- All cryptographic operations use verified libraries
- Input validation and sanitization throughout
- Memory bounds checking and overflow protection
- Secure RPC communication with proper error handling

### Performance
- Memory layout optimized for ZisK execution
- Efficient data structures and algorithms
- Minimal memory allocation during execution
- Optimized proof generation pipeline

### Scalability
- Modular architecture for component replacement
- Configurable memory and compute limits
- Efficient account data caching
- Batch processing capabilities

## Contributing

This project follows standard Rust development practices:

1. Fork the repository
2. Create a feature branch
3. Implement changes with tests
4. Ensure all tests pass
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Solana Labs for the RBPF implementation
- ZisK team for the zero-knowledge virtual machine
- Rust community for excellent tooling and ecosystem
- Solana community for blockchain integration patterns
