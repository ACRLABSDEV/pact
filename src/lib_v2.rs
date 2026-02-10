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

pub mod instructions_v2;
pub use instructions_v2::*;

// Program ID - TO BE UPDATED AFTER DEPLOY
pub const ID: Pubkey = [0u8; 32]; // Placeholder

// Instruction discriminators
pub const IX_CREATE_ESCROW: u8 = 0;
pub const IX_MARK_DELIVERED: u8 = 1;
pub const IX_ACCEPT_DELIVERY: u8 = 2;
pub const IX_RELEASE: u8 = 3;
pub const IX_REFUND: u8 = 4;
pub const IX_DISPUTE: u8 = 5;
pub const IX_ARBITRATE: u8 = 6;

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let (discriminator, data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match *discriminator {
        IX_CREATE_ESCROW => CreateEscrowV2::process(program_id, accounts, data),
        IX_MARK_DELIVERED => MarkDelivered::process(accounts),
        IX_ACCEPT_DELIVERY => AcceptDelivery::process(accounts),
        IX_RELEASE => ReleaseV2::process(accounts),
        IX_REFUND => RefundV2::process(accounts),
        IX_DISPUTE => Dispute::process(accounts),
        IX_ARBITRATE => Arbitrate::process(accounts, data),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
