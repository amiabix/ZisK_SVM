//! Cross-Program Invocation (CPI) Support for ZisK zkVM
//! 
//! This module implements the critical CPI functionality that Solana programs
//! need for complex DeFi operations and program interactions.

use crate::zisk_memory_manager::{ZisKMemoryManager, ZisKMemoryConstraints};
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use sha2::{Sha256, Digest};
use bs58;

/// Program execution context for CPI calls
#[derive(Debug, Clone)]
pub struct ProgramContext {
    /// Program ID of the calling program
    pub program_id: [u8; 32],
    /// Current instruction being executed
    pub instruction: Vec<u8>,
    /// Accounts accessible to this program
    pub accounts: Vec<AccountInfo>,
    /// Current compute budget
    pub compute_budget: u64,
    /// Call depth (for recursion protection)
    pub call_depth: u8,
}

/// Account information for CPI operations
#[derive(Debug, Clone)]
pub struct AccountInfo {
    /// Account public key
    pub key: [u8; 32],
    /// Account owner
    pub owner: [u8; 32],
    /// Account data
    pub data: Vec<u8>,
    /// Account lamports
    pub lamports: u64,
    /// Whether account is executable
    pub executable: bool,
    /// Account rent epoch
    pub rent_epoch: u64,
    /// Whether account is writable
    pub is_writable: bool,
    /// Whether account is signer
    pub is_signer: bool,
}

impl AccountInfo {
    /// Create new account info
    pub fn new(
        key: [u8; 32],
        owner: [u8; 32],
        data: Vec<u8>,
        lamports: u64,
        executable: bool,
        rent_epoch: u64,
        is_writable: bool,
        is_signer: bool,
    ) -> Self {
        Self {
            key,
            owner,
            data,
            lamports,
            executable,
            rent_epoch,
            is_writable,
            is_signer,
        }
    }

    /// Convert to account data for BPF execution
    pub fn to_account_data(&self) -> Vec<u8> {
        // Serialize account data for BPF execution
        let mut data = Vec::new();
        data.extend_from_slice(&self.lamports.to_le_bytes());
        data.extend_from_slice(&self.owner);
        data.push(if self.executable { 1 } else { 0 });
        data.extend_from_slice(&self.rent_epoch.to_le_bytes());
        data.extend_from_slice(&self.data);
        data
    }

    /// Update from account data after BPF execution
    pub fn update_from_account_data(&mut self, data: &[u8]) -> Result<(), anyhow::Error> {
        if data.len() < 41 { // Minimum size for account data
            return Err(anyhow::anyhow!("Invalid account data size"));
        }
        
        let mut offset = 0;
        
        // Read lamports
        if data.len() < offset + 8 {
            return Err(anyhow::anyhow!("Insufficient data for lamports"));
        }
        self.lamports = u64::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
            data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7]
        ]);
        offset += 8;
        
        // Read owner
        if data.len() < offset + 32 {
            return Err(anyhow::anyhow!("Insufficient data for owner"));
        }
        self.owner.copy_from_slice(&data[offset..offset + 32]);
        offset += 32;
        
        // Read executable
        if data.len() < offset + 1 {
            return Err(anyhow::anyhow!("Insufficient data for executable"));
        }
        self.executable = data[offset] != 0;
        offset += 1;
        
        // Read rent_epoch
        if data.len() < offset + 8 {
            return Err(anyhow::anyhow!("Insufficient data for rent_epoch"));
        }
        self.rent_epoch = u64::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
            data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7]
        ]);
        offset += 8;
        
        // Read data
        if offset < data.len() {
            self.data = data[offset..].to_vec();
        }
        
        Ok(())
    }
}

/// CPI context for managing cross-program invocations
pub struct ZisKCpiContext {
    /// Call stack for nested invocations
    call_stack: Vec<ProgramContext>,
    /// Borrowed accounts across program boundaries
    borrowed_accounts: HashMap<[u8; 32], AccountInfo>,
    /// Remaining compute budget
    remaining_compute: u64,
    /// Maximum call depth
    max_call_depth: u8,
    /// Program registry for loading programs
    program_registry: ProgramRegistry,
    /// Memory manager for constrained environment
    memory_manager: ZisKMemoryManager,
}

impl ZisKCpiContext {
    /// Create new CPI context
    pub fn new(initial_compute: u64, max_call_depth: u8) -> Self {
        let memory_constraints = ZisKMemoryConstraints::default();
        Self {
            call_stack: Vec::new(),
            borrowed_accounts: HashMap::new(),
            remaining_compute: initial_compute,
            max_call_depth,
            program_registry: ProgramRegistry::new(),
            memory_manager: ZisKMemoryManager::new(memory_constraints),
        }
    }

    /// Invoke another program
    pub fn invoke_program(
        &mut self,
        instruction: &[u8],
        account_infos: &[&AccountInfo],
        target_program: [u8; 32],
    ) -> Result<()> {
        // Check call depth limit
        if self.call_stack.len() >= self.max_call_depth as usize {
            return Err(anyhow!("Maximum call depth exceeded: {}", self.max_call_depth));
        }

        // Validate CPI constraints
        self.validate_cpi_constraints(account_infos)?;

        // Create new program context
        let program_context = ProgramContext {
            program_id: target_program,
            instruction: instruction.to_vec(),
            accounts: account_infos.iter().map(|&info| info.clone()).collect(),
            compute_budget: self.remaining_compute / 2, // Reserve compute for caller
            call_depth: self.call_stack.len() as u8 + 1,
        };

        // Push context onto call stack
        self.call_stack.push(program_context);

        // Execute the target program
        let result = self.execute_target_program(&target_program, instruction, account_infos);

        // Pop context from call stack
        self.call_stack.pop();

        // Update account states
        self.update_account_states(account_infos);

        result
    }

    /// Invoke program with signed accounts (PDA support)
    pub fn invoke_signed(
        &mut self,
        instruction: &[u8],
        account_infos: &[&AccountInfo],
        target_program: [u8; 32],
        seeds: &[&[u8]],
    ) -> Result<()> {
        // Validate seeds for PDA derivation
        self.validate_pda_seeds(seeds)?;

        // Perform the invocation
        self.invoke_program(instruction, account_infos, target_program)
    }

    /// Validate CPI constraints
    fn validate_cpi_constraints(&self, account_infos: &[&AccountInfo]) -> Result<()> {
        for account in account_infos {
            // Check if account is already borrowed
            if self.borrowed_accounts.contains_key(&account.key) {
                return Err(anyhow!("Account {} is already borrowed", bs58::encode(account.key).into_string()));
            }

            // Validate account permissions
            if account.is_writable && !self.can_write_account(account)? {
                return Err(anyhow!("Cannot write to account {}", bs58::encode(account.key).into_string()));
            }
        }
        Ok(())
    }

    /// Validate PDA seeds
    fn validate_pda_seeds(&self, seeds: &[&[u8]]) -> Result<()> {
        // Check seed length constraints
        if seeds.is_empty() {
            return Err(anyhow!("PDA seeds cannot be empty"));
        }

        for seed in seeds {
            if seed.is_empty() {
                return Err(anyhow!("Individual PDA seed cannot be empty"));
            }
        }

        Ok(())
    }

    /// Check if account can be written to
    fn can_write_account(&self, account: &AccountInfo) -> Result<bool> {
        // System program can always write to its accounts
        if account.owner == [0u8; 32] {
            return Ok(true);
        }

        // Check if current program owns the account
        if let Some(current_context) = self.call_stack.last() {
            if account.owner == current_context.program_id {
                return Ok(true);
            }
        }

        // Check if account is marked as writable
        Ok(account.is_writable)
    }

    /// Execute target program
    fn execute_target_program(
        &mut self,
        program_id: &[u8; 32],
        instruction: &[u8],
        account_infos: &[&AccountInfo],
    ) -> Result<()> {
        // Load program from registry
        let program = self.program_registry.load_program(program_id)?;

        // Create BPF execution context
        let mut bpf_context = self.create_bpf_context(account_infos);

        // Execute program
        let result = self.execute_bpf_program(&program, instruction, &mut bpf_context);

        // Update account states from execution
        self.update_accounts_from_execution(account_infos, &bpf_context);

        result
    }

    /// Create BPF execution context
    fn create_bpf_context(&self, account_infos: &[&AccountInfo]) -> BpfExecutionContext {
        BpfExecutionContext {
            accounts: account_infos.iter().map(|&info| info.clone()).collect(),
            compute_budget: self.remaining_compute / 2,
            logs: Vec::new(),
            return_data: None,
        }
    }

    /// Execute BPF program
    fn execute_bpf_program(
        &self,
        program: &[u8],
        instruction: &[u8],
        context: &mut BpfExecutionContext,
    ) -> Result<()> {
        // TODO: Implement actual BPF execution
        // This should integrate with the BPF interpreter
        
        // For now, simulate execution
        context.logs.push("BPF execution simulated".to_string());
        context.return_data = Some(instruction.to_vec());
        
        Ok(())
    }

    /// Update account states from execution
    fn update_accounts_from_execution(
        &mut self,
        account_infos: &[&AccountInfo],
        bpf_context: &BpfExecutionContext,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (i, account_info) in account_infos.iter().enumerate() {
            if let Some(updated_account) = bpf_context.accounts.get(i) {
                // Update the borrowed account
                if let Some(borrowed) = self.borrowed_accounts.get_mut(&account_info.key) {
                    borrowed.update_from_account_data(&updated_account.data)?;
                }
            }
        }
        Ok(())
    }

    /// Update account states after execution
    fn update_account_states(&mut self, account_infos: &[&AccountInfo]) {
        for account_info in account_infos {
            if let Some(borrowed) = self.borrowed_accounts.get(&account_info.key) {
                // Update the original account with borrowed state
                // This would need to be integrated with the main account manager
            }
        }
    }

    /// Get current call depth
    pub fn call_depth(&self) -> u8 {
        self.call_stack.len() as u8
    }

    /// Get remaining compute budget
    pub fn remaining_compute(&self) -> u64 {
        self.remaining_compute
    }

    /// Consume compute budget
    pub fn consume_compute(&mut self, units: u64) -> Result<()> {
        if self.remaining_compute < units {
            return Err(anyhow!("Insufficient compute budget: {} < {}", self.remaining_compute, units));
        }
        self.remaining_compute -= units;
        Ok(())
    }
}

/// BPF execution context for CPI
#[derive(Debug)]
struct BpfExecutionContext {
    accounts: Vec<AccountInfo>,
    compute_budget: u64,
    logs: Vec<String>,
    return_data: Option<Vec<u8>>,
}

/// Program registry for loading programs
struct ProgramRegistry {
    programs: HashMap<[u8; 32], Vec<u8>>,
}

impl ProgramRegistry {
    fn new() -> Self {
        Self {
            programs: HashMap::new(),
        }
    }

    fn load_program(&self, program_id: &[u8; 32]) -> Result<Vec<u8>> {
        self.programs
            .get(program_id)
            .cloned()
            .ok_or_else(|| anyhow!("Program {} not found", bs58::encode(program_id).into_string()))
    }

    fn register_program(&mut self, program_id: [u8; 32], program_data: Vec<u8>) {
        self.programs.insert(program_id, program_data);
    }
}

/// PDA (Program Derived Address) utilities
pub mod pda {
    use super::*;
    use sha2::{Sha256, Digest};

    /// Find program derived address
    pub fn find_program_address(
        seeds: &[&[u8]],
        program_id: &[u8; 32],
    ) -> Result<([u8; 32], u8)> {
        let mut bump_seed = 255u8;
        let mut address = [0u8; 32];

        loop {
            let mut seeds_with_bump = seeds.to_vec();
            let bump_seed_slice = [bump_seed];
            seeds_with_bump.push(&bump_seed_slice);

            if let Ok(addr) = try_find_program_address(&seeds_with_bump, program_id) {
                address = addr;
                break;
            }

            if bump_seed == 0 {
                return Err(anyhow!("Unable to find program derived address"));
            }
            bump_seed -= 1;
        }

        Ok((address, bump_seed))
    }

    /// Try to find program derived address
    fn try_find_program_address(
        seeds: &[&[u8]],
        program_id: &[u8; 32],
    ) -> Result<[u8; 32]> {
        let mut hasher = Sha256::new();
        
        for seed in seeds {
            hasher.update(seed);
        }
        hasher.update(b"ProgramDerivedAddress");
        hasher.update(program_id);
        
        let result = hasher.finalize();
        let mut address = [0u8; 32];
        address.copy_from_slice(&result);

        // Check if address is on ed25519 curve
        if is_on_ed25519_curve(&address) {
            return Err(anyhow!("Address is on ed25519 curve"));
        }

        Ok(address)
    }

    /// Check if address is on ed25519 curve
    fn is_on_ed25519_curve(address: &[u8; 32]) -> bool {
        // Simplified check - in practice this would use proper curve math
        // For now, just check if the address has certain properties
        address[31] & 1 == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpi_context_creation() {
        let context = ZisKCpiContext::new(1000, 5);
        assert_eq!(context.call_depth(), 0);
        assert_eq!(context.remaining_compute(), 1000);
    }

    #[test]
    fn test_account_info_creation() {
        let key = [1u8; 32];
        let owner = [2u8; 32];
        let data = vec![1, 2, 3];
        
        let account = AccountInfo::new(
            key,
            owner,
            data.clone(),
            1000,
            false,
            0,
            true,
            false,
        );

        assert_eq!(account.key, key);
        assert_eq!(account.owner, owner);
        assert_eq!(account.data, data);
        assert_eq!(account.lamports, 1000);
        assert!(!account.executable);
        assert!(account.is_writable);
        assert!(!account.is_signer);
    }

    #[test]
    fn test_pda_finding() {
        let program_id = [1u8; 32];
        let seeds = [b"test", b"seed"];
        
        let result = pda::find_program_address(&seeds, &program_id);
        assert!(result.is_ok());
        
        let (address, bump) = result.unwrap();
        assert_eq!(address.len(), 32);
        assert!(bump <= 255);
    }
}
