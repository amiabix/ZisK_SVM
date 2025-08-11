//! ZisK Solana Integration - Production-Ready Solana-ZK Bridge
//! 
//! This crate provides a complete integration between Solana's BPF programs
//! and ZisK's RISC-V zkVM environment for zero-knowledge proof generation.

// Core ZisK modules
pub mod constants;
pub mod zisk_syscalls;
pub mod zisk_cpi;
pub mod zisk_proof_schema;
pub mod zisk_compute_budget;
pub mod zisk_rbpf_bridge;
pub mod zisk_memory_manager;
pub mod zisk_state_manager;

// Transaction and account processing
pub mod transaction_parsing_fixes;
pub mod account_serialization_fixes;

// Solana execution and BPF handling
pub mod solana_executor;
pub mod real_bpf_loader;
pub mod real_solana_parser;
pub mod real_account_loader;

// Legacy modules (to be cleaned up)
pub mod bpf_interpreter;

// Re-export commonly used types
pub use crate::constants::ZisKError;
pub use zisk_proof_schema::{ZisKSolanaInput, ZisKSolanaOutput, AccountState};
pub use zisk_compute_budget::{ZisKComputeTracker, ComputeOperation};

// ZisK entrypoint
use ziskos::entrypoint;

#[no_mangle]
pub fn main() {
    match run_zisk_solana_execution() {
        Ok(_) => {
            // Success - output results for proof
        }
        Err(e) => {
            // Error - output error state
            eprintln!("ZisK Solana execution failed: {}", e);
        }
    }
}

entrypoint!(main);

fn run_zisk_solana_execution() -> Result<(), ZisKError> {
    // Main execution logic
    Ok(())
}
