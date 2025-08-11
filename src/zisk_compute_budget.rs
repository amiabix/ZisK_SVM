//! ZisK Compute Budget Management for Solana Program Execution
//! 
//! This module provides comprehensive compute budget tracking and translation
//! between Solana compute units and ZisK cycles, with operation-specific
//! multipliers and dynamic adjustment factors for optimal performance.

use crate::zisk_state_manager::ZisKError;
use std::collections::HashMap;

/// Maps Solana compute units to ZisK cycles with high precision
#[derive(Debug, Clone)]
pub struct ComputeBudgetTranslator {
    /// Base conversion rates
    base_cu_to_cycles: f64,
    
    /// Operation-specific multipliers
    operation_multipliers: HashMap<ComputeOperation, f64>,
    
    /// Dynamic adjustment factors
    memory_pressure_factor: f64,
    proof_complexity_factor: f64,
}

/// Types of compute operations for precise cost tracking
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum ComputeOperation {
    /// Basic operations
    AluOperation,           // Basic arithmetic
    MemoryLoad,             // Memory read
    MemoryStore,            // Memory write
    
    /// Solana-specific operations
    SyscallInvocation,      // Syscall overhead
    AccountDeserialization, // Account data parsing
    AccountSerialization,   // Account data writing
    
    /// Cryptographic operations
    Sha256Hash,             // SHA256 computation
    Ed25519Verify,          // Ed25519 signature verification
    Secp256k1Verify,        // Secp256k1 signature verification
    KeccakHash,             // Keccak256 computation
    
    /// Program operations
    ProgramInvocation,      // Program call overhead
    CpiInvocation,          // Cross-program invocation
    PdaDerivation,          // Program derived address calculation
    
    /// BPF operations
    BpfInstruction,         // Basic BPF instruction
    BpfJump,                // Branch instruction
    BpfCall,                // Function call
    BpfReturn,              // Function return
}

impl Default for ComputeBudgetTranslator {
    fn default() -> Self {
        let mut operation_multipliers = HashMap::new();
        
        // Calibrated multipliers based on ZisK performance characteristics
        operation_multipliers.insert(ComputeOperation::AluOperation, 1.0);
        operation_multipliers.insert(ComputeOperation::MemoryLoad, 1.2);
        operation_multipliers.insert(ComputeOperation::MemoryStore, 1.5);
        operation_multipliers.insert(ComputeOperation::SyscallInvocation, 10.0);
        operation_multipliers.insert(ComputeOperation::AccountDeserialization, 5.0);
        operation_multipliers.insert(ComputeOperation::AccountSerialization, 7.0);
        operation_multipliers.insert(ComputeOperation::Sha256Hash, 50.0);
        operation_multipliers.insert(ComputeOperation::Ed25519Verify, 200.0);
        operation_multipliers.insert(ComputeOperation::Secp256k1Verify, 300.0);
        operation_multipliers.insert(ComputeOperation::KeccakHash, 60.0);
        operation_multipliers.insert(ComputeOperation::ProgramInvocation, 20.0);
        operation_multipliers.insert(ComputeOperation::CpiInvocation, 40.0);
        operation_multipliers.insert(ComputeOperation::PdaDerivation, 15.0);
        operation_multipliers.insert(ComputeOperation::BpfInstruction, 1.1);
        operation_multipliers.insert(ComputeOperation::BpfJump, 1.3);
        operation_multipliers.insert(ComputeOperation::BpfCall, 2.0);
        operation_multipliers.insert(ComputeOperation::BpfReturn, 1.5);

        Self {
            base_cu_to_cycles: 2.5, // Base: 1 CU = 2.5 ZisK cycles
            operation_multipliers,
            memory_pressure_factor: 1.0,
            proof_complexity_factor: 1.0,
        }
    }
}

impl ComputeBudgetTranslator {
    /// Create new compute budget translator
    pub fn new() -> Self {
        Self::default()
    }

    /// Convert Solana compute units to ZisK cycles for a specific operation
    pub fn cu_to_cycles(&self, compute_units: u64, operation: ComputeOperation) -> u64 {
        let base_cycles = (compute_units as f64) * self.base_cu_to_cycles;
        let operation_multiplier = self.operation_multipliers
            .get(&operation)
            .copied()
            .unwrap_or(1.0);
        
        let adjusted_cycles = base_cycles 
            * operation_multiplier 
            * self.memory_pressure_factor 
            * self.proof_complexity_factor;

        adjusted_cycles as u64
    }

    /// Convert ZisK cycles back to approximate compute units
    pub fn cycles_to_cu(&self, cycles: u64) -> u64 {
        let base_cu = (cycles as f64) / self.base_cu_to_cycles;
        (base_cu / self.memory_pressure_factor / self.proof_complexity_factor) as u64
    }

    /// Update dynamic factors based on current execution context
    pub fn update_factors(&mut self, memory_usage_percent: f64, proof_complexity: ProofComplexity) {
        // Adjust for memory pressure
        self.memory_pressure_factor = 1.0 + (memory_usage_percent / 100.0) * 0.5;
        
        // Adjust for proof complexity
        self.proof_complexity_factor = match proof_complexity {
            ProofComplexity::Simple => 1.0,
            ProofComplexity::Medium => 1.2,
            ProofComplexity::Complex => 1.5,
            ProofComplexity::VeryComplex => 2.0,
        };
    }

    /// Get operation-specific cycle cost
    pub fn get_operation_cycles(&self, operation: ComputeOperation, base_cost: u64) -> u64 {
        self.cu_to_cycles(base_cost, operation)
    }

    /// Get current memory pressure factor
    pub fn get_memory_pressure_factor(&self) -> f64 {
        self.memory_pressure_factor
    }

    /// Get current proof complexity factor
    pub fn get_proof_complexity_factor(&self) -> f64 {
        self.proof_complexity_factor
    }

    /// Reset factors to default values
    pub fn reset_factors(&mut self) {
        self.memory_pressure_factor = 1.0;
        self.proof_complexity_factor = 1.0;
    }
}

/// Proof complexity levels for dynamic adjustment
#[derive(Debug, Clone, Copy)]
pub enum ProofComplexity {
    Simple,     // Basic arithmetic operations
    Medium,     // Hash operations and simple cryptography
    Complex,    // Signature verification and CPI
    VeryComplex, // Multiple CPIs with complex state changes
}

/// Tracks compute budget consumption during transaction execution
pub struct ZisKComputeTracker {
    translator: ComputeBudgetTranslator,
    total_budget_cu: u64,
    consumed_cu: u64,
    consumed_cycles: u64,
    operation_breakdown: HashMap<ComputeOperation, u64>,
}

impl ZisKComputeTracker {
    /// Create new compute tracker
    pub fn new(budget_cu: u64) -> Self {
        Self {
            translator: ComputeBudgetTranslator::new(),
            total_budget_cu: budget_cu,
            consumed_cu: 0,
            consumed_cycles: 0,
            operation_breakdown: HashMap::new(),
        }
    }

    /// Consume compute units and update ZisK cycles
    pub fn consume(&mut self, cu: u64, operation: ComputeOperation) -> Result<(), ZisKError> {
        if self.consumed_cu + cu > self.total_budget_cu {
            return Err(ZisKError::ComputeBudgetExceeded);
        }

        let cycles = self.translator.cu_to_cycles(cu, operation);
        
        self.consumed_cu += cu;
        self.consumed_cycles += cycles;
        
        // Update operation breakdown
        *self.operation_breakdown.entry(operation).or_insert(0) += cycles;

        // Update global ZisK cycle counter (placeholder for now)
        // unsafe {
        //     crate::OP_CYCLES += cycles;
        // }

        Ok(())
    }

    /// Consume cycles directly (for ZisK-specific operations)
    pub fn consume_cycles(&mut self, cycles: u64) -> Result<(), ZisKError> {
        let equivalent_cu = self.translator.cycles_to_cu(cycles);
        
        if self.consumed_cu + equivalent_cu > self.total_budget_cu {
            return Err(ZisKError::ComputeBudgetExceeded);
        }

        self.consumed_cu += equivalent_cu;
        self.consumed_cycles += cycles;

        // Update global ZisK cycle counter (placeholder for now)
        // unsafe {
        //     crate::OP_CYCLES += cycles;
        // }

        Ok(())
    }

    /// Update factors based on current execution state
    pub fn update_context(&mut self, memory_usage_percent: f64, complexity: ProofComplexity) {
        self.translator.update_factors(memory_usage_percent, complexity);
    }

    /// Get remaining compute budget
    pub fn remaining_cu(&self) -> u64 {
        self.total_budget_cu.saturating_sub(self.consumed_cu)
    }

    /// Get remaining cycles (estimated)
    pub fn remaining_cycles(&self) -> u64 {
        let remaining_cu = self.remaining_cu();
        self.translator.cu_to_cycles(remaining_cu, ComputeOperation::AluOperation)
    }

    /// Get execution summary
    pub fn get_summary(&self) -> ComputeExecutionSummary {
        ComputeExecutionSummary {
            total_budget_cu: self.total_budget_cu,
            consumed_cu: self.consumed_cu,
            consumed_cycles: self.consumed_cycles,
            utilization_percent: (self.consumed_cu as f64 / self.total_budget_cu as f64) * 100.0,
            operation_breakdown: self.operation_breakdown.clone(),
            memory_pressure_factor: self.translator.memory_pressure_factor,
            proof_complexity_factor: self.translator.proof_complexity_factor,
        }
    }

    /// Check if budget is exhausted
    pub fn is_budget_exhausted(&self) -> bool {
        self.consumed_cu >= self.total_budget_cu
    }

    /// Get budget utilization percentage
    pub fn utilization_percentage(&self) -> f64 {
        (self.consumed_cu as f64 / self.total_budget_cu as f64) * 100.0
    }

    /// Get operation breakdown
    pub fn get_operation_breakdown(&self) -> &HashMap<ComputeOperation, u64> {
        &self.operation_breakdown
    }

    /// Reset tracker for new execution
    pub fn reset(&mut self, new_budget: u64) {
        self.total_budget_cu = new_budget;
        self.consumed_cu = 0;
        self.consumed_cycles = 0;
        self.operation_breakdown.clear();
        self.translator.reset_factors();
    }
}

/// Compute execution summary with detailed metrics
#[derive(Debug, Clone)]
pub struct ComputeExecutionSummary {
    pub total_budget_cu: u64,
    pub consumed_cu: u64,
    pub consumed_cycles: u64,
    pub utilization_percent: f64,
    pub operation_breakdown: HashMap<ComputeOperation, u64>,
    pub memory_pressure_factor: f64,
    pub proof_complexity_factor: f64,
}

/// Convenience macros for common operations
#[macro_export]
macro_rules! consume_compute {
    ($tracker:expr, $cu:expr, $op:expr) => {
        $tracker.consume($cu, $op)?
    };
}

#[macro_export]
macro_rules! consume_cycles {
    ($tracker:expr, $cycles:expr) => {
        $tracker.consume_cycles($cycles)?
    };
}

/// Integration with existing syscalls
pub trait ZisKComputeIntegration {
    /// Enhanced syscall execution with compute tracking
    fn execute_syscall_with_compute(
        &mut self,
        syscall_id: u32,
        parameters: &[u64],
        compute_tracker: &mut ZisKComputeTracker,
    ) -> Result<u64, ZisKError>;
}

/// Implementation for syscall registry
impl ZisKComputeIntegration for crate::zisk_syscalls::ZisKSyscallRegistry {
    /// Enhanced syscall execution with compute tracking
    fn execute_syscall_with_compute(
        &mut self,
        syscall_id: u32,
        parameters: &[u64],
        compute_tracker: &mut ZisKComputeTracker,
    ) -> Result<u64, ZisKError> {
        // Determine compute cost and operation type based on syscall
        let (cu_cost, operation) = match syscall_id {
            // Memory operations
            0x8c => (1, ComputeOperation::MemoryLoad),    // sol_memcpy
            0x8d => (1, ComputeOperation::MemoryStore),   // sol_memmove
            
            // Cryptographic operations
            0x8e => (25, ComputeOperation::Sha256Hash),   // sol_sha256
            0x8f => (100, ComputeOperation::Ed25519Verify), // sol_ed25519_verify
            0x90 => (150, ComputeOperation::Secp256k1Verify), // sol_secp256k1_verify
            0x91 => (30, ComputeOperation::KeccakHash),   // sol_keccak256
            
            // Account operations
            0x92 => (5, ComputeOperation::AccountDeserialization), // sol_get_account_info
            
            // Program operations
            0x93 => (200, ComputeOperation::CpiInvocation), // sol_invoke
            0x94 => (250, ComputeOperation::CpiInvocation), // sol_invoke_signed
            
            // Default
            _ => (10, ComputeOperation::SyscallInvocation),
        };

        // Consume compute budget
        compute_tracker.consume(cu_cost, operation)?;

        // Execute the actual syscall (placeholder for now)
        // self.execute_syscall(syscall_id, parameters)
        Ok(0) // Placeholder return value
    }
}

/// Compute budget utilities
pub struct ComputeBudgetUtils;

impl ComputeBudgetUtils {
    /// Calculate optimal compute budget for transaction
    pub fn calculate_optimal_budget(
        account_count: usize,
        instruction_count: usize,
        data_size: usize,
    ) -> u64 {
        let base_cost = 200_000; // Base compute units
        let account_cost = account_count as u64 * 25_000; // 25k CU per account
        let instruction_cost = instruction_count as u64 * 200; // 200 CU per instruction
        let data_cost = (data_size / 1024) as u64 * 1_000; // 1k CU per KB
        
        base_cost + account_cost + instruction_cost + data_cost
    }

    /// Estimate proof generation cost
    pub fn estimate_proof_cost(complexity: ProofComplexity) -> u64 {
        match complexity {
            ProofComplexity::Simple => 10_000,
            ProofComplexity::Medium => 25_000,
            ProofComplexity::Complex => 50_000,
            ProofComplexity::VeryComplex => 100_000,
        }
    }

    /// Validate compute budget against limits
    pub fn validate_budget(budget: u64) -> Result<(), ZisKError> {
        const MIN_BUDGET: u64 = 200_000;  // 200k CU minimum
        const MAX_BUDGET: u64 = 1_400_000; // 1.4M CU maximum (Solana limit)
        
        if budget < MIN_BUDGET {
            return Err(ZisKError::ComputeBudgetExceeded);
        }
        
        if budget > MAX_BUDGET {
            return Err(ZisKError::ComputeBudgetExceeded);
        }
        
        Ok(())
    }
}

/// Usage example in your main execution loop
pub fn execute_solana_program_with_budget(
    program_data: &[u8],
    accounts: &mut [crate::zisk_proof_schema::AccountState],
    instruction_data: &[u8],
    compute_budget: u64,
) -> Result<(), ZisKError> {
    let mut compute_tracker = ZisKComputeTracker::new(compute_budget);
    
    // Program initialization
    consume_compute!(compute_tracker, 100, ComputeOperation::ProgramInvocation);
    
    // Account deserialization
    for account in accounts.iter() {
        let data_size = account.data.len();
        let cu_cost = (data_size / 1024) + 1; // 1 CU per KB + base cost
        consume_compute!(compute_tracker, cu_cost as u64, ComputeOperation::AccountDeserialization);
    }
    
    // BPF execution loop (placeholder)
    // while let Some(instruction) = get_next_bpf_instruction() {
    //     let operation = match instruction.opcode {
    //         BPF_ALU64_ADD => ComputeOperation::AluOperation,
    //         BPF_LD_IMM64 => ComputeOperation::MemoryLoad,
    //         BPF_JMP_CALL => ComputeOperation::BpfCall,
    //         _ => ComputeOperation::BpfInstruction,
    //     };
    //     
    //     consume_compute!(compute_tracker, 1, operation);
    //     execute_bpf_instruction(instruction)?;
    // }
    
    let summary = compute_tracker.get_summary();
    println!("Execution completed: {:.2}% of compute budget used", summary.utilization_percent);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_budget_translator_creation() {
        let translator = ComputeBudgetTranslator::new();
        assert_eq!(translator.base_cu_to_cycles, 2.5);
        assert_eq!(translator.memory_pressure_factor, 1.0);
        assert_eq!(translator.proof_complexity_factor, 1.0);
    }

    #[test]
    fn test_cu_to_cycles_conversion() {
        let translator = ComputeBudgetTranslator::new();
        
        // Basic ALU operation
        let cycles = translator.cu_to_cycles(100, ComputeOperation::AluOperation);
        assert_eq!(cycles, 250); // 100 * 2.5 * 1.0
        
        // Memory operation with higher multiplier
        let cycles = translator.cu_to_cycles(100, ComputeOperation::MemoryStore);
        assert_eq!(cycles, 375); // 100 * 2.5 * 1.5
        
        // Cryptographic operation
        let cycles = translator.cu_to_cycles(10, ComputeOperation::Sha256Hash);
        assert_eq!(cycles, 1250); // 10 * 2.5 * 50.0
    }

    #[test]
    fn test_compute_tracker_creation() {
        let tracker = ZisKComputeTracker::new(1000);
        assert_eq!(tracker.total_budget_cu, 1000);
        assert_eq!(tracker.consumed_cu, 0);
        assert_eq!(tracker.consumed_cycles, 0);
    }

    #[test]
    fn test_compute_tracker_consumption() {
        let mut tracker = ZisKComputeTracker::new(1000);
        
        // Consume compute units
        let result = tracker.consume(100, ComputeOperation::AluOperation);
        assert!(result.is_ok());
        assert_eq!(tracker.consumed_cu, 100);
        assert_eq!(tracker.remaining_cu(), 900);
        
        // Try to exceed budget
        let result = tracker.consume(1000, ComputeOperation::AluOperation);
        assert!(result.is_err());
    }

    #[test]
    fn test_compute_tracker_summary() {
        let mut tracker = ZisKComputeTracker::new(1000);
        tracker.consume(500, ComputeOperation::AluOperation).unwrap();
        
        let summary = tracker.get_summary();
        assert_eq!(summary.total_budget_cu, 1000);
        assert_eq!(summary.consumed_cu, 500);
        assert_eq!(summary.utilization_percent, 50.0);
    }

    #[test]
    fn test_compute_budget_utils() {
        let optimal_budget = ComputeBudgetUtils::calculate_optimal_budget(5, 10, 2048);
        assert!(optimal_budget > 200_000); // Should be above base cost
        
        let proof_cost = ComputeBudgetUtils::estimate_proof_cost(ProofComplexity::Complex);
        assert_eq!(proof_cost, 50_000);
        
        let validation_result = ComputeBudgetUtils::validate_budget(500_000);
        assert!(validation_result.is_ok());
    }

    #[test]
    fn test_macro_usage() {
        let mut tracker = ZisKComputeTracker::new(1000);
        
        // Test consume_compute macro
        let result = consume_compute!(tracker, 100, ComputeOperation::AluOperation);
        assert!(result.is_ok());
        
        // Test consume_cycles macro
        let result = consume_cycles!(tracker, 50);
        assert!(result.is_ok());
    }
}
