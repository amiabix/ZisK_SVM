use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

// Declare and export the program's entrypoint
entrypoint!(process_instruction);

// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("🎉 Hello from ZisK-SVM BPF execution!");
    msg!("Program ID: {}", program_id);
    msg!("Number of accounts: {}", accounts.len());
    msg!("Instruction data length: {}", instruction_data.len());
    
    if !instruction_data.is_empty() {
        msg!("First byte of instruction data: {}", instruction_data[0]);
    }
    
    // Process accounts
    let accounts_iter = &mut accounts.iter();
    if let Ok(account) = next_account_info(accounts_iter) {
        msg!("First account key: {}", account.key);
        msg!("Account lamports: {}", account.lamports());
    }
    
    msg!("✅ ZisK-SVM BPF program executed successfully!");
    Ok(())
}
