use std::fs;
use std::path::Path;

// ZisK-specific build configurations
const ZISK_MEMORY_LAYOUT: &str = "zisk-memory.x";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=zisk-memory.x");
    
    // ZisK-specific build flags
    if cfg!(target_arch = "riscv64") {
        println!("cargo:rustc-link-arg=-T{}", ZISK_MEMORY_LAYOUT);
        println!("cargo:rustc-link-arg=-Wl,--gc-sections");
        println!("cargo:rustc-link-arg=-Wl,--strip-all");
        println!("cargo:rustc-link-arg=-nostdlib");
        println!("cargo:rustc-link-arg=-static");
    }
    
    // Create output directory for ZK program
    let output_dir = Path::new("build");
    if !output_dir.exists() {
        fs::create_dir_all(output_dir).expect("Failed to create build directory");
    }
    
    // Create ZisK input file with test data
    create_zisk_input_file();
}

fn create_zisk_input_file() {
    let input_data = create_test_input_data();
    let input_path = Path::new("build/input.bin");
    
    // Ensure build directory exists
    if let Some(parent) = input_path.parent() {
        fs::create_dir_all(parent).expect("Failed to create build directory");
    }
    
    // Write input data
    fs::write(input_path, input_data).expect("Failed to write ZisK input file");
    println!("cargo:warning=Created ZisK input file: {:?}", input_path);
}

fn create_test_input_data() -> Vec<u8> {
    // Create test input data for ZisK execution
    // This simulates the input that would be provided to the ZK program
    let mut data = Vec::new();
    
    // Add test transaction data
    data.extend_from_slice(&[0x01, 0x02, 0x03, 0x04]); // Test signature
    data.extend_from_slice(&[0x05, 0x06, 0x07, 0x08]); // Test account key
    data.extend_from_slice(&[0x09, 0x0A, 0x0B, 0x0C]); // Test instruction data
    
    // Add test BPF program
    let test_program = vec![0x61, 0x01, 0x02, 0x95, 0x00, 0x00, 0x00, 0x00]; // LdReg + Exit
    data.extend_from_slice(&test_program);
    
    data
}
