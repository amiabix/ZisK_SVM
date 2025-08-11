//! ZisK Proof Input/Output Schema for Solana Transaction Verification
//! 
//! This module defines the standardized format for inputs and outputs that can be
//! verified by the ZisK proving system, ensuring consistency between Solana
//! execution and ZK proof generation.

use crate::bpf_interpreter::SolanaAccount;
use crate::real_solana_parser::RealSolanaTransaction;
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

/// Input to ZisK proving system for Solana transaction execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZisKSolanaInput {
    /// Transaction to be executed
    pub transaction: EncodedTransaction,
    /// Account states before execution
    pub account_states: Vec<AccountState>,
    /// Block information
    pub block_context: BlockContext,
    /// Execution parameters
    pub execution_params: ExecutionParams,
    /// Metadata for proof generation
    pub proof_metadata: ProofMetadata,
}

/// Encoded transaction data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodedTransaction {
    /// Transaction signatures
    pub signatures: Vec<String>,
    /// Transaction message
    pub message: TransactionMessage,
    /// Transaction metadata (if available)
    pub meta: Option<TransactionMeta>,
}

/// Transaction message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionMessage {
    /// Transaction header
    pub header: MessageHeader,
    /// Account keys involved in transaction
    pub account_keys: Vec<String>,
    /// Recent blockhash
    pub recent_blockhash: String,
    /// Instructions to execute
    pub instructions: Vec<CompiledInstruction>,
}

/// Transaction header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHeader {
    /// Number of required signatures
    pub num_required_signatures: u8,
    /// Number of read-only signed accounts
    pub num_readonly_signed_accounts: u8,
    /// Number of read-only unsigned accounts
    pub num_readonly_unsigned_accounts: u8,
}



/// Compiled instruction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledInstruction {
    /// Program ID index
    pub program_id_index: u8,
    /// Account indices
    pub accounts: Vec<u8>,
    /// Instruction data
    pub data: Vec<u8>,
}

/// Transaction metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionMeta {
    /// Error if transaction failed
    pub err: Option<serde_json::Value>,
    /// Transaction fee
    pub fee: u64,
    /// Pre-balances
    pub pre_balances: Vec<u64>,
    /// Post-balances
    pub post_balances: Vec<u64>,
    /// Inner instructions
    pub inner_instructions: Option<Vec<serde_json::Value>>,
    /// Log messages
    pub log_messages: Option<Vec<String>>,
    /// Compute units consumed
    pub compute_units_consumed: Option<u64>,
}

/// Account state representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountState {
    /// Account public key
    pub pubkey: solana_sdk::pubkey::Pubkey,
    /// Account owner
    pub owner: solana_sdk::pubkey::Pubkey,
    /// Account lamports
    pub lamports: u64,
    /// Whether account is executable
    pub executable: bool,
    /// Account rent epoch
    pub rent_epoch: u64,
    /// Account data
    pub data: Vec<u8>,
    /// Account rent exemption status
    pub rent_exempt_reserve: u64,
}

/// Block context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockContext {
    /// Block hash
    pub blockhash: String,
    /// Block height
    pub block_height: u64,
    /// Block timestamp
    pub timestamp: i64,
    /// Epoch information
    pub epoch: u64,
    /// Slot information
    pub slot: u64,
}

/// Execution parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionParams {
    /// Maximum compute units
    pub max_compute_units: u64,
    /// Priority fee
    pub priority_fee: u64,
    /// Compute unit price
    pub compute_unit_price: u64,
    /// Maximum call depth
    pub max_call_depth: u8,
}

/// Proof generation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofMetadata {
    /// ZisK version
    pub zisk_version: String,
    /// Proof generation timestamp
    pub generated_at: i64,
    /// Target RISC-V architecture
    pub target_arch: String,
    /// Memory layout configuration
    pub memory_layout: MemoryLayout,
    /// Cycle budget
    pub cycle_budget: u64,
}

/// Memory layout configuration for ZisK
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLayout {
    /// Code section origin
    pub code_origin: u64,
    /// Code section length
    pub code_length: u64,
    /// Data section origin
    pub data_origin: u64,
    /// Data section length
    pub data_length: u64,
    /// Stack section origin
    pub stack_origin: u64,
    /// Stack section length
    pub stack_length: u64,
    /// Heap section origin
    pub heap_origin: u64,
    /// Heap section length
    pub heap_length: u64,
}

/// Output from ZisK proving system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZisKSolanaOutput {
    /// Execution result
    pub execution_result: TransactionResult,
    /// State changes
    pub state_changes: Vec<AccountDelta>,
    /// Compute units consumed
    pub compute_units_consumed: u64,
    /// ZisK cycles consumed
    pub zisk_cycles_consumed: u64,
    /// Program logs
    pub logs: Vec<String>,
    /// Return data
    pub return_data: Option<Vec<u8>>,
    /// Proof metadata
    pub proof_metadata: ProofResultMetadata,
}

/// Transaction execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResult {
    /// Whether transaction succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Instruction results
    pub instruction_results: Vec<InstructionResult>,
    /// Transaction fee charged
    pub fee_charged: u64,
}

/// Instruction execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionResult {
    /// Whether instruction succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Logs from instruction execution
    pub logs: Vec<String>,
    /// Return data from instruction
    pub return_data: Option<Vec<u8>>,
    /// Compute units consumed by instruction
    pub compute_units_consumed: u64,
}

/// Account state change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountDelta {
    /// Account public key
    pub pubkey: String,
    /// Change type
    pub change_type: AccountChangeType,
    /// Previous state
    pub previous_state: Option<AccountState>,
    /// New state
    pub new_state: Option<AccountState>,
    /// Lamport change
    pub lamport_change: i64,
}

/// Type of account change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountChangeType {
    /// Account created
    Created,
    /// Account updated
    Updated,
    /// Account deleted
    Deleted,
    /// Account unchanged
    Unchanged,
}

/// Proof result metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofResultMetadata {
    /// Proof generation timestamp
    pub generated_at: i64,
    /// Proof verification status
    pub verification_status: ProofVerificationStatus,
    /// Proof size in bytes
    pub proof_size: usize,
    /// Verification time in microseconds
    pub verification_time_us: u64,
}

/// Proof verification status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofVerificationStatus {
    /// Proof verified successfully
    Verified,
    /// Proof verification failed
    Failed,
    /// Proof verification pending
    Pending,
}

/// ZisK proof schema validator
pub struct ZisKProofValidator;

impl ZisKProofValidator {
    /// Validate input schema
    pub fn validate_input(input: &ZisKSolanaInput) -> Result<()> {
        // Validate transaction structure
        Self::validate_transaction(&input.transaction)?;
        
        // Validate account states
        Self::validate_account_states(&input.account_states)?;
        
        // Validate block context
        Self::validate_block_context(&input.block_context)?;
        
        // Validate execution parameters
        Self::validate_execution_params(&input.execution_params)?;
        
        // Validate proof metadata
        Self::validate_proof_metadata(&input.proof_metadata)?;
        
        Ok(())
    }

    /// Validate output schema
    pub fn validate_output(output: &ZisKSolanaOutput) -> Result<()> {
        // Validate execution result
        Self::validate_execution_result(&output.execution_result)?;
        
        // Validate state changes
        Self::validate_state_changes(&output.state_changes)?;
        
        // Validate proof metadata
        Self::validate_proof_result_metadata(&output.proof_metadata)?;
        
        Ok(())
    }

    /// Validate transaction structure
    fn validate_transaction(transaction: &EncodedTransaction) -> Result<()> {
        if transaction.signatures.is_empty() {
            return Err(anyhow!("Transaction must have at least one signature"));
        }
        
        if transaction.message.account_keys.is_empty() {
            return Err(anyhow!("Transaction must have at least one account key"));
        }
        
        if transaction.message.instructions.is_empty() {
            return Err(anyhow!("Transaction must have at least one instruction"));
        }
        
        Ok(())
    }

    /// Validate account states
    fn validate_account_states(account_states: &[AccountState]) -> Result<()> {
        for (i, account) in account_states.iter().enumerate() {
            if account.pubkey == solana_sdk::pubkey::Pubkey::default() {
                return Err(anyhow!("Account {} has default public key", i));
            }
            
            if account.owner == solana_sdk::pubkey::Pubkey::default() {
                return Err(anyhow!("Account {} has default owner", i));
            }
        }
        
        Ok(())
    }

    /// Validate block context
    fn validate_block_context(block_context: &BlockContext) -> Result<()> {
        if block_context.blockhash.is_empty() {
            return Err(anyhow!("Block hash cannot be empty"));
        }
        
        if block_context.timestamp <= 0 {
            return Err(anyhow!("Invalid block timestamp"));
        }
        
        Ok(())
    }

    /// Validate execution parameters
    fn validate_execution_params(params: &ExecutionParams) -> Result<()> {
        if params.max_compute_units == 0 {
            return Err(anyhow!("Maximum compute units must be greater than 0"));
        }
        
        if params.max_call_depth == 0 {
            return Err(anyhow!("Maximum call depth must be greater than 0"));
        }
        
        Ok(())
    }

    /// Validate proof metadata
    fn validate_proof_metadata(metadata: &ProofMetadata) -> Result<()> {
        if metadata.zisk_version.is_empty() {
            return Err(anyhow!("ZisK version cannot be empty"));
        }
        
        if metadata.target_arch.is_empty() {
            return Err(anyhow!("Target architecture cannot be empty"));
        }
        
        if metadata.cycle_budget == 0 {
            return Err(anyhow!("Cycle budget must be greater than 0"));
        }
        
        Ok(())
    }

    /// Validate execution result
    fn validate_execution_result(result: &TransactionResult) -> Result<()> {
        if result.instruction_results.is_empty() {
            return Err(anyhow!("Transaction result must have at least one instruction result"));
        }
        
        Ok(())
    }

    /// Validate state changes
    fn validate_state_changes(changes: &[AccountDelta]) -> Result<()> {
        for (i, change) in changes.iter().enumerate() {
            if change.pubkey.is_empty() {
                return Err(anyhow!("State change {} has empty public key", i));
            }
        }
        
        Ok(())
    }

    /// Validate proof result metadata
    fn validate_proof_result_metadata(metadata: &ProofResultMetadata) -> Result<()> {
        if metadata.proof_size == 0 {
            return Err(anyhow!("Proof size must be greater than 0"));
        }
        
        Ok(())
    }
}

/// Schema converter utilities
pub struct SchemaConverter;

impl SchemaConverter {
    /// Convert RealSolanaTransaction to ZisKSolanaInput
    pub fn transaction_to_input(
        transaction: &RealSolanaTransaction,
        account_states: Vec<AccountState>,
        block_context: BlockContext,
        execution_params: ExecutionParams,
    ) -> Result<ZisKSolanaInput> {
        let encoded_transaction = EncodedTransaction {
            signatures: transaction.signatures.clone(),
            message: TransactionMessage {
                header: MessageHeader {
                    num_required_signatures: transaction.message.header.num_required_signatures,
                    num_readonly_signed_accounts: transaction.message.header.num_readonly_signed_accounts,
                    num_readonly_unsigned_accounts: transaction.message.header.num_readonly_unsigned_accounts,
                },
                account_keys: transaction.message.account_keys.clone(),
                recent_blockhash: transaction.message.recent_blockhash.clone(),
                instructions: transaction.message.instructions.iter()
                    .map(|inst| CompiledInstruction {
                        program_id_index: inst.program_id_index,
                        accounts: inst.accounts.clone(),
                        data: bs58::decode(&inst.data).into_vec().unwrap_or_default(),
                    })
                    .collect(),
            },
            meta: transaction.meta.as_ref().map(|m| TransactionMeta {
                err: m.err.clone(),
                fee: m.fee,
                pre_balances: m.pre_balances.clone(),
                post_balances: m.post_balances.clone(),
                inner_instructions: m.inner_instructions.clone(),
                log_messages: m.log_messages.clone(),
                compute_units_consumed: m.compute_units_consumed,
            }),
        };

        let proof_metadata = ProofMetadata {
            zisk_version: env!("CARGO_PKG_VERSION").to_string(),
            generated_at: chrono::Utc::now().timestamp(),
            target_arch: "riscv64imac-unknown-none-elf".to_string(),
            memory_layout: MemoryLayout {
                code_origin: 0x1000,
                code_length: 0x100000,
                data_origin: 0x200000,
                data_length: 0x100000,
                stack_origin: 0x300000,
                stack_length: 0x10000,
                heap_origin: 0x400000,
                heap_length: 0x80000,
            },
            cycle_budget: 1_000_000,
        };

        Ok(ZisKSolanaInput {
            transaction: encoded_transaction,
            account_states,
            block_context,
            execution_params,
            proof_metadata,
        })
    }

    /// Convert SolanaAccount to AccountState
    pub fn solana_account_to_state(
        pubkey: &str,
        account: &SolanaAccount,
        rent_exempt_reserve: u64,
    ) -> AccountState {
        AccountState {
            pubkey: solana_sdk::pubkey::Pubkey::from_str(pubkey).unwrap_or_default(),
            owner: solana_sdk::pubkey::Pubkey::new_from_array(account.owner),
            lamports: account.lamports,
            executable: account.executable,
            rent_epoch: account.rent_epoch,
            data: account.data.clone(),
            rent_exempt_reserve,
        }
    }
}

/// Serialization utilities for ZisK integration
pub mod serialization {
    use super::*;
    use std::fs;
    use std::path::Path;

    /// Save input schema to file
    pub fn save_input_to_file(input: &ZisKSolanaInput, filepath: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(input)?;
        fs::write(filepath, json)?;
        Ok(())
    }

    /// Load input schema from file
    pub fn load_input_from_file(filepath: &Path) -> Result<ZisKSolanaInput> {
        let json = fs::read_to_string(filepath)?;
        let input: ZisKSolanaInput = serde_json::from_str(&json)?;
        Ok(input)
    }

    /// Save output schema to file
    pub fn save_output_to_file(output: &ZisKSolanaOutput, filepath: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(output)?;
        fs::write(filepath, json)?;
        Ok(())
    }

    /// Load output schema from file
    pub fn load_output_from_file(filepath: &Path) -> Result<ZisKSolanaOutput> {
        let json = fs::read_to_string(filepath)?;
        let output: ZisKSolanaOutput = serde_json::from_str(&json)?;
        Ok(output)
    }

    /// Convert input to binary format for ZisK
    pub fn input_to_binary(input: &ZisKSolanaInput) -> Result<Vec<u8>> {
        bincode::serialize(input).map_err(|e| anyhow!("Serialization failed: {}", e))
    }

    /// Convert binary to input format
    pub fn binary_to_input(data: &[u8]) -> Result<ZisKSolanaInput> {
        bincode::deserialize(data).map_err(|e| anyhow!("Deserialization failed: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_validation() {
        let input = ZisKSolanaInput {
            transaction: EncodedTransaction {
                signatures: vec!["signature1".to_string()],
                message: TransactionMessage {
                    header: MessageHeader {
                        num_required_signatures: 1,
                        num_readonly_signed_accounts: 0,
                        num_readonly_unsigned_accounts: 0,
                    },
                    account_keys: vec!["account1".to_string()],
                    recent_blockhash: "blockhash1".to_string(),
                    instructions: vec![CompiledInstruction {
                        program_id_index: 0,
                        accounts: vec![0],
                        data: vec![1, 2, 3],
                    }],
                },
                meta: None,
            },
            account_states: vec![AccountState {
            rent_exempt_reserve: 0,
                pubkey: "account1".to_string(),
                owner: "owner1".to_string(),
                lamports: 1000,
                executable: false,
                rent_epoch: 0,
                data: vec![],
                rent_exempt_reserve: 0,
            }],
            block_context: BlockContext {
                blockhash: "blockhash1".to_string(),
                block_height: 1000,
                timestamp: 1000000,
                epoch: 10,
                slot: 10000,
            },
            execution_params: ExecutionParams {
                max_compute_units: 1000,
                priority_fee: 0,
                compute_unit_price: 0,
                max_call_depth: 5,
            },
            proof_metadata: ProofMetadata {
                zisk_version: "0.1.0".to_string(),
                generated_at: 1000000,
                target_arch: "riscv64".to_string(),
                memory_layout: MemoryLayout {
                    code_origin: 0x1000,
                    code_length: 0x100000,
                    data_origin: 0x200000,
                    data_length: 0x100000,
                    stack_origin: 0x300000,
                    stack_length: 0x10000,
                    heap_origin: 0x400000,
                    heap_length: 0x80000,
                },
                cycle_budget: 1000000,
            },
        };

        assert!(ZisKProofValidator::validate_input(&input).is_ok());
    }

    #[test]
    fn test_output_validation() {
        let output = ZisKSolanaOutput {
            execution_result: TransactionResult {
                success: true,
                error: None,
                instruction_results: vec![InstructionResult {
                    success: true,
                    error: None,
                    logs: vec!["log1".to_string()],
                    return_data: None,
                    compute_units_consumed: 100,
                }],
                fee_charged: 5000,
            },
            state_changes: vec![AccountDelta {
                pubkey: "account1".to_string(),
                change_type: AccountChangeType::Updated,
                previous_state: None,
                new_state: None,
                lamport_change: 100,
            }],
            compute_units_consumed: 100,
            zisk_cycles_consumed: 1000,
            logs: vec!["log1".to_string()],
            return_data: None,
            proof_metadata: ProofResultMetadata {
                generated_at: 1000000,
                verification_status: ProofVerificationStatus::Verified,
                proof_size: 1000,
                verification_time_us: 100,
            },
        };

        assert!(ZisKProofValidator::validate_output(&output).is_ok());
    }
}
