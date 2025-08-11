// Custom random number generator implementation
fn custom_getrandom(dest: &mut [u8]) -> Result<(), Box<dyn std::error::Error>> {
    // Simple deterministic "random" generator
    for (i, byte) in dest.iter_mut().enumerate() {
        *byte = ((i * 7 + 13) % 256) as u8;
    }
    Ok(())
}

// Export the main modules
pub mod constants;
pub mod bpf_interpreter;
pub mod solana_executor;
pub mod real_bpf_loader;
pub mod real_solana_parser;
pub mod real_account_loader;
pub mod zisk_syscalls;
pub mod zisk_cpi;
pub mod zisk_proof_schema;
pub mod zisk_memory_manager;
pub mod zisk_state_manager;
pub mod zisk_rbpf_bridge;
pub mod zisk_compute_budget;
