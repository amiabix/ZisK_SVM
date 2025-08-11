//! Real Solana Account Loader for ZisK zkVM
//! 
//! This module implements loading of actual Solana account states
//! from the network using RPC calls.

use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use solana_account_decoder::UiAccount;
use anyhow::{Result, Context};
use std::collections::HashMap;
use crate::real_solana_parser::{RealSolanaAccount, RealSolanaParser};

/// Real Solana Account Loader
pub struct RealAccountLoader {
    rpc_url: String,
    accounts: HashMap<String, RealSolanaAccount>,
    parser: RealSolanaParser,
}

/// Solana RPC Response for Account Info
#[derive(Debug, Deserialize)]
struct SolanaRpcAccountResponse {
    jsonrpc: String,
    id: u64,
    result: Option<SolanaRpcAccountResult>,
    error: Option<SolanaRpcError>,
}

#[derive(Debug, Deserialize)]
struct SolanaRpcAccountResult {
    value: Option<UiAccount>,
    context: SolanaRpcContext,
}

#[derive(Debug, Deserialize)]
struct SolanaRpcContext {
    slot: u64,
}

#[derive(Debug, Deserialize)]
struct SolanaRpcError {
    code: i32,
    message: String,
}

/// Solana RPC Response for Multiple Accounts
#[derive(Debug, Deserialize)]
struct SolanaRpcMultipleAccountsResponse {
    jsonrpc: String,
    id: u64,
    result: Option<SolanaRpcMultipleAccountsResult>,
    error: Option<SolanaRpcError>,
}

#[derive(Debug, Deserialize)]
struct SolanaRpcMultipleAccountsResult {
    value: Vec<Option<UiAccount>>,
    context: SolanaRpcContext,
}

impl RealAccountLoader {
    pub fn new(rpc_url: String) -> Self {
        Self {
            rpc_url,
            accounts: HashMap::new(),
            parser: RealSolanaParser::new(),
        }
    }
    
    /// Load a single account by public key
    pub async fn load_account(&mut self, pubkey: &str) -> Result<Option<RealSolanaAccount>> {
        // Check if we already have this account
        if let Some(account) = self.accounts.get(pubkey) {
            return Ok(Some(account.clone()));
        }
        
        // Fetch account from RPC
        let account_data = self.fetch_account_rpc(pubkey).await?;
        
        if let Some(ui_account) = account_data {
            // Load the account into our parser
            self.parser.load_account(pubkey, &ui_account)?;
            
            // Get the loaded account
            if let Some(account) = self.parser.get_account(pubkey) {
                // Convert to RealSolanaAccount format
                let real_account = RealSolanaAccount {
                    pubkey: pubkey.to_string(),
                    lamports: account.lamports,
                    owner: bs58::encode(account.owner).into_string(),
                    executable: account.executable,
                    rent_epoch: account.rent_epoch,
                    data: account.data.clone(),
                };
                
                self.accounts.insert(pubkey.to_string(), real_account.clone());
                Ok(Some(real_account))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
    
    /// Load multiple accounts by public keys
    pub async fn load_multiple_accounts(&mut self, pubkeys: &[String]) -> Result<Vec<Option<RealSolanaAccount>>> {
        let mut results = Vec::new();
        
        for pubkey in pubkeys {
            let account = self.load_account(pubkey).await?;
            results.push(account);
        }
        
        Ok(results)
    }
    
    /// Load accounts for a transaction
    pub async fn load_transaction_accounts(&mut self, transaction: &crate::real_solana_parser::RealSolanaTransaction) -> Result<Vec<RealSolanaAccount>> {
        let mut accounts = Vec::new();
        
        for pubkey in &transaction.message.account_keys {
            if let Some(account) = self.load_account(pubkey).await? {
                accounts.push(account);
            }
        }
        
        Ok(accounts)
    }
    
    /// Fetch account data from Solana RPC
    async fn fetch_account_rpc(&self, pubkey: &str) -> Result<Option<UiAccount>> {
        let client = reqwest::Client::new();
        
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getAccountInfo",
            "params": [
                pubkey,
                {
                    "encoding": "base64",
                    "commitment": "confirmed"
                }
            ]
        });
        
        let response = client
            .post(&self.rpc_url)
            .json(&request_body)
            .send()
            .await
            .context("Failed to send RPC request")?;
        
        let response_text = response.text().await
            .context("Failed to read response text")?;
        
        let rpc_response: SolanaRpcAccountResponse = serde_json::from_str(&response_text)
            .context("Failed to parse RPC response")?;
        
        if let Some(error) = rpc_response.error {
            anyhow::bail!("RPC error: {} (code: {})", error.message, error.code);
        }
        
        if let Some(result) = rpc_response.result {
            Ok(result.value)
        } else {
            Ok(None)
        }
    }
    
    /// Fetch multiple accounts from Solana RPC
    async fn fetch_multiple_accounts_rpc(&self, pubkeys: &[String]) -> Result<Vec<Option<UiAccount>>> {
        let client = reqwest::Client::new();
        
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getMultipleAccounts",
            "params": [
                pubkeys,
                {
                    "encoding": "base64",
                    "commitment": "confirmed"
                }
            ]
        });
        
        let response = client
            .post(&self.rpc_url)
            .json(&request_body)
            .send()
            .await
            .context("Failed to send RPC request")?;
        
        let response_text = response.text().await
            .context("Failed to read response text")?;
        
        let rpc_response: SolanaRpcMultipleAccountsResponse = serde_json::from_str(&response_text)
            .context("Failed to parse RPC response")?;
        
        if let Some(error) = rpc_response.error {
            anyhow::bail!("RPC error: {} (code: {})", error.message, error.code);
        }
        
        if let Some(result) = rpc_response.result {
            Ok(result.value)
        } else {
            Ok(Vec::new())
        }
    }
    
    /// Get account balance
    pub async fn get_account_balance(&self, pubkey: &str) -> Result<u64> {
        let client = reqwest::Client::new();
        
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getBalance",
            "params": [pubkey]
        });
        
        let response = client
            .post(&self.rpc_url)
            .json(&request_body)
            .send()
            .await
            .context("Failed to send RPC request")?;
        
        let response_text = response.text().await
            .context("Failed to read response text")?;
        
        #[derive(Deserialize)]
        struct BalanceResponse {
            jsonrpc: String,
            id: u64,
            result: Option<u64>,
            error: Option<SolanaRpcError>,
        }
        
        let balance_response: BalanceResponse = serde_json::from_str(&response_text)
            .context("Failed to parse balance response")?;
        
        if let Some(error) = balance_response.error {
            anyhow::bail!("RPC error: {} (code: {})", error.message, error.code);
        }
        
        balance_response.result.ok_or_else(|| {
            anyhow::anyhow!("No balance result in response")
        })
    }
    
    /// Get cached account
    pub fn get_cached_account(&self, pubkey: &str) -> Option<&RealSolanaAccount> {
        self.accounts.get(pubkey)
    }
    
    /// Clear account cache
    pub fn clear_cache(&mut self) {
        self.accounts.clear();
    }
    
    /// Get cache statistics
    pub fn get_cache_stats(&self) -> CacheStats {
        CacheStats {
            total_accounts: self.accounts.len(),
            total_memory: self.accounts.values()
                .map(|acc| acc.data.len() + 64) // Approximate memory usage
                .sum(),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_accounts: usize,
    pub total_memory: usize,
}

// Note: Test functions have been removed for production use.
// The account loader now focuses solely on real Solana account data processing.
