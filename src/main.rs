use bpf_riscv_transpiler::BpfTranspiler;

fn main() {
    println!("🚀 BPF to RISC-V Transpiler with Real ZisK Integration");
    println!("=====================================================\n");

    // Create a simple BPF program: MOV64_IMM R0, 42; EXIT
    let bpf_program = vec![
        0xb7, 0x00, 0x00, 0x00, 0x2a, 0x00, 0x00, 0x00, // MOV64_IMM R0, 42
        0x95, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // EXIT
    ];

    println!("📋 BPF Program:");
    println!("  MOV64_IMM R0, 42");
    println!("  EXIT");
    println!("  Size: {} bytes\n", bpf_program.len());

    // Create transpiler
    let mut transpiler = BpfTranspiler::new();

    // Transpile to RISC-V assembly
    match transpiler.transpile_to_assembly(&bpf_program) {
        Ok(assembly) => {
            println!("🔧 Generated RISC-V Assembly:");
            println!("{}", assembly);
            println!();

            // Execute in ZisK
            println!("⚡ Executing in ZisK...");
            match transpiler.execute_in_zisk(&bpf_program) {
                Ok(result) => {
                    println!("✅ Execution successful!");
                    println!("  Exit code: {}", result.exit_code);
                    println!("  Instructions executed: {}", result.instructions_executed);
                    println!("  Execution time: {:?}", result.execution_time);
                }
                Err(e) => {
                    println!("❌ ZisK execution failed: {}", e);
                    println!("  This is expected if ZisK is not fully configured");
                }
            }
        }
        Err(e) => {
            println!("❌ Transpilation failed: {}", e);
        }
    }

    println!("\n🎯 Next Steps for Full ZisK Integration:");
    println!("  1. Install ZisK toolchain: curl https://raw.githubusercontent.com/0xPolygonHermez/zisk/main/ziskup/install.sh | bash");
    println!("  2. Navigate to generated project: cd zisk_bpf_project");
    println!("  3. Build RISC-V program: cargo-zisk build --release");
    println!("  4. Execute in ZisK emulator: ziskemu -e target/riscv64ima-zisk-zkvm-elf/release/bpf_program");
    println!("  5. Generate proof: cargo-zisk rom-setup -e target/riscv64ima-zisk-zkvm-elf/release/bpf_program");
    println!("  6. Generate final proof: cargo-zisk prove -e target/riscv64ima-zisk-zkvm-elf/release/bpf_program -o proof -a -y");
    println!("  7. Verify proof: cargo-zisk verify -p ./proof/vadcop_final_proof.bin");
    println!("\n📚 See README.md for detailed instructions and examples.");
}
