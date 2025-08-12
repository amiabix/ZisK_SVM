// =================================================================
// ZISK-SVM: MINIMAL LIBRARY IMPLEMENTATION
// =================================================================
//
// This is the library version for ZisK integration
// All Solana dependencies have been removed

use anyhow::Result;

// Export our core modules
pub mod complete_bpf_interpreter;
pub mod bpf_zisk_integration;
pub mod bpf_test_utils;

// Re-export key types
pub use complete_bpf_interpreter::{BpfExecutionContext, BpfInstruction, BpfRegisters, BpfMemory, RealBpfInterpreter, ExecutionResult};
pub use bpf_zisk_integration::{ZiskBpfExecutor, ZiskExecutionConfig, execute_solana_transaction_in_zisk, SolanaAccount, ZiskTransactionContext};

/// ZisK-SVM library entry point
pub fn zisk_svm_main() -> Result<()> {
    println!("ZisK-SVM: BPF Interpreter Library");
    Ok(())
}

/// Test function for the library
#[cfg(test)]
mod tests {
    use super::*;
    use complete_bpf_interpreter::BpfExecutionContext; // Import for test

    #[test]
    fn test_zisk_svm_main() {
        assert!(zisk_svm_main().is_ok());
    }

    #[test]
    fn test_bpf_interpreter_import() {
        let _context = BpfExecutionContext::new(b"test".to_vec(), 1_000_000); // Pass program data and limit
        println!("BPF interpreter imported successfully");
    }

    #[test]
    fn test_proof_generator_import() {
        let _generator = ZiskBpfExecutor::new(ZiskExecutionConfig::default());
        println!("Proof generator imported successfully");
    }
}
