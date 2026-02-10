#![no_std]

use pinocchio::{
    account_info::AccountInfo,
    entrypoint,
    nostd_panic_handler,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

entrypoint!(process_instruction);
nostd_panic_handler!();

pub mod instructions;
pub use instructions::*;

// Program ID: S64L6x9bZqDewocv5MrCLeTAq1MKatLqrWfrLpdcDKM
pub const ID: Pubkey = [6, 109, 61, 24, 47, 212, 198, 93, 67, 166, 114, 173, 203, 164, 21, 164, 119, 215, 219, 39, 121, 169, 222, 136, 239, 59, 180, 118, 32, 77, 105, 48];

fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let (discriminator, data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match discriminator {
        0 => CreateEscrow::try_from((data, accounts))?.process(),
        1 => Release::try_from(accounts)?.process(),
        2 => Refund::try_from(accounts)?.process(),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
