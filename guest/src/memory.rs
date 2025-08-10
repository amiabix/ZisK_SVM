use crate::shared::constants::{zk_assert, ZkError};

/// Memory layout constants for ZisK zkVM
/// Optimized for RISC-V architecture and zero-knowledge proofs
pub const MEMORY_SIZE: usize = 1024 * 256; // 256KB total memory
pub const STACK_SIZE: usize = 1024 * 64;   // 64KB stack
pub const HEAP_SIZE: usize = 1024 * 128;   // 128KB heap
pub const CODE_SIZE: usize = 1024 * 64;    // 64KB code section

/// Memory regions for ZisK optimization
#[repr(C)]
pub struct MemoryLayout {
    pub code: [u8; CODE_SIZE],
    pub stack: [u8; STACK_SIZE],
    pub heap: [u8; HEAP_SIZE],
    pub data: [u8; HEAP_SIZE],
}

impl MemoryLayout {
    pub fn new() -> Self {
        Self {
            code: [0; CODE_SIZE],
            stack: [0; STACK_SIZE],
            heap: [0; HEAP_SIZE],
            data: [0; HEAP_SIZE],
        }
    }
    
    /// Get total memory usage
    pub fn total_size(&self) -> usize {
        CODE_SIZE + STACK_SIZE + HEAP_SIZE + HEAP_SIZE
    }
    
    /// Check if address is within valid range
    pub fn is_valid_address(&self, addr: usize) -> bool {
        addr < self.total_size()
    }
    
    /// Get memory region for address
    pub fn get_region(&self, addr: usize) -> Option<&[u8]> {
        if addr < CODE_SIZE {
            Some(&self.code[addr..])
        } else if addr < CODE_SIZE + STACK_SIZE {
            Some(&self.stack[addr - CODE_SIZE..])
        } else if addr < CODE_SIZE + STACK_SIZE + HEAP_SIZE {
            Some(&self.heap[addr - CODE_SIZE - STACK_SIZE..])
        } else if addr < CODE_SIZE + STACK_SIZE + HEAP_SIZE + HEAP_SIZE {
            Some(&self.data[addr - CODE_SIZE - STACK_SIZE - HEAP_SIZE..])
        } else {
            None
        }
    }
    
    /// Get mutable memory region for address
    pub fn get_region_mut(&mut self, addr: usize) -> Option<&mut [u8]> {
        if addr < CODE_SIZE {
            Some(&mut self.code[addr..])
        } else if addr < CODE_SIZE + STACK_SIZE {
            Some(&mut self.stack[addr - CODE_SIZE..])
        } else if addr < CODE_SIZE + HEAP_SIZE {
            Some(&mut self.heap[addr - CODE_SIZE - STACK_SIZE..])
        } else if addr < CODE_SIZE + STACK_SIZE + HEAP_SIZE + HEAP_SIZE {
            Some(&mut self.data[addr - CODE_SIZE - STACK_SIZE - HEAP_SIZE..])
        } else {
            None
        }
    }
}

/// Account state with 32-byte alignment for RISC-V optimization
#[repr(align(32))]
pub struct AccountState {
    pub key: [u8; 32],
    pub lamports: u64,
    pub owner: [u8; 32],
    pub executable: bool,
    pub rent_epoch: u64,
    pub data_len: u32,
    pub data: [u8; 128], // Fixed size for ZK optimization
}

impl AccountState {
    pub fn new(key: [u8; 32], owner: [u8; 32]) -> Self {
        Self {
            key,
            lamports: 0,
            owner,
            executable: false,
            rent_epoch: 0,
            data_len: 0,
            data: [0; 128],
        }
    }
    
    /// Set account data with bounds checking
    pub fn set_data(&mut self, data: &[u8]) -> Result<(), ZkError> {
        zk_assert!(data.len() <= 128, ZkError::InvalidInput);
        
        self.data_len = data.len() as u32;
        self.data[..data.len()].copy_from_slice(data);
        
        Ok(())
    }
    
    /// Get account data
    pub fn get_data(&self) -> &[u8] {
        &self.data[..self.data_len as usize]
    }
    
    /// Check if account has sufficient lamports
    pub fn has_sufficient_lamports(&self, amount: u64) -> bool {
        self.lamports >= amount
    }
    
    /// Transfer lamports to another account
    pub fn transfer_lamports(&mut self, to: &mut AccountState, amount: u64) -> Result<(), ZkError> {
        zk_assert!(self.has_sufficient_lamports(amount), ZkError::InvalidInput);
        zk_assert!(amount > 0, ZkError::InvalidInput);
        
        self.lamports -= amount;
        to.lamports += amount;
        
        Ok(())
    }
    
    /// Get serialized size for ZK input
    pub fn serialized_size(&self) -> usize {
        32 + 8 + 32 + 1 + 8 + 4 + self.data_len as usize
    }
}

/// BPF memory with proper alignment
#[repr(align(32))]
pub struct BpfMemory {
    pub data: [u8; MEMORY_SIZE],
    pub stack_pointer: usize,
    pub heap_pointer: usize,
}

impl BpfMemory {
    pub fn new() -> Self {
        Self {
            data: [0; MEMORY_SIZE],
            stack_pointer: MEMORY_SIZE - STACK_SIZE,
            heap_pointer: CODE_SIZE,
        }
    }
    
    /// Allocate memory from heap
    pub fn allocate(&mut self, size: usize) -> Result<usize, ZkError> {
        zk_assert!(size > 0, ZkError::InvalidInput);
        zk_assert!(self.heap_pointer + size <= self.stack_pointer, ZkError::MemoryOutOfBounds);
        
        let addr = self.heap_pointer;
        self.heap_pointer += size;
        
        Ok(addr)
    }
    
    /// Push to stack
    pub fn push(&mut self, value: u64) -> Result<(), ZkError> {
        zk_assert!(self.stack_pointer >= 8, ZkError::StackOverflow);
        
        self.stack_pointer -= 8;
        let bytes = value.to_le_bytes();
        self.data[self.stack_pointer..self.stack_pointer + 8].copy_from_slice(&bytes);
        
        Ok(())
    }
    
    /// Pop from stack
    pub fn pop(&mut self) -> Result<u64, ZkError> {
        zk_assert!(self.stack_pointer + 8 <= MEMORY_SIZE, ZkError::StackUnderflow);
        
        let bytes = self.data[self.stack_pointer..self.stack_pointer + 8].try_into()
            .map_err(|_| ZkError::InvalidInput)?;
        let value = u64::from_le_bytes(bytes);
        self.stack_pointer += 8;
        
        Ok(value)
    }
    
    /// Read memory at address
    pub fn read_u8(&self, addr: usize) -> Result<u8, ZkError> {
        zk_assert!(addr < MEMORY_SIZE, ZkError::MemoryOutOfBounds);
        Ok(self.data[addr])
    }
    
    /// Read memory at address (16-bit)
    pub fn read_u16(&self, addr: usize) -> Result<u16, ZkError> {
        zk_assert!(addr + 1 < MEMORY_SIZE, ZkError::MemoryOutOfBounds);
        let bytes = [self.data[addr], self.data[addr + 1]];
        Ok(u16::from_le_bytes(bytes))
    }
    
    /// Read memory at address (32-bit)
    pub fn read_u32(&self, addr: usize) -> Result<u32, ZkError> {
        zk_assert!(addr + 3 < MEMORY_SIZE, ZkError::MemoryOutOfBounds);
        let bytes = [
            self.data[addr],
            self.data[addr + 1],
            self.data[addr + 2],
            self.data[addr + 3],
        ];
        Ok(u32::from_le_bytes(bytes))
    }
    
    /// Read memory at address (64-bit)
    pub fn read_u64(&self, addr: usize) -> Result<u64, ZkError> {
        zk_assert!(addr + 7 < MEMORY_SIZE, ZkError::MemoryOutOfBounds);
        let bytes = [
            self.data[addr],
            self.data[addr + 1],
            self.data[addr + 2],
            self.data[addr + 3],
            self.data[addr + 4],
            self.data[addr + 5],
            self.data[addr + 6],
            self.data[addr + 7],
        ];
        Ok(u64::from_le_bytes(bytes))
    }
    
    /// Write memory at address
    pub fn write_u8(&mut self, addr: usize, value: u8) -> Result<(), ZkError> {
        zk_assert!(addr < MEMORY_SIZE, ZkError::MemoryOutOfBounds);
        self.data[addr] = value;
        Ok(())
    }
    
    /// Write memory at address (16-bit)
    pub fn write_u16(&mut self, addr: usize, value: u16) -> Result<(), ZkError> {
        zk_assert!(addr + 1 < MEMORY_SIZE, ZkError::MemoryOutOfBounds);
        let bytes = value.to_le_bytes();
        self.data[addr..addr + 2].copy_from_slice(&bytes);
        Ok(())
    }
    
    /// Write memory at address (32-bit)
    pub fn write_u32(&mut self, addr: usize, value: u32) -> Result<(), ZkError> {
        zk_assert!(addr + 3 < MEMORY_SIZE, ZkError::MemoryOutOfBounds);
        let bytes = value.to_le_bytes();
        self.data[addr..addr + 4].copy_from_slice(&bytes);
        Ok(())
    }
    
    /// Write memory at address (64-bit)
    pub fn write_u64(&mut self, addr: usize, value: u64) -> Result<(), ZkError> {
        zk_assert!(addr + 7 < MEMORY_SIZE, ZkError::MemoryOutOfBounds);
        let bytes = value.to_le_bytes();
        self.data[addr..addr + 8].copy_from_slice(&bytes);
        Ok(())
    }
    
    /// Get memory usage statistics
    pub fn get_usage_stats(&self) -> MemoryStats {
        MemoryStats {
            total_size: MEMORY_SIZE,
            code_used: self.heap_pointer - CODE_SIZE,
            heap_used: self.heap_pointer - CODE_SIZE,
            stack_used: MEMORY_SIZE - self.stack_pointer,
            free_space: self.stack_pointer - self.heap_pointer,
        }
    }
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_size: usize,
    pub code_used: usize,
    pub heap_used: usize,
    pub stack_used: usize,
    pub free_space: usize,
}

impl MemoryStats {
    /// Get memory utilization percentage
    pub fn utilization_percent(&self) -> f64 {
        let used = self.code_used + self.heap_used + self.stack_used;
        (used as f64 / self.total_size as f64) * 100.0
    }
    
    /// Check if memory is critically low
    pub fn is_critical(&self) -> bool {
        self.free_space < 1024 // Less than 1KB free
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_layout_creation() {
        let layout = MemoryLayout::new();
        assert_eq!(layout.total_size(), MEMORY_SIZE);
        assert!(layout.is_valid_address(0));
        assert!(layout.is_valid_address(MEMORY_SIZE - 1));
        assert!(!layout.is_valid_address(MEMORY_SIZE));
    }
    
    #[test]
    fn test_account_state_alignment() {
        let account = AccountState::new([0x01; 32], [0x02; 32]);
        assert_eq!(std::mem::size_of::<AccountState>() % 32, 0);
        assert_eq!(account.serialized_size(), 32 + 8 + 32 + 1 + 8 + 4);
    }
    
    #[test]
    fn test_bpf_memory_operations() {
        let mut memory = BpfMemory::new();
        
        // Test allocation
        let addr = memory.allocate(16).unwrap();
        assert_eq!(addr, CODE_SIZE);
        
        // Test read/write
        memory.write_u32(addr, 0x12345678).unwrap();
        let value = memory.read_u32(addr).unwrap();
        assert_eq!(value, 0x12345678);
        
        // Test stack operations
        memory.push(0xDEADBEEF).unwrap();
        let popped = memory.pop().unwrap();
        assert_eq!(popped, 0xDEADBEEF);
    }
    
    #[test]
    fn test_memory_bounds_checking() {
        let memory = BpfMemory::new();
        
        // Test out of bounds read
        let result = memory.read_u8(MEMORY_SIZE);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ZkError::MemoryOutOfBounds);
    }
}
