//! Fixed BPF Test Programs with Correct Memory and Opcodes
//! 
//! This fixes the two major issues:
//! 1. Correct heap memory addresses (0x100000000 instead of 0x2a)
//! 2. Valid BPF opcodes according to actual BPF specification

use anyhow::Result;

// ================================================================
// CORRECTED BPF OPCODE DEFINITIONS
// ================================================================

// These are the ACTUAL BPF opcodes according to Linux BPF spec
// Our previous implementation had some wrong opcodes

pub const BPF_LD_IMM64: u8 = 0x18;     // Load 64-bit immediate
pub const BPF_LDX_W: u8 = 0x61;        // dst = *(u32 *)(src + offset)
pub const BPF_LDX_H: u8 = 0x69;        // dst = *(u16 *)(src + offset)  
pub const BPF_LDX_B: u8 = 0x71;        // dst = *(u8 *)(src + offset)
pub const BPF_LDX_DW: u8 = 0x79;       // dst = *(u64 *)(src + offset)

pub const BPF_STX_W: u8 = 0x63;        // *(u32 *)(dst + offset) = src
pub const BPF_STX_H: u8 = 0x6B;        // *(u16 *)(dst + offset) = src
pub const BPF_STX_B: u8 = 0x73;        // *(u8 *)(dst + offset) = src
pub const BPF_STX_DW: u8 = 0x7B;       // *(u64 *)(dst + offset) = src

pub const BPF_ST_W: u8 = 0x62;         // *(u32 *)(dst + offset) = imm
pub const BPF_ST_H: u8 = 0x6A;         // *(u16 *)(dst + offset) = imm
pub const BPF_ST_B: u8 = 0x72;         // *(u8 *)(dst + offset) = imm
pub const BPF_ST_DW: u8 = 0x7A;        // *(u64 *)(dst + offset) = imm

pub const BPF_ALU64_ADD_IMM: u8 = 0x07; // dst += imm
pub const BPF_ALU64_ADD_REG: u8 = 0x0F; // dst += src
pub const BPF_ALU64_SUB_IMM: u8 = 0x17; // dst -= imm
pub const BPF_ALU64_SUB_REG: u8 = 0x1F; // dst -= src
pub const BPF_ALU64_MUL_IMM: u8 = 0x27; // dst *= imm
pub const BPF_ALU64_MUL_REG: u8 = 0x2F; // dst *= src
pub const BPF_ALU64_DIV_IMM: u8 = 0x37; // dst /= imm
pub const BPF_ALU64_DIV_REG: u8 = 0x3F; // dst /= src
pub const BPF_ALU64_MOV_IMM: u8 = 0xB7; // dst = imm
pub const BPF_ALU64_MOV_REG: u8 = 0xBF; // dst = src

pub const BPF_JMP_JA: u8 = 0x05;        // pc += offset
pub const BPF_JMP_JEQ_IMM: u8 = 0x15;   // if dst == imm goto pc+offset
pub const BPF_JMP_JEQ_REG: u8 = 0x1D;   // if dst == src goto pc+offset
pub const BPF_JMP_JNE_IMM: u8 = 0x55;   // if dst != imm goto pc+offset
pub const BPF_JMP_JNE_REG: u8 = 0x5D;   // if dst != src goto pc+offset
pub const BPF_JMP_CALL: u8 = 0x85;      // call function
pub const BPF_JMP_EXIT: u8 = 0x95;      // exit program

// ================================================================
// MEMORY LAYOUT CONSTANTS (CORRECTED)
// ================================================================

pub const HEAP_START: u64 = 0x100000000;     // 4GB - correct heap start
pub const STACK_START: u64 = 0x200000000;    // 8GB - stack start
pub const ACCOUNT_START: u64 = 0x300000000;  // 12GB - account data start

// ================================================================
// CORRECTED TEST PROGRAMS
// ================================================================

/// Create a corrected arithmetic test program
pub fn create_corrected_arithmetic_program() -> Result<Vec<u8>> {
    let mut program = Vec::new();
    
    // Test arithmetic operations with CORRECT opcodes
    program.extend_from_slice(&[
        // MOV r1, 10
        BPF_ALU64_MOV_IMM, 0x01, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00,
        
        // MOV r2, 32  
        BPF_ALU64_MOV_IMM, 0x02, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00,
        
        // ADD r1, r2 (r1 = r1 + r2 = 42)
        BPF_ALU64_ADD_REG, 0x21, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        
        // MUL r1, 2 (r1 = r1 * 2 = 84)
        BPF_ALU64_MUL_IMM, 0x01, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
        
        // MOV r0, r1 (return value = 84)
        BPF_ALU64_MOV_REG, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        
        // EXIT
        BPF_JMP_EXIT, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ]);
    
    println!("✅ Created arithmetic program: {} bytes", program.len());
    Ok(program)
}

/// Create a corrected memory operations test program with PROPER heap addresses
pub fn create_corrected_memory_program() -> Result<Vec<u8>> {
    let mut program = Vec::new();
    
    // Test memory operations with CORRECT heap address
    program.extend_from_slice(&[
        // Load heap address into r3 (0x100000000 = 4GB)
        // This is a 64-bit immediate load (wide instruction)
        BPF_LD_IMM64, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // First part
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,         // Second part (high 32 bits)
        
        // MOV r1, 0x12345678 (test value)
        BPF_ALU64_MOV_IMM, 0x01, 0x00, 0x00, 0x78, 0x56, 0x34, 0x12,
        
        // Store r1 at heap address: STX [r3 + 0], r1
        BPF_STX_DW, 0x13, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        
        // Load back from heap: LDX r2, [r3 + 0]
        BPF_LDX_DW, 0x23, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        
        // Compare loaded value with original: JNE r2, r1, +3
        BPF_JMP_JNE_REG, 0x12, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00,
        
        // Success: MOV r0, 1
        BPF_ALU64_MOV_IMM, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        
        // Jump to exit: JA +2
        BPF_JMP_JA, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00,
        
        // Failure: MOV r0, 0
        BPF_ALU64_MOV_IMM, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        
        // EXIT
        BPF_JMP_EXIT, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ]);
    
    println!("✅ Created memory program with heap at 0x{:x}: {} bytes", HEAP_START, program.len());
    Ok(program)
}

/// Create a corrected syscall test program
pub fn create_corrected_syscall_program() -> Result<Vec<u8>> {
    let mut program = Vec::new();
    
    // Test Solana syscalls with correct opcodes
    program.extend_from_slice(&[
        // Set up arguments for sol_log syscall
        // MOV r1, heap_addr (message address)
        BPF_LD_IMM64, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // First part
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,         // Second part (heap)
        
        // MOV r2, 12 (message length)
        BPF_ALU64_MOV_IMM, 0x02, 0x00, 0x00, 0x0C, 0x00, 0x00, 0x00,
        
        // Call sol_log syscall (syscall number as immediate)
        BPF_JMP_CALL, 0x00, 0x00, 0x00, 0xF0, 0xC5, 0x6F, 0x7C, // sol_log syscall ID
        
        // Test sol_log_64 syscall
        // MOV r1, 42
        BPF_ALU64_MOV_IMM, 0x01, 0x00, 0x00, 0x2A, 0x00, 0x00, 0x00,
        
        // MOV r2, 100
        BPF_ALU64_MOV_IMM, 0x02, 0x00, 0x00, 0x64, 0x00, 0x00, 0x00,
        
        // Call sol_log_64 syscall
        BPF_JMP_CALL, 0x00, 0x00, 0x00, 0x2C, 0x20, 0x6B, 0x7B, // sol_log_64 syscall ID
        
        // Set return data
        // MOV r1, heap_addr + 64 (return data address)
        BPF_LD_IMM64, 0x01, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, // First part (offset 64)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,         // Second part (heap)
        
        // MOV r2, 8 (return data length)
        BPF_ALU64_MOV_IMM, 0x02, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00,
        
        // Call sol_set_return_data syscall
        BPF_JMP_CALL, 0x00, 0x00, 0x00, 0xA3, 0x38, 0x2A, 0x26, // sol_set_return_data syscall ID
        
        // Exit successfully: MOV r0, 0
        BPF_ALU64_MOV_IMM, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        
        // EXIT
        BPF_JMP_EXIT, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ]);
    
    println!("✅ Created syscall program: {} bytes", program.len());
    Ok(program)
}

/// Create a comprehensive test program that exercises multiple features
pub fn create_comprehensive_test_program() -> Result<Vec<u8>> {
    let mut program = Vec::new();
    
    // Comprehensive test: arithmetic + memory + conditional jumps
    program.extend_from_slice(&[
        // Phase 1: Arithmetic
        // MOV r1, 10
        BPF_ALU64_MOV_IMM, 0x01, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00,
        
        // ADD r1, 32 (r1 = 42)
        BPF_ALU64_ADD_IMM, 0x01, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00,
        
        // Phase 2: Memory operations with correct heap address
        // Load heap address: LD_IMM64 r2, 0x100000000
        BPF_LD_IMM64, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // First part
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,         // Second part (4GB)
        
        // Store r1 to heap: STX [r2], r1
        BPF_STX_DW, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        
        // Load from heap: LDX r3, [r2]
        BPF_LDX_DW, 0x32, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        
        // Phase 3: Conditional logic
        // Compare: JEQ r3, 42, +3 (if r3 == 42, jump to success)
        BPF_JMP_JEQ_IMM, 0x03, 0x03, 0x00, 0x2A, 0x00, 0x00, 0x00,
        
        // Failure path: MOV r0, 0
        BPF_ALU64_MOV_IMM, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        
        // Jump to exit: JA +2
        BPF_JMP_JA, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00,
        
        // Success path: MOV r0, 1
        BPF_ALU64_MOV_IMM, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        
        // EXIT
        BPF_JMP_EXIT, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ]);
    
    println!("✅ Created comprehensive test program: {} bytes", program.len());
    Ok(program)
}

/// Create a minimal working program for quick validation
pub fn create_minimal_test_program() -> Result<Vec<u8>> {
    let mut program = Vec::new();
    
    // Minimal program: just set return value and exit
    program.extend_from_slice(&[
        // MOV r0, 42 (return value)
        BPF_ALU64_MOV_IMM, 0x00, 0x00, 0x00, 0x2A, 0x00, 0x00, 0x00,
        
        // EXIT
        BPF_JMP_EXIT, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ]);
    
    println!("✅ Created minimal test program: {} bytes", program.len());
    Ok(program)
}

// ================================================================
// UPDATED BUILD.RS INTEGRATION
// ================================================================

/// Updated function for build.rs that creates corrected test programs
pub fn create_corrected_test_bpf_program() -> Result<Vec<u8>> {
    // Choose which test program to use based on environment variable
    let program_type = std::env::var("BPF_TEST_TYPE").unwrap_or_else(|_| "comprehensive".to_string());
    
    match program_type.as_str() {
        "arithmetic" => create_corrected_arithmetic_program(),
        "memory" => create_corrected_memory_program(),
        "syscall" => create_corrected_syscall_program(),
        "minimal" => create_minimal_test_program(),
        "comprehensive" | _ => create_comprehensive_test_program(),
    }
}

/// Validate that a BPF program has correct instruction structure
pub fn validate_bpf_program(program: &[u8]) -> Result<()> {
    if program.len() % 8 != 0 {
        return Err(anyhow::anyhow!("BPF program length must be multiple of 8 bytes, got {}", program.len()));
    }
    
    if program.is_empty() {
        return Err(anyhow::anyhow!("BPF program cannot be empty"));
    }
    
    // Check that the program ends with EXIT instruction
    let last_instruction = &program[program.len() - 8..];
    if last_instruction[0] != BPF_JMP_EXIT {
        return Err(anyhow::anyhow!("BPF program must end with EXIT instruction (0x95), found 0x{:02x}", last_instruction[0]));
    }
    
    // Validate each instruction
    let mut offset = 0;
    while offset < program.len() {
        let opcode = program[offset];
        
        // Check for valid opcodes
        match opcode {
            BPF_LD_IMM64 => {
                // Wide instruction - check we have enough bytes
                if offset + 16 > program.len() {
                    return Err(anyhow::anyhow!("Wide instruction at offset {} extends beyond program", offset));
                }
                offset += 16; // Wide instruction is 16 bytes
            },
            BPF_ALU64_MOV_IMM | BPF_ALU64_MOV_REG | BPF_ALU64_ADD_IMM | BPF_ALU64_ADD_REG |
            BPF_ALU64_SUB_IMM | BPF_ALU64_SUB_REG | BPF_ALU64_MUL_IMM | BPF_ALU64_MUL_REG |
            BPF_ALU64_DIV_IMM | BPF_ALU64_DIV_REG | BPF_LDX_W | BPF_LDX_H | BPF_LDX_B | BPF_LDX_DW |
            BPF_STX_W | BPF_STX_H | BPF_STX_B | BPF_STX_DW | BPF_ST_W | BPF_ST_H | BPF_ST_B | BPF_ST_DW |
            BPF_JMP_JA | BPF_JMP_JEQ_IMM | BPF_JMP_JEQ_REG | BPF_JMP_JNE_IMM | BPF_JMP_JNE_REG |
            BPF_JMP_CALL | BPF_JMP_EXIT => {
                offset += 8; // Standard instruction is 8 bytes
            },
            _ => {
                return Err(anyhow::anyhow!("Unknown BPF opcode 0x{:02x} at offset {}", opcode, offset));
            }
        }
    }
    
    println!("✅ BPF program validation passed: {} instructions", program.len() / 8);
    Ok(())
}

// ================================================================
// MEMORY LAYOUT TESTING
// ================================================================

/// Test that memory addresses are within valid ranges
pub fn test_memory_layout() -> Result<()> {
    println!("🧪 Testing memory layout...");
    
    // Test heap address calculation
    let heap_base = HEAP_START;
    let heap_offset_1kb = heap_base + 1024;
    let heap_offset_64kb = heap_base + 64 * 1024;
    
    assert!(heap_base >= 0x100000000, "Heap should start at 4GB boundary");
    assert!(heap_offset_1kb > heap_base, "Heap offset calculation should work");
    assert!(heap_offset_64kb < 0x200000000, "Heap should not overlap with stack");
    
    println!("✅ Heap layout: 0x{:x} - 0x{:x}", heap_base, heap_base + 64 * 1024);
    
    // Test stack address
    let stack_base = STACK_START;
    assert!(stack_base >= 0x200000000, "Stack should start at 8GB boundary");
    assert!(stack_base > heap_base, "Stack should be after heap");
    
    println!("✅ Stack layout: 0x{:x} - 0x{:x}", stack_base, stack_base + 8 * 1024);
    
    // Test account data address
    let account_base = ACCOUNT_START;
    assert!(account_base >= 0x300000000, "Account data should start at 12GB boundary");
    assert!(account_base > stack_base, "Account data should be after stack");
    
    println!("✅ Account layout: 0x{:x}+", account_base);
    println!("✅ Memory layout test passed!");
    
    Ok(())
}

// ================================================================
// USAGE EXAMPLES
// ================================================================

/// Example usage for different test scenarios
pub fn create_test_programs_for_scenarios() -> Result<()> {
    println!("🎯 Creating test programs for different scenarios...");
    
    // Arithmetic test
    let arithmetic = create_corrected_arithmetic_program()?;
    validate_bpf_program(&arithmetic)?;
    std::fs::write("build/test_arithmetic.bpf", arithmetic)?;
    
    // Memory test with correct addresses
    let memory = create_corrected_memory_program()?;
    validate_bpf_program(&memory)?;
    std::fs::write("build/test_memory.bpf", memory)?;
    
    // Syscall test
    let syscall = create_corrected_syscall_program()?;
    validate_bpf_program(&syscall)?;
    std::fs::write("build/test_syscall.bpf", syscall)?;
    
    // Comprehensive test
    let comprehensive = create_comprehensive_test_program()?;
    validate_bpf_program(&comprehensive)?;
    std::fs::write("build/test_comprehensive.bpf", comprehensive)?;
    
    // Minimal test
    let minimal = create_minimal_test_program()?;
    validate_bpf_program(&minimal)?;
    std::fs::write("build/test_minimal.bpf", minimal)?;
    
    println!("✅ All test programs created and validated!");
    println!("📁 Test files written to build/ directory");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_corrected_arithmetic_program() {
        let program = create_corrected_arithmetic_program().unwrap();
        validate_bpf_program(&program).unwrap();
        
        // Check that it ends with EXIT
        assert_eq!(program[program.len() - 8], BPF_JMP_EXIT);
    }
    
    #[test]
    fn test_corrected_memory_program() {
        let program = create_corrected_memory_program().unwrap();
        validate_bpf_program(&program).unwrap();
        
        // Check that it contains LD_IMM64 for heap address loading
        assert!(program.contains(&BPF_LD_IMM64));
    }
    
    #[test]
    fn test_memory_layout_constants() {
        test_memory_layout().unwrap();
    }
    
    #[test]
    fn test_all_program_types() {
        create_test_programs_for_scenarios().unwrap();
    }
}
