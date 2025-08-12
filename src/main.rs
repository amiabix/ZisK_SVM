use bpf_riscv_transpiler::{BpfTranspiler, TranspilerError};
use std::env;
use std::fs;
use std::path::Path;

fn main() -> Result<(), TranspilerError> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }
    
    match args[1].as_str() {
        "transpile" => {
            if args.len() < 4 {
                eprintln!("Usage: {} transpile <input.bpf> <output.riscv>", args[0]);
                return Ok(());
            }
            
            let input_file = &args[2];
            let output_file = &args[3];
            
            transpile_bpf_to_riscv(input_file, output_file)?;
        }
        
        "execute" => {
            if args.len() < 3 {
                eprintln!("Usage: {} execute <input.bpf>", args[0]);
                return Ok(());
            }
            
            let input_file = &args[2];
            execute_bpf_in_zisk(input_file)?;
        }
        
        "test" => {
            run_tests()?;
        }
        
        "demo" => {
            run_demo()?;
        }
        
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage();
        }
    }
    
    Ok(())
}

fn print_usage() {
    println!("BPF to RISC-V Transpiler for ZisK Integration");
    println!();
    println!("Usage:");
    println!("  {} transpile <input.bpf> <output.riscv>  - Transpile BPF to RISC-V", env::args().next().unwrap());
    println!("  {} execute <input.bpf>                   - Execute BPF in ZisK", env::args().next().unwrap());
    println!("  {} test                                  - Run tests", env::args().next().unwrap());
    println!("  {} demo                                  - Run demonstration", env::args().next().unwrap());
    println!();
    println!("Examples:");
    println!("  {} transpile program.bpf program.riscv", env::args().next().unwrap());
    println!("  {} execute program.bpf", env::args().next().unwrap());
}

fn transpile_bpf_to_riscv(input_file: &str, output_file: &str) -> Result<(), TranspilerError> {
    println!("🔄 Transpiling BPF to RISC-V...");
    println!("   Input:  {}", input_file);
    println!("   Output: {}", output_file);
    
    // Read BPF bytecode
    let bpf_bytecode = fs::read(input_file)
        .map_err(|e| TranspilerError::MemoryError { 
            message: format!("Failed to read input file: {}", e) 
        })?;
    
    println!("   BPF size: {} bytes", bpf_bytecode.len());
    
    // Create transpiler and transpile
    let mut transpiler = BpfTranspiler::new();
    let riscv_code = transpiler.transpile(&bpf_bytecode)?;
    
    println!("   RISC-V size: {} bytes", riscv_code.len());
    
    // Write RISC-V code
    fs::write(output_file, riscv_code)
        .map_err(|e| TranspilerError::MemoryError { 
            message: format!("Failed to write output file: {}", e) 
        })?;
    
    println!("✅ Transpilation completed successfully!");
    Ok(())
}

fn execute_bpf_in_zisk(input_file: &str) -> Result<(), TranspilerError> {
    println!("🚀 Executing BPF in ZisK...");
    println!("   Input: {}", input_file);
    
    // Read BPF bytecode
    let bpf_bytecode = fs::read(input_file)
        .map_err(|e| TranspilerError::MemoryError { 
            message: format!("Failed to read input file: {}", e) 
        })?;
    
    println!("   BPF size: {} bytes", bpf_bytecode.len());
    
    // Create transpiler and execute
    let mut transpiler = BpfTranspiler::new();
    let result = transpiler.execute_in_zisk(&bpf_bytecode)?;
    
    println!("✅ Execution completed successfully!");
    println!("   Exit code: {}", result.exit_code);
    println!("   Instructions executed: {}", result.instructions_executed);
    println!("   Execution time: {:?}", result.execution_time);
    println!("   Register R0: {}", result.registers[0]);
    println!("   Register R1: {}", result.registers[1]);
    
    Ok(())
}

fn run_tests() -> Result<(), TranspilerError> {
    println!("🧪 Running tests...");
    
    // Test BPF parsing
    println!("   Testing BPF parser...");
    let test_bpf = vec![
        0xb7, 0x00, 0x00, 0x00, 0x2a, 0x00, 0x00, 0x00, // MOV64_IMM R0, 42
        0x07, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x00, // ADD64_IMM R0, 10
        0x95, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // EXIT
    ];
    
    let mut transpiler = BpfTranspiler::new();
    let result = transpiler.transpile(&test_bpf)?;
    
    println!("   Test BPF (24 bytes) -> RISC-V ({} bytes)", result.len());
    println!("✅ Tests passed!");
    
    Ok(())
}

fn run_demo() -> Result<(), TranspilerError> {
    println!("🎯 Running demonstration...");
    
    // Create a simple BPF program: MOV64_IMM R0, 42; ADD64_IMM R0, 10; EXIT
    let demo_bpf = vec![
        0xb7, 0x00, 0x00, 0x00, 0x2a, 0x00, 0x00, 0x00, // MOV64_IMM R0, 42
        0x07, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x00, // ADD64_IMM R0, 10
        0x95, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // EXIT
    ];
    
    println!("   Demo BPF program:");
    println!("     MOV64_IMM R0, 42");
    println!("     ADD64_IMM R0, 10");
    println!("     EXIT");
    println!("   Expected result: R0 = 52");
    
    // Transpile to RISC-V
    let mut transpiler = BpfTranspiler::new();
    let riscv_code = transpiler.transpile(&demo_bpf)?;
    
    println!("   Transpiled to {} bytes of RISC-V code", riscv_code.len());
    
    // Execute in ZisK
    let result = transpiler.execute_in_zisk(&demo_bpf)?;
    
    println!("   Execution result:");
    println!("     Exit code: {}", result.exit_code);
    println!("     Instructions executed: {}", result.instructions_executed);
    println!("     Register R0: {}", result.registers[0]);
    println!("     Execution time: {:?}", result.execution_time);
    
    if result.registers[0] == 52 {
        println!("✅ Demo completed successfully! R0 = 52 as expected.");
    } else {
        println!("❌ Demo failed! Expected R0 = 52, got R0 = {}", result.registers[0]);
    }
    
    Ok(())
}
