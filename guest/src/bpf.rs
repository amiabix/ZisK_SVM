use crate::shared::constants::{MAX_CYCLES, OP_CYCLES, zk_assert, ZkError};

/// BPF Virtual Machine for ZisK zkVM
/// Optimized for zero-knowledge proof generation
pub struct BpfVm {
    pub registers: [u64; 10],
    pub memory: [u8; 1024 * 256], // 256KB memory
    pub pc: usize,
    pub remaining_cycles: u32,
    pub stack: Vec<u64>,
    pub stack_pointer: usize,
}

impl BpfVm {
    pub fn new() -> Self {
        Self {
            registers: [0; 10],
            memory: [0; 1024 * 256],
            pc: 0,
            remaining_cycles: MAX_CYCLES,
            stack: Vec::with_capacity(1024),
            stack_pointer: 0,
        }
    }
    
    pub fn execute(&mut self, program: &[u8]) -> Result<(), ZkError> {
        while self.pc < program.len() && self.remaining_cycles > 0 {
            let opcode = program[self.pc];
            let cycles_needed = OP_CYCLES[opcode as usize];
            
            // Critical: Check cycle availability before execution
            zk_assert!(self.remaining_cycles >= cycles_needed, ZkError::InsufficientCycles);
            self.remaining_cycles -= cycles_needed;
            
            self.execute_opcode(opcode, program)?;
            self.pc += 1;
        }
        
        if self.remaining_cycles == 0 {
            return Err(ZkError::InsufficientCycles);
        }
        
        Ok(())
    }
    
    fn execute_opcode(&mut self, opcode: u8, program: &[u8]) -> Result<(), ZkError> {
        match opcode {
            // Arithmetic operations
            0x00 => self.op_add_reg()?,
            0x01 => self.op_sub_reg()?,
            0x02 => self.op_mul_reg()?,
            0x03 => self.op_div_reg()?,
            0x07 => self.op_add_imm()?,
            0x17 => self.op_sub_imm()?,
            0x27 => self.op_mul_imm()?,
            0x37 => self.op_div_imm()?,
            
            // Bitwise operations
            0x5F => self.op_and_reg()?,
            0x6F => self.op_or_reg()?,
            0x7F => self.op_xor_reg()?,
            0x6C => self.op_lsh_reg()?,
            0x7C => self.op_rsh_reg()?,
            0x54 => self.op_and_imm()?,
            0x64 => self.op_or_imm()?,
            0x74 => self.op_xor_imm()?,
            0x6D => self.op_lsh_imm()?,
            0x7D => self.op_rsh_imm()?,
            
            // Load operations
            0x30 => self.op_ld_abs_b()?,
            0x28 => self.op_ld_abs_h()?,
            0x20 => self.op_ld_abs_w()?,
            0x18 => self.op_ld_abs_dw()?,
            0x50 => self.op_ld_ind_b()?,
            0x48 => self.op_ld_ind_h()?,
            0x40 => self.op_ld_ind_w()?,
            0x38 => self.op_ld_ind_dw()?,
            
            // Store operations
            0x62 => self.op_st_reg()?,
            0x63 => self.op_st_reg_imm()?,
            
            // Comparison and jumps
            0x1D => self.op_jeq_reg()?,
            0x5D => self.op_jne_reg()?,
            0x2D => self.op_jgt_reg()?,
            0x3D => self.op_jge_reg()?,
            0xAD => self.op_jlt_reg()?,
            0xBD => self.op_jle_reg()?,
            0x15 => self.op_jeq_imm()?,
            0x55 => self.op_jne_imm()?,
            0x25 => self.op_jgt_imm()?,
            0x35 => self.op_jge_imm()?,
            0xA5 => self.op_jlt_imm()?,
            0xB5 => self.op_jle_imm()?,
            0x05 => self.op_ja()?,
            
            // Control flow
            0x85 => self.op_call()?,
            0x95 => self.op_exit()?,
            
            // Solana-specific operations
            0xE0 => self.op_sol_call()?,
            0xE1 => self.op_sol_log()?,
            0xE2 => self.op_sol_return()?,
            
            _ => return Err(ZkError::InvalidOpcode),
        }
        
        Ok(())
    }
    
    // Arithmetic operations
    fn op_add_reg(&mut self) -> Result<(), ZkError> {
        let dst = self.read_register(1)?;
        let src = self.read_register(2)?;
        self.write_register(0, dst.wrapping_add(src))?;
        Ok(())
    }
    
    fn op_sub_reg(&mut self) -> Result<(), ZkError> {
        let dst = self.read_register(1)?;
        let src = self.read_register(2)?;
        self.write_register(0, dst.wrapping_sub(src))?;
        Ok(())
    }
    
    fn op_mul_reg(&mut self) -> Result<(), ZkError> {
        let dst = self.read_register(1)?;
        let src = self.read_register(2)?;
        self.write_register(0, dst.wrapping_mul(src))?;
        Ok(())
    }
    
    fn op_div_reg(&mut self) -> Result<(), ZkError> {
        let dst = self.read_register(1)?;
        let src = self.read_register(2)?;
        if src == 0 {
            return Err(ZkError::InvalidInput);
        }
        self.write_register(0, dst / src)?;
        Ok(())
    }
    
    fn op_add_imm(&mut self) -> Result<(), ZkError> {
        let dst = self.read_register(1)?;
        let imm = self.read_immediate()?;
        self.write_register(0, dst.wrapping_add(imm))?;
        Ok(())
    }
    
    fn op_sub_imm(&mut self) -> Result<(), ZkError> {
        let dst = self.read_register(1)?;
        let imm = self.read_immediate()?;
        self.write_register(0, dst.wrapping_sub(imm))?;
        Ok(())
    }
    
    fn op_mul_imm(&mut self) -> Result<(), ZkError> {
        let dst = self.read_register(1)?;
        let imm = self.read_immediate()?;
        self.write_register(0, dst.wrapping_mul(imm))?;
        Ok(())
    }
    
    fn op_div_imm(&mut self) -> Result<(), ZkError> {
        let dst = self.read_register(1)?;
        let imm = self.read_immediate()?;
        if imm == 0 {
            return Err(ZkError::InvalidInput);
        }
        self.write_register(0, dst / imm)?;
        Ok(())
    }
    
    // Bitwise operations
    fn op_and_reg(&mut self) -> Result<(), ZkError> {
        let dst = self.read_register(1)?;
        let src = self.read_register(2)?;
        self.write_register(0, dst & src)?;
        Ok(())
    }
    
    fn op_or_reg(&mut self) -> Result<(), ZkError> {
        let dst = self.read_register(1)?;
        let src = self.read_register(2)?;
        self.write_register(0, dst | src)?;
        Ok(())
    }
    
    fn op_xor_reg(&mut self) -> Result<(), ZkError> {
        let dst = self.read_register(1)?;
        let src = self.read_register(2)?;
        self.write_register(0, dst ^ src)?;
        Ok(())
    }
    
    fn op_lsh_reg(&mut self) -> Result<(), ZkError> {
        let dst = self.read_register(1)?;
        let src = self.read_register(2)?;
        self.write_register(0, dst << (src % 64))?;
        Ok(())
    }
    
    fn op_rsh_reg(&mut self) -> Result<(), ZkError> {
        let dst = self.read_register(1)?;
        let src = self.read_register(2)?;
        self.write_register(0, dst >> (src % 64))?;
        Ok(())
    }
    
    fn op_and_imm(&mut self) -> Result<(), ZkError> {
        let dst = self.read_register(1)?;
        let imm = self.read_immediate()?;
        self.write_register(0, dst & imm)?;
        Ok(())
    }
    
    fn op_or_imm(&mut self) -> Result<(), ZkError> {
        let dst = self.read_register(1)?;
        let imm = self.read_immediate()?;
        self.write_register(0, dst | imm)?;
        Ok(())
    }
    
    fn op_xor_imm(&mut self) -> Result<(), ZkError> {
        let dst = self.read_register(1)?;
        let imm = self.read_immediate()?;
        self.write_register(0, dst ^ imm)?;
        Ok(())
    }
    
    fn op_lsh_imm(&mut self) -> Result<(), ZkError> {
        let dst = self.read_register(1)?;
        let imm = self.read_immediate()?;
        self.write_register(0, dst << (imm % 64))?;
        Ok(())
    }
    
    fn op_rsh_imm(&mut self) -> Result<(), ZkError> {
        let dst = self.read_register(1)?;
        let imm = self.read_immediate()?;
        self.write_register(0, dst >> (imm % 64))?;
        Ok(())
    }
    
    // Load operations
    fn op_ld_abs_b(&mut self) -> Result<(), ZkError> {
        let offset = self.read_immediate()? as usize;
        zk_assert!(offset < self.memory.len(), ZkError::MemoryOutOfBounds);
        let value = self.memory[offset] as u64;
        self.write_register(0, value)?;
        Ok(())
    }
    
    fn op_ld_abs_h(&mut self) -> Result<(), ZkError> {
        let offset = self.read_immediate()? as usize;
        zk_assert!(offset + 1 < self.memory.len(), ZkError::MemoryOutOfBounds);
        let value = u16::from_le_bytes([self.memory[offset], self.memory[offset + 1]]) as u64;
        self.write_register(0, value)?;
        Ok(())
    }
    
    fn op_ld_abs_w(&mut self) -> Result<(), ZkError> {
        let offset = self.read_immediate()? as usize;
        zk_assert!(offset + 3 < self.memory.len(), ZkError::MemoryOutOfBounds);
        let value = u32::from_le_bytes([
            self.memory[offset],
            self.memory[offset + 1],
            self.memory[offset + 2],
            self.memory[offset + 3],
        ]) as u64;
        self.write_register(0, value)?;
        Ok(())
    }
    
    fn op_ld_abs_dw(&mut self) -> Result<(), ZkError> {
        let offset = self.read_immediate()? as usize;
        zk_assert!(offset + 7 < self.memory.len(), ZkError::MemoryOutOfBounds);
        let value = u64::from_le_bytes([
            self.memory[offset],
            self.memory[offset + 1],
            self.memory[offset + 2],
            self.memory[offset + 3],
            self.memory[offset + 4],
            self.memory[offset + 5],
            self.memory[offset + 6],
            self.memory[offset + 7],
        ]);
        self.write_register(0, value)?;
        Ok(())
    }
    
    // Store operations
    fn op_st_reg(&mut self) -> Result<(), ZkError> {
        let value = self.read_register(1)?;
        let offset = self.read_register(2)? as usize;
        zk_assert!(offset < self.memory.len(), ZkError::MemoryOutOfBounds);
        self.memory[offset] = value as u8;
        Ok(())
    }
    
    fn op_st_reg_imm(&mut self) -> Result<(), ZkError> {
        let value = self.read_register(1)?;
        let offset = self.read_immediate()? as usize;
        zk_assert!(offset < self.memory.len(), ZkError::MemoryOutOfBounds);
        self.memory[offset] = value as u8;
        Ok(())
    }
    
    // Jump operations
    fn op_jeq_reg(&mut self) -> Result<(), ZkError> {
        let a = self.read_register(1)?;
        let b = self.read_register(2)?;
        if a == b {
            let offset = self.read_immediate()? as i64;
            self.pc = (self.pc as i64 + offset) as usize;
        }
        Ok(())
    }
    
    fn op_jne_reg(&mut self) -> Result<(), ZkError> {
        let a = self.read_register(1)?;
        let b = self.read_register(2)?;
        if a != b {
            let offset = self.read_immediate()? as i64;
            self.pc = (self.pc as i64 + offset) as usize;
        }
        Ok(())
    }
    
    fn op_jgt_reg(&mut self) -> Result<(), ZkError> {
        let a = self.read_register(1)?;
        let b = self.read_register(2)?;
        if a > b {
            let offset = self.read_immediate()? as i64;
            self.pc = (self.pc as i64 + offset) as usize;
        }
        Ok(())
    }
    
    fn op_jge_reg(&mut self) -> Result<(), ZkError> {
        let a = self.read_register(1)?;
        let b = self.read_register(2)?;
        if a >= b {
            let offset = self.read_immediate()? as i64;
            self.pc = (self.pc as i64 + offset) as usize;
        }
        Ok(())
    }
    
    fn op_jlt_reg(&mut self) -> Result<(), ZkError> {
        let a = self.read_register(1)?;
        let b = self.read_register(2)?;
        if a < b {
            let offset = self.read_immediate()? as i64;
            self.pc = (self.pc as i64 + offset) as usize;
        }
        Ok(())
    }
    
    fn op_jle_reg(&mut self) -> Result<(), ZkError> {
        let a = self.read_register(1)?;
        let b = self.read_register(2)?;
        if a <= b {
            let offset = self.read_immediate()? as i64;
            self.pc = (self.pc as i64 + offset) as usize;
        }
        Ok(())
    }
    
    fn op_jeq_imm(&mut self) -> Result<(), ZkError> {
        let a = self.read_register(1)?;
        let imm = self.read_immediate()?;
        if a == imm {
            let offset = self.read_immediate()? as i64;
            self.pc = (self.pc as i64 + offset) as usize;
        }
        Ok(())
    }
    
    fn op_jne_imm(&mut self) -> Result<(), ZkError> {
        let a = self.read_register(1)?;
        let imm = self.read_immediate()?;
        if a != imm {
            let offset = self.read_immediate()? as i64;
            self.pc = (self.pc as i64 + offset) as usize;
        }
        Ok(())
    }
    
    fn op_jgt_imm(&mut self) -> Result<(), ZkError> {
        let a = self.read_register(1)?;
        let imm = self.read_immediate()?;
        if a > imm {
            let offset = self.read_immediate()? as i64;
            self.pc = (self.pc as i64 + offset) as usize;
        }
        Ok(())
    }
    
    fn op_jge_imm(&mut self) -> Result<(), ZkError> {
        let a = self.read_register(1)?;
        let imm = self.read_immediate()?;
        if a >= imm {
            let offset = self.read_immediate()? as i64;
            self.pc = (self.pc as i64 + offset) as usize;
        }
        Ok(())
    }
    
    fn op_jlt_imm(&mut self) -> Result<(), ZkError> {
        let a = self.read_register(1)?;
        let imm = self.read_immediate()?;
        if a < imm {
            let offset = self.read_immediate()? as i64;
            self.pc = (self.pc as i64 + offset) as usize;
        }
        Ok(())
    }
    
    fn op_jle_imm(&mut self) -> Result<(), ZkError> {
        let a = self.read_register(1)?;
        let imm = self.read_immediate()?;
        if a <= imm {
            let offset = self.read_immediate()? as i64;
            self.pc = (self.pc as i64 + offset) as usize;
        }
        Ok(())
    }
    
    fn op_ja(&mut self) -> Result<(), ZkError> {
        let offset = self.read_immediate()? as i64;
        self.pc = (self.pc as i64 + offset) as usize;
        Ok(())
    }
    
    // Control flow
    fn op_call(&mut self) -> Result<(), ZkError> {
        zk_assert!(self.stack.len() < 1024, ZkError::StackOverflow);
        self.stack.push(self.pc as u64);
        let target = self.read_immediate()? as usize;
        self.pc = target;
        Ok(())
    }
    
    fn op_exit(&mut self) -> Result<(), ZkError> {
        if let Some(return_pc) = self.stack.pop() {
            self.pc = return_pc as usize;
        } else {
            return Err(ZkError::StackUnderflow);
        }
        Ok(())
    }
    
    // Solana-specific operations
    fn op_sol_call(&mut self) -> Result<(), ZkError> {
        // Cross-program invocation - high gas cost
        const CPI_COST: u32 = 5000;
        zk_assert!(self.remaining_cycles >= CPI_COST, ZkError::InsufficientCycles);
        self.remaining_cycles -= CPI_COST;
        
        // TODO: Implement actual CPI logic
        Ok(())
    }
    
    fn op_sol_log(&mut self) -> Result<(), ZkError> {
        // Log operation - low gas cost
        const LOG_COST: u32 = 100;
        zk_assert!(self.remaining_cycles >= LOG_COST, ZkError::InsufficientCycles);
        self.remaining_cycles -= LOG_COST;
        
        // TODO: Implement actual logging logic
        Ok(())
    }
    
    fn op_sol_return(&mut self) -> Result<(), ZkError> {
        // Return operation - low gas cost
        const RETURN_COST: u32 = 50;
        zk_assert!(self.remaining_cycles >= RETURN_COST, ZkError::InsufficientCycles);
        self.remaining_cycles -= RETURN_COST;
        
        // TODO: Implement actual return logic
        Ok(())
    }
    
    // Helper methods
    fn read_register(&self, reg: u8) -> Result<u64, ZkError> {
        zk_assert!(reg < 10, ZkError::InvalidInput);
        Ok(self.registers[reg as usize])
    }
    
    fn write_register(&mut self, reg: u8, value: u64) -> Result<(), ZkError> {
        zk_assert!(reg < 10, ZkError::InvalidInput);
        self.registers[reg as usize] = value;
        Ok(())
    }
    
    fn read_immediate(&mut self) -> Result<u64, ZkError> {
        // For simplicity, read next 8 bytes as immediate
        // In real implementation, this would parse instruction format
        zk_assert!(self.pc + 7 < self.memory.len(), ZkError::MemoryOutOfBounds);
        
        let mut bytes = [0u8; 8];
        for i in 0..8 {
            bytes[i] = self.memory[self.pc + i];
        }
        
        Ok(u64::from_le_bytes(bytes))
    }
    
    pub fn get_remaining_cycles(&self) -> u32 {
        self.remaining_cycles
    }
    
    pub fn get_memory_usage(&self) -> usize {
        self.memory.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vm_creation() {
        let vm = BpfVm::new();
        assert_eq!(vm.remaining_cycles, MAX_CYCLES);
        assert_eq!(vm.memory.len(), 1024 * 256);
        assert_eq!(vm.registers.len(), 10);
    }
    
    #[test]
    fn test_cycle_accounting() {
        let mut vm = BpfVm::new();
        let initial_cycles = vm.remaining_cycles;
        
        // Simple program: ADD operation (cost: 1)
        let program = vec![0x00]; // ADD opcode
        
        let result = vm.execute(&program);
        assert!(result.is_ok());
        assert_eq!(vm.remaining_cycles, initial_cycles - 1);
    }
    
    #[test]
    fn test_insufficient_cycles() {
        let mut vm = BpfVm::new();
        vm.remaining_cycles = 1; // Only 1 cycle left
        
        // Program that needs more cycles
        let program = vec![0x00, 0x00]; // Two ADD operations
        
        let result = vm.execute(&program);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ZkError::InsufficientCycles);
    }
}
