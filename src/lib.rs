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
