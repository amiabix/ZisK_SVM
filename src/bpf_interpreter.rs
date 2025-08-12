use crate::types::{BpfInstruction, BpfOpcode, BpfProgram};
use crate::error::{InterpreterError, TranspilerError};
use std::collections::HashMap;

/// BPF interpreter that runs natively in ZisK
pub struct BpfInterpreter {
    registers: [u64; 11],        // BPF registers R0-R10
    memory: Vec<u8>,             // Memory space for BPF operations
    program_counter: usize,      // Current instruction pointer
    max_memory: usize,           // Maximum memory size
}

impl BpfInterpreter {
    /// Create a new BPF interpreter
    pub fn new() -> Self {
        Self {
            registers: [0; 11],
            memory: vec![0; 1024 * 1024], // 1MB memory
            program_counter: 0,
            max_memory: 1024 * 1024,
        }
    }

    /// Reset interpreter state
    pub fn reset(&mut self) {
        self.registers = [0; 11];
        self.memory = vec![0; self.max_memory];
        self.program_counter = 0;
    }

    /// Get current register values
    pub fn get_registers(&self) -> [u64; 11] {
        self.registers
    }

    /// Set register value
    pub fn set_register(&mut self, reg: u8, value: u64) -> Result<(), TranspilerError> {
        if reg > 10 {
            return Err(TranspilerError::InterpreterError(InterpreterError::InvalidRegister { register: reg }));
        }
        self.registers[reg as usize] = value;
        Ok(())
    }

    /// Get register value
    pub fn get_register(&self, reg: u8) -> Result<u64, TranspilerError> {
        if reg > 10 {
            return Err(TranspilerError::InterpreterError(InterpreterError::InvalidRegister { register: reg }));
        }
        Ok(self.registers[reg as usize])
    }

    /// Read memory at address
    pub fn read_memory(&self, address: usize, size: usize) -> Result<&[u8], TranspilerError> {
        if address + size > self.memory.len() {
            return Err(TranspilerError::InterpreterError(InterpreterError::MemoryAccessViolation { 
                address, 
                size, 
                max_address: self.memory.len() 
            }));
        }
        Ok(&self.memory[address..address + size])
    }

    /// Write memory at address
    pub fn write_memory(&mut self, address: usize, data: &[u8]) -> Result<(), TranspilerError> {
        if address + data.len() > self.memory.len() {
            return Err(TranspilerError::InterpreterError(InterpreterError::MemoryAccessViolation { 
                address, 
                size: data.len(), 
                max_address: self.memory.len() 
            }));
        }
        self.memory[address..address + data.len()].copy_from_slice(data);
        Ok(())
    }

    /// Execute a single BPF instruction
    pub fn execute_instruction(&mut self, instruction: &BpfInstruction) -> Result<(), TranspilerError> {
        match instruction.opcode {
            // ALU Operations
            BpfOpcode::Add64Imm => {
                let dst = instruction.dst_reg;
                let value = self.get_register(dst)?;
                let result = value.wrapping_add(instruction.immediate as u64);
                self.set_register(dst, result)?;
            }
            
            BpfOpcode::Add64Reg => {
                let dst = instruction.dst_reg;
                let src = instruction.src_reg;
                let dst_val = self.get_register(dst)?;
                let src_val = self.get_register(src)?;
                let result = dst_val.wrapping_add(src_val);
                self.set_register(dst, result)?;
            }
            
            BpfOpcode::Sub64Imm => {
                let dst = instruction.dst_reg;
                let value = self.get_register(dst)?;
                let result = value.wrapping_sub(instruction.immediate as u64);
                self.set_register(dst, result)?;
            }
            
            BpfOpcode::Sub64Reg => {
                let dst = instruction.dst_reg;
                let src = instruction.src_reg;
                let dst_val = self.get_register(dst)?;
                let src_val = self.get_register(src)?;
                let result = dst_val.wrapping_sub(src_val);
                self.set_register(dst, result)?;
            }
            
            BpfOpcode::Mul64Imm => {
                let dst = instruction.dst_reg;
                let value = self.get_register(dst)?;
                let result = value.wrapping_mul(instruction.immediate as u64);
                self.set_register(dst, result)?;
            }
            
            BpfOpcode::Mul64Reg => {
                let dst = instruction.dst_reg;
                let src = instruction.src_reg;
                let dst_val = self.get_register(dst)?;
                let src_val = self.get_register(src)?;
                let result = dst_val.wrapping_mul(src_val);
                self.set_register(dst, result)?;
            }
            
            BpfOpcode::Div64Imm => {
                let dst = instruction.dst_reg;
                let value = self.get_register(dst)?;
                let divisor = instruction.immediate as u64;
                if divisor == 0 {
                    return Err(TranspilerError::InterpreterError(InterpreterError::DivisionByZero));
                }
                let result = value / divisor;
                self.set_register(dst, result)?;
            }
            
            BpfOpcode::Div64Reg => {
                let dst = instruction.dst_reg;
                let src = instruction.src_reg;
                let dst_val = self.get_register(dst)?;
                let src_val = self.get_register(src)?;
                if src_val == 0 {
                    return Err(TranspilerError::InterpreterError(InterpreterError::DivisionByZero));
                }
                let result = dst_val / src_val;
                self.set_register(dst, result)?;
            }
            
            BpfOpcode::Mod64Imm => {
                let dst = instruction.dst_reg;
                let value = self.get_register(dst)?;
                let divisor = instruction.immediate as u64;
                if divisor == 0 {
                    return Err(TranspilerError::InterpreterError(InterpreterError::DivisionByZero));
                }
                let result = value % divisor;
                self.set_register(dst, result)?;
            }
            
            BpfOpcode::Mod64Reg => {
                let dst = instruction.dst_reg;
                let src = instruction.src_reg;
                let dst_val = self.get_register(dst)?;
                let src_val = self.get_register(src)?;
                if src_val == 0 {
                    return Err(TranspilerError::InterpreterError(InterpreterError::DivisionByZero));
                }
                let result = dst_val % src_val;
                self.set_register(dst, result)?;
            }
            
            BpfOpcode::And64Imm => {
                let dst = instruction.dst_reg;
                let value = self.get_register(dst)?;
                let result = value & (instruction.immediate as u64);
                self.set_register(dst, result)?;
            }
            
            BpfOpcode::And64Reg => {
                let dst = instruction.dst_reg;
                let src = instruction.src_reg;
                let dst_val = self.get_register(dst)?;
                let src_val = self.get_register(src)?;
                let result = dst_val & src_val;
                self.set_register(dst, result)?;
            }
            
            BpfOpcode::Or64Imm => {
                let dst = instruction.dst_reg;
                let value = self.get_register(dst)?;
                let result = value | (instruction.immediate as u64);
                self.set_register(dst, result)?;
            }
            
            BpfOpcode::Or64Reg => {
                let dst = instruction.dst_reg;
                let src = instruction.src_reg;
                let dst_val = self.get_register(dst)?;
                let src_val = self.get_register(src)?;
                let result = dst_val | src_val;
                self.set_register(dst, result)?;
            }
            
            BpfOpcode::Xor64Imm => {
                let dst = instruction.dst_reg;
                let value = self.get_register(dst)?;
                let result = value ^ (instruction.immediate as u64);
                self.set_register(dst, result)?;
            }
            
            BpfOpcode::Xor64Reg => {
                let dst = instruction.dst_reg;
                let src = instruction.src_reg;
                let dst_val = self.get_register(dst)?;
                let src_val = self.get_register(src)?;
                let result = dst_val ^ src_val;
                self.set_register(dst, result)?;
            }
            
            BpfOpcode::Lsh64Imm => {
                let dst = instruction.dst_reg;
                let value = self.get_register(dst)?;
                let shift = (instruction.immediate as u64) % 64;
                let result = value << shift;
                self.set_register(dst, result)?;
            }
            
            BpfOpcode::Lsh64Reg => {
                let dst = instruction.dst_reg;
                let src = instruction.src_reg;
                let dst_val = self.get_register(dst)?;
                let src_val = self.get_register(src)?;
                let shift = src_val % 64;
                let result = dst_val << shift;
                self.set_register(dst, result)?;
            }
            
            BpfOpcode::Rsh64Imm => {
                let dst = instruction.dst_reg;
                let value = self.get_register(dst)?;
                let shift = (instruction.immediate as u64) % 64;
                let result = value >> shift;
                self.set_register(dst, result)?;
            }
            
            BpfOpcode::Rsh64Reg => {
                let dst = instruction.dst_reg;
                let src = instruction.src_reg;
                let dst_val = self.get_register(dst)?;
                let src_val = self.get_register(src)?;
                let shift = src_val % 64;
                let result = dst_val >> shift;
                self.set_register(dst, result)?;
            }
            
            BpfOpcode::Neg64 => {
                let dst = instruction.dst_reg;
                let value = self.get_register(dst)?;
                let result = value.wrapping_neg();
                self.set_register(dst, result)?;
            }
            
            BpfOpcode::Mov64Imm => {
                let dst = instruction.dst_reg;
                let value = instruction.immediate as u64;
                self.set_register(dst, value)?;
            }
            
            BpfOpcode::Mov64Reg => {
                let dst = instruction.dst_reg;
                let src = instruction.src_reg;
                let value = self.get_register(src)?;
                self.set_register(dst, value)?;
            }
            
            // Memory Operations
            BpfOpcode::LdImm64 => {
                let dst = instruction.dst_reg;
                let value = instruction.immediate as u64;
                self.set_register(dst, value)?;
            }
            
            BpfOpcode::LdAbs8 => {
                let dst = instruction.dst_reg;
                let address = instruction.offset as usize;
                let data = self.read_memory(address, 1)?;
                let value = data[0] as u64;
                self.set_register(dst, value)?;
            }
            
            BpfOpcode::LdAbs16 => {
                let dst = instruction.dst_reg;
                let address = instruction.offset as usize;
                let data = self.read_memory(address, 2)?;
                let value = u16::from_le_bytes([data[0], data[1]]) as u64;
                self.set_register(dst, value)?;
            }
            
            BpfOpcode::LdAbs32 => {
                let dst = instruction.dst_reg;
                let address = instruction.offset as usize;
                let data = self.read_memory(address, 4)?;
                let value = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as u64;
                self.set_register(dst, value)?;
            }
            
            BpfOpcode::LdAbs64 => {
                let dst = instruction.dst_reg;
                let address = instruction.offset as usize;
                let data = self.read_memory(address, 8)?;
                let value = u64::from_le_bytes([
                    data[0], data[1], data[2], data[3],
                    data[4], data[5], data[6], data[7]
                ]);
                self.set_register(dst, value)?;
            }
            
            BpfOpcode::St8 => {
                let src = instruction.src_reg;
                let address = instruction.offset as usize;
                let value = self.get_register(src)? as u8;
                self.write_memory(address, &[value])?;
            }
            
            BpfOpcode::St16 => {
                let src = instruction.src_reg;
                let address = instruction.offset as usize;
                let value = self.get_register(src)? as u16;
                let bytes = value.to_le_bytes();
                self.write_memory(address, &bytes)?;
            }
            
            BpfOpcode::St32 => {
                let src = instruction.src_reg;
                let address = instruction.offset as usize;
                let value = self.get_register(src)? as u32;
                let bytes = value.to_le_bytes();
                self.write_memory(address, &bytes)?;
            }
            
            BpfOpcode::St64 => {
                let src = instruction.src_reg;
                let address = instruction.offset as usize;
                let value = self.get_register(src)?;
                let bytes = value.to_le_bytes();
                self.write_memory(address, &bytes)?;
            }
            
            // Branch Operations
            BpfOpcode::Ja => {
                let offset = instruction.offset as isize;
                self.program_counter = (self.program_counter as isize + offset) as usize;
                return Ok(()); // Skip normal PC increment
            }
            
            BpfOpcode::JeqImm => {
                let dst = instruction.dst_reg;
                let dst_val = self.get_register(dst)?;
                let imm = instruction.immediate as u64;
                if dst_val == imm {
                    let offset = instruction.offset as isize;
                    self.program_counter = (self.program_counter as isize + offset) as usize;
                    return Ok(()); // Skip normal PC increment
                }
            }
            
            BpfOpcode::JeqReg => {
                let dst = instruction.dst_reg;
                let src = instruction.src_reg;
                let dst_val = self.get_register(dst)?;
                let src_val = self.get_register(src)?;
                if dst_val == src_val {
                    let offset = instruction.offset as isize;
                    self.program_counter = (self.program_counter as isize + offset) as usize;
                    return Ok(()); // Skip normal PC increment
                }
            }
            
            BpfOpcode::Exit => {
                // Exit instruction - handled by caller
                return Ok(());
            }
            
            // Unsupported opcodes
            _ => {
                return Err(TranspilerError::InterpreterError(InterpreterError::UnsupportedOpcode { 
                    opcode: instruction.opcode as u8 
                }));
            }
        }
        
        // Increment program counter for next instruction
        self.program_counter += 1;
        Ok(())
    }

    /// Execute a complete BPF program
    pub fn execute_program(&mut self, program: &BpfProgram) -> Result<u64, TranspilerError> {
        self.reset();
        
        let mut instructions_executed = 0;
        let start_time = std::time::Instant::now();
        
        while self.program_counter < program.instructions.len() {
            let instruction = &program.instructions[self.program_counter];
            
            // Handle exit instruction
            if instruction.opcode == BpfOpcode::Exit {
                let exit_code = self.get_register(0)?; // R0 contains exit code
                return Ok(exit_code);
            }
            
            // Execute instruction
            self.execute_instruction(instruction)?;
            instructions_executed += 1;
            
            // Safety check to prevent infinite loops
            if instructions_executed > 100_000 {
                return Err(TranspilerError::InterpreterError(InterpreterError::ExecutionLimitExceeded));
            }
        }
        
        // Program completed without exit
        Ok(0)
    }
}

impl Default for BpfInterpreter {
    fn default() -> Self {
        Self::new()
    }
}
