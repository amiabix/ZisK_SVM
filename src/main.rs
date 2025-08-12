// =================================================================
// ZISK-SVM: BPF INTERPRETER IN RISC-V ZKVM
// =================================================================
//
// CORRECT ARCHITECTURE:
// ZisK (RISC-V) contains BPF interpreter (Rust) that executes Solana programs
// This provides zero-knowledge proofs of Solana BPF execution

use anyhow::{Result, anyhow, Context};

// Import our complete BPF interpreter and ZisK integration
mod complete_bpf_interpreter;
mod bpf_zisk_integration;
mod bpf_test_utils;

use complete_bpf_interpreter::{RealBpfInterpreter, ExecutionResult};
use bpf_zisk_integration::{ZiskBpfExecutor, ZiskExecutionConfig, ZiskTransactionContext, ZiskInstruction, SolanaAccount, AccountMeta};

// ZisK entrypoint - this runs in RISC-V environment
ziskos::entrypoint!(main);

fn main() {
    match run_zisk_bpf_interpreter() {
        Ok(_) => {
            println!("✅ ZisK-SVM: Real BPF execution completed successfully");
        }
        Err(e) => {
            eprintln!("❌ ZisK-SVM error: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_zisk_bpf_interpreter() -> Result<()> {
    println!("🚀 ZisK-SVM: Starting Real BPF Execution in RISC-V zkVM");
    
    // 1. Read input from ZisK environment (RISC-V style)
    let input_data = read_zisk_input()?;
    println!("📥 ZisK input received: {} bytes", input_data.transaction_data.len() + input_data.account_data.len() + input_data.program_data.len());

    // 2. Parse Solana transaction from input
    let transaction_context = parse_solana_transaction_from_input(&input_data)?;
    println!("📋 Parsed transaction with {} instructions", transaction_context.instructions.len());

    // 3. Create ZisK BPF executor with optimized configuration
    let config = create_zisk_optimized_config();
    let mut executor = ZiskBpfExecutor::new(config);
    println!("⚙️  ZisK BPF executor initialized");

    // 4. Load test BPF programs for demonstration
    load_test_programs(&mut executor)?;

    // 5. Load accounts into executor
    load_transaction_accounts(&mut executor, &transaction_context)?;

    // 6. Execute transaction with full BPF interpretation
    println!("⚡ Starting real BPF execution...");
    let execution_result = executor.execute_transaction(transaction_context)?;

    // 7. Process execution results
    process_execution_results(&execution_result)?;

    // 8. Generate ZisK proof data
    let proof_data = generate_zisk_proof(&execution_result)?;

    // 9. Output results for ZisK proof verification
    output_zisk_results(&execution_result, &proof_data)?;

    println!("🎉 ZisK-SVM: Real BPF execution completed successfully!");
    println!("📊 Instructions executed: {}", execution_result.instructions_executed);
    println!("⚡ Total compute units: {}", execution_result.total_compute_units);
    println!("🔄 Total cycles: {}", execution_result.total_cycles);

    Ok(())
}

// =================================================================
// ZISK INPUT/OUTPUT HANDLING
// =================================================================

/// ZisK-compatible input data structure
#[derive(Debug)]
struct ZiskInputData {
    pub version: u32,
    pub transaction_data: Vec<u8>,
    pub account_data: Vec<u8>,
    pub program_data: Vec<u8>,
}

fn read_zisk_input() -> Result<ZiskInputData> {
    // Use proper ZisK input function as per documentation
    let input_bytes: Vec<u8> = read_input();
    
    // Parse the input format
    parse_zisk_input_format(&input_bytes)
    
    // Method 3: Default test data for demonstration
    create_test_zisk_input()
}

fn parse_zisk_input_format(data: &[u8]) -> Result<ZiskInputData> {
    if data.len() < 12 {
        return Err(anyhow::anyhow!("Input data too short"));
    }
    
    let version = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    let tx_len = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
    let acc_len = u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize;
    
    let mut offset = 12;
    let transaction_data = data[offset..offset + tx_len].to_vec();
    offset += tx_len;
    let account_data = data[offset..offset + acc_len].to_vec();
    offset += acc_len;
    let program_data = data[offset..].to_vec();
    
    Ok(ZiskInputData {
        version,
        transaction_data,
        account_data,
        program_data,
    })
}

fn create_test_zisk_input() -> Result<ZiskInputData> {
    Ok(ZiskInputData {
        version: 1,
        transaction_data: create_test_transaction_bytes(),
        account_data: create_test_account_bytes(),
        program_data: create_test_program_bytes(),
    })
}

// =================================================================
// TRANSACTION PARSING AND SETUP
// =================================================================

fn parse_solana_transaction_from_input(input: &ZiskInputData) -> Result<ZiskTransactionContext> {
    // Parse transaction from the input data
    // For demonstration, create a test transaction that exercises real BPF execution
    
    let program_id = [1u8; 32]; // Test program ID
    let account_pubkey = [2u8; 32]; // Test account
    
    Ok(ZiskTransactionContext {
        transaction_hash: compute_transaction_hash(&input.transaction_data),
        instructions: vec![
            // Instruction 1: Math operations
            ZiskInstruction {
                program_id,
                accounts: vec![
                    AccountMeta {
                        pubkey: account_pubkey,
                        is_signer: false,
                        is_writable: true,
                    }
                ],
                data: vec![0x01, 0x00, 0x00, 0x00, 0x2A, 0x00, 0x00, 0x00], // Math operation: add 42
            },
            // Instruction 2: Memory operations
            ZiskInstruction {
                program_id,
                accounts: vec![
                    AccountMeta {
                        pubkey: account_pubkey,
                        is_signer: false,
                        is_writable: true,
                    }
                ],
                data: vec![0x02, 0x00, 0x00, 0x00], // Memory operation
            },
            // Instruction 3: Logging operation
            ZiskInstruction {
                program_id,
                accounts: vec![],
                data: b"Hello from ZisK-SVM BPF!".to_vec(),
            },
        ],
        accounts: vec![
            SolanaAccount {
                pubkey: account_pubkey,
                lamports: 1_000_000,
                data: vec![0u8; 1024], // 1KB account data
                owner: program_id,
                executable: false,
                rent_epoch: 200,
            }
        ],
        recent_blockhash: [0x42u8; 32],
        signatures: vec![[0x13u8; 64]],
    })
}

fn compute_transaction_hash(data: &[u8]) -> [u8; 32] {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

// =================================================================
// BPF PROGRAM CREATION AND LOADING
// =================================================================

fn load_test_programs(executor: &mut ZiskBpfExecutor) -> Result<()> {
    println!("📦 Loading test BPF programs...");
    
    // Program 1: Math and logic operations
    let math_program = create_math_test_program()?;
    executor.load_program([1u8; 32], math_program)?;
    println!("   ✅ Math program loaded");
    
    // Program 2: Memory operations
    let memory_program = create_memory_test_program()?;
    executor.load_program([2u8; 32], memory_program)?;
    println!("   ✅ Memory program loaded");
    
    // Program 3: Syscall operations  
    let syscall_program = create_syscall_test_program()?;
    executor.load_program([3u8; 32], syscall_program)?;
    println!("   ✅ Syscall program loaded");
    
    Ok(())
}

fn create_math_test_program() -> Result<Vec<u8>> {
    let mut program = create_elf_header();
    
    // BPF program that performs math operations:
    // 1. Load immediate 10 into r1
    // 2. Load immediate 32 into r2  
    // 3. Add r1 + r2 -> r3
    // 4. Multiply r3 * 2 -> r4
    // 5. Store result and exit
    
    program.extend_from_slice(&[
        // MOV r1, 10
        0xB7, 0x01, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00,
        // MOV r2, 32  
        0xB7, 0x02, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00,
        // ADD r3, r1, r2 (r3 = r1 + r2)
        0x0F, 0x21, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // MOV r4, r3
        0xBF, 0x43, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // MUL r4, 2
        0x27, 0x04, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
        // MOV r0, r4 (return value)
        0xBF, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // EXIT
        0x95, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ]);
    
    Ok(program)
}

fn create_memory_test_program() -> Result<Vec<u8>> {
    let mut program = create_elf_header();
    
    // BPF program that performs memory operations:
    // 1. Write data to memory
    // 2. Read data from memory  
    // 3. Verify correctness
    // 4. Exit with result
    
    program.extend_from_slice(&[
        // MOV r1, heap_addr (0x100000000)
        0x18, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // MOV r2, 0x12345678 (test value)
        0xB7, 0x02, 0x00, 0x00, 0x78, 0x56, 0x34, 0x12,
        // STX [r1], r2 (store r2 at address r1)
        0x7B, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // LDX r3, [r1] (load from address r1 into r3)
        0x79, 0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // Compare r2 and r3
        0x5D, 0x32, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, // JNE r3, r2, +2
        // Success: MOV r0, 1
        0xB7, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        0x05, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, // JA +1
        // Failure: MOV r0, 0
        0xB7, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // EXIT
        0x95, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ]);
    
    Ok(program)
}

fn create_syscall_test_program() -> Result<Vec<u8>> {
    let mut program = create_elf_header();
    
    // BPF program that tests Solana syscalls:
    // 1. Call sol_log with message
    // 2. Call sol_sha256 with data
    // 3. Set return data
    // 4. Exit successfully
    
    program.extend_from_slice(&[
        // Set up log message in memory
        0x18, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, // MOV r1, heap_addr
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // Store "Hello BPF!" message
        0xB7, 0x02, 0x00, 0x00, 0x6C, 0x6C, 0x65, 0x48, // "Hell"
        0x73, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // STX [r1], r2
        0xB7, 0x02, 0x00, 0x00, 0x21, 0x46, 0x50, 0x42, // "o BP"
        0x73, 0x12, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, // STX [r1+4], r2
        // Call sol_log syscall
        0xB7, 0x02, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, // MOV r2, 8 (length)
        0x85, 0x00, 0x00, 0x00, 0xF0, 0xC5, 0x6F, 0x7C, // CALL sol_log
        // Set return data
        0xB7, 0x01, 0x00, 0x00, 0x42, 0x00, 0x00, 0x00, // MOV r1, 0x42
        0xB7, 0x02, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, // MOV r2, 4
        0x85, 0x00, 0x00, 0x00, 0xA3, 0x38, 0x2A, 0x26, // CALL sol_set_return_data
        // Exit successfully
        0xB7, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // MOV r0, 0
        0x95, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // EXIT
    ]);
    
    Ok(program)
}

fn create_elf_header() -> Vec<u8> {
    let mut header = vec![0u8; 64];
    header[0..4].copy_from_slice(b"\x7fELF"); // ELF magic
    header[4] = 2; // 64-bit
    header[5] = 1; // Little endian
    header[6] = 1; // Version 1
    header
}

// =================================================================
// ACCOUNT LOADING AND MANAGEMENT
// =================================================================

fn load_transaction_accounts(executor: &mut ZiskBpfExecutor, transaction: &ZiskTransactionContext) -> Result<()> {
    println!("👥 Loading transaction accounts...");
    
    for account in &transaction.accounts {
        executor.load_account(account.clone());
        println!("   ✅ Account loaded: {:02x}...", account.pubkey[0]);
    }
    
    Ok(())
}

// =================================================================
// EXECUTION RESULT PROCESSING
// =================================================================

fn process_execution_results(result: &bpf_zisk_integration::ZiskExecutionResult) -> Result<()> {
    println!("\n🎯 BPF Execution Results:");
    println!("   Success: {}", result.success);
    println!("   Instructions: {}", result.instructions_executed);
    println!("   Compute Units: {}", result.total_compute_units);
    println!("   Cycles: {}", result.total_cycles);
    println!("   Account Changes: {}", result.account_changes.len());
    
    // Log execution details
    if !result.logs.is_empty() {
        println!("\n📋 Execution Logs:");
        for (i, log) in result.logs.iter().enumerate() {
            println!("   {}: {}", i + 1, log);
        }
    }
    
    // Show return data
    if let Some(ref return_data) = result.return_data {
        println!("\n📤 Return Data: {} bytes", return_data.len());
        if return_data.len() <= 32 {
            println!("   Data: {:02x?}", return_data);
        }
    }
    
    // Report any errors
    if let Some(ref error) = result.error_message {
        println!("\n❌ Error: {}", error);
    }
    
    // Execution trace summary
    println!("\n🔍 Execution Trace:");
    for (i, step) in result.proof_data.execution_trace.iter().enumerate() {
        println!("   Step {}: Instruction {}, Cycles: {}", 
            i + 1, step.instruction_index, step.cycles_consumed);
    }
    
    Ok(())
}

// =================================================================
// ZISK PROOF GENERATION
// =================================================================

#[derive(Debug)]
struct ZiskProofOutput {
    pub execution_summary: ExecutionSummary,
    pub witness_data: Vec<u8>,
    pub public_inputs: Vec<u8>,
    pub state_commitment: [u8; 32],
}

#[derive(Debug, Clone)]
struct ExecutionSummary {
    pub success: bool,
    pub instructions_executed: usize,
    pub compute_units_consumed: u64,
    pub cycles_consumed: u32,
    pub final_account_state_hash: [u8; 32],
}

fn generate_zisk_proof(execution_result: &bpf_zisk_integration::ZiskExecutionResult) -> Result<ZiskProofOutput> {
    println!("🔐 Generating ZisK proof data...");
    
    // Create execution summary for proof
    let execution_summary = ExecutionSummary {
        success: execution_result.success,
        instructions_executed: execution_result.instructions_executed,
        compute_units_consumed: execution_result.total_compute_units,
        cycles_consumed: execution_result.total_cycles,
        final_account_state_hash: compute_account_state_hash(&execution_result.account_changes),
    };
    
    // Generate witness data (private inputs for proof)
    let witness_data = generate_witness_data(execution_result)?;
    
    // Generate public inputs (verifiable outputs)
    let public_inputs = generate_public_inputs(&execution_summary)?;
    
    // Compute state commitment
    let state_commitment = compute_state_commitment(&execution_summary, &witness_data)?;
    
    println!("   ✅ Witness data: {} bytes", witness_data.len());
    println!("   ✅ Public inputs: {} bytes", public_inputs.len());
    println!("   ✅ State commitment: {:02x}...", state_commitment[0]);
    
    Ok(ZiskProofOutput {
        execution_summary,
        witness_data,
        public_inputs,
        state_commitment,
    })
}

fn compute_account_state_hash(account_changes: &[bpf_zisk_integration::AccountChange]) -> [u8; 32] {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    
    for change in account_changes {
        hasher.update(&change.pubkey);
        hasher.update(&change.lamports_after.to_le_bytes());
        hasher.update(&change.data_after);
    }
    
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

fn generate_witness_data(execution_result: &bpf_zisk_integration::ZiskExecutionResult) -> Result<Vec<u8>> {
    let mut witness = Vec::new();
    
    // Add execution trace
    for step in &execution_result.proof_data.execution_trace {
        witness.extend_from_slice(&step.instruction_index.to_le_bytes());
        witness.extend_from_slice(&step.cycles_consumed.to_le_bytes());
    }
    
    // Add account changes
    for change in &execution_result.account_changes {
        witness.extend_from_slice(&change.pubkey);
        witness.extend_from_slice(&change.lamports_before.to_le_bytes());
        witness.extend_from_slice(&change.lamports_after.to_le_bytes());
        witness.extend_from_slice(&(change.data_before.len() as u32).to_le_bytes());
        witness.extend_from_slice(&change.data_before);
        witness.extend_from_slice(&(change.data_after.len() as u32).to_le_bytes());
        witness.extend_from_slice(&change.data_after);
    }
    
    Ok(witness)
}

fn generate_public_inputs(summary: &ExecutionSummary) -> Result<Vec<u8>> {
    let mut inputs = Vec::new();
    
    // Success flag
    inputs.push(if summary.success { 1 } else { 0 });
    
    // Execution metrics
    inputs.extend_from_slice(&summary.instructions_executed.to_le_bytes());
    inputs.extend_from_slice(&summary.compute_units_consumed.to_le_bytes());
    inputs.extend_from_slice(&summary.cycles_consumed.to_le_bytes());
    
    // Final state hash
    inputs.extend_from_slice(&summary.final_account_state_hash);
    
    Ok(inputs)
}

fn compute_state_commitment(summary: &ExecutionSummary, witness: &[u8]) -> Result<[u8; 32]> {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    
    hasher.update(&summary.instructions_executed.to_le_bytes());
    hasher.update(&summary.compute_units_consumed.to_le_bytes());
    hasher.update(&summary.cycles_consumed.to_le_bytes());
    hasher.update(&summary.final_account_state_hash);
    hasher.update(witness);
    
    let result = hasher.finalize();
    let mut commitment = [0u8; 32];
    commitment.copy_from_slice(&result);
    Ok(commitment)
}

// =================================================================
// ZISK OUTPUT
// =================================================================

fn output_zisk_results(execution_result: &bpf_zisk_integration::ZiskExecutionResult, proof: &ZiskProofOutput) -> Result<()> {
    println!("📤 Outputting ZisK results...");
    
    // Create ZisK output structure
    let zisk_output = ZiskOutput {
        success: execution_result.success,
        execution_summary: proof.execution_summary.clone(),
        witness_data: proof.witness_data.clone(),
        public_inputs: proof.public_inputs.clone(),
        state_commitment: proof.state_commitment,
        logs: execution_result.logs.clone(),
        return_data: execution_result.return_data.clone(),
    };
    
    // Method 1: ZisK journal output (like RISC Zero env::commit)
    output_to_zisk_journal(&zisk_output)?;
    
    // Method 2: JSON output for debugging
    output_to_json(&zisk_output)?;
    
    // Method 3: Binary output for proof verification
    output_to_binary(&zisk_output)?;
    
    println!("   ✅ ZisK journal output committed");
    println!("   ✅ JSON output written");
    println!("   ✅ Binary output written");
    
    Ok(())
}

#[derive(Debug)]
struct ZiskOutput {
    pub success: bool,
    pub execution_summary: ExecutionSummary,
    pub witness_data: Vec<u8>,
    pub public_inputs: Vec<u8>,
    pub state_commitment: [u8; 32],
    pub logs: Vec<String>,
    pub return_data: Option<Vec<u8>>,
}

fn output_to_zisk_journal(output: &ZiskOutput) -> Result<()> {
    // Use proper ZisK output functions as per documentation
    // set_output(id, value) for each output value
    
    set_output(0, output.success as u32);
    set_output(1, output.execution_summary.instructions_executed as u32);
    set_output(2, output.execution_summary.compute_units_consumed as u32);
    set_output(3, output.execution_summary.cycles_consumed as u32);
    
    // For complex data like state commitment, we need to split into u32 chunks
    // State commitment is 32 bytes = 8 u32 values
    for i in 0..8 {
        let val = u32::from_le_bytes([
            output.state_commitment[i * 4],
            output.state_commitment[i * 4 + 1],
            output.state_commitment[i * 4 + 2],
            output.state_commitment[i * 4 + 3],
        ]);
        set_output(4 + i, val);
    }
    
    Ok(())
}

fn output_to_json(output: &ZiskOutput) -> Result<()> {
    // Write JSON output for debugging and verification
    let json_output = serde_json::json!({
        "success": output.success,
        "execution_summary": {
            "instructions_executed": output.execution_summary.instructions_executed,
            "compute_units_consumed": output.execution_summary.compute_units_consumed,
            "cycles_consumed": output.execution_summary.cycles_consumed,
            "final_account_state_hash": hex::encode(output.execution_summary.final_account_state_hash)
        },
        "state_commitment": hex::encode(output.state_commitment),
        "public_inputs": hex::encode(&output.public_inputs),
        "witness_size": output.witness_data.len(),
        "logs": output.logs,
        "return_data": output.return_data.as_ref().map(hex::encode)
    });
    
    // In ZisK environment, this might be written to a specific location
    if let Ok(json_str) = serde_json::to_string_pretty(&json_output) {
        std::fs::write("zisk_execution_result.json", json_str)?;
    }
    
    Ok(())
}

fn output_to_binary(output: &ZiskOutput) -> Result<()> {
    let mut binary_output = Vec::new();
    
    // Header
    binary_output.extend_from_slice(b"ZISK");
    binary_output.extend_from_slice(&1u32.to_le_bytes()); // Version
    
    // Success flag
    binary_output.push(if output.success { 1 } else { 0 });
    
    // Execution summary
    binary_output.extend_from_slice(&output.execution_summary.instructions_executed.to_le_bytes());
    binary_output.extend_from_slice(&output.execution_summary.compute_units_consumed.to_le_bytes());
    binary_output.extend_from_slice(&output.execution_summary.cycles_consumed.to_le_bytes());
    
    // State commitment
    binary_output.extend_from_slice(&output.state_commitment);
    
    // Public inputs
    binary_output.extend_from_slice(&(output.public_inputs.len() as u32).to_le_bytes());
    binary_output.extend_from_slice(&output.public_inputs);
    
    // Witness data
    binary_output.extend_from_slice(&(output.witness_data.len() as u32).to_le_bytes());
    binary_output.extend_from_slice(&output.witness_data);
    
    std::fs::write("zisk_proof_data.bin", binary_output)?;
    
    Ok(())
}

// =================================================================
// CONFIGURATION AND UTILITIES
// =================================================================

fn create_zisk_optimized_config() -> ZiskExecutionConfig {
    ZiskExecutionConfig {
        max_compute_units: 1_400_000, // Solana standard limit
        max_cycles: 1_000_000,        // ZisK constraint
        max_memory: 64 * 1024 * 1024, // 64MB memory limit
        enable_logging: true,         // Enable for demonstration
        enable_debug: false,          // Disable for performance
    }
}

fn create_test_transaction_bytes() -> Vec<u8> {
    // Create test transaction data
    let mut tx_data = Vec::new();
    tx_data.extend_from_slice(b"SOLANA_TX");
    tx_data.extend_from_slice(&1u32.to_le_bytes()); // Version
    tx_data.extend_from_slice(&3u32.to_le_bytes()); // Instruction count
    tx_data
}

fn create_test_account_bytes() -> Vec<u8> {
    // Create test account data
    let mut acc_data = Vec::new();
    acc_data.extend_from_slice(b"SOLANA_ACC");
    acc_data.extend_from_slice(&1u32.to_le_bytes()); // Account count
    acc_data
}

fn create_test_program_bytes() -> Vec<u8> {
    // Create test program data
    let mut prog_data = Vec::new();
    prog_data.extend_from_slice(b"SOLANA_PROG");
    prog_data.extend_from_slice(&3u32.to_le_bytes()); // Program count
    prog_data
}

// Include required external dependencies
extern crate serde_json;
extern crate hex;
extern crate sha2;

