// ZisK-compatible getrandom implementation
use getrandom::register_custom_getrandom;

// Custom random number generator for ZisK zkVM
fn custom_getrandom(dest: &mut [u8]) -> Result<(), getrandom::Error> {
    // For ZisK zkVM, we'll use a simple deterministic "random" generator
    // In production, you'd want to use ZisK's built-in randomness or external entropy
    for (i, byte) in dest.iter_mut().enumerate() {
        *byte = ((i * 7 + 13) % 256) as u8;
    }
    Ok(())
}

// Register our custom implementation
register_custom_getrandom!(custom_getrandom);

// Export the main modules for ZisK integration
pub mod constants;
pub mod bpf_interpreter;
pub mod solana_executor;
