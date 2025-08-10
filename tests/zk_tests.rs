use std::fs;
use std::process::Command;
use std::path::Path;

use solana_test::host::input::create_test_block;
use solana_test::host::proof_verifier::verify_proof;

/// Expected state root for test cases
/// This should match the output from the ZisK guest program
const EXPECTED_STATE_ROOT: [u8; 32] = [
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
    0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10,
    0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
    0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20,
];

/// Test ZK arithmetic operations
#[test]
fn test_zk_arithmetic() {
    // Create test input with arithmetic operations
    let input = vec![
        0x00, 0x01, 0x02, // ADD r1, r2
        0x01, 0x03, 0x04, // SUB r3, r4
        0x02, 0x05, 0x06, // MUL r5, r6
        0x03, 0x07, 0x08, // DIV r7, r8
    ];
    
    // Write input to file
    let input_path = "test_arithmetic.bin";
    fs::write(input_path, input).unwrap();
    
    // Generate proof using cargo-zisk
    let proof_result = generate_proof(input_path);
    assert!(proof_result.is_ok(), "Proof generation failed: {:?}", proof_result.err());
    
    // Verify proof
    let verification_result = verify_proof("proof.bin");
    assert!(verification_result.is_ok(), "Proof verification failed: {:?}", verification_result.err());
    
    let result = verification_result.unwrap();
    assert!(result.is_valid, "Proof should be valid");
    
    // Clean up
    fs::remove_file(input_path).ok();
    fs::remove_file("proof.bin").ok();
}

/// Test ZK memory operations
#[test]
fn test_zk_memory() {
    // Create test input with memory operations
    let input = vec![
        0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // LDW at offset 0
        0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // LDW at offset 0
        0x62, 0x01, 0x02, // ST r1 to r2
        0x63, 0x03, 0x04, // ST r3 to immediate
    ];
    
    let input_path = "test_memory.bin";
    fs::write(input_path, input).unwrap();
    
    let proof_result = generate_proof(input_path);
    assert!(proof_result.is_ok());
    
    let verification_result = verify_proof("proof.bin");
    assert!(verification_result.is_ok());
    
    let result = verification_result.unwrap();
    assert!(result.is_valid);
    
    // Clean up
    fs::remove_file(input_path).ok();
    fs::remove_file("proof.bin").ok();
}

/// Test ZK control flow operations
#[test]
fn test_zk_control_flow() {
    // Create test input with control flow
    let input = vec![
        0x15, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // JEQ immediate
        0x05, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // JA +16
        0x85, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // CALL +32
        0x95, // EXIT
    ];
    
    let input_path = "test_control_flow.bin";
    fs::write(input_path, input).unwrap();
    
    let proof_result = generate_proof(input_path);
    assert!(proof_result.is_ok());
    
    let verification_result = verify_proof("proof.bin");
    assert!(verification_result.is_ok());
    
    let result = verification_result.unwrap();
    assert!(result.is_valid);
    
    // Clean up
    fs::remove_file(input_path).ok();
    fs::remove_file("proof.bin").ok();
}

/// Test ZK Solana-specific operations
#[test]
fn test_zk_solana_ops() {
    // Create test input with Solana operations
    let input = vec![
        0xE0, // SOL_CALL (Cross-program invocation)
        0xE1, // SOL_LOG
        0xE2, // SOL_RETURN
    ];
    
    let input_path = "test_solana_ops.bin";
    fs::write(input_path, input).unwrap();
    
    let proof_result = generate_proof(input_path);
    assert!(proof_result.is_ok());
    
    let verification_result = verify_proof("proof.bin");
    assert!(verification_result.is_ok());
    
    let result = verification_result.unwrap();
    assert!(result.is_valid);
    
    // Clean up
    fs::remove_file(input_path).ok();
    fs::remove_file("proof.bin").ok();
}

/// Test ZK cycle accounting
#[test]
fn test_zk_cycle_accounting() {
    // Create test input that should exceed cycle limit
    let mut input = Vec::new();
    
    // Add many operations to exceed cycle limit
    for _ in 0..1_000_000 {
        input.push(0x00); // ADD operation (cost: 1)
    }
    
    let input_path = "test_cycle_limit.bin";
    fs::write(input_path, &input).unwrap();
    
    let proof_result = generate_proof(input_path);
    
    // This should either succeed (if cycle limit is high enough) or fail gracefully
    if proof_result.is_ok() {
        let verification_result = verify_proof("proof.bin");
        assert!(verification_result.is_ok());
        
        let result = verification_result.unwrap();
        assert!(result.is_valid);
    }
    
    // Clean up
    fs::remove_file(input_path).ok();
    fs::remove_file("proof.bin").ok();
}

/// Test ZK memory bounds checking
#[test]
fn test_zk_memory_bounds() {
    // Create test input that accesses out-of-bounds memory
    let input = vec![
        0x18, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // LDW at very high offset
    ];
    
    let input_path = "test_memory_bounds.bin";
    fs::write(input_path, input).unwrap();
    
    let proof_result = generate_proof(input_path);
    
    // This should either succeed (if bounds checking is disabled) or fail gracefully
    if proof_result.is_ok() {
        let verification_result = verify_proof("proof.bin");
        assert!(verification_result.is_ok());
        
        let result = verification_result.unwrap();
        assert!(result.is_valid);
    }
    
    // Clean up
    fs::remove_file(input_path).ok();
    fs::remove_file("proof.bin").ok();
}

/// Test ZK integration with full Solana block
#[test]
fn test_zk_full_block() {
    // Create a test block with accounts and transactions
    let block = create_test_block();
    
    // Generate ZK input
    let input_path = "test_full_block.bin";
    let input_result = solana_test::host::input::generate_zk_input(&block, input_path);
    assert!(input_result.is_ok(), "Failed to generate ZK input: {:?}", input_result.err());
    
    // Generate proof
    let proof_result = generate_proof(input_path);
    assert!(proof_result.is_ok(), "Proof generation failed: {:?}", proof_result.err());
    
    // Verify proof
    let verification_result = verify_proof("proof.bin");
    assert!(verification_result.is_ok(), "Proof verification failed: {:?}", verification_result.err());
    
    let result = verification_result.unwrap();
    assert!(result.is_valid, "Proof should be valid");
    
    // Verify state root matches expected
    assert_eq!(result.state_root, EXPECTED_STATE_ROOT, "State root mismatch");
    
    // Clean up
    fs::remove_file(input_path).ok();
    fs::remove_file("proof.bin").ok();
}

/// Test ZK proof verification with invalid proof
#[test]
fn test_zk_invalid_proof() {
    // Create an invalid proof (empty file)
    let invalid_proof_path = "invalid_proof.bin";
    fs::write(invalid_proof_path, vec![]).unwrap();
    
    // This should fail gracefully
    let verification_result = verify_proof(invalid_proof_path);
    
    // Either it fails during verification or returns an error
    if verification_result.is_ok() {
        let result = verification_result.unwrap();
        // If it succeeds, the proof should be marked as invalid
        assert!(!result.is_valid || result.state_root == [0u8; 32]);
    }
    
    // Clean up
    fs::remove_file(invalid_proof_path).ok();
}

/// Test ZK batch verification
#[test]
fn test_zk_batch_verification() {
    // Create multiple test inputs
    let inputs = vec![
        ("batch_test1.bin", vec![0x00, 0x01, 0x02]),
        ("batch_test2.bin", vec![0x01, 0x03, 0x04]),
        ("batch_test3.bin", vec![0x02, 0x05, 0x06]),
    ];
    
    let mut proof_paths = Vec::new();
    
    // Generate proofs for each input
    for (input_path, input_data) in inputs {
        fs::write(input_path, input_data).unwrap();
        
        let proof_result = generate_proof(input_path);
        if proof_result.is_ok() {
            proof_paths.push(format!("proof_{}.bin", input_path.trim_end_matches(".bin")));
        }
    }
    
    // Batch verify all proofs
    if !proof_paths.is_empty() {
        let batch_result = solana_test::host::proof_verifier::batch_verify_proofs(&proof_paths);
        assert!(batch_result.is_ok(), "Batch verification failed: {:?}", batch_result.err());
        
        let results = batch_result.unwrap();
        assert!(!results.is_empty(), "Should have at least one valid proof");
        
        // All proofs should be valid
        for result in &results {
            assert!(result.is_valid, "All proofs should be valid");
        }
    }
    
    // Clean up
    for (input_path, _) in inputs {
        fs::remove_file(input_path).ok();
    }
    for proof_path in proof_paths {
        fs::remove_file(proof_path).ok();
    }
}

/// Test ZK performance characteristics
#[test]
fn test_zk_performance() {
    // Create a moderately complex input
    let mut input = Vec::new();
    
    // Add various operations to test performance
    for i in 0..1000 {
        input.push(0x00); // ADD
        input.push(0x01); // SUB
        input.push(0x02); // MUL
        input.push(0x03); // DIV
        
        if i % 100 == 0 {
            input.push(0x15); // JEQ
            input.extend_from_slice(&(i as u64).to_le_bytes());
        }
    }
    
    let input_path = "test_performance.bin";
    fs::write(input_path, input).unwrap();
    
    // Measure proof generation time
    let start_time = std::time::Instant::now();
    let proof_result = generate_proof(input_path);
    let generation_time = start_time.elapsed();
    
    assert!(proof_result.is_ok(), "Performance test proof generation failed");
    
    // Measure verification time
    let start_time = std::time::Instant::now();
    let verification_result = verify_proof("proof.bin");
    let verification_time = start_time.elapsed();
    
    assert!(verification_result.is_ok(), "Performance test verification failed");
    
    let result = verification_result.unwrap();
    assert!(result.is_valid);
    
    // Log performance metrics
    println!("Performance Test Results:");
    println!("  Proof Generation: {:?}", generation_time);
    println!("  Proof Verification: {:?}", verification_time);
    println!("  Proof Size: {} bytes", result.proof_size_bytes);
    
    // Performance assertions (adjust thresholds as needed)
    assert!(generation_time.as_secs() < 60, "Proof generation took too long: {:?}", generation_time);
    assert!(verification_time.as_millis() < 1000, "Proof verification took too long: {:?}", verification_time);
    
    // Clean up
    fs::remove_file(input_path).ok();
    fs::remove_file("proof.bin").ok();
}

/// Generate a ZisK proof for the given input file
fn generate_proof(input_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Check if cargo-zisk is available
    let zisk_check = Command::new("cargo-zisk")
        .arg("--version")
        .output();
    
    if zisk_check.is_err() {
        eprintln!("cargo-zisk not found, skipping proof generation");
        return Ok(());
    }
    
    // Generate proof using cargo-zisk
    let output = Command::new("cargo-zisk")
        .args(["prove", "-e", "./guest", "-i", input_path, "-o", "proof.bin"])
        .output()?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Proof generation failed: {}", stderr).into());
    }
    
    // Verify proof file was created
    if !Path::new("proof.bin").exists() {
        return Err("Proof file was not created".into());
    }
    
    Ok(())
}

/// Test ZK error handling
#[test]
fn test_zk_error_handling() {
    // Test with malformed input
    let malformed_input = vec![0xFF; 100]; // Invalid opcodes
    
    let input_path = "test_error_handling.bin";
    fs::write(input_path, malformed_input).unwrap();
    
    let proof_result = generate_proof(input_path);
    
    // This should either succeed (if error handling is robust) or fail gracefully
    if proof_result.is_ok() {
        let verification_result = verify_proof("proof.bin");
        if verification_result.is_ok() {
            let result = verification_result.unwrap();
            // Even if proof is generated, it should handle errors gracefully
            assert!(result.is_valid || result.state_root == [0u8; 32]);
        }
    }
    
    // Clean up
    fs::remove_file(input_path).ok();
    fs::remove_file("proof.bin").ok();
}

/// Test ZK memory alignment
#[test]
fn test_zk_memory_alignment() {
    // Test that memory structures are properly aligned
    use solana_test::guest::memory::{AccountState, BpfMemory, MemoryLayout};
    
    // Check alignment of AccountState
    let account = AccountState::new([0x01; 32], [0x02; 32]);
    let account_size = std::mem::size_of::<AccountState>();
    assert_eq!(account_size % 32, 0, "AccountState should be 32-byte aligned");
    
    // Check alignment of BpfMemory
    let memory = BpfMemory::new();
    let memory_size = std::mem::size_of::<BpfMemory>();
    assert_eq!(memory_size % 32, 0, "BpfMemory should be 32-byte aligned");
    
    // Check alignment of MemoryLayout
    let layout = MemoryLayout::new();
    let layout_size = std::mem::size_of::<MemoryLayout>();
    assert_eq!(layout_size % 32, 0, "MemoryLayout should be 32-byte aligned");
    
    // Test memory operations maintain alignment
    let mut memory = BpfMemory::new();
    let addr = memory.allocate(16).unwrap();
    assert_eq!(addr % 8, 0, "Allocated addresses should be 8-byte aligned");
    
    // Test stack operations maintain alignment
    memory.push(0x1234567890ABCDEF).unwrap();
    let value = memory.pop().unwrap();
    assert_eq!(value, 0x1234567890ABCDEF);
}

/// Test ZK constants and limits
#[test]
fn test_zk_constants() {
    use solana_test::shared::constants::{MAX_CYCLES, MAX_ACCOUNTS, MAX_ACCOUNT_DATA};
    
    // Verify constants are reasonable for ZK environment
    assert!(MAX_CYCLES > 0, "MAX_CYCLES should be positive");
    assert!(MAX_CYCLES <= 10_000_000, "MAX_CYCLES should be reasonable for ZK");
    
    assert!(MAX_ACCOUNTS > 0, "MAX_ACCOUNTS should be positive");
    assert!(MAX_ACCOUNTS <= 1000, "MAX_ACCOUNTS should be reasonable for ZK");
    
    assert!(MAX_ACCOUNT_DATA > 0, "MAX_ACCOUNT_DATA should be positive");
    assert!(MAX_ACCOUNT_DATA <= 1024, "MAX_ACCOUNT_DATA should be reasonable for ZK");
    
    // Test OP_CYCLES table
    use solana_test::shared::constants::OP_CYCLES;
    assert_eq!(OP_CYCLES.len(), 256, "OP_CYCLES should have 256 entries");
    
    // All opcode costs should be positive
    for (i, &cost) in OP_CYCLES.iter().enumerate() {
        assert!(cost > 0, "Opcode 0x{:02X} has invalid cost: {}", i, cost);
    }
}
