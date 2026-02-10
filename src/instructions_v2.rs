use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Seed, Signer},
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::{find_program_address, Pubkey},
    sysvars::{clock::Clock, rent::Rent, Sysvar},
    ProgramResult,
};

// ============================================================================
// Constants
// ============================================================================

const SYSTEM_PROGRAM_ID: Pubkey = [0u8; 32];

// Escrow discriminator: "PACTESCR" as u64 LE
const ESCROW_DISC: u64 = 0x5041435445534352;

// Escrow account size (v2)
// discriminator(8) + buyer(32) + seller(32) + arbitrator(32) + mint(32) + 
// amount(8) + created_at(8) + timeout_seconds(8) + terms_hash(32) + 
// status(1) + flags(1) + bump(1) = 195 bytes
const ESCROW_SIZE: usize = 195;

// Status values
const STATUS_ACTIVE: u8 = 0;
const STATUS_DELIVERED: u8 = 1;
const STATUS_ACCEPTED: u8 = 2;
const STATUS_DISPUTED: u8 = 3;
const STATUS_RELEASED: u8 = 4;
const STATUS_REFUNDED: u8 = 5;

// Flag bits
const FLAG_SELLER_DELIVERED: u8 = 1 << 0;
const FLAG_BUYER_ACCEPTED: u8 = 1 << 1;
const FLAG_BUYER_DISPUTED: u8 = 1 << 2;
const FLAG_SELLER_DISPUTED: u8 = 1 << 3;

// Account layout offsets
const OFF_DISC: usize = 0;
const OFF_BUYER: usize = 8;
const OFF_SELLER: usize = 40;
const OFF_ARBITRATOR: usize = 72;
const OFF_MINT: usize = 104;
const OFF_AMOUNT: usize = 136;
const OFF_CREATED_AT: usize = 144;
const OFF_TIMEOUT: usize = 152;
const OFF_TERMS_HASH: usize = 160;
const OFF_STATUS: usize = 192;
const OFF_FLAGS: usize = 193;
const OFF_BUMP: usize = 194;

// ============================================================================
// Helpers
// ============================================================================

fn derive_escrow(buyer: &Pubkey, seller: &Pubkey, seed: u64, program_id: &Pubkey) -> (Pubkey, u8) {
    find_program_address(
        &[b"escrow", buyer, seller, &seed.to_le_bytes()],
        program_id,
    )
}

fn read_pubkey(data: &[u8], offset: usize) -> Pubkey {
    let mut pk = [0u8; 32];
    pk.copy_from_slice(&data[offset..offset + 32]);
    pk
}

fn read_u64(data: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap())
}

fn write_pubkey(data: &mut [u8], offset: usize, pk: &Pubkey) {
    data[offset..offset + 32].copy_from_slice(pk);
}

fn write_u64(data: &mut [u8], offset: usize, val: u64) {
    data[offset..offset + 8].copy_from_slice(&val.to_le_bytes());
}

// ============================================================================
// CreateEscrowV2
// ============================================================================

pub struct CreateEscrowV2;

impl CreateEscrowV2 {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
        // Accounts: buyer, seller, arbitrator, escrow, system_program
        if accounts.len() < 5 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let buyer = &accounts[0];
        let seller = &accounts[1];
        let arbitrator = &accounts[2];
        let escrow = &accounts[3];
        let system_program = &accounts[4];

        // Parse instruction data: amount(8) + seed(8) + timeout_seconds(8) + terms_hash(32) = 56 bytes
        if data.len() < 56 {
            return Err(ProgramError::InvalidInstructionData);
        }

        let amount = u64::from_le_bytes(data[0..8].try_into().unwrap());
        let seed = u64::from_le_bytes(data[8..16].try_into().unwrap());
        let timeout_seconds = u64::from_le_bytes(data[16..24].try_into().unwrap());
        let mut terms_hash = [0u8; 32];
        terms_hash.copy_from_slice(&data[24..56]);

        // Validate
        if !buyer.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }
        if amount == 0 {
            return Err(ProgramError::InvalidInstructionData);
        }

        // Derive and validate PDA
        let (expected_pda, bump) = derive_escrow(buyer.key(), seller.key(), seed, program_id);
        if escrow.key() != &expected_pda {
            return Err(ProgramError::InvalidSeeds);
        }

        // Get current timestamp
        let clock = Clock::get()?;
        let created_at = clock.unix_timestamp as u64;

        // Calculate rent
        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(ESCROW_SIZE);

        // Create account
        let bump_bytes = [bump];
        let seed_bytes = seed.to_le_bytes();
        let signer_seeds = [
            Seed::from(b"escrow".as_slice()),
            Seed::from(buyer.key().as_ref()),
            Seed::from(seller.key().as_ref()),
            Seed::from(seed_bytes.as_ref()),
            Seed::from(bump_bytes.as_ref()),
        ];
        let signer = Signer::from(&signer_seeds);

        let mut create_data = [0u8; 52];
        create_data[0..4].copy_from_slice(&0u32.to_le_bytes());
        create_data[4..12].copy_from_slice(&lamports.to_le_bytes());
        create_data[12..20].copy_from_slice(&(ESCROW_SIZE as u64).to_le_bytes());
        create_data[20..52].copy_from_slice(program_id);

        let create_accounts = [
            AccountMeta::writable_signer(buyer.key()),
            AccountMeta::writable_signer(escrow.key()),
        ];

        let create_ix = Instruction {
            program_id: system_program.key(),
            accounts: &create_accounts,
            data: &create_data,
        };

        invoke_signed(&create_ix, &[buyer, escrow], &[signer])?;

        // Initialize escrow data
        let mut escrow_data = escrow.try_borrow_mut_data()?;
        
        write_u64(&mut escrow_data, OFF_DISC, ESCROW_DISC);
        write_pubkey(&mut escrow_data, OFF_BUYER, buyer.key());
        write_pubkey(&mut escrow_data, OFF_SELLER, seller.key());
        write_pubkey(&mut escrow_data, OFF_ARBITRATOR, arbitrator.key());
        // For v2 native SOL, we just use zeroes for mint
        escrow_data[OFF_MINT..OFF_MINT + 32].copy_from_slice(&[0u8; 32]);
        write_u64(&mut escrow_data, OFF_AMOUNT, amount);
        write_u64(&mut escrow_data, OFF_CREATED_AT, created_at);
        write_u64(&mut escrow_data, OFF_TIMEOUT, timeout_seconds);
        escrow_data[OFF_TERMS_HASH..OFF_TERMS_HASH + 32].copy_from_slice(&terms_hash);
        escrow_data[OFF_STATUS] = STATUS_ACTIVE;
        escrow_data[OFF_FLAGS] = 0;
        escrow_data[OFF_BUMP] = bump;
        
        drop(escrow_data);

        // Transfer funds to escrow
        let mut transfer_data = [0u8; 12];
        transfer_data[0..4].copy_from_slice(&2u32.to_le_bytes());
        transfer_data[4..12].copy_from_slice(&amount.to_le_bytes());

        let transfer_accounts = [
            AccountMeta::writable_signer(buyer.key()),
            AccountMeta::writable(escrow.key()),
        ];

        let transfer_ix = Instruction {
            program_id: system_program.key(),
            accounts: &transfer_accounts,
            data: &transfer_data,
        };

        invoke_signed::<2>(&transfer_ix, &[buyer, escrow], &[])?;

        Ok(())
    }
}

// ============================================================================
// MarkDelivered
// ============================================================================

pub struct MarkDelivered;

impl MarkDelivered {
    pub fn process(accounts: &[AccountInfo]) -> ProgramResult {
        if accounts.len() < 2 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let seller = &accounts[0];
        let escrow = &accounts[1];

        if !seller.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut escrow_data = escrow.try_borrow_mut_data()?;

        // Validate discriminator
        let disc = read_u64(&escrow_data, OFF_DISC);
        if disc != ESCROW_DISC {
            return Err(ProgramError::InvalidAccountData);
        }

        // Validate seller
        let stored_seller = read_pubkey(&escrow_data, OFF_SELLER);
        if seller.key() != &stored_seller {
            return Err(ProgramError::InvalidAccountData);
        }

        // Validate status
        let status = escrow_data[OFF_STATUS];
        if status != STATUS_ACTIVE {
            return Err(ProgramError::InvalidAccountData);
        }

        // Update flags and status
        escrow_data[OFF_FLAGS] |= FLAG_SELLER_DELIVERED;
        escrow_data[OFF_STATUS] = STATUS_DELIVERED;

        Ok(())
    }
}

// ============================================================================
// AcceptDelivery
// ============================================================================

pub struct AcceptDelivery;

impl AcceptDelivery {
    pub fn process(accounts: &[AccountInfo]) -> ProgramResult {
        if accounts.len() < 3 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let buyer = &accounts[0];
        let seller = &accounts[1];
        let escrow = &accounts[2];

        if !buyer.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut escrow_data = escrow.try_borrow_mut_data()?;

        // Validate
        let disc = read_u64(&escrow_data, OFF_DISC);
        if disc != ESCROW_DISC {
            return Err(ProgramError::InvalidAccountData);
        }

        let stored_buyer = read_pubkey(&escrow_data, OFF_BUYER);
        let stored_seller = read_pubkey(&escrow_data, OFF_SELLER);
        if buyer.key() != &stored_buyer || seller.key() != &stored_seller {
            return Err(ProgramError::InvalidAccountData);
        }

        let status = escrow_data[OFF_STATUS];
        if status != STATUS_DELIVERED {
            return Err(ProgramError::InvalidAccountData);
        }

        let amount = read_u64(&escrow_data, OFF_AMOUNT);

        // Update status
        escrow_data[OFF_FLAGS] |= FLAG_BUYER_ACCEPTED;
        escrow_data[OFF_STATUS] = STATUS_RELEASED;
        drop(escrow_data);

        // Transfer funds to seller
        unsafe {
            let escrow_lamports = escrow.borrow_mut_lamports_unchecked();
            let seller_lamports = seller.borrow_mut_lamports_unchecked();
            *seller_lamports = seller_lamports.checked_add(amount).ok_or(ProgramError::ArithmeticOverflow)?;
            *escrow_lamports = escrow_lamports.checked_sub(amount).ok_or(ProgramError::InsufficientFunds)?;
        }

        Ok(())
    }
}

// ============================================================================
// ReleaseV2
// ============================================================================

pub struct ReleaseV2;

impl ReleaseV2 {
    pub fn process(accounts: &[AccountInfo]) -> ProgramResult {
        if accounts.len() < 3 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let buyer = &accounts[0];
        let seller = &accounts[1];
        let escrow = &accounts[2];

        if !buyer.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut escrow_data = escrow.try_borrow_mut_data()?;

        let disc = read_u64(&escrow_data, OFF_DISC);
        if disc != ESCROW_DISC {
            return Err(ProgramError::InvalidAccountData);
        }

        let stored_buyer = read_pubkey(&escrow_data, OFF_BUYER);
        let stored_seller = read_pubkey(&escrow_data, OFF_SELLER);
        if buyer.key() != &stored_buyer || seller.key() != &stored_seller {
            return Err(ProgramError::InvalidAccountData);
        }

        let status = escrow_data[OFF_STATUS];
        // Can release from Active, Delivered, or Accepted (but not Disputed)
        if status == STATUS_DISPUTED || status == STATUS_RELEASED || status == STATUS_REFUNDED {
            return Err(ProgramError::InvalidAccountData);
        }

        let amount = read_u64(&escrow_data, OFF_AMOUNT);
        escrow_data[OFF_STATUS] = STATUS_RELEASED;
        drop(escrow_data);

        unsafe {
            let escrow_lamports = escrow.borrow_mut_lamports_unchecked();
            let seller_lamports = seller.borrow_mut_lamports_unchecked();
            *seller_lamports = seller_lamports.checked_add(amount).ok_or(ProgramError::ArithmeticOverflow)?;
            *escrow_lamports = escrow_lamports.checked_sub(amount).ok_or(ProgramError::InsufficientFunds)?;
        }

        Ok(())
    }
}

// ============================================================================
// RefundV2
// ============================================================================

pub struct RefundV2;

impl RefundV2 {
    pub fn process(accounts: &[AccountInfo]) -> ProgramResult {
        if accounts.len() < 4 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let authority = &accounts[0];
        let buyer = &accounts[1];
        let seller = &accounts[2];
        let escrow = &accounts[3];

        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut escrow_data = escrow.try_borrow_mut_data()?;

        let disc = read_u64(&escrow_data, OFF_DISC);
        if disc != ESCROW_DISC {
            return Err(ProgramError::InvalidAccountData);
        }

        let stored_buyer = read_pubkey(&escrow_data, OFF_BUYER);
        let stored_seller = read_pubkey(&escrow_data, OFF_SELLER);
        let stored_arbitrator = read_pubkey(&escrow_data, OFF_ARBITRATOR);

        if buyer.key() != &stored_buyer || seller.key() != &stored_seller {
            return Err(ProgramError::InvalidAccountData);
        }

        let status = escrow_data[OFF_STATUS];
        if status == STATUS_RELEASED || status == STATUS_REFUNDED {
            return Err(ProgramError::InvalidAccountData);
        }

        let amount = read_u64(&escrow_data, OFF_AMOUNT);
        let created_at = read_u64(&escrow_data, OFF_CREATED_AT);
        let timeout_seconds = read_u64(&escrow_data, OFF_TIMEOUT);

        // Check who can refund
        let is_seller = authority.key() == &stored_seller;
        let is_buyer = authority.key() == &stored_buyer;
        let is_arbitrator = authority.key() == &stored_arbitrator;

        let clock = Clock::get()?;
        let now = clock.unix_timestamp as u64;
        let timeout_reached = timeout_seconds > 0 && now >= created_at + timeout_seconds;

        // Seller can always refund
        // Buyer can refund if: timeout reached OR status is Active (no delivery yet)
        // Arbitrator can refund if disputed
        let can_refund = is_seller 
            || (is_buyer && (timeout_reached || status == STATUS_ACTIVE))
            || (is_arbitrator && status == STATUS_DISPUTED);

        if !can_refund {
            return Err(ProgramError::InvalidAccountData);
        }

        escrow_data[OFF_STATUS] = STATUS_REFUNDED;
        drop(escrow_data);

        unsafe {
            let escrow_lamports = escrow.borrow_mut_lamports_unchecked();
            let buyer_lamports = buyer.borrow_mut_lamports_unchecked();
            *buyer_lamports = buyer_lamports.checked_add(amount).ok_or(ProgramError::ArithmeticOverflow)?;
            *escrow_lamports = escrow_lamports.checked_sub(amount).ok_or(ProgramError::InsufficientFunds)?;
        }

        Ok(())
    }
}

// ============================================================================
// Dispute
// ============================================================================

pub struct Dispute;

impl Dispute {
    pub fn process(accounts: &[AccountInfo]) -> ProgramResult {
        if accounts.len() < 2 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let authority = &accounts[0];
        let escrow = &accounts[1];

        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut escrow_data = escrow.try_borrow_mut_data()?;

        let disc = read_u64(&escrow_data, OFF_DISC);
        if disc != ESCROW_DISC {
            return Err(ProgramError::InvalidAccountData);
        }

        let stored_buyer = read_pubkey(&escrow_data, OFF_BUYER);
        let stored_seller = read_pubkey(&escrow_data, OFF_SELLER);

        let is_buyer = authority.key() == &stored_buyer;
        let is_seller = authority.key() == &stored_seller;

        if !is_buyer && !is_seller {
            return Err(ProgramError::InvalidAccountData);
        }

        let status = escrow_data[OFF_STATUS];
        // Can only dispute Active or Delivered
        if status != STATUS_ACTIVE && status != STATUS_DELIVERED {
            return Err(ProgramError::InvalidAccountData);
        }

        // Set dispute flag
        if is_buyer {
            escrow_data[OFF_FLAGS] |= FLAG_BUYER_DISPUTED;
        } else {
            escrow_data[OFF_FLAGS] |= FLAG_SELLER_DISPUTED;
        }
        escrow_data[OFF_STATUS] = STATUS_DISPUTED;

        Ok(())
    }
}

// ============================================================================
// Arbitrate
// ============================================================================

pub struct Arbitrate;

impl Arbitrate {
    pub fn process(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
        if accounts.len() < 4 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let arbitrator = &accounts[0];
        let buyer = &accounts[1];
        let seller = &accounts[2];
        let escrow = &accounts[3];

        if !arbitrator.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Decision: 0 = refund, 1 = release
        if data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        let decision = data[0];

        let mut escrow_data = escrow.try_borrow_mut_data()?;

        let disc = read_u64(&escrow_data, OFF_DISC);
        if disc != ESCROW_DISC {
            return Err(ProgramError::InvalidAccountData);
        }

        let stored_buyer = read_pubkey(&escrow_data, OFF_BUYER);
        let stored_seller = read_pubkey(&escrow_data, OFF_SELLER);
        let stored_arbitrator = read_pubkey(&escrow_data, OFF_ARBITRATOR);

        if arbitrator.key() != &stored_arbitrator {
            return Err(ProgramError::InvalidAccountData);
        }
        if buyer.key() != &stored_buyer || seller.key() != &stored_seller {
            return Err(ProgramError::InvalidAccountData);
        }

        // Check arbitrator is not zero (no arbitrator set)
        if stored_arbitrator == [0u8; 32] {
            return Err(ProgramError::InvalidAccountData);
        }

        let status = escrow_data[OFF_STATUS];
        if status != STATUS_DISPUTED {
            return Err(ProgramError::InvalidAccountData);
        }

        let amount = read_u64(&escrow_data, OFF_AMOUNT);

        if decision == 0 {
            // Refund to buyer
            escrow_data[OFF_STATUS] = STATUS_REFUNDED;
            drop(escrow_data);

            unsafe {
                let escrow_lamports = escrow.borrow_mut_lamports_unchecked();
                let buyer_lamports = buyer.borrow_mut_lamports_unchecked();
                *buyer_lamports = buyer_lamports.checked_add(amount).ok_or(ProgramError::ArithmeticOverflow)?;
                *escrow_lamports = escrow_lamports.checked_sub(amount).ok_or(ProgramError::InsufficientFunds)?;
            }
        } else {
            // Release to seller
            escrow_data[OFF_STATUS] = STATUS_RELEASED;
            drop(escrow_data);

            unsafe {
                let escrow_lamports = escrow.borrow_mut_lamports_unchecked();
                let seller_lamports = seller.borrow_mut_lamports_unchecked();
                *seller_lamports = seller_lamports.checked_add(amount).ok_or(ProgramError::ArithmeticOverflow)?;
                *escrow_lamports = escrow_lamports.checked_sub(amount).ok_or(ProgramError::InsufficientFunds)?;
            }
        }

        Ok(())
    }
}
