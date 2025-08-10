use ziskos::{read_input, set_output};
use crate::shared::constants::{zk_assert, ZkError, MAX_ACCOUNTS, MAX_ACCOUNT_DATA};
use crate::bpf::BpfVm;

/// Solana Virtual Machine for ZisK zkVM
/// Processes blocks and generates state roots
pub struct SolanaVm {
    accounts: Vec<([u8; 32], u64)>, // (pubkey, lamports)
    bpf_vm: BpfVm,
}

impl SolanaVm {
    pub fn new(accounts: Vec<([u8; 32], u64)>) -> Self {
        zk_assert!(accounts.len() <= MAX_ACCOUNTS, ZkError::InvalidInput);
        
        Self {
            accounts,
            bpf_vm: BpfVm::new(),
        }
    }
    
    /// Execute transactions and return state root
    pub fn execute(&mut self, prev_hash: &[u8], transaction_data: &[u8]) -> [u8; 32] {
        // Process transactions
        let mut state_root = [0u8; 32];
        
        // For now, create a simple hash-based state root
        // In production, this would be a proper Merkle tree
        state_root.copy_from_slice(prev_hash);
        
        // Process each transaction
        let mut cursor = 0;
        while cursor < transaction_data.len() {
            if let Ok(transaction) = self.parse_transaction(&transaction_data[cursor..]) {
                self.execute_transaction(transaction)?;
                cursor += transaction.len();
            } else {
                break;
            }
        }
        
        // Generate final state root
        self.generate_state_root()
    }
    
    /// Parse a single transaction from input data
    fn parse_transaction(&self, input: &[u8]) -> Result<Transaction, ZkError> {
        zk_assert!(input.len() >= 64, ZkError::InvalidInput); // Sig + message
        
        let mut cursor = 0;
        
        // Parse signature (64 bytes)
        let signature = input[cursor..cursor+64].try_into()
            .map_err(|_| ZkError::InvalidInput)?;
        cursor += 64;
        
        // Parse message length (4 bytes)
        zk_assert!(cursor + 4 <= input.len(), ZkError::InvalidInput);
        let message_len = u32::from_le_bytes(input[cursor..cursor+4].try_into()
            .map_err(|_| ZkError::InvalidInput)?);
        cursor += 4;
        
        // Parse message
        zk_assert!(cursor + message_len as usize <= input.len(), ZkError::InvalidInput);
        let message = input[cursor..cursor+message_len as usize].to_vec();
        cursor += message_len as usize;
        
        // Parse compute units (4 bytes)
        zk_assert!(cursor + 4 <= input.len(), ZkError::InvalidInput);
        let compute_units = u32::from_le_bytes(input[cursor..cursor+4].try_into()
            .map_err(|_| ZkError::InvalidInput)?);
        cursor += 4;
        
        Ok(Transaction {
            signature,
            message,
            compute_units,
        })
    }
    
    /// Execute a single transaction
    fn execute_transaction(&mut self, transaction: Transaction) -> Result<(), ZkError> {
        // Validate compute units
        zk_assert!(transaction.compute_units <= self.bpf_vm.get_remaining_cycles() as u32, 
                   ZkError::InsufficientCycles);
        
        // Execute BPF program if message contains one
        if !transaction.message.is_empty() {
            self.bpf_vm.execute(&transaction.message)?;
        }
        
        // Update account balances (simplified)
        // In production, this would handle proper account updates
        for (pubkey, lamports) in &mut self.accounts {
            if *pubkey == transaction.signature[..32] {
                *lamports = lamports.saturating_add(1000); // Reward for transaction
                break;
            }
        }
        
        Ok(())
    }
    
    /// Generate final state root
    fn generate_state_root(&self) -> [u8; 32] {
        let mut hasher = sha2::Sha256::new();
        
        // Hash all account states
        for (pubkey, lamports) in &self.accounts {
            hasher.update(pubkey);
            hasher.update(&lamports.to_le_bytes());
        }
        
        // Convert hash to bytes
        let result = hasher.finalize();
        let mut state_root = [0u8; 32];
        state_root.copy_from_slice(&result);
        
        state_root
    }
    
    /// Get transaction length for parsing
    fn get_transaction_length(&self, input: &[u8]) -> Result<usize, ZkError> {
        if input.len() < 68 {
            return Err(ZkError::InvalidInput);
        }
        
        let message_len = u32::from_le_bytes(input[64..68].try_into()
            .map_err(|_| ZkError::InvalidInput)?);
        
        Ok(64 + 4 + message_len as usize + 4) // sig + msg_len + msg + compute_units
    }
}

/// Transaction structure for ZisK processing
#[derive(Debug, Clone)]
struct Transaction {
    signature: [u8; 64],
    message: Vec<u8>,
    compute_units: u32,
}

impl Transaction {
    fn len(&self) -> usize {
        64 + 4 + self.message.len() + 4 // sig + msg_len + msg + compute_units
    }
}

/// Main entry point for ZisK zkVM
/// This function is called by the ZisK runtime
#[no_mangle]
pub extern "C" fn main() {
    // Read input from ZisK
    let input = read_input();
    let mut cursor = 0;
    
    // 1. Parse prev_hash (bytes 0..32)
    zk_assert!(cursor + 32 <= input.len(), ZkError::InvalidInput);
    let prev_hash = &input[cursor..cursor+32];
    cursor += 32;
    
    // 2. Parse slot (8 bytes)
    zk_assert!(cursor + 8 <= input.len(), ZkError::InvalidInput);
    let _slot = u64::from_le_bytes(input[cursor..cursor+8].try_into().unwrap());
    cursor += 8;
    
    // 3. Parse account count (4 bytes)
    zk_assert!(cursor + 4 <= input.len(), ZkError::InvalidInput);
    let account_count = u32::from_le_bytes(input[cursor..cursor+4].try_into().unwrap());
    cursor += 4;
    
    zk_assert!(account_count <= MAX_ACCOUNTS as u32, ZkError::InvalidInput);
    
    // 4. Parse accounts
    let mut accounts = Vec::new();
    for _ in 0..account_count {
        zk_assert!(cursor + 40 <= input.len(), ZkError::InvalidInput);
        
        // Public key (32 bytes)
        let key = input[cursor..cursor+32].try_into().unwrap();
        cursor += 32;
        
        // Lamports (8 bytes)
        let lamports = u64::from_le_bytes(input[cursor..cursor+8].try_into().unwrap());
        cursor += 8;
        
        accounts.push((key, lamports));
    }
    
    // 5. Parse transaction count (4 bytes)
    zk_assert!(cursor + 4 <= input.len(), ZkError::InvalidInput);
    let transaction_count = u32::from_le_bytes(input[cursor..cursor+4].try_into().unwrap());
    cursor += 4;
    
    // 6. Parse transactions
    let mut transactions = Vec::new();
    for _ in 0..transaction_count {
        if cursor >= input.len() {
            break;
        }
        
        // Parse transaction length
        let tx_len = if cursor + 68 <= input.len() {
            let msg_len = u32::from_le_bytes(input[cursor+64..cursor+68].try_into().unwrap());
            64 + 4 + msg_len as usize + 4
        } else {
            break;
        };
        
        zk_assert!(cursor + tx_len <= input.len(), ZkError::InvalidInput);
        
        let tx_data = &input[cursor..cursor+tx_len];
        if let Ok(transaction) = parse_transaction_simple(tx_data) {
            transactions.push(transaction);
        }
        
        cursor += tx_len;
    }
    
    // 7. Create and execute Solana VM
    let mut svm = SolanaVm::new(accounts);
    let state_root = svm.execute(prev_hash, &input[cursor..]);
    
    // 8. Output state root as u32 chunks for ZisK
    for (i, chunk) in state_root.chunks_exact(4).enumerate() {
        let val = u32::from_le_bytes(chunk.try_into().unwrap());
        set_output(i as u32, val);
    }
}

/// Simplified transaction parsing for the guest program
fn parse_transaction_simple(input: &[u8]) -> Result<Transaction, ZkError> {
    zk_assert!(input.len() >= 68, ZkError::InvalidInput);
    
    let signature = input[0..64].try_into().unwrap();
    let message_len = u32::from_le_bytes(input[64..68].try_into().unwrap());
    
    zk_assert!(68 + message_len as usize <= input.len(), ZkError::InvalidInput);
    
    let message = input[68..68+message_len as usize].to_vec();
    let compute_units = if 68 + message_len as usize + 4 <= input.len() {
        u32::from_le_bytes(input[68+message_len as usize..68+message_len as usize+4].try_into().unwrap())
    } else {
        200_000 // Default compute units
    };
    
    Ok(Transaction {
        signature,
        message,
        compute_units,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_solana_vm_creation() {
        let accounts = vec![([0x01; 32], 1000), ([0x02; 32], 2000)];
        let svm = SolanaVm::new(accounts);
        assert_eq!(svm.accounts.len(), 2);
    }
    
    #[test]
    fn test_transaction_parsing() {
        let mut tx_data = vec![0u8; 100];
        tx_data[0..64].copy_from_slice(&[0xAA; 64]); // Signature
        tx_data[64..68].copy_from_slice(&4u32.to_le_bytes()); // Message length
        tx_data[68..72].copy_from_slice(b"test"); // Message
        tx_data[72..76].copy_from_slice(&1000u32.to_le_bytes()); // Compute units
        
        let transaction = parse_transaction_simple(&tx_data).unwrap();
        assert_eq!(transaction.signature, [0xAA; 64]);
        assert_eq!(transaction.message, b"test");
        assert_eq!(transaction.compute_units, 1000);
    }
    
    #[test]
    fn test_state_root_generation() {
        let accounts = vec![([0x01; 32], 1000)];
        let svm = SolanaVm::new(accounts);
        let state_root = svm.generate_state_root();
        assert_eq!(state_root.len(), 32);
    }
}
