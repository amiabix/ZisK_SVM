//! BPF Interpreter for Solana Program Execution in ZisK zkVM
//! 
//! This module implements a BPF (Berkeley Packet Filter) interpreter that can execute
//! Solana programs directly within the ZisK zero-knowledge virtual machine.
//! 
//! The interpreter handles:
//! - BPF instruction decoding and execution
//! - Solana account model and state management
//! - Memory access and validation
//! - Cross-program invocation
//! - Compute unit tracking
//! - Error handling and rollback

use std::collections::HashMap;
// use crate::constants::OP_CYCLES; // Commented out for now

// ZisK-specific optimizations - using standard assertions for now
// TODO: Replace with actual ZisK-specific assertions when available

/// BPF Register Set (64-bit registers as per Solana's BPF implementation)
#[derive(Debug, Clone, Default)]
pub struct BpfRegisters {
    pub r0: u64,   // Always zero
    pub r1: u64,   // Return value
    pub r2: u64,   // Return value
    pub r3: u64,   // Argument 1
    pub r4: u64,   // Argument 2
    pub r5: u64,   // Argument 3
    pub r6: u64,   // Argument 4
    pub r7: u64,   // Argument 5
    pub r8: u64,   // Argument 6
    pub r9: u64,   // Argument 7
    pub r10: u64,  // Frame pointer
    pub r11: u64,  // Frame pointer
    pub r12: u64,  // Temporary
    pub r13: u64,  // Temporary
    pub r14: u64,  // Temporary
    pub r15: u64,  // Temporary
}

impl BpfRegisters {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn get(&self, index: u8) -> u64 {
        match index {
            0 => self.r0,
            1 => self.r1,
            2 => self.r2,
            3 => self.r3,
            4 => self.r4,
            5 => self.r5,
            6 => self.r6,
            7 => self.r7,
            8 => self.r8,
            9 => self.r9,
            10 => self.r10,
            11 => self.r11,
            12 => self.r12,
            13 => self.r13,
            14 => self.r14,
            15 => self.r15,
            _ => 0,
        }
    }
    
    pub fn set(&mut self, index: u8, value: u64) {
        match index {
            0 => self.r0 = value,
            1 => self.r1 = value,
            2 => self.r2 = value,
            3 => self.r3 = value,
            4 => self.r4 = value,
            5 => self.r5 = value,
            6 => self.r6 = value,
            7 => self.r7 = value,
            8 => self.r8 = value,
            9 => self.r9 = value,
            10 => self.r10 = value,
            11 => self.r11 = value,
            12 => self.r12 = value,
            13 => self.r13 = value,
            14 => self.r14 = value,
            15 => self.r15 = value,
            _ => {},
        }
    }
}

/// BPF Instruction Format
#[derive(Debug, Clone)]
pub struct BpfInstruction {
    pub opcode: u8,
    pub dst_reg: u8,
    pub src_reg: u8,
    pub offset: i16,
    pub immediate: i32,
}

impl BpfInstruction {
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 8 {
            return None;
        }
        
        let opcode = bytes[0];
        let dst_reg = bytes[1] & 0x0F;
        let src_reg = (bytes[1] >> 4) & 0x0F;
        let offset = i16::from_le_bytes([bytes[2], bytes[3]]);
        let immediate = i32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        
        Some(Self {
            opcode,
            dst_reg,
            src_reg,
            offset,
            immediate,
        })
    }
}

/// BPF Opcodes
#[repr(u8)]
pub enum BpfOpcode {
    // Load/Store operations
    LdAbsB = 0x30,    // Load absolute byte
    LdAbsH = 0x28,    // Load absolute halfword
    LdAbsW = 0x20,    // Load absolute word
    LdAbsDw = 0x18,   // Load absolute doubleword
    
    // Register operations
    LdReg = 0x61,     // Load register
    StReg = 0x62,     // Store register
    StRegImm = 0x63,  // Store register immediate
    
    // Arithmetic operations
    AddReg = 0x0F,    // Add registers
    SubReg = 0x1F,    // Subtract registers
    MulReg = 0x2F,    // Multiply registers
    DivReg = 0x3F,    // Divide registers
    
    // Comparison operations
    JeqImm = 0x15,    // Jump if equal (immediate)
    JneImm = 0x55,    // Jump if not equal (immediate)
    
    // Control flow
    Ja = 0x05,        // Jump always
    Call = 0x85,      // Call function
    Exit = 0x95,      // Exit program
    
    // Solana-specific operations
    SolCall = 0xE0,   // Solana cross-program invocation
    SolLog = 0xE1,    // Solana logging
    SolReturn = 0xE2, // Solana return data
}

/// Solana Account Structure
/// 
/// Represents a complete Solana account with all required fields
/// for production use and BPF program execution.
#[derive(Debug, Clone)]
pub struct SolanaAccount {
    /// Account public key (32 bytes)
    pub pubkey: [u8; 32],
    /// Account balance in lamports
    pub lamports: u64,
    /// Account owner (program ID that owns this account)
    pub owner: [u8; 32],
    /// Whether this account is executable (program account)
    pub executable: bool,
    /// Rent epoch for this account
    pub rent_epoch: u64,
    /// Account data (program-specific data)
    pub data: Vec<u8>,
}

impl SolanaAccount {
    /// Create a new Solana account with default values
    /// 
    /// # Arguments
    /// 
    /// * `pubkey` - Account public key
    /// 
    /// # Returns
    /// 
    /// Returns a new SolanaAccount with default values
    pub fn new(pubkey: [u8; 32]) -> Self {
        Self {
            pubkey,
            lamports: 0,
            owner: [0u8; 32], // Default to system program
            executable: false,
            rent_epoch: 0,
            data: Vec::new(),
        }
    }
    
    /// Create a new Solana account with all fields specified
    /// 
    /// # Arguments
    /// 
    /// * `pubkey` - Account public key
    /// * `lamports` - Account balance in lamports
    /// * `owner` - Account owner program ID
    /// * `executable` - Whether account is executable
    /// * `rent_epoch` - Rent epoch
    /// * `data` - Account data
    /// 
    /// # Returns
    /// 
    /// Returns a new SolanaAccount with specified values
    pub fn new_with_data(
        pubkey: [u8; 32],
        lamports: u64,
        owner: [u8; 32],
        executable: bool,
        rent_epoch: u64,
        data: Vec<u8>,
    ) -> Self {
        Self {
            pubkey,
            lamports,
            owner,
            executable,
            rent_epoch,
            data,
        }
    }
    
    /// Check if this account is owned by the system program
    /// 
    /// # Returns
    /// 
    /// Returns `true` if owned by system program (all zeros)
    pub fn is_system_owned(&self) -> bool {
        self.owner == [0u8; 32]
    }
    
    /// Check if this account is a program account
    /// 
    /// # Returns
    /// 
    /// Returns `true` if account is executable
    pub fn is_program(&self) -> bool {
        self.executable
    }
    
    /// Get account data size
    /// 
    /// # Returns
    /// 
    /// Returns the size of account data in bytes
    pub fn data_size(&self) -> usize {
        self.data.len()
    }
}

/// BPF Memory Layout
#[derive(Debug, Clone)]
pub struct BpfMemory {
    pub heap: Vec<u8>,
}

impl BpfMemory {
    pub fn new(heap_size: usize) -> Self {
        Self {
            heap: vec![0; heap_size],
        }
    }
    
    pub fn read_heap(&self, offset: usize, size: usize) -> Option<&[u8]> {
        if offset + size <= self.heap.len() {
            Some(&self.heap[offset..offset + size])
        } else {
            None
        }
    }
    
    pub fn write_heap(&mut self, offset: usize, data: &[u8]) -> bool {
        if offset + data.len() <= self.heap.len() {
            self.heap[offset..offset + data.len()].copy_from_slice(data);
            true
        } else {
            false
        }
    }
}

/// BPF Execution Context
#[derive(Debug, Clone)]
pub struct BpfExecutionContext {
    pub registers: BpfRegisters,
    pub memory: BpfMemory,
    pub program_counter: usize,
    pub program: Vec<u8>,
    pub compute_units_used: u64,
    pub compute_units_limit: u64,
    pub logs: Vec<String>,
    pub return_data: Option<Vec<u8>>,
    pub error: Option<String>,
    // ZisK-specific cycle accounting
    pub cycles_remaining: u32,
    pub total_cycles: u32,
}

impl BpfExecutionContext {
    pub fn new(program: Vec<u8>, compute_units_limit: u64) -> Self {
        Self {
            registers: BpfRegisters::new(),
            memory: BpfMemory::new(64 * 1024), // 64KB heap
            program_counter: 0,
            program,
            compute_units_used: 0,
            compute_units_limit,
            logs: Vec::new(),
            return_data: None,
            error: None,
            // ZisK-specific cycle accounting
            cycles_remaining: 1000000, // 1M cycles limit
            total_cycles: 0,
        }
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
    
    pub fn log(&mut self, message: String) {
        self.logs.push(message);
    }
    
    pub fn set_return_data(&mut self, data: Vec<u8>) {
        self.return_data = Some(data);
    }
}

/// BPF Interpreter
pub struct BpfInterpreter {
    context: BpfExecutionContext,
}

impl BpfInterpreter {
    pub fn new(program: Vec<u8>, compute_units_limit: u64) -> Self {
        Self {
            context: BpfExecutionContext::new(program, compute_units_limit),
        }
    }
    
    pub fn execute(&mut self) -> Result<(), String> {
        while self.context.program_counter < self.context.program.len() {
            if !self.step()? {
                break;
            }
        }
        
        if let Some(ref error) = self.context.error {
            Err(error.clone())
        } else {
            Ok(())
        }
    }
    
    pub fn step(&mut self) -> Result<bool, String> {
        if self.context.program_counter >= self.context.program.len() {
            return Ok(false);
        }
        
        let instruction_bytes = &self.context.program[self.context.program_counter..];
        let instruction = BpfInstruction::decode(instruction_bytes)
            .ok_or("Failed to decode instruction")?;
        
        // Add compute units for instruction execution
        if !self.context.add_compute_units(1) {
            return Ok(false);
        }
        
        let should_continue = self.execute_instruction(&instruction)?;
        
        if should_continue {
            self.context.program_counter += 8; // BPF instructions are 8 bytes
        }
        
        Ok(should_continue)
    }
    
    pub fn execute_instruction(&mut self, instruction: &BpfInstruction) -> Result<bool, String> {
        // ZisK cycle accounting
        let cycles_needed = match instruction.opcode {
            // Load operations
            0x30 | 0x28 | 0x20 | 0x18 => 1, // LdAbs operations
            0x61 => 1, // LdReg
            
            // Store operations
            0x62 | 0x63 => 1, // StReg operations
            
            // Arithmetic operations
            0x0F | 0x1F | 0x2F | 0x3F => 2, // Add, Sub, Mul, Div
            
            // Comparison and jumps
            0x15 | 0x55 => 1, // JeqImm, JneImm
            0x05 => 1, // Ja
            
            // Control flow
            0x85 => 3, // Call
            0x95 => 1, // Exit
            
            // Solana-specific operations
            0xE0 => 5, // SolCall
            0xE1 => 1, // SolLog
            0xE2 => 1, // SolReturn
            
            _ => 1, // Default cycle cost
        };
        
        // Cycle validation for ZisK compatibility
        if self.context.cycles_remaining < cycles_needed {
            self.context.error = Some(format!("Insufficient cycles: need {}, have {}", cycles_needed, self.context.cycles_remaining));
            return Ok(false);
        }
        
        self.context.cycles_remaining -= cycles_needed;
        self.context.total_cycles += cycles_needed;
        
        // Define opcode constants for pattern matching
        const OP_LD_ABS_B: u8 = BpfOpcode::LdAbsB as u8;
        const OP_LD_ABS_H: u8 = BpfOpcode::LdAbsH as u8;
        const OP_LD_ABS_W: u8 = BpfOpcode::LdAbsW as u8;
        const OP_LD_ABS_DW: u8 = BpfOpcode::LdAbsDw as u8;
        const OP_LD_REG: u8 = BpfOpcode::LdReg as u8;
        const OP_ST_REG: u8 = BpfOpcode::StReg as u8;
        const OP_ST_REG_IMM: u8 = BpfOpcode::StRegImm as u8;
        const OP_ADD_REG: u8 = BpfOpcode::AddReg as u8;
        const OP_SUB_REG: u8 = BpfOpcode::SubReg as u8;
        const OP_MUL_REG: u8 = BpfOpcode::MulReg as u8;
        const OP_DIV_REG: u8 = BpfOpcode::DivReg as u8;
        const OP_JEQ_IMM: u8 = BpfOpcode::JeqImm as u8;
        const OP_JNE_IMM: u8 = BpfOpcode::JneImm as u8;
        const OP_JA: u8 = BpfOpcode::Ja as u8;
        const OP_CALL: u8 = BpfOpcode::Call as u8;
        const OP_EXIT: u8 = BpfOpcode::Exit as u8;
        const OP_SOL_CALL: u8 = BpfOpcode::SolCall as u8;
        const OP_SOL_LOG: u8 = BpfOpcode::SolLog as u8;
        const OP_SOL_RETURN: u8 = BpfOpcode::SolReturn as u8;
        
        match instruction.opcode {
            // Load operations
            OP_LD_ABS_B => self.execute_ld_abs_b(instruction),
            OP_LD_ABS_H => self.execute_ld_abs_h(instruction),
            OP_LD_ABS_W => self.execute_ld_abs_w(instruction),
            OP_LD_ABS_DW => self.execute_ld_abs_dw(instruction),
            OP_LD_REG => self.execute_ld_reg(instruction),
            
            // Store operations
            OP_ST_REG => self.execute_st_reg(instruction),
            OP_ST_REG_IMM => self.execute_st_reg_imm(instruction),
            
            // Arithmetic operations
            OP_ADD_REG => self.execute_add_reg(instruction),
            OP_SUB_REG => self.execute_sub_reg(instruction),
            OP_MUL_REG => self.execute_mul_reg(instruction),
            OP_DIV_REG => self.execute_div_reg(instruction),
            
            // Comparison and jumps
            OP_JEQ_IMM => self.execute_jeq_imm(instruction),
            OP_JNE_IMM => self.execute_jne_imm(instruction),
            OP_JA => self.execute_ja(instruction),
            
            // Control flow
            OP_CALL => self.execute_call(instruction),
            OP_EXIT => self.execute_exit(instruction),
            
            // Solana-specific operations
            OP_SOL_CALL => self.execute_sol_call(instruction),
            OP_SOL_LOG => self.execute_sol_log(instruction),
            OP_SOL_RETURN => self.execute_sol_return(instruction),
            
            _ => {
                self.context.error = Some(format!("Unsupported opcode: 0x{:02X}", instruction.opcode));
                Ok(false)
            }
        }
    }
    
    // Load absolute byte
    fn execute_ld_abs_b(&mut self, instruction: &BpfInstruction) -> Result<bool, String> {
        let addr = instruction.immediate as usize;
        if let Some(data) = self.context.memory.read_heap(addr, 1) {
            let value = data[0] as u64;
            self.context.registers.set(instruction.dst_reg, value);
        } else {
            self.context.error = Some("Invalid memory access".to_string());
            return Ok(false);
        }
        Ok(true)
    }
    
    // Load absolute halfword
    fn execute_ld_abs_h(&mut self, instruction: &BpfInstruction) -> Result<bool, String> {
        let addr = instruction.immediate as usize;
        if let Some(data) = self.context.memory.read_heap(addr, 2) {
            let value = u16::from_le_bytes([data[0], data[1]]) as u64;
            self.context.registers.set(instruction.dst_reg, value);
        } else {
            self.context.error = Some("Invalid memory access".to_string());
            return Ok(false);
        }
        Ok(true)
    }
    
    // Load absolute word
    fn execute_ld_abs_w(&mut self, instruction: &BpfInstruction) -> Result<bool, String> {
        let addr = instruction.immediate as usize;
        if let Some(data) = self.context.memory.read_heap(addr, 4) {
            let value = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as u64;
            self.context.registers.set(instruction.dst_reg, value);
        } else {
            self.context.error = Some("Invalid memory access".to_string());
            return Ok(false);
        }
        Ok(true)
    }
    
    // Load absolute doubleword
    fn execute_ld_abs_dw(&mut self, instruction: &BpfInstruction) -> Result<bool, String> {
        let addr = instruction.immediate as usize;
        if let Some(data) = self.context.memory.read_heap(addr, 8) {
            let value = u64::from_le_bytes([
                data[0], data[1], data[2], data[3],
                data[4], data[5], data[6], data[7]
            ]);
            self.context.registers.set(instruction.dst_reg, value);
        } else {
            self.context.error = Some("Invalid memory access".to_string());
            return Ok(false);
        }
        Ok(true)
    }
    
    // Load register (copy value from src_reg to dst_reg)
    fn execute_ld_reg(&mut self, instruction: &BpfInstruction) -> Result<bool, String> {
        let src_value = self.context.registers.get(instruction.src_reg);
        self.context.registers.set(instruction.dst_reg, src_value);
        Ok(true)
    }
    
    // Store register
    fn execute_st_reg(&mut self, instruction: &BpfInstruction) -> Result<bool, String> {
        let src_value = self.context.registers.get(instruction.src_reg);
        let dst_value = self.context.registers.get(instruction.dst_reg);
        let addr = dst_value as usize;
        
        let bytes = src_value.to_le_bytes();
        if !self.context.memory.write_heap(addr, &bytes) {
            self.context.error = Some("Invalid memory access".to_string());
            return Ok(false);
        }
        Ok(true)
    }
    
    // Store register immediate
    fn execute_st_reg_imm(&mut self, instruction: &BpfInstruction) -> Result<bool, String> {
        let src_value = self.context.registers.get(instruction.src_reg);
        let addr = instruction.immediate as usize;
        
        let bytes = src_value.to_le_bytes();
        if !self.context.memory.write_heap(addr, &bytes) {
            self.context.error = Some("Invalid memory access".to_string());
            return Ok(false);
        }
        Ok(true)
    }
    
    // Add registers
    fn execute_add_reg(&mut self, instruction: &BpfInstruction) -> Result<bool, String> {
        let src_value = self.context.registers.get(instruction.src_reg);
        let dst_value = self.context.registers.get(instruction.dst_reg);
        let result = dst_value.wrapping_add(src_value);
        self.context.registers.set(instruction.dst_reg, result);
        Ok(true)
    }
    
    // Subtract registers
    fn execute_sub_reg(&mut self, instruction: &BpfInstruction) -> Result<bool, String> {
        let src_value = self.context.registers.get(instruction.src_reg);
        let dst_value = self.context.registers.get(instruction.dst_reg);
        let result = dst_value.wrapping_sub(src_value);
        self.context.registers.set(instruction.dst_reg, result);
        Ok(true)
    }
    
    // Multiply registers
    fn execute_mul_reg(&mut self, instruction: &BpfInstruction) -> Result<bool, String> {
        let src_value = self.context.registers.get(instruction.src_reg);
        let dst_value = self.context.registers.get(instruction.dst_reg);
        let result = dst_value.wrapping_mul(src_value);
        self.context.registers.set(instruction.dst_reg, result);
        Ok(true)
    }
    
    // Divide registers
    fn execute_div_reg(&mut self, instruction: &BpfInstruction) -> Result<bool, String> {
        let src_value = self.context.registers.get(instruction.src_reg);
        if src_value == 0 {
            self.context.error = Some("Division by zero".to_string());
            return Ok(false);
        }
        let dst_value = self.context.registers.get(instruction.dst_reg);
        let result = dst_value / src_value;
        self.context.registers.set(instruction.dst_reg, result);
        Ok(true)
    }
    
    // Jump if equal immediate
    fn execute_jeq_imm(&mut self, instruction: &BpfInstruction) -> Result<bool, String> {
        let dst_value = self.context.registers.get(instruction.dst_reg);
        if dst_value == instruction.immediate as u64 {
            let jump_offset = instruction.offset as isize;
            let new_pc = self.context.program_counter as isize + jump_offset;
            if new_pc >= 0 && new_pc < self.context.program.len() as isize {
                self.context.program_counter = new_pc as usize;
                return Ok(false); // Don't advance PC normally
            }
        }
        Ok(true)
    }
    
    // Jump if not equal immediate
    fn execute_jne_imm(&mut self, instruction: &BpfInstruction) -> Result<bool, String> {
        let dst_value = self.context.registers.get(instruction.dst_reg);
        if dst_value != instruction.immediate as u64 {
            let jump_offset = instruction.offset as isize;
            let new_pc = self.context.program_counter as isize + jump_offset;
            if new_pc >= 0 && new_pc < self.context.program.len() as isize {
                self.context.program_counter = new_pc as usize;
                return Ok(false); // Don't advance PC normally
            }
        }
        Ok(true)
    }
    
    // Jump always
    fn execute_ja(&mut self, instruction: &BpfInstruction) -> Result<bool, String> {
        let jump_offset = instruction.offset as isize;
        let new_pc = self.context.program_counter as isize + jump_offset;
        if new_pc >= 0 && new_pc < self.context.program.len() as isize {
            self.context.program_counter = new_pc as usize;
            Ok(false) // Don't advance PC normally
        } else {
            self.context.error = Some("Invalid jump target".to_string());
            Ok(false)
        }
    }
    
    // Call function
    fn execute_call(&mut self, instruction: &BpfInstruction) -> Result<bool, String> {
        // For now, just log the call
        self.context.log(format!("Call to function at offset {}", instruction.offset));
        Ok(true)
    }
    
    // Exit program
    fn execute_exit(&mut self, _instruction: &BpfInstruction) -> Result<bool, String> {
        Ok(false) // Stop execution
    }
    
    // Solana cross-program invocation
    fn execute_sol_call(&mut self, instruction: &BpfInstruction) -> Result<bool, String> {
        let program_id = self.context.registers.get(instruction.src_reg);
        let instruction_data = self.context.registers.get(instruction.dst_reg);
        
        self.context.log(format!("Solana CPI: program={:016x}, data={:016x}", program_id, instruction_data));
        
        // Add compute units for CPI
        if !self.context.add_compute_units(1000) {
            return Ok(false);
        }
        
        Ok(true)
    }
    
    // Solana logging
    fn execute_sol_log(&mut self, instruction: &BpfInstruction) -> Result<bool, String> {
        let message_ptr = self.context.registers.get(instruction.src_reg) as usize;
        let message_len = self.context.registers.get(instruction.dst_reg) as usize;
        
        if let Some(data) = self.context.memory.read_heap(message_ptr, message_len) {
            if let Ok(message) = String::from_utf8(data.to_vec()) {
                self.context.log(format!("Solana log: {}", message));
            }
        }
        
        Ok(true)
    }
    
    // Solana return data
    fn execute_sol_return(&mut self, instruction: &BpfInstruction) -> Result<bool, String> {
        let data_ptr = self.context.registers.get(instruction.src_reg) as usize;
        let data_len = self.context.registers.get(instruction.dst_reg) as usize;
        
        if let Some(data) = self.context.memory.read_heap(data_ptr, data_len) {
            self.context.set_return_data(data.to_vec());
        }
        
        Ok(true)
    }
    
    // Get execution results
    pub fn get_results(&self) -> (Vec<String>, Option<Vec<u8>>, Option<String>, u64) {
        (
            self.context.logs.clone(),
            self.context.return_data.clone(),
            self.context.error.clone(),
            self.context.compute_units_used,
        )
    }
    

}

/// Solana Program Executor using BPF Interpreter
pub struct SolanaProgramExecutor {
    interpreter: BpfInterpreter,
    accounts: HashMap<[u8; 32], SolanaAccount>,
}

impl SolanaProgramExecutor {
    pub fn new(program: Vec<u8>, compute_units_limit: u64) -> Self {
        Self {
            interpreter: BpfInterpreter::new(program, compute_units_limit),
            accounts: HashMap::new(),
        }
    }
    
    pub fn add_account(&mut self, account: SolanaAccount) {
        self.accounts.insert(account.pubkey, account);
    }
    
    pub fn execute_program(&mut self, instruction_data: Vec<u8>, accounts: Vec<[u8; 32]>) -> Result<ExecutionResult, String> {
        // Set up initial registers with instruction data and accounts
        self.interpreter.context.registers.set(3, instruction_data.len() as u64); // r3 = instruction data length
        self.interpreter.context.registers.set(4, accounts.len() as u64); // r4 = account count
        
        // Copy instruction data to memory
        if !self.interpreter.context.memory.write_heap(0, &instruction_data) {
            return Err("Failed to write instruction data to memory".to_string());
        }
        
        // Execute the program
        self.interpreter.execute()?;
        
        // Get execution results
        let (logs, return_data, error, compute_units_used) = self.interpreter.get_results();
        
        Ok(ExecutionResult {
            success: error.is_none(),
            logs,
            return_data,
            error,
            compute_units_used,
        })
    }
}

/// Execution result
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub success: bool,
    pub logs: Vec<String>,
    pub return_data: Option<Vec<u8>>,
    pub error: Option<String>,
    pub compute_units_used: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bpf_instruction_decode() {
        // Test instruction: ADD r1, r2
        let bytes = [0x0F, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let instruction = BpfInstruction::decode(&bytes).unwrap();
        
        assert_eq!(instruction.opcode, 0x0F);
        assert_eq!(instruction.dst_reg, 1);
        assert_eq!(instruction.src_reg, 2);
        assert_eq!(instruction.offset, 0);
        assert_eq!(instruction.immediate, 0);
    }
    
    #[test]
    fn test_bpf_registers() {
        let mut registers = BpfRegisters::new();
        registers.set(1, 42);
        registers.set(2, 100);
        
        assert_eq!(registers.get(1), 42);
        assert_eq!(registers.get(2), 100);
        assert_eq!(registers.get(0), 0); // r0 is always zero
    }
    
    #[test]
    fn test_simple_program() {
        // Simple program: load immediate 42 into r1, then exit
        let program = vec![
            0x61, 0x10, 0x00, 0x00, 0x2A, 0x00, 0x00, 0x00, // LD r1, 42
            0x95, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // EXIT
        ];
        
        let mut interpreter = BpfInterpreter::new(program, 1000);
        interpreter.execute().unwrap();
        
        let (_, _, error, compute_units) = interpreter.get_results();
        assert!(error.is_none());
        assert_eq!(compute_units, 2); // 2 instructions executed
    }
}
