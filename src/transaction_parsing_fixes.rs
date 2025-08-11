//! Transaction Parsing Fixes for Solana SDK v2.3.7 Compatibility
//! 
//! This module provides comprehensive transaction parsing that handles
//! multiple Solana SDK versions and resolves type compatibility issues
//! between different response formats.

use solana_sdk::{
    message::{Message, VersionedMessage},
    transaction::{VersionedTransaction, Transaction},
    pubkey::Pubkey,
    hash::Hash,
};
use serde::{Deserialize, Serialize};
use crate::zisk_state_manager::ZisKError;

/// Fixed transaction types for Solana SDK v2.3.7 compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZisKCompatibleTransaction {
    pub signatures: Vec<String>,
    pub message: ZisKCompatibleMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZisKCompatibleMessage {
    pub account_keys: Vec<String>,
    pub header: MessageHeader,
    pub instructions: Vec<ZisKCompatibleInstruction>,
    pub recent_blockhash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHeader {
    pub num_required_signatures: u8,
    pub num_readonly_signed_accounts: u8,
    pub num_readonly_unsigned_accounts: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZisKCompatibleInstruction {
    pub program_id_index: u8,
    pub accounts: Vec<u8>,
    pub data: Vec<u8>,
}

/// Transaction parser that handles SDK v2.3.7 compatibility
pub struct ZisKTransactionParser;

impl ZisKTransactionParser {
    /// Parse transaction from JSON-RPC response (handles multiple SDK versions)
    pub fn parse_transaction_response(
        transaction_data: &serde_json::Value,
    ) -> Result<ZisKCompatibleTransaction, ZisKError> {
        // Handle different response formats from Solana RPC
        if let Some(transaction) = transaction_data.get("transaction") {
            Self::parse_transaction_object(transaction)
        } else {
            // Direct transaction object
            Self::parse_transaction_object(transaction_data)
        }
    }

    fn parse_transaction_object(
        tx_obj: &serde_json::Value,
    ) -> Result<ZisKCompatibleTransaction, ZisKError> {
        // Extract signatures
        let signatures = tx_obj
            .get("signatures")
            .and_then(|s| s.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|sig| sig.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        // Extract message with fallback handling
        let message = if let Some(msg) = tx_obj.get("message") {
            Self::parse_message_object(msg)?
        } else {
            return Err(ZisKError::TransactionParseError("No message found in transaction".to_string()));
        };

        Ok(ZisKCompatibleTransaction {
            signatures,
            message,
        })
    }

    fn parse_message_object(
        msg_obj: &serde_json::Value,
    ) -> Result<ZisKCompatibleMessage, ZisKError> {
        // Handle both raw and parsed message formats
        match msg_obj {
            serde_json::Value::Object(obj) => {
                // Check if this is a versioned message
                if obj.contains_key("v0") || obj.contains_key("legacy") {
                    Self::parse_versioned_message(msg_obj)
                } else {
                    Self::parse_legacy_message(msg_obj)
                }
            }
            _ => Err(ZisKError::TransactionParseError("Invalid message format".to_string())),
        }
    }

    fn parse_versioned_message(
        msg_obj: &serde_json::Value,
    ) -> Result<ZisKCompatibleMessage, ZisKError> {
        // Handle v0 message format
        if let Some(v0_msg) = msg_obj.get("v0") {
            return Self::parse_message_content(v0_msg);
        }

        // Handle legacy message format
        if let Some(legacy_msg) = msg_obj.get("legacy") {
            return Self::parse_message_content(legacy_msg);
        }

        // Fallback to direct parsing
        Self::parse_message_content(msg_obj)
    }

    fn parse_legacy_message(
        msg_obj: &serde_json::Value,
    ) -> Result<ZisKCompatibleMessage, ZisKError> {
        Self::parse_message_content(msg_obj)
    }

    fn parse_message_content(
        msg_content: &serde_json::Value,
    ) -> Result<ZisKCompatibleMessage, ZisKError> {
        // Extract account keys with multiple fallback patterns
        let account_keys = Self::extract_account_keys(msg_content)?;
        
        // Extract header
        let header = Self::extract_message_header(msg_content)?;
        
        // Extract instructions
        let instructions = Self::extract_instructions(msg_content)?;
        
        // Extract recent blockhash
        let recent_blockhash = msg_content
            .get("recentBlockhash")
            .or_else(|| msg_content.get("recent_blockhash"))
            .and_then(|h| h.as_str())
            .unwrap_or("")
            .to_string();

        Ok(ZisKCompatibleMessage {
            account_keys,
            header,
            instructions,
            recent_blockhash,
        })
    }

    fn extract_account_keys(msg_content: &serde_json::Value) -> Result<Vec<String>, ZisKError> {
        // Try multiple field names for account keys
        let account_keys_value = msg_content
            .get("accountKeys")
            .or_else(|| msg_content.get("account_keys"))
            .or_else(|| msg_content.get("staticAccountKeys"))
            .or_else(|| msg_content.get("static_account_keys"));

        match account_keys_value {
            Some(keys) => {
                if let Some(keys_array) = keys.as_array() {
                    let account_keys: Vec<String> = keys_array
                        .iter()
                        .filter_map(|key| {
                            // Handle both string format and object format
                            if let Some(key_str) = key.as_str() {
                                Some(key_str.to_string())
                            } else if let Some(pubkey_obj) = key.get("pubkey") {
                                pubkey_obj.as_str().map(|s| s.to_string())
                            } else {
                                None
                            }
                        })
                        .collect();
                    Ok(account_keys)
                } else {
                    Ok(Vec::new())
                }
            }
            None => Ok(Vec::new()),
        }
    }

    fn extract_message_header(msg_content: &serde_json::Value) -> Result<MessageHeader, ZisKError> {
        let header_obj = msg_content.get("header");
        
        match header_obj {
            Some(header) => {
                let num_required_signatures = header
                    .get("numRequiredSignatures")
                    .or_else(|| header.get("num_required_signatures"))
                    .and_then(|n| n.as_u64())
                    .unwrap_or(0) as u8;

                let num_readonly_signed_accounts = header
                    .get("numReadonlySignedAccounts")
                    .or_else(|| header.get("num_readonly_signed_accounts"))
                    .and_then(|n| n.as_u64())
                    .unwrap_or(0) as u8;

                let num_readonly_unsigned_accounts = header
                    .get("numReadonlyUnsignedAccounts")
                    .or_else(|| header.get("num_readonly_unsigned_accounts"))
                    .and_then(|n| n.as_u64())
                    .unwrap_or(0) as u8;

                Ok(MessageHeader {
                    num_required_signatures,
                    num_readonly_signed_accounts,
                    num_readonly_unsigned_accounts,
                })
            }
            None => {
                // Default header if not present
                Ok(MessageHeader {
                    num_required_signatures: 1,
                    num_readonly_signed_accounts: 0,
                    num_readonly_unsigned_accounts: 0,
                })
            }
        }
    }

    fn extract_instructions(msg_content: &serde_json::Value) -> Result<Vec<ZisKCompatibleInstruction>, ZisKError> {
        let instructions_value = msg_content
            .get("instructions")
            .or_else(|| msg_content.get("compiledInstructions"))
            .or_else(|| msg_content.get("compiled_instructions"));

        match instructions_value {
            Some(instructions) => {
                if let Some(instructions_array) = instructions.as_array() {
                    let mut result = Vec::new();
                    
                    for instruction in instructions_array {
                        let program_id_index = instruction
                            .get("programIdIndex")
                            .or_else(|| instruction.get("program_id_index"))
                            .and_then(|i| i.as_u64())
                            .unwrap_or(0) as u8;

                        let accounts = instruction
                            .get("accounts")
                            .and_then(|a| a.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|a| a.as_u64().map(|n| n as u8))
                                    .collect()
                            })
                            .unwrap_or_default();

                        let data = instruction
                            .get("data")
                            .and_then(|d| d.as_str())
                            .map(|s| {
                                // Handle base58 or base64 encoded data
                                if s.chars().all(|c| c.is_ascii_alphanumeric() || "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz".contains(c)) {
                                    // Likely base58
                                    bs58::decode(s).into_vec().unwrap_or_default()
                                } else {
                                    // Likely base64 or hex
                                    base64::engine::general_purpose::STANDARD.decode(s).unwrap_or_else(|_| {
                                        hex::decode(s).unwrap_or_default()
                                    })
                                }
                            })
                            .unwrap_or_default();

                        result.push(ZisKCompatibleInstruction {
                            program_id_index,
                            accounts,
                            data,
                        });
                    }
                    
                    Ok(result)
                } else {
                    Ok(Vec::new())
                }
            }
            None => Ok(Vec::new()),
        }
    }

    /// Convert to internal transaction format for ZisK processing
    pub fn to_zisk_transaction(
        compat_tx: &ZisKCompatibleTransaction,
    ) -> Result<crate::zisk_proof_schema::ZisKSolanaInput, ZisKError> {
        // Convert account keys to Pubkey objects
        let account_keys: Result<Vec<Pubkey>, _> = compat_tx
            .message
            .account_keys
            .iter()
            .map(|key| key.parse::<Pubkey>())
            .collect();

        let account_keys = account_keys
            .map_err(|e| ZisKError::TransactionParseError(format!("Invalid account key: {}", e)))?;

        // Convert recent blockhash
        let recent_blockhash = compat_tx
            .message
            .recent_blockhash
            .parse::<Hash>()
            .map_err(|e| ZisKError::TransactionParseError(format!("Invalid blockhash: {}", e)))?;

        // Create transaction input
        Ok(crate::zisk_proof_schema::ZisKSolanaInput {
            transaction: crate::zisk_proof_schema::EncodedTransaction {
                signatures: compat_tx.signatures.clone(),
                message: crate::zisk_proof_schema::TransactionMessage {
                    header: crate::zisk_proof_schema::MessageHeader {
                        num_required_signatures: compat_tx.message.header.num_required_signatures,
                        num_readonly_signed_accounts: compat_tx.message.header.num_readonly_signed_accounts,
                        num_readonly_unsigned_accounts: compat_tx.message.header.num_readonly_unsigned_accounts,
                    },
                    account_keys: compat_tx.message.account_keys.clone(),
                    recent_blockhash: compat_tx.message.recent_blockhash.clone(),
                    instructions: compat_tx.message.instructions.iter().map(|i| crate::zisk_proof_schema::CompiledInstruction {
                        program_id_index: i.program_id_index,
                        accounts: i.accounts.clone(),
                        data: i.data.clone(),
                    }).collect(),
                },
                meta: None,
            },
            account_states: Vec::new(), // Will be populated separately
            block_context: crate::zisk_proof_schema::BlockContext {
                blockhash: compat_tx.message.recent_blockhash.clone(),
                block_height: 0,
                timestamp: 0,
                epoch: 0,
                slot: 0,
            },
            execution_params: crate::zisk_proof_schema::ExecutionParams {
                max_compute_units: 1_400_000,
                priority_fee: 0,
                compute_unit_price: 0,
                max_call_depth: 64,
            },
            proof_metadata: crate::zisk_proof_schema::ProofMetadata {
                zisk_version: "1.0.0".to_string(),
                generated_at: 0,
                target_arch: "riscv64imac".to_string(),
                memory_layout: crate::zisk_proof_schema::MemoryLayout {
                    code_origin: 0x1000,
                    code_length: 0x10000,
                    data_origin: 0x20000,
                    data_length: 0x10000,
                    stack_origin: 0x30000,
                    stack_length: 0x8000,
                    heap_origin: 0x40000,
                    heap_length: 0x40000,
                },
                cycle_budget: 1_000_000,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_header_creation() {
        let header = MessageHeader {
            num_required_signatures: 1,
            num_readonly_signed_accounts: 0,
            num_readonly_unsigned_accounts: 0,
        };
        
        assert_eq!(header.num_required_signatures, 1);
        assert_eq!(header.num_readonly_signed_accounts, 0);
        assert_eq!(header.num_readonly_unsigned_accounts, 0);
    }

    #[test]
    fn test_instruction_creation() {
        let instruction = ZisKCompatibleInstruction {
            program_id_index: 0,
            accounts: vec![1, 2, 3],
            data: vec![0x01, 0x02, 0x03],
        };
        
        assert_eq!(instruction.program_id_index, 0);
        assert_eq!(instruction.accounts, vec![1, 2, 3]);
        assert_eq!(instruction.data, vec![0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_transaction_creation() {
        let message = ZisKCompatibleMessage {
            account_keys: vec!["11111111111111111111111111111111".to_string()],
            header: MessageHeader {
                num_required_signatures: 1,
                num_readonly_signed_accounts: 0,
                num_readonly_unsigned_accounts: 0,
            },
            instructions: vec![],
            recent_blockhash: "11111111111111111111111111111111".to_string(),
        };
        
        let transaction = ZisKCompatibleTransaction {
            signatures: vec!["signature".to_string()],
            message,
        };
        
        assert_eq!(transaction.signatures.len(), 1);
        assert_eq!(transaction.message.account_keys.len(), 1);
    }
}
