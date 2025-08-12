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
    program_id: &Pubkey, // The public key of the account the program was loaded into
    accounts: &[AccountInfo], // The account to write to
    instruction_data: &[u8], // Ignored, all programs are hard-coded and simple
) -> ProgramResult {
    msg!("Simple Calculator program entrypoint");

    // Iterating accounts is safer than indexing
    let accounts_iter = &mut accounts.iter();

    // Get the account to write to
    let account = next_account_info(accounts_iter)?;

    // The account must be owned by the program in order to modify its data
    if account.owner != program_id {
        msg!("Account does not have the correct program id");
        return Err(ProgramError::IncorrectProgramId);
    }

    // Decode the instruction data
    if instruction_data.len() < 1 {
        msg!("Instruction data too short");
        return Err(ProgramError::InvalidInstructionData);
    }

    let operation = instruction_data[0];
    let mut result: u64 = 0;

    // Perform the calculation based on operation
    match operation {
        0 => {
            // Addition: instruction_data[1:9] = first number, instruction_data[9:17] = second number
            if instruction_data.len() < 17 {
                msg!("Addition operation requires 16 bytes of data");
                return Err(ProgramError::InvalidInstructionData);
            }
            let a = u64::from_le_bytes([
                instruction_data[1], instruction_data[2], instruction_data[3], instruction_data[4],
                instruction_data[5], instruction_data[6], instruction_data[7], instruction_data[8]
            ]);
            let b = u64::from_le_bytes([
                instruction_data[9], instruction_data[10], instruction_data[11], instruction_data[12],
                instruction_data[13], instruction_data[14], instruction_data[15], instruction_data[16]
            ]);
            result = a.wrapping_add(b);
            msg!("Addition: {} + {} = {}", a, b, result);
        },
        1 => {
            // Multiplication: instruction_data[1:9] = first number, instruction_data[9:17] = second number
            if instruction_data.len() < 17 {
                msg!("Multiplication operation requires 16 bytes of data");
                return Err(ProgramError::InvalidInstructionData);
            }
            let a = u64::from_le_bytes([
                instruction_data[1], instruction_data[2], instruction_data[3], instruction_data[4],
                instruction_data[5], instruction_data[6], instruction_data[7], instruction_data[8]
            ]);
            let b = u64::from_le_bytes([
                instruction_data[9], instruction_data[10], instruction_data[11], instruction_data[12],
                instruction_data[13], instruction_data[14], instruction_data[15], instruction_data[16]
            ]);
            result = a.wrapping_mul(b);
            msg!("Multiplication: {} * {} = {}", a, b, result);
        },
        2 => {
            // Division: instruction_data[1:9] = dividend, instruction_data[9:17] = divisor
            if instruction_data.len() < 17 {
                msg!("Division operation requires 16 bytes of data");
                return Err(ProgramError::InvalidInstructionData);
            }
            let a = u64::from_le_bytes([
                instruction_data[1], instruction_data[2], instruction_data[3], instruction_data[4],
                instruction_data[5], instruction_data[6], instruction_data[7], instruction_data[8]
            ]);
            let b = u64::from_le_bytes([
                instruction_data[9], instruction_data[10], instruction_data[11], instruction_data[12],
                instruction_data[13], instruction_data[14], instruction_data[15], instruction_data[16]
            ]);
            if b == 0 {
                msg!("Division by zero");
                return Err(ProgramError::InvalidInstructionData);
            }
            result = a / b;
            msg!("Division: {} / {} = {}", a, b, result);
        },
        _ => {
            msg!("Unknown operation: {}", operation);
            return Err(ProgramError::InvalidInstructionData);
        }
    }

    // Write the result to the account data
    let mut data = account.data.borrow_mut();
    data[0..8].copy_from_slice(&result.to_le_bytes());
    data[8] = operation; // Store the operation performed

    msg!("Calculation completed successfully. Result: {}", result);
    Ok(())
}
