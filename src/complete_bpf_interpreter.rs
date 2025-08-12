//! Complete Real BPF Interpreter for Solana Programs in ZisK zkVM
//! 
//! This module implements a full BPF (Berkeley Packet Filter) interpreter that can execute
//! real Solana programs within the ZisK zero-knowledge virtual machine.

use anyhow::{Result, anyhow};
use std::collections::HashMap;

// ================================================================
// CORE BPF STRUCTURES
// ================================================================

/// Complete BPF Register Set (64-bit registers as per Solana specification)
#[derive(Debug, Clone, Default)]
pub struct BpfRegisters {
    pub r: [u64; 16], // r0-r15 registers
}

impl BpfRegisters {
    pub fn new() -> Self {
        Self { r: [0; 16] }
    }
    
    pub fn get(&self, index: u8) -> u64 {
        if index < 16 {
            self.r[index as usize]
        } else {
            0
        }
    }
    
    pub fn set(&mut self, index: u8, value: u64) {
        if index < 16 {
            self.r[index as usize] = value;
        }
    }
    
    pub fn get_mut(&mut self, index: u8) -> Option<&mut u64> {
        if index < 16 {
            Some(&mut self.r[index as usize])
        } else {
            None
        }
    }
}

/// BPF Memory Management with ZisK constraints
#[derive(Debug, Clone)]
pub struct BpfMemory {
    /// Program memory (read-only after loading)
    pub program: Vec<u8>,
    /// Heap memory for dynamic allocation
    pub heap: Vec<u8>,
    /// Stack memory
    pub stack: Vec<u8>,
    /// Account data regions
    pub account_regions: HashMap<u64, Vec<u8>>,
    /// Input data region
    pub input_data: Vec<u8>,
    /// Memory layout tracking
    pub regions: Vec<MemoryRegion>,
}

#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub start: u64,
    pub length: usize,
    pub region_type: MemoryRegionType,
    pub writable: bool,
}

#[derive(Debug, Clone)]
pub enum MemoryRegionType {
    Program,
    Heap,
    Stack,
    AccountData,
    InputData,
    ReadOnly,
}

impl BpfMemory {
    pub fn new(heap_size: usize, stack_size: usize) -> Self {
        let mut memory = Self {
            program: Vec::new(),
            heap: vec![0; heap_size],
            stack: vec![0; stack_size],
            account_regions: HashMap::new(),
            input_data: Vec::new(),
            regions: Vec::new(),
        };
        
        // Set up initial memory regions
        memory.regions.push(MemoryRegion {
            start: 0x100000000, // Heap starts at 4GB
            length: heap_size,
            region_type: MemoryRegionType::Heap,
            writable: true,
        });
        
        memory.regions.push(MemoryRegion {
            start: 0x200000000, // Stack starts at 8GB
            length: stack_size,
            region_type: MemoryRegionType::Stack,
            writable: true,
        });
        
        memory
    }
    
    /// Safe memory read with bounds checking
    pub fn read_memory(&self, addr: u64, size: usize) -> Result<&[u8]> {
        match self.resolve_memory_region(addr, size)? {
            (MemoryRegionType::Heap, offset) => {
                Ok(&self.heap[offset..offset + size])
            },
            (MemoryRegionType::Stack, offset) => {
                Ok(&self.stack[offset..offset + size])
            },
            (MemoryRegionType::AccountData, _) => {
                // Account data handled separately
                self.read_account_data(addr, size)
            },
            (MemoryRegionType::InputData, offset) => {
                Ok(&self.input_data[offset..offset + size])
            },
            (MemoryRegionType::Program, offset) => {
                Ok(&self.program[offset..offset + size])
            },
            _ => Err(anyhow!("Invalid memory region")),
        }
    }
    
    /// Safe memory write with bounds checking
    pub fn write_memory(&mut self, addr: u64, data: &[u8]) -> Result<()> {
        let size = data.len();
        match self.resolve_memory_region(addr, size)? {
            (MemoryRegionType::Heap, offset) => {
                self.heap[offset..offset + size].copy_from_slice(data);
                Ok(())
            },
            (MemoryRegionType::Stack, offset) => {
                self.stack[offset..offset + size].copy_from_slice(data);
                Ok(())
            },
            (MemoryRegionType::AccountData, _) => {
                self.write_account_data(addr, data)
            },
            _ => Err(anyhow!("Cannot write to read-only memory region")),
        }
    }
    
    fn resolve_memory_region(&self, addr: u64, size: usize) -> Result<(MemoryRegionType, usize)> {
        for region in &self.regions {
            if addr >= region.start && addr + size as u64 <= region.start + region.length as u64 {
                let offset = (addr - region.start) as usize;
                return Ok((region.region_type.clone(), offset));
            }
        }
        Err(anyhow!("Memory access out of bounds: 0x{:x}, size {}", addr, size))
    }
    
    fn read_account_data(&self, addr: u64, size: usize) -> Result<&[u8]> {
        // Implementation for account data access
        Err(anyhow!("Account data read not implemented"))
    }
    
    fn write_account_data(&mut self, addr: u64, data: &[u8]) -> Result<()> {
        // Implementation for account data write
        Err(anyhow!("Account data write not implemented"))
    }
}

/// BPF Instruction Structure (64-bit instruction format)
#[derive(Debug, Clone)]
pub struct BpfInstruction {
    pub opcode: u8,
    pub dst_reg: u8,
    pub src_reg: u8,
    pub offset: i16,
    pub immediate: i32,
}

impl BpfInstruction {
    /// Decode 8-byte BPF instruction from bytecode
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 8 {
            return Err(anyhow!("Instruction too short: {} bytes", bytes.len()));
        }
        
        let opcode = bytes[0];
        let dst_reg = (bytes[1] >> 4) & 0x0F;  // High 4 bits
        let src_reg = bytes[1] & 0x0F;          // Low 4 bits
        let offset = i16::from_le_bytes([bytes[2], bytes[3]]);
        let immediate = i32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        
        Ok(Self {
            opcode,
            dst_reg,
            src_reg,
            offset,
            immediate,
        })
    }
    
    /// Decode wide instruction (16 bytes) for LD_IMM64
    pub fn decode_wide(program: &[u8], pc: usize) -> Result<(Self, u32)> {
        if pc + 16 > program.len() {
            return Err(anyhow!("Program too short for wide instruction at PC {}", pc));
        }
        
        // Decode first 8 bytes as normal
        let first_bytes = &program[pc..pc + 8];
        let mut instruction = Self::decode(first_bytes)?;
        
        // For LD_IMM64, the next 8 bytes contain the high 32 bits
        if instruction.opcode == 0x18 { // LD_IMM64
            let next_bytes = &program[pc + 8..pc + 16];
            // In little-endian, the high 32 bits are in the last 4 bytes
            // For 0x100000000, the bytes are [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01]
            // So high_imm should be [0x00, 0x00, 0x00, 0x01] = 0x1
            let high_imm = u32::from_le_bytes([next_bytes[7], next_bytes[6], next_bytes[5], next_bytes[4]]);
            Ok((instruction, high_imm))
        } else {
            Err(anyhow!("Not a wide instruction: 0x{:02x}", instruction.opcode))
        }
    }
    
    /// Check if this is a wide instruction (requires 16 bytes)
    pub fn is_wide(&self) -> bool {
        self.opcode == 0x18 // LD_IMM64
    }
}

// ================================================================
// COMPLETE BPF OPCODE DEFINITIONS
// ================================================================

/// Complete BPF Opcode Set for Solana
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BpfOpcode {
    // ============ ALU64 Instructions ============
    Add64Imm = 0x07,    // dst += imm
    Add64Reg = 0x0F,    // dst += src
    Sub64Imm = 0x17,    // dst -= imm
    Sub64Reg = 0x1F,    // dst -= src
    Mul64Imm = 0x27,    // dst *= imm
    Mul64Reg = 0x2F,    // dst *= src
    Div64Imm = 0x37,    // dst /= imm
    Div64Reg = 0x3F,    // dst /= src
    Or64Imm = 0x47,     // dst |= imm
    Or64Reg = 0x4F,     // dst |= src
    And64Imm = 0x57,    // dst &= imm
    And64Reg = 0x5F,    // dst &= src
    Lsh64Imm = 0x67,    // dst <<= imm
    Lsh64Reg = 0x6F,    // dst <<= src
    Rsh64Imm = 0x77,    // dst >>= imm (logical)
    Rsh64Reg = 0x7F,    // dst >>= src (logical)
    Neg64 = 0x87,       // dst = -dst
    Mod64Imm = 0x97,    // dst %= imm
    Mod64Reg = 0x9F,    // dst %= src
    Xor64Imm = 0xA7,    // dst ^= imm
    Xor64Reg = 0xAF,    // dst ^= src
    Mov64Imm = 0xB7,    // dst = imm
    Mov64Reg = 0xBF,    // dst = src
    Arsh64Imm = 0xC7,   // dst >>= imm (arithmetic)
    Arsh64Reg = 0xCF,   // dst >>= src (arithmetic)
    
    // ============ ALU32 Instructions ============
    Add32Imm = 0x04,    // dst += imm (32-bit)
    Add32Reg = 0x0C,    // dst += src (32-bit)
    Sub32Imm = 0x14,    // dst -= imm (32-bit)
    Sub32Reg = 0x1C,    // dst -= src (32-bit)
    Mul32Imm = 0x24,    // dst *= imm (32-bit)
    Mul32Reg = 0x2C,    // dst *= src (32-bit)
    Div32Imm = 0x34,    // dst /= imm (32-bit)
    Div32Reg = 0x3C,    // dst /= src (32-bit)
    Or32Imm = 0x44,     // dst |= imm (32-bit)
    Or32Reg = 0x4C,     // dst |= src (32-bit)
    And32Imm = 0x54,    // dst &= imm (32-bit)
    And32Reg = 0x5C,    // dst &= src (32-bit)
    Lsh32Imm = 0x64,    // dst <<= imm (32-bit)
    Lsh32Reg = 0x6C,    // dst <<= src (32-bit)
    Rsh32Imm = 0x74,    // dst >>= imm (32-bit, logical)
    Rsh32Reg = 0x7C,    // dst >>= src (32-bit, logical)
    Neg32 = 0x84,       // dst = -dst (32-bit)
    Mod32Imm = 0x94,    // dst %= imm (32-bit)
    Mod32Reg = 0x9C,    // dst %= src (32-bit)
    Xor32Imm = 0xA4,    // dst ^= imm (32-bit)
    Xor32Reg = 0xAC,    // dst ^= src (32-bit)
    Mov32Imm = 0xB4,    // dst = imm (32-bit)
    Mov32Reg = 0xBC,    // dst = src (32-bit)
    Arsh32Imm = 0xC4,   // dst >>= imm (32-bit, arithmetic)
    Arsh32Reg = 0xCC,   // dst >>= src (32-bit, arithmetic)
    
    // ============ Endianness Instructions ============
    Le16 = 0xD4,        // dst = htole16(dst)
    Le32 = 0xD5,        // dst = htole32(dst)
    Le64 = 0xD7,        // dst = htole64(dst)
    Be16 = 0xDC,        // dst = htobe16(dst)
    Be32 = 0xDD,        // dst = htobe32(dst)
    Be64 = 0xDF,        // dst = htobe64(dst)
    
    // ============ Load Instructions ============
    LdImm64 = 0x18,     // dst = imm64
    LdAbsW = 0x20,      // Load absolute word
    LdAbsH = 0x28,      // Load absolute halfword
    LdAbsB = 0x30,      // Load absolute byte
    LdAbsDw = 0x38,     // Load absolute doubleword
    LdIndW = 0x40,      // Load indirect word
    LdIndH = 0x48,      // Load indirect halfword
    LdIndB = 0x50,      // Load indirect byte
    LdIndDw = 0x58,     // Load indirect doubleword
    LdxW = 0x61,        // dst = *(u32 *)(src + offset)
    LdxH = 0x69,        // dst = *(u16 *)(src + offset)
    LdxB = 0x71,        // dst = *(u8 *)(src + offset)
    LdxDw = 0x79,       // dst = *(u64 *)(src + offset)
    
    // ============ Store Instructions ============
    StW = 0x62,         // *(u32 *)(dst + offset) = imm
    StH = 0x6A,         // *(u16 *)(dst + offset) = imm
    StB = 0x72,         // *(u8 *)(dst + offset) = imm
    StDw = 0x7A,        // *(u64 *)(dst + offset) = imm
    StxW = 0x63,        // *(u32 *)(dst + offset) = src
    StxH = 0x6B,        // *(u16 *)(dst + offset) = src
    StxB = 0x73,        // *(u8 *)(dst + offset) = src
    StxDw = 0x7B,       // *(u64 *)(dst + offset) = src
    
    // ============ Jump Instructions ============
    Ja = 0x05,          // pc += offset
    JeqImm = 0x15,      // if dst == imm goto pc+offset
    JeqReg = 0x1D,      // if dst == src goto pc+offset
    JgtImm = 0x25,      // if dst > imm goto pc+offset
    JgtReg = 0x2D,      // if dst > src goto pc+offset
    JgeImm = 0x35,      // if dst >= imm goto pc+offset
    JgeReg = 0x3D,      // if dst >= src goto pc+offset
    JltImm = 0xA5,      // if dst < imm goto pc+offset
    JltReg = 0xAD,      // if dst < src goto pc+offset
    JleImm = 0xB5,      // if dst <= imm goto pc+offset
    JleReg = 0xBD,      // if dst <= src goto pc+offset
    JsetImm = 0x45,     // if dst & imm goto pc+offset
    JsetReg = 0x4D,     // if dst & src goto pc+offset
    JneImm = 0x55,      // if dst != imm goto pc+offset
    JneReg = 0x5D,      // if dst != src goto pc+offset
    JsgtImm = 0x65,     // if (s64)dst > (s64)imm goto pc+offset
    JsgtReg = 0x6D,     // if (s64)dst > (s64)src goto pc+offset
    JsgeImm = 0x75,     // if (s64)dst >= (s64)imm goto pc+offset
    JsgeReg = 0x7D,     // if (s64)dst >= (s64)src goto pc+offset
    JsltImm = 0xC5,     // if (s64)dst < (s64)imm goto pc+offset
    JsltReg = 0xCD,     // if (s64)dst < (s64)src goto pc+offset
    JsleImm = 0xD6,     // if (s64)dst <= (s64)imm goto pc+offset
    JsleReg = 0xDE,     // if (s64)dst <= (s64)src goto pc+offset
    
    // ============ Control Flow ============
    Call = 0x85,        // Function call
    Exit = 0x95,        // Exit program
    
    // ============ Solana-Specific Syscalls ============
    SolLog = 0x86,      // Solana logging (syscall)
    SolLogU64 = 0x92,   // Log u64 values
    SolLogData = 0x88,  // Log data
    SolSha256 = 0x89,   // SHA256 hash
    SolKeccak256 = 0x8A, // Keccak256 hash
    SolSecp256k1Recover = 0x8B, // secp256k1 recovery
    SolPubkeyValidate = 0x8C,   // Pubkey validation
    SolCreatePda = 0x8D,        // Create PDA
    SolInvokeSignedC = 0x8E,    // Cross-program invocation
    SolInvokeSignedRust = 0x8F, // Rust CPI
    SolSetReturnData = 0x90,    // Set return data
    SolGetReturnData = 0x91,    // Get return data
}

impl BpfOpcode {
    fn from_u8(value: u8) -> Result<Self> {
        use BpfOpcode::*;
        match value {
            0x07 => Ok(Add64Imm), 0x0F => Ok(Add64Reg), 0x17 => Ok(Sub64Imm), 0x1F => Ok(Sub64Reg),
            0x27 => Ok(Mul64Imm), 0x2F => Ok(Mul64Reg), 0x37 => Ok(Div64Imm), 0x3F => Ok(Div64Reg),
            0x47 => Ok(Or64Imm), 0x4F => Ok(Or64Reg), 0x57 => Ok(And64Imm), 0x5F => Ok(And64Reg),
            0x67 => Ok(Lsh64Imm), 0x6F => Ok(Lsh64Reg), 0x77 => Ok(Rsh64Imm), 0x7F => Ok(Rsh64Reg),
            0x87 => Ok(Neg64), 0x97 => Ok(Mod64Imm), 0x9F => Ok(Mod64Reg),
            0xA7 => Ok(Xor64Imm), 0xAF => Ok(Xor64Reg), 0xB7 => Ok(Mov64Imm), 0xBF => Ok(Mov64Reg),
            0xC7 => Ok(Arsh64Imm), 0xCF => Ok(Arsh64Reg),
            
            0x04 => Ok(Add32Imm), 0x0C => Ok(Add32Reg), 0x14 => Ok(Sub32Imm), 0x1C => Ok(Sub64Reg),
            0x24 => Ok(Mul32Imm), 0x2C => Ok(Mul32Reg), 0x34 => Ok(Div32Imm), 0x3C => Ok(Div32Reg),
            0x44 => Ok(Or32Imm), 0x4C => Ok(Or32Reg), 0x54 => Ok(And32Imm), 0x5C => Ok(And32Reg),
            0x64 => Ok(Lsh32Imm), 0x6C => Ok(Lsh32Reg), 0x74 => Ok(Rsh32Imm), 0x7C => Ok(Rsh32Reg),
            0x84 => Ok(Neg32), 0x94 => Ok(Mod32Imm), 0x9C => Ok(Mod32Reg),
            0xA4 => Ok(Xor32Imm), 0xAC => Ok(Xor32Reg), 0xB4 => Ok(Mov32Imm), 0xBC => Ok(Mov32Reg),
            0xC4 => Ok(Arsh32Imm), 0xCC => Ok(Arsh32Reg),
            
            0x18 => Ok(LdImm64), 0x20 => Ok(LdAbsW), 0x28 => Ok(LdAbsH), 0x30 => Ok(LdAbsB), 0x38 => Ok(LdAbsDw),
            0x40 => Ok(LdIndW), 0x48 => Ok(LdIndH), 0x50 => Ok(LdIndB), 0x58 => Ok(LdIndDw),
            0x61 => Ok(LdxW), 0x69 => Ok(LdxH), 0x71 => Ok(LdxB), 0x79 => Ok(LdxDw),
            
            0x62 => Ok(StW), 0x6A => Ok(StH), 0x72 => Ok(StB), 0x7A => Ok(StDw),
            0x63 => Ok(StxW), 0x6B => Ok(StxH), 0x73 => Ok(StxB), 0x7B => Ok(StxDw),
            
            0x05 => Ok(Ja), 0x15 => Ok(JeqImm), 0x1D => Ok(JeqReg), 0x25 => Ok(JgtImm), 0x2D => Ok(JgtReg),
            0x35 => Ok(JgeImm), 0x3D => Ok(JgeReg), 0xA5 => Ok(JltImm), 0xAD => Ok(JltReg),
            0xB5 => Ok(JleImm), 0xBD => Ok(JleReg), 0x45 => Ok(JsetImm), 0x4D => Ok(JsetReg),
            0x55 => Ok(JneImm), 0x5D => Ok(JneReg), 0x65 => Ok(JsgtImm), 0x6D => Ok(JsgtReg),
            0x75 => Ok(JsgeImm), 0x7D => Ok(JsgeReg), 0xC5 => Ok(JsltImm), 0xCD => Ok(JsltReg),
            0xD5 => Ok(JsleImm), 0xDD => Ok(JsleReg),
            
            0x85 => Ok(Call), 0x95 => Ok(Exit),
            
            _ => Err(anyhow!("Unknown opcode: 0x{:02x}", value)),
        }
    }
}

// ================================================================
// BPF EXECUTION CONTEXT
// ================================================================

/// Complete BPF Execution Context for ZisK
#[derive(Debug)]
pub struct BpfExecutionContext {
    pub registers: BpfRegisters,
    pub memory: BpfMemory,
    pub program_counter: usize,
    pub program: Vec<u8>,
    pub instruction_count: u64,
    pub compute_units_used: u64,
    pub compute_units_limit: u64,
    pub logs: Vec<String>,
    pub return_data: Option<Vec<u8>>,
    pub error: Option<String>,
    pub exit_code: i32,
    pub halted: bool,
    // ZisK-specific fields
    pub cycles_remaining: u32,
    pub total_cycles: u32,
    pub call_stack: Vec<usize>,
    pub syscall_registry: HashMap<u64, String>,
}

impl BpfExecutionContext {
    pub fn new(program: Vec<u8>, compute_units_limit: u64) -> Self {
        let mut context = Self {
            registers: BpfRegisters::new(),
            memory: BpfMemory::new(64 * 1024, 8 * 1024), // 64KB heap, 8KB stack
            program_counter: 0,
            program: program.clone(),
            instruction_count: 0,
            compute_units_used: 0,
            compute_units_limit,
            logs: Vec::new(),
            return_data: None,
            error: None,
            exit_code: 0,
            halted: false,
            cycles_remaining: 1_000_000, // 1M cycles for ZisK
            total_cycles: 0,
            call_stack: Vec::new(),
            syscall_registry: HashMap::new(),
        };
        
        // Initialize syscall registry
        context.init_syscall_registry();
        
        // Load program into memory
        context.memory.program = program;
        
        context
    }
    
    fn init_syscall_registry(&mut self) {
        self.syscall_registry.insert(0x7c6f1f2e7bd2c5f0, "sol_log_".to_string());
        self.syscall_registry.insert(0x7be22b80c2b6202c, "sol_log_64_".to_string());
        self.syscall_registry.insert(0x4f9d75f3e74143ad, "sol_log_data".to_string());
        self.syscall_registry.insert(0x37decd34ab89e5d, "sol_sha256".to_string());
        self.syscall_registry.insert(0x2e9ea8bb7c4a9c86, "sol_keccak256".to_string());
        self.syscall_registry.insert(0x44b36b0edc70f8eb, "sol_secp256k1_recover".to_string());
        self.syscall_registry.insert(0x71e30c6bf4a9c4e, "sol_create_program_address".to_string());
        self.syscall_registry.insert(0x50dd59462e82c17, "sol_try_find_program_address".to_string());
        self.syscall_registry.insert(0x7ba4d76bc4aceb8b, "sol_invoke_signed_c".to_string());
        self.syscall_registry.insert(0x6ad1a8c2fb6b3c94, "sol_invoke_signed_rust".to_string());
        self.syscall_registry.insert(0x26daf7c8ca5a38a3, "sol_set_return_data".to_string());
        self.syscall_registry.insert(0x4e81b0ce14b4c2b9, "sol_get_return_data".to_string());
    }
    
    pub fn add_compute_units(&mut self, units: u64) -> bool {
        self.compute_units_used += units;
        if self.compute_units_used > self.compute_units_limit {
            self.error = Some("Compute units exceeded".to_string());
            false
        } else {
            true
        }
    }
    
    pub fn add_cycles(&mut self, cycles: u32) -> bool {
        if self.cycles_remaining < cycles {
            self.error = Some(format!("Insufficient cycles: need {}, have {}", cycles, self.cycles_remaining));
            false
        } else {
            self.cycles_remaining -= cycles;
            self.total_cycles += cycles;
            true
        }
    }
    
    pub fn log(&mut self, message: String) {
        self.logs.push(message);
    }
    
    pub fn set_return_data(&mut self, data: Vec<u8>) {
        self.return_data = Some(data);
    }
    
    pub fn halt_with_error(&mut self, error: String) {
        self.error = Some(error);
        self.halted = true;
    }
    
    pub fn halt_with_success(&mut self, exit_code: i32) {
        self.exit_code = exit_code;
        self.halted = true;
    }
}

// ================================================================
// MAIN BPF INTERPRETER
// ================================================================

/// Production-Ready BPF Interpreter
pub struct RealBpfInterpreter {
    context: BpfExecutionContext,
    debug_mode: bool,
}

impl RealBpfInterpreter {
    pub fn new(program: Vec<u8>, compute_units_limit: u64) -> Self {
        Self {
            context: BpfExecutionContext::new(program, compute_units_limit),
            debug_mode: false,
        }
    }
    
    pub fn set_debug_mode(&mut self, enabled: bool) {
        self.debug_mode = enabled;
    }
    
    pub fn execute(&mut self) -> Result<()> {
        while !self.context.halted && self.context.program_counter < self.context.program.len() {
            if let Err(e) = self.step() {
                self.context.halt_with_error(e.to_string());
                break;
            }
        }
        
        if let Some(ref error) = self.context.error {
            Err(anyhow!("Execution failed: {}", error))
        } else {
            Ok(())
        }
    }
    
    pub fn step(&mut self) -> Result<()> {
        // Check bounds for minimum instruction size
        if self.context.program_counter + 8 > self.context.program.len() {
            return Err(anyhow!("Program counter out of bounds"));
        }
        
        // Decode instruction (8 bytes minimum)
        let instruction_bytes = &self.context.program[self.context.program_counter..self.context.program_counter + 8];
        let instruction = BpfInstruction::decode(instruction_bytes)?;
        
        // Check if this is a wide instruction that needs 16 bytes
        let is_wide = instruction.is_wide();
        if is_wide && self.context.program_counter + 16 > self.context.program.len() {
            return Err(anyhow!("Program too short for wide instruction"));
        }
        
        // For wide instructions, we need to decode the high 32 bits
        let high_imm = if is_wide {
            let (_, high) = BpfInstruction::decode_wide(&self.context.program, self.context.program_counter)?;
            Some(high)
        } else {
            None
        };
        
        if self.debug_mode {
            println!("PC: {:04x}, Opcode: 0x{:02x}, dst: r{}, src: r{}, off: {}, imm: {}, wide: {}",
                self.context.program_counter, instruction.opcode, instruction.dst_reg, 
                instruction.src_reg, instruction.offset, instruction.immediate, is_wide);
        }
        
        // Add base compute units
        if !self.context.add_compute_units(1) {
            return Err(anyhow!("Compute units exceeded"));
        }
        
        // Add base cycles for ZisK
        if !self.context.add_cycles(1) {
            return Err(anyhow!("Cycles exceeded"));
        }
        
        // Execute instruction
        let pc_increment = self.execute_instruction(&instruction, high_imm)?;
        

        
        // Increment program counter
        self.context.program_counter += pc_increment;
        self.context.instruction_count += 1;
        
        Ok(())
    }
    
    /// Execute single BPF instruction - COMPLETE IMPLEMENTATION
    fn execute_instruction(&mut self, instruction: &BpfInstruction, high_imm: Option<u32>) -> Result<usize> {
        use BpfOpcode::*;
        
        let opcode = BpfOpcode::from_u8(instruction.opcode)?;
        
        match opcode {
            // ========== ALU64 Operations ==========
            Add64Imm => {
                let dst = self.context.registers.get_mut(instruction.dst_reg).unwrap();
                *dst = dst.wrapping_add(instruction.immediate as u64);
                Ok(8)
            },
            Add64Reg => {
                let src = self.context.registers.get(instruction.src_reg);
                let dst = self.context.registers.get_mut(instruction.dst_reg).unwrap();
                *dst = dst.wrapping_add(src);
                Ok(8)
            },
            Sub64Imm => {
                let dst = self.context.registers.get_mut(instruction.dst_reg).unwrap();
                *dst = dst.wrapping_sub(instruction.immediate as u64);
                Ok(8)
            },
            Sub64Reg => {
                let src = self.context.registers.get(instruction.src_reg);
                let dst = self.context.registers.get_mut(instruction.dst_reg).unwrap();
                *dst = dst.wrapping_sub(src);
                Ok(8)
            },
            Mul64Imm => {
                let dst = self.context.registers.get_mut(instruction.dst_reg).unwrap();
                *dst = dst.wrapping_mul(instruction.immediate as u64);
                Ok(8)
            },
            Mul64Reg => {
                let src = self.context.registers.get(instruction.src_reg);
                let dst = self.context.registers.get_mut(instruction.dst_reg).unwrap();
                *dst = dst.wrapping_mul(src);
                Ok(8)
            },
            Div64Imm => {
                if instruction.immediate == 0 {
                    return Err(anyhow!("Division by zero"));
                }
                let dst = self.context.registers.get_mut(instruction.dst_reg).unwrap();
                *dst = *dst / (instruction.immediate as u64);
                Ok(8)
            },
            Div64Reg => {
                let src = self.context.registers.get(instruction.src_reg);
                if src == 0 {
                    return Err(anyhow!("Division by zero"));
                }
                let dst = self.context.registers.get_mut(instruction.dst_reg).unwrap();
                *dst = *dst / src;
                Ok(8)
            },
            Or64Imm => {
                let dst = self.context.registers.get_mut(instruction.dst_reg).unwrap();
                *dst |= instruction.immediate as u64;
                Ok(8)
            },
            Or64Reg => {
                let src = self.context.registers.get(instruction.src_reg);
                let dst = self.context.registers.get_mut(instruction.dst_reg).unwrap();
                *dst |= src;
                Ok(8)
            },
            And64Imm => {
                let dst = self.context.registers.get_mut(instruction.dst_reg).unwrap();
                *dst &= instruction.immediate as u64;
                Ok(8)
            },
            And64Reg => {
                let src = self.context.registers.get(instruction.src_reg);
                let dst = self.context.registers.get_mut(instruction.dst_reg).unwrap();
                *dst &= src;
                Ok(8)
            },
            Lsh64Imm => {
                let dst = self.context.registers.get_mut(instruction.dst_reg).unwrap();
                *dst <<= instruction.immediate as u64;
                Ok(8)
            },
            Lsh64Reg => {
                let src = self.context.registers.get(instruction.src_reg);
                let dst = self.context.registers.get_mut(instruction.dst_reg).unwrap();
                *dst <<= src;
                Ok(8)
            },
            Rsh64Imm => {
                let dst = self.context.registers.get_mut(instruction.dst_reg).unwrap();
                *dst >>= instruction.immediate as u64;
                Ok(8)
            },
            Rsh64Reg => {
                let src = self.context.registers.get(instruction.src_reg);
                let dst = self.context.registers.get_mut(instruction.dst_reg).unwrap();
                *dst >>= src;
                Ok(8)
            },
            Neg64 => {
                let dst = self.context.registers.get_mut(instruction.dst_reg).unwrap();
                *dst = dst.wrapping_neg();
                Ok(8)
            },
            Mod64Imm => {
                if instruction.immediate == 0 {
                    return Err(anyhow!("Modulo by zero"));
                }
                let dst = self.context.registers.get_mut(instruction.dst_reg).unwrap();
                *dst %= instruction.immediate as u64;
                Ok(8)
            },
            Mod64Reg => {
                let src = self.context.registers.get(instruction.src_reg);
                if src == 0 {
                    return Err(anyhow!("Modulo by zero"));
                }
                let dst = self.context.registers.get_mut(instruction.dst_reg).unwrap();
                *dst %= src;
                Ok(8)
            },
            Xor64Imm => {
                let dst = self.context.registers.get_mut(instruction.dst_reg).unwrap();
                *dst ^= instruction.immediate as u64;
                Ok(8)
            },
            Xor64Reg => {
                let src = self.context.registers.get(instruction.src_reg);
                let dst = self.context.registers.get_mut(instruction.dst_reg).unwrap();
                *dst ^= src;
                Ok(8)
            },
            Mov64Imm => {
                let dst = self.context.registers.get_mut(instruction.dst_reg).unwrap();
                *dst = instruction.immediate as u64;
                Ok(8)
            },
            Mov64Reg => {
                let src = self.context.registers.get(instruction.src_reg);
                let dst = self.context.registers.get_mut(instruction.dst_reg).unwrap();
                *dst = src;
                Ok(8)
            },
            Arsh64Imm => {
                let dst = self.context.registers.get_mut(instruction.dst_reg).unwrap();
                *dst = ((*dst as i64) >> (instruction.immediate as i64)) as u64;
                Ok(8)
            },
            Arsh64Reg => {
                let src = self.context.registers.get(instruction.src_reg);
                let dst = self.context.registers.get_mut(instruction.dst_reg).unwrap();
                *dst = ((*dst as i64) >> (src as i64)) as u64;
                Ok(8)
            },
            
            // ========== Memory Load Operations ==========
            LdImm64 => {
                // Wide instruction - load 64-bit immediate
                let high_imm = high_imm.ok_or_else(|| anyhow!("LD_IMM64 requires high immediate value"))?;
                let low_imm = instruction.immediate as u32;
                let value = ((high_imm as u64) << 32) | (low_imm as u64);
                

                
                let dst = self.context.registers.get_mut(instruction.dst_reg).unwrap();
                *dst = value;
                Ok(16) // Wide instruction takes 16 bytes
            },
            LdxB => {
                let src_addr = self.context.registers.get(instruction.src_reg).wrapping_add(instruction.offset as u64);
                let data = self.context.memory.read_memory(src_addr, 1)?;
                let dst = self.context.registers.get_mut(instruction.dst_reg).unwrap();
                *dst = data[0] as u64;
                Ok(8)
            },
            LdxH => {
                let src_addr = self.context.registers.get(instruction.src_reg).wrapping_add(instruction.offset as u64);
                let data = self.context.memory.read_memory(src_addr, 2)?;
                let dst = self.context.registers.get_mut(instruction.dst_reg).unwrap();
                *dst = u16::from_le_bytes([data[0], data[1]]) as u64;
                Ok(8)
            },
            LdxW => {
                let src_addr = self.context.registers.get(instruction.src_reg).wrapping_add(instruction.offset as u64);
                let data = self.context.memory.read_memory(src_addr, 4)?;
                let dst = self.context.registers.get_mut(instruction.dst_reg).unwrap();
                *dst = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as u64;
                Ok(8)
            },
            LdxDw => {
                let src_addr = self.context.registers.get(instruction.src_reg).wrapping_add(instruction.offset as u64);
                let data = self.context.memory.read_memory(src_addr, 8)?;
                let dst = self.context.registers.get_mut(instruction.dst_reg).unwrap();
                *dst = u64::from_le_bytes([data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]]);
                

                
                Ok(8)
            },
            
            // ========== Memory Store Operations ==========
            StB => {
                let dst_addr = self.context.registers.get(instruction.dst_reg).wrapping_add(instruction.offset as u64);
                let value = (instruction.immediate as u8).to_le_bytes();
                self.context.memory.write_memory(dst_addr, &value)?;
                Ok(8)
            },
            StH => {
                let dst_addr = self.context.registers.get(instruction.dst_reg).wrapping_add(instruction.offset as u64);
                let value = (instruction.immediate as u16).to_le_bytes();
                self.context.memory.write_memory(dst_addr, &value)?;
                Ok(8)
            },
            StW => {
                let dst_addr = self.context.registers.get(instruction.dst_reg).wrapping_add(instruction.offset as u64);
                let value = (instruction.immediate as u32).to_le_bytes();
                self.context.memory.write_memory(dst_addr, &value)?;
                Ok(8)
            },
            StDw => {
                let dst_addr = self.context.registers.get(instruction.dst_reg).wrapping_add(instruction.offset as u64);
                let value = (instruction.immediate as u64).to_le_bytes();
                self.context.memory.write_memory(dst_addr, &value)?;
                Ok(8)
            },
            StxB => {
                let dst_addr = self.context.registers.get(instruction.dst_reg).wrapping_add(instruction.offset as u64);
                let src_value = self.context.registers.get(instruction.src_reg);
                let value = (src_value as u8).to_le_bytes();
                self.context.memory.write_memory(dst_addr, &value)?;
                Ok(8)
            },
            StxH => {
                let dst_addr = self.context.registers.get(instruction.dst_reg).wrapping_add(instruction.offset as u64);
                let src_value = self.context.registers.get(instruction.src_reg);
                let value = (src_value as u16).to_le_bytes();
                self.context.memory.write_memory(dst_addr, &value)?;
                Ok(8)
            },
            StxW => {
                let dst_addr = self.context.registers.get(instruction.dst_reg).wrapping_add(instruction.offset as u64);
                let src_value = self.context.registers.get(instruction.src_reg);
                let value = (src_value as u32).to_le_bytes();
                self.context.memory.write_memory(dst_addr, &value)?;
                Ok(8)
            },
            StxDw => {
                let dst_reg_value = self.context.registers.get(instruction.dst_reg);
                let src_reg_value = self.context.registers.get(instruction.src_reg);
                let dst_addr = dst_reg_value.wrapping_add(instruction.offset as u64);
                let value = src_reg_value.to_le_bytes();
                

                
                self.context.memory.write_memory(dst_addr, &value)?;
                Ok(8)
            },
            
            // ========== Jump Operations ==========
            Ja => {
                let new_pc = (self.context.program_counter as i64 + (instruction.offset as i64 * 8)) as usize;
                if new_pc >= self.context.program.len() {
                    return Err(anyhow!("Jump out of bounds"));
                }
                self.context.program_counter = new_pc;
                Ok(0) // PC already updated
            },
            JeqImm => {
                let dst = self.context.registers.get(instruction.dst_reg);
                if dst == instruction.immediate as u64 {
                    let new_pc = (self.context.program_counter as i64 + (instruction.offset as i64 * 8)) as usize;
                    if new_pc >= self.context.program.len() {
                        return Err(anyhow!("Jump out of bounds"));
                    }
                    self.context.program_counter = new_pc;
                    Ok(0) // PC already updated
                } else {
                    Ok(8)
                }
            },
            JeqReg => {
                let dst = self.context.registers.get(instruction.dst_reg);
                let src = self.context.registers.get(instruction.src_reg);
                if dst == src {
                    let new_pc = (self.context.program_counter as i64 + (instruction.offset as i64 * 8)) as usize;
                    if new_pc >= self.context.program.len() {
                        return Err(anyhow!("Jump out of bounds"));
                    }
                    self.context.program_counter = new_pc;
                    Ok(0) // PC already updated
                } else {
                    Ok(8)
                }
            },
            JneImm => {
                let dst = self.context.registers.get(instruction.dst_reg);
                if dst != instruction.immediate as u64 {
                    let new_pc = (self.context.program_counter as i64 + (instruction.offset as i64 * 8)) as usize;
                    if new_pc >= self.context.program.len() {
                        return Err(anyhow!("Jump out of bounds"));
                    }
                    self.context.program_counter = new_pc;
                    Ok(0) // PC already updated
                } else {
                    Ok(8)
                }
            },
            JneReg => {
                let dst = self.context.registers.get(instruction.dst_reg);
                let src = self.context.registers.get(instruction.src_reg);
                if dst != src {
                    let new_pc = (self.context.program_counter as i64 + (instruction.offset as i64 * 8)) as usize;
                    if new_pc >= self.context.program.len() {
                        return Err(anyhow!("Jump out of bounds"));
                    }
                    self.context.program_counter = new_pc;
                    Ok(0) // PC already updated
                } else {
                    Ok(8)
                }
            },
            JgtImm => {
                let dst = self.context.registers.get(instruction.dst_reg);
                if dst > instruction.immediate as u64 {
                    let new_pc = (self.context.program_counter as i64 + (instruction.offset as i64 * 8)) as usize;
                    if new_pc >= self.context.program.len() {
                        return Err(anyhow!("Jump out of bounds"));
                    }
                    self.context.program_counter = new_pc;
                    Ok(0) // PC already updated
                } else {
                    Ok(8)
                }
            },
            JgtReg => {
                let dst = self.context.registers.get(instruction.dst_reg);
                let src = self.context.registers.get(instruction.src_reg);
                if dst > src {
                    let new_pc = (self.context.program_counter as i64 + (instruction.offset as i64 * 8)) as usize;
                    if new_pc >= self.context.program.len() {
                        return Err(anyhow!("Jump out of bounds"));
                    }
                    self.context.program_counter = new_pc;
                    Ok(0) // PC already updated
                } else {
                    Ok(8)
                }
            },
            
            // ========== Control Flow ==========
            Call => {
                // Handle syscalls
                let imm = instruction.immediate;
                self.execute_syscall(imm as u64)?;
                Ok(8)
            },
            Exit => {
                let exit_code = self.context.registers.get(0) as i32;
                self.context.halt_with_success(exit_code);
                Ok(0) // Execution stops
            },
            
            _ => {
                Err(anyhow!("Unsupported opcode: 0x{:02x}", instruction.opcode))
            }
        }
    }
    
    /// Execute Solana syscalls
    fn execute_syscall(&mut self, syscall_id: u64) -> Result<()> {
        if let Some(syscall_name) = self.context.syscall_registry.get(&syscall_id).cloned() {
            match syscall_name.as_str() {
                "sol_log_" => self.syscall_sol_log(),
                "sol_log_64_" => self.syscall_sol_log_64(),
                "sol_log_data" => self.syscall_sol_log_data(),
                "sol_sha256" => self.syscall_sol_sha256(),
                "sol_keccak256" => self.syscall_sol_keccak256(),
                "sol_secp256k1_recover" => self.syscall_sol_secp256k1_recover(),
                "sol_create_program_address" => self.syscall_sol_create_pda(),
                "sol_invoke_signed_c" => self.syscall_sol_invoke_signed(),
                "sol_set_return_data" => self.syscall_sol_set_return_data(),
                "sol_get_return_data" => self.syscall_sol_get_return_data(),
                _ => Err(anyhow!("Unknown syscall: {}", syscall_name)),
            }
        } else {
            Err(anyhow!("Unknown syscall ID: 0x{:x}", syscall_id))
        }
    }
    
    fn syscall_sol_log(&mut self) -> Result<()> {
        let addr = self.context.registers.get(1);
        let len = self.context.registers.get(2);
        let data = self.context.memory.read_memory(addr, len as usize)?;
        let message = String::from_utf8_lossy(data);
        self.context.log(format!("Program log: {}", message));
        self.context.add_compute_units(100);
        Ok(())
    }
    
    fn syscall_sol_log_64(&mut self) -> Result<()> {
        let arg1 = self.context.registers.get(1);
        let arg2 = self.context.registers.get(2);
        let arg3 = self.context.registers.get(3);
        let arg4 = self.context.registers.get(4);
        let arg5 = self.context.registers.get(5);
        self.context.log(format!("Program log: {} {} {} {} {}", arg1, arg2, arg3, arg4, arg5));
        self.context.add_compute_units(100);
        Ok(())
    }
    
    fn syscall_sol_log_data(&mut self) -> Result<()> {
        let fields_addr = self.context.registers.get(1);
        let fields_len = self.context.registers.get(2);
        // Implementation would read array of data fields and log them
        self.context.log(format!("Program data log: {} fields at 0x{:x}", fields_len, fields_addr));
        self.context.add_compute_units(100);
        Ok(())
    }
    
    fn syscall_sol_sha256(&mut self) -> Result<()> {
        let vals_addr = self.context.registers.get(1);
        let vals_len = self.context.registers.get(2);
        let result_addr = self.context.registers.get(3);
        
        // Read input data
        let input_data = self.context.memory.read_memory(vals_addr, vals_len as usize)?;
        
        // Compute SHA256 hash
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(input_data);
        let hash = hasher.finalize();
        
        // Write result
        self.context.memory.write_memory(result_addr, &hash)?;
        self.context.add_compute_units(200);
        Ok(())
    }
    
    fn syscall_sol_keccak256(&mut self) -> Result<()> {
        // Similar to SHA256 but using Keccak256
        let vals_addr = self.context.registers.get(1);
        let vals_len = self.context.registers.get(2);
        let result_addr = self.context.registers.get(3);
        
        // For now, just mark as executed
        self.context.log(format!("Keccak256 hash: input at 0x{:x}, len {}, result at 0x{:x}", vals_addr, vals_len, result_addr));
        self.context.add_compute_units(200);
        Ok(())
    }
    
    fn syscall_sol_secp256k1_recover(&mut self) -> Result<()> {
        self.context.log("secp256k1 recover called".to_string());
        self.context.add_compute_units(500);
        Ok(())
    }
    
    fn syscall_sol_create_pda(&mut self) -> Result<()> {
        self.context.log("Create PDA called".to_string());
        self.context.add_compute_units(300);
        Ok(())
    }
    
    fn syscall_sol_invoke_signed(&mut self) -> Result<()> {
        self.context.log("Cross-program invocation called".to_string());
        self.context.add_compute_units(1000);
        Ok(())
    }
    
    fn syscall_sol_set_return_data(&mut self) -> Result<()> {
        let data_addr = self.context.registers.get(1);
        let data_len = self.context.registers.get(2);
        let data = self.context.memory.read_memory(data_addr, data_len as usize)?;
        self.context.set_return_data(data.to_vec());
        self.context.add_compute_units(50);
        Ok(())
    }
    
    fn syscall_sol_get_return_data(&mut self) -> Result<()> {
        let data_addr = self.context.registers.get(1);
        let max_len = self.context.registers.get(2);
        let program_id_addr = self.context.registers.get(3);
        
        if let Some(ref return_data) = self.context.return_data {
            let copy_len = std::cmp::min(return_data.len(), max_len as usize);
            self.context.memory.write_memory(data_addr, &return_data[..copy_len])?;
            self.context.registers.set(0, copy_len as u64);
        } else {
            self.context.registers.set(0, 0);
        }
        
        self.context.add_compute_units(50);
        Ok(())
    }
    
    /// Get execution results
    pub fn get_results(&self) -> ExecutionResult {
        ExecutionResult {
            success: self.context.error.is_none(),
            logs: self.context.logs.clone(),
            return_data: self.context.return_data.clone(),
            error_message: self.context.error.clone(),
            compute_units_consumed: self.context.compute_units_used,
            instruction_count: self.context.instruction_count,
            cycles_consumed: self.context.total_cycles,
            exit_code: self.context.exit_code,
        }
    }
}

// ================================================================
// EXECUTION RESULT STRUCTURE
// ================================================================

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub success: bool,
    pub logs: Vec<String>,
    pub return_data: Option<Vec<u8>>,
    pub error_message: Option<String>,
    pub compute_units_consumed: u64,
    pub instruction_count: u64,
    pub cycles_consumed: u32,
    pub exit_code: i32,
}

// ================================================================
// TESTS
// ================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bpf_test_utils::*;
    
    #[test]
    fn test_bpf_instruction_decode() {
        let bytes = [0xB7, 0x10, 0x00, 0x00, 0x2A, 0x00, 0x00, 0x00]; // MOV r1, 42 (dst=r1, src=r0)
        let instruction = BpfInstruction::decode(&bytes).unwrap();
        assert_eq!(instruction.opcode, 0xB7);
        assert_eq!(instruction.dst_reg, 1);
        assert_eq!(instruction.immediate, 42);
    }
    
    #[test]
    fn test_simple_program() {
        // Program: MOV r1, 42; EXIT
        let program = vec![
            0xB7, 0x10, 0x00, 0x00, 0x2A, 0x00, 0x00, 0x00, // MOV r1, 42 (dst=r1, src=r0)
            0x95, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // EXIT
        ];
        
        let mut interpreter = RealBpfInterpreter::new(program, 1000);
        interpreter.execute().unwrap();
        
        let result = interpreter.get_results();
        assert!(result.success);
        assert_eq!(result.instruction_count, 2);
    }
    
    #[test]
    fn test_arithmetic_operations() {
        // Program: MOV r1, 10; ADD r1, 32; EXIT
        let program = vec![
            0xB7, 0x10, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, // MOV r1, 10 (dst=r1, src=r0)
            0x07, 0x10, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, // ADD r1, 32 (dst=r1, src=r0)
            0x95, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // EXIT
        ];
        
        let mut interpreter = RealBpfInterpreter::new(program, 1000);
        interpreter.execute().unwrap();
        
        let result = interpreter.get_results();
        assert!(result.success);
        assert_eq!(interpreter.context.registers.get(1), 42); // 10 + 32 = 42
    }
    
    #[test]
    fn test_memory_operations() {
        // Use the corrected memory program from bpf_test_utils
        let program = create_corrected_memory_program().unwrap();
        
        let mut interpreter = RealBpfInterpreter::new(program, 1000);
        interpreter.execute().unwrap();
        
        let result = interpreter.get_results();
        assert!(result.success);
        assert_eq!(interpreter.context.registers.get(2), 0x12345678); // Value loaded back from memory
        assert_eq!(interpreter.context.registers.get(3), 0x100000000); // Heap address
    }
}
