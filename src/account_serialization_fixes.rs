//! Account Serialization Fixes for Type Conversion Issues
//! 
//! This module provides comprehensive account serialization utilities
//! that handle type conversions and provide proper error handling
//! for Solana account data processing.

use solana_sdk::{
    account::{Account, AccountSharedData},
    pubkey::Pubkey,
    rent::Rent,
};
use serde::{Deserialize, Serialize};
use crate::zisk_state_manager::{AccountState, ZisKError};
use base64::{Engine as _, engine::general_purpose};
use std::str::FromStr;

/// Account serialization utilities with proper type conversions
pub struct ZisKAccountSerializer;

impl ZisKAccountSerializer {
    /// Convert Solana Account to ZisK AccountState (with proper type handling)
    pub fn solana_account_to_zisk(
        pubkey: &Pubkey,
        account: &Account,
    ) -> AccountState {
        AccountState {
            rent_exempt_reserve: 0,
            pubkey: *pubkey,
            lamports: account.lamports,
            data: account.data.clone(),
            owner: account.owner,
            executable: account.executable,
            rent_epoch: account.rent_epoch,
        }
    }

    /// Convert ZisK AccountState to Solana Account
    pub fn zisk_account_to_solana(account_state: &AccountState) -> Account {
        Account {
            lamports: account_state.lamports,
            data: account_state.data.clone(),
            owner: account_state.owner,
            executable: account_state.executable,
            rent_epoch: account_state.rent_epoch,
        }
    }

    /// Parse account from JSON RPC response with type safety
    pub fn parse_account_from_json(
        pubkey_str: &str,
        account_json: &serde_json::Value,
    ) -> Result<AccountState, ZisKError> {
        // Parse pubkey with error handling
        let pubkey = Pubkey::from_str(pubkey_str)
            .map_err(|e| ZisKError::AccountParseError(format!("Invalid pubkey {}: {}", pubkey_str, e)))?;

        // Extract lamports with type conversion
        let lamports = Self::extract_u64_field(account_json, "lamports", 0)?;

        // Extract data with base64 decoding
        let data = Self::extract_account_data(account_json)?;

        // Extract owner with pubkey parsing
        let owner = Self::extract_pubkey_field(account_json, "owner", &solana_sdk::system_program::ID)?;

        // Extract executable flag
        let executable = Self::extract_bool_field(account_json, "executable", false)?;

        // Extract rent epoch
        let rent_epoch = Self::extract_u64_field(account_json, "rentEpoch", 0)?;

        Ok(AccountState {
            rent_exempt_reserve: 0,
            pubkey,
            lamports,
            data,
            owner,
            executable,
            rent_epoch,
        })
    }

    /// Extract u64 field with multiple type conversion attempts
    fn extract_u64_field(
        json: &serde_json::Value,
        field_name: &str,
        default: u64,
    ) -> Result<u64, ZisKError> {
        match json.get(field_name) {
            Some(value) => {
                if let Some(num) = value.as_u64() {
                    Ok(num)
                } else if let Some(num) = value.as_i64() {
                    if num >= 0 {
                        Ok(num as u64)
                    } else {
                        Err(ZisKError::AccountParseError(format!("Negative value for {}: {}", field_name, num)))
                    }
                } else if let Some(str_val) = value.as_str() {
                    str_val.parse::<u64>()
                        .map_err(|e| ZisKError::AccountParseError(format!("Cannot parse {} as u64: {}", field_name, e)))
                } else {
                    Err(ZisKError::AccountParseError(format!("Invalid type for {}", field_name)))
                }
            }
            None => Ok(default),
        }
    }

    /// Extract boolean field with type conversion
    fn extract_bool_field(
        json: &serde_json::Value,
        field_name: &str,
        default: bool,
    ) -> Result<bool, ZisKError> {
        match json.get(field_name) {
            Some(value) => {
                if let Some(bool_val) = value.as_bool() {
                    Ok(bool_val)
                } else if let Some(str_val) = value.as_str() {
                    match str_val.to_lowercase().as_str() {
                        "true" | "1" => Ok(true),
                        "false" | "0" => Ok(false),
                        _ => Err(ZisKError::AccountParseError(format!("Invalid boolean value for {}: {}", field_name, str_val)))
                    }
                } else if let Some(num) = value.as_u64() {
                    Ok(num != 0)
                } else {
                    Err(ZisKError::AccountParseError(format!("Invalid type for {}", field_name)))
                }
            }
            None => Ok(default),
        }
    }

    /// Extract pubkey field with parsing
    fn extract_pubkey_field(
        json: &serde_json::Value,
        field_name: &str,
        default: &Pubkey,
    ) -> Result<Pubkey, ZisKError> {
        match json.get(field_name) {
            Some(value) => {
                if let Some(pubkey_str) = value.as_str() {
                    Pubkey::from_str(pubkey_str)
                        .map_err(|e| ZisKError::AccountParseError(format!("Invalid pubkey for {}: {}", field_name, e)))
                } else {
                    Err(ZisKError::AccountParseError(format!("Invalid type for {}", field_name)))
                }
            }
            None => Ok(*default),
        }
    }

    /// Extract account data with base64/base58 decoding
    fn extract_account_data(json: &serde_json::Value) -> Result<Vec<u8>, ZisKError> {
        match json.get("data") {
            Some(data_value) => {
                match data_value {
                    // Handle array format: ["base64string", "base64"]
                    serde_json::Value::Array(arr) => {
                        if let Some(data_str) = arr.get(0).and_then(|v| v.as_str()) {
                            if let Some(encoding) = arr.get(1).and_then(|v| v.as_str()) {
                                match encoding {
                                    "base64" | "base64+zstd" => {
                                        general_purpose::STANDARD_NO_PAD.decode(data_str)
                                            .map_err(|e| ZisKError::AccountParseError(format!("Base64 decode error: {}", e)))
                                    }
                                    "jsonParsed" => {
                                        // For parsed data, return empty vec or implement specific parsing
                                        Ok(Vec::new())
                                    }
                                    _ => {
                                        Err(ZisKError::AccountParseError(format!("Unsupported encoding: {}", encoding)))
                                    }
                                }
                            } else {
                                // Default to base64
                                general_purpose::STANDARD_NO_PAD.decode(data_str)
                                    .map_err(|e| ZisKError::AccountParseError(format!("Base64 decode error: {}", e)))
                            }
                        } else {
                            Ok(Vec::new())
                        }
                    }
                    // Handle string format (assume base64)
                    serde_json::Value::String(data_str) => {
                        general_purpose::STANDARD_NO_PAD.decode(data_str)
                            .map_err(|e| ZisKError::AccountParseError(format!("Base64 decode error: {}", e)))
                    }
                    // Handle object format (parsed data)
                    serde_json::Value::Object(_) => {
                        // For parsed data, we might need to serialize it back
                        // For now, return empty vec
                        Ok(Vec::new())
                    }
                    _ => Ok(Vec::new()),
                }
            }
            None => Ok(Vec::new()),
        }
    }

    /// Serialize account for Solana program input (with proper type conversions)
    pub fn serialize_for_program_input(accounts: &[AccountState]) -> Result<Vec<u8>, ZisKError> {
        let mut input = Vec::new();

        // Serialize account count (8 bytes, little endian)
        input.extend_from_slice(&(accounts.len() as u64).to_le_bytes());

        for account in accounts {
            // Serialize duplicate flag (1 byte) - not a duplicate
            input.push(0xff);

            // Serialize signer flag (1 byte) - assume not signer for now
            input.push(0x00);

            // Serialize writable flag (1 byte) - assume writable for now
            input.push(0x01);

            // Serialize executable flag (1 byte)
            input.push(if account.executable { 0x01 } else { 0x00 });

            // Padding (4 bytes)
            input.extend_from_slice(&[0u8; 4]);

            // Serialize pubkey (32 bytes)
            input.extend_from_slice(account.pubkey.as_ref());

            // Serialize owner (32 bytes)
            input.extend_from_slice(account.owner.as_ref());

            // Serialize lamports (8 bytes, little endian)
            input.extend_from_slice(&account.lamports.to_le_bytes());

            // Serialize data length (8 bytes, little endian)
            input.extend_from_slice(&(account.data.len() as u64).to_le_bytes());

            // Serialize data
            input.extend_from_slice(&account.data);

            // Serialize rent epoch (8 bytes, little endian)
            input.extend_from_slice(&account.rent_epoch.to_le_bytes());
        }

        Ok(input)
    }

    /// Deserialize account changes from program output
    pub fn deserialize_account_changes(
        output_data: &[u8],
        original_accounts: &[AccountState],
    ) -> Result<Vec<AccountState>, ZisKError> {
        // This is a simplified implementation
        // In practice, you'd need to parse the actual output format from the BPF program
        
        if output_data.is_empty() {
            // No changes, return original accounts
            return Ok(original_accounts.to_vec());
        }

        // For now, assume accounts weren't modified and return originals
        // In a real implementation, you'd parse the modified account data from the BPF program's memory
        Ok(original_accounts.to_vec())
    }

    /// Convert string to Pubkey with better error handling
    pub fn parse_pubkey_safe(pubkey_str: &str) -> Result<Pubkey, ZisKError> {
        if pubkey_str.is_empty() {
            return Err(ZisKError::AccountParseError("Empty pubkey string".to_string()));
        }

        if pubkey_str.len() != 44 {
            return Err(ZisKError::AccountParseError(
                format!("Invalid pubkey length: {} (expected 44)", pubkey_str.len())
            ));
        }

        Pubkey::from_str(pubkey_str)
            .map_err(|e| ZisKError::AccountParseError(format!("Invalid pubkey format: {}", e)))
    }

    /// Validate account state consistency
    pub fn validate_account_state(account: &AccountState) -> Result<(), ZisKError> {
        // Check rent exemption
        let rent = Rent::default();
        if !rent.is_exempt(account.lamports, account.data.len()) {
            let minimum_balance = rent.minimum_balance(account.data.len());
            if account.lamports < minimum_balance {
                return Err(ZisKError::AccountValidationError(
                    format!("Account {} is not rent exempt and has insufficient balance", account.pubkey)
                ));
            }
        }

        // Check executable accounts have no data or are owned by loader
        if account.executable && !account.data.is_empty() {
            // Executable accounts should be owned by a loader program
            if account.owner == solana_sdk::system_program::ID {
                return Err(ZisKError::AccountValidationError(
                    format!("Executable account {} cannot be owned by system program", account.pubkey)
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::system_program;

    #[test]
    fn test_solana_account_to_zisk() {
        let pubkey = Pubkey::new_unique();
        let account = Account {
            lamports: 1000,
            data: vec![1, 2, 3],
            owner: system_program::ID,
            executable: false,
            rent_epoch: 0,
        };

        let zisk_account = ZisKAccountSerializer::solana_account_to_zisk(&pubkey, &account);
        
        assert_eq!(zisk_account.pubkey, pubkey);
        assert_eq!(zisk_account.lamports, 1000);
        assert_eq!(zisk_account.data, vec![1, 2, 3]);
        assert_eq!(zisk_account.owner, system_program::ID);
        assert_eq!(zisk_account.executable, false);
        assert_eq!(zisk_account.rent_epoch, 0);
    }

    #[test]
    fn test_zisk_account_to_solana() {
        let zisk_account = AccountState {
            rent_exempt_reserve: 0,
            pubkey: Pubkey::new_unique(),
            lamports: 2000,
            data: vec![4, 5, 6],
            owner: system_program::ID,
            executable: true,
            rent_epoch: 1,
        };

        let solana_account = ZisKAccountSerializer::zisk_account_to_solana(&zisk_account);
        
        assert_eq!(solana_account.lamports, 2000);
        assert_eq!(solana_account.data, vec![4, 5, 6]);
        assert_eq!(solana_account.owner, system_program::ID);
        assert_eq!(solana_account.executable, true);
        assert_eq!(solana_account.rent_epoch, 1);
    }

    #[test]
    fn test_parse_pubkey_safe() {
        let valid_pubkey = "11111111111111111111111111111111";
        let result = ZisKAccountSerializer::parse_pubkey_safe(valid_pubkey);
        assert!(result.is_ok());

        let invalid_pubkey = "invalid";
        let result = ZisKAccountSerializer::parse_pubkey_safe(invalid_pubkey);
        assert!(result.is_err());

        let empty_pubkey = "";
        let result = ZisKAccountSerializer::parse_pubkey_safe(empty_pubkey);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_account_state() {
        let valid_account = AccountState {
            rent_exempt_reserve: 0,
            pubkey: Pubkey::new_unique(),
            lamports: 1000000, // Sufficient for rent exemption
            data: vec![0; 100],
            owner: system_program::ID,
            executable: false,
            rent_epoch: 0,
        };

        let result = ZisKAccountSerializer::validate_account_state(&valid_account);
        assert!(result.is_ok());
    }
}
