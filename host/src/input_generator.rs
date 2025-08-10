use std::fs::File;
use std::io::Write;
use std::collections::HashMap;

/// Solana block data structure for ZisK input
#[derive(Debug, Clone)]
pub struct Block {
    pub prev_hash: [u8; 32],
    pub slot: u64,
    pub accounts: HashMap<[u8; 32], Account>,
    pub transactions: Vec<Transaction>,
}

/// Solana account data
#[derive(Debug, Clone)]
pub struct Account {
    pub lamports: u64,
    pub owner: [u8; 32],
    pub executable: bool,
    pub rent_epoch: u64,
    pub data: Vec<u8>,
}

/// Solana transaction data
#[derive(Debug, Clone)]
pub struct Transaction {
    pub signature: [u8; 64],
    pub message: Vec<u8>,
    pub compute_units: u32,
}

impl Block {
    pub fn new(prev_hash: [u8; 32], slot: u64) -> Self {
        Self {
            prev_hash,
            slot,
            accounts: HashMap::new(),
            transactions: Vec::new(),
        }
    }
    
    pub fn add_account(&mut self, pubkey: [u8; 32], account: Account) {
        self.accounts.insert(pubkey, account);
    }
    
    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
    }
}

impl Account {
    pub fn new(owner: [u8; 32]) -> Self {
        Self {
            lamports: 0,
            owner,
            executable: false,
            rent_epoch: 0,
            data: Vec::new(),
        }
    }
    
    pub fn with_lamports(mut self, lamports: u64) -> Self {
        self.lamports = lamports;
        self
    }
    
    pub fn with_data(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }
}

impl Transaction {
    pub fn new(signature: [u8; 64], message: Vec<u8>) -> Self {
        Self {
            signature,
            message,
            compute_units: 200_000, // Default compute units
        }
    }
    
    pub fn with_compute_units(mut self, compute_units: u32) -> Self {
        self.compute_units = compute_units;
        self
    }
}

/// Generate raw binary input for ZisK zkVM
/// This replaces Solana-specific serialization with manual binary packing
pub fn generate_zk_input(block: &Block, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(output_path)?;
    
    // 1. Write prev_hash (fixed 32 bytes)
    file.write_all(&block.prev_hash)?;
    
    // 2. Write slot (8 bytes)
    file.write_all(&block.slot.to_le_bytes())?;
    
    // 3. Write account count (4 bytes)
    let account_count = block.accounts.len() as u32;
    file.write_all(&account_count.to_le_bytes())?;
    
    // 4. Write accounts (key + lamports + owner + executable + rent_epoch + data_len + data)
    for (pubkey, account) in &block.accounts {
        // Public key (32 bytes)
        file.write_all(pubkey)?;
        
        // Lamports (8 bytes)
        file.write_all(&account.lamports.to_le_bytes())?;
        
        // Owner (32 bytes)
        file.write_all(&account.owner)?;
        
        // Executable (1 byte)
        file.write_all(&[if account.executable { 1 } else { 0 }])?;
        
        // Rent epoch (8 bytes)
        file.write_all(&account.rent_epoch.to_le_bytes())?;
        
        // Data length (4 bytes)
        let data_len = account.data.len() as u32;
        file.write_all(&data_len.to_le_bytes())?;
        
        // Data (variable length)
        if !account.data.is_empty() {
            file.write_all(&account.data)?;
        }
    }
    
    // 5. Write transaction count (4 bytes)
    let transaction_count = block.transactions.len() as u32;
    file.write_all(&transaction_count.to_le_bytes())?;
    
    // 6. Write transactions (signature + message_len + message + compute_units)
    for tx in &block.transactions {
        // Signature (64 bytes)
        file.write_all(&tx.signature)?;
        
        // Message length (4 bytes)
        let message_len = tx.message.len() as u32;
        file.write_all(&message_len.to_le_bytes())?;
        
        // Message (variable length)
        if !tx.message.is_empty() {
            file.write_all(&tx.message)?;
        }
        
        // Compute units (4 bytes)
        file.write_all(&tx.compute_units.to_le_bytes())?;
    }
    
    Ok(())
}

/// Create a test block for development and testing
pub fn create_test_block() -> Block {
    let mut block = Block::new(
        [0x42; 32], // Test prev_hash
        12345,      // Test slot
    );
    
    // Add test accounts
    let test_account = Account::new([0x01; 32])
        .with_lamports(1_000_000)
        .with_data(vec![0x01, 0x02, 0x03, 0x04]);
    
    block.add_account([0xAA; 32], test_account);
    
    // Add test transaction
    let test_tx = Transaction::new(
        [0xBB; 64], // Test signature
        vec![0x01, 0x02, 0x03], // Test message
    );
    
    block.add_transaction(test_tx);
    
    block
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_input_generation() {
        let block = create_test_block();
        let temp_path = "/tmp/test_input.bin";
        
        // Generate input file
        assert!(generate_zk_input(&block, temp_path).is_ok());
        
        // Verify file was created and has expected size
        let metadata = std::fs::metadata(temp_path).unwrap();
        assert!(metadata.len() > 0);
        
        // Cleanup
        std::fs::remove_file(temp_path).unwrap();
    }
    
    #[test]
    fn test_block_creation() {
        let block = Block::new([0x42; 32], 12345);
        assert_eq!(block.slot, 12345);
        assert_eq!(block.prev_hash, [0x42; 32]);
        assert_eq!(block.accounts.len(), 0);
        assert_eq!(block.transactions.len(), 0);
    }
    
    #[test]
    fn test_account_creation() {
        let owner = [0x01; 32];
        let account = Account::new(owner)
            .with_lamports(1_000_000)
            .with_data(vec![0x01, 0x02, 0x03]);
        
        assert_eq!(account.owner, owner);
        assert_eq!(account.lamports, 1_000_000);
        assert_eq!(account.data, vec![0x01, 0x02, 0x03]);
        assert!(!account.executable);
        assert_eq!(account.rent_epoch, 0);
    }
}
