use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Seed, Signer},
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::{find_program_address, Pubkey},
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};

// System Program ID
const SYSTEM_PROGRAM_ID: Pubkey = [0u8; 32];

// Escrow account layout:
// [0..8]   discriminator
// [8..40]  buyer pubkey
// [40..72] seller pubkey
// [72..80] amount (u64)
// [80]     status (u8): 0=Active, 1=Released, 2=Refunded
const ESCROW_DISC: u64 = 0x5041435445534352; // "PACTESCR"
const ESCROW_SIZE: usize = 81;

const STATUS_ACTIVE: u8 = 0;
const STATUS_RELEASED: u8 = 1;
const STATUS_REFUNDED: u8 = 2;

/// Derive escrow PDA from buyer + seller + seed
fn derive_escrow(buyer: &Pubkey, seller: &Pubkey, seed: u64, program_id: &Pubkey) -> (Pubkey, u8) {
    find_program_address(
        &[
            b"escrow",
            buyer,
            seller,
            &seed.to_le_bytes(),
        ],
        program_id,
    )
}

// ============================================================================
// CreateEscrow - Buyer creates and funds an escrow
// ============================================================================

pub struct CreateEscrow<'a> {
    pub buyer: &'a AccountInfo,
    pub seller: &'a AccountInfo,
    pub escrow: &'a AccountInfo,
    pub system_program: &'a AccountInfo,
    pub amount: u64,
    pub seed: u64,
}

impl<'a> CreateEscrow<'a> {
    pub fn process(self) -> ProgramResult {
        let Self {
            buyer,
            seller,
            escrow,
            system_program,
            amount,
            seed,
        } = self;

        // Validate buyer is signer
        if !buyer.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Validate amount > 0
        if amount == 0 {
            return Err(ProgramError::InvalidInstructionData);
        }

        // Get program ID from escrow's owner (before creation, it should be system program)
        let program_id = crate::ID;

        // Derive and validate escrow PDA
        let (expected_pda, bump) = derive_escrow(buyer.key(), seller.key(), seed, &program_id);
        if escrow.key() != &expected_pda {
            return Err(ProgramError::InvalidSeeds);
        }

        // Calculate rent
        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(ESCROW_SIZE);

        // Build signer seeds for PDA
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

        // Create account via CPI to System Program
        // Instruction data: discriminator (0) + lamports + space + owner
        let mut create_data = [0u8; 52];
        create_data[0..4].copy_from_slice(&0u32.to_le_bytes()); // CreateAccount = 0
        create_data[4..12].copy_from_slice(&lamports.to_le_bytes());
        create_data[12..20].copy_from_slice(&(ESCROW_SIZE as u64).to_le_bytes());
        create_data[20..52].copy_from_slice(&program_id);

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
        escrow_data[0..8].copy_from_slice(&ESCROW_DISC.to_le_bytes());
        escrow_data[8..40].copy_from_slice(buyer.key());
        escrow_data[40..72].copy_from_slice(seller.key());
        escrow_data[72..80].copy_from_slice(&amount.to_le_bytes());
        escrow_data[80] = STATUS_ACTIVE;
        drop(escrow_data);

        // Transfer funds to escrow via CPI
        let mut transfer_data = [0u8; 12];
        transfer_data[0..4].copy_from_slice(&2u32.to_le_bytes()); // Transfer = 2
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

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for CreateEscrow<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        if accounts.len() < 4 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }
        if data.len() < 16 {
            return Err(ProgramError::InvalidInstructionData);
        }

        let amount = u64::from_le_bytes(data[0..8].try_into().unwrap());
        let seed = u64::from_le_bytes(data[8..16].try_into().unwrap());

        Ok(Self {
            buyer: &accounts[0],
            seller: &accounts[1],
            escrow: &accounts[2],
            system_program: &accounts[3],
            amount,
            seed,
        })
    }
}

// ============================================================================
// Release - Buyer releases funds to seller
// ============================================================================

pub struct Release<'a> {
    pub buyer: &'a AccountInfo,
    pub seller: &'a AccountInfo,
    pub escrow: &'a AccountInfo,
}

impl<'a> Release<'a> {
    pub fn process(self) -> ProgramResult {
        let Self { buyer, seller, escrow } = self;

        // Validate buyer is signer
        if !buyer.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Validate escrow ownership
        if escrow.owner() != &crate::ID {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // Read and validate escrow data
        let escrow_data = escrow.try_borrow_data()?;
        
        // Check discriminator
        let disc = u64::from_le_bytes(escrow_data[0..8].try_into().unwrap());
        if disc != ESCROW_DISC {
            return Err(ProgramError::InvalidAccountData);
        }

        let stored_buyer: Pubkey = escrow_data[8..40].try_into().unwrap();
        let stored_seller: Pubkey = escrow_data[40..72].try_into().unwrap();
        let amount = u64::from_le_bytes(escrow_data[72..80].try_into().unwrap());
        let status = escrow_data[80];
        drop(escrow_data);

        // Validate accounts match
        if buyer.key() != &stored_buyer {
            return Err(ProgramError::InvalidAccountData);
        }
        if seller.key() != &stored_seller {
            return Err(ProgramError::InvalidAccountData);
        }

        // Validate status is active
        if status != STATUS_ACTIVE {
            return Err(ProgramError::InvalidAccountData);
        }

        // Update status to released
        let mut escrow_data = escrow.try_borrow_mut_data()?;
        escrow_data[80] = STATUS_RELEASED;
        drop(escrow_data);

        // Transfer funds from escrow to seller (direct lamport manipulation)
        unsafe {
            let escrow_lamports = escrow.borrow_mut_lamports_unchecked();
            let seller_lamports = seller.borrow_mut_lamports_unchecked();

            *seller_lamports = seller_lamports
                .checked_add(amount)
                .ok_or(ProgramError::ArithmeticOverflow)?;
            *escrow_lamports = escrow_lamports
                .checked_sub(amount)
                .ok_or(ProgramError::InsufficientFunds)?;
        }

        Ok(())
    }
}

impl<'a> TryFrom<&'a [AccountInfo]> for Release<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        if accounts.len() < 3 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }
        Ok(Self {
            buyer: &accounts[0],
            seller: &accounts[1],
            escrow: &accounts[2],
        })
    }
}

// ============================================================================
// Refund - Seller refunds buyer
// ============================================================================

pub struct Refund<'a> {
    pub buyer: &'a AccountInfo,
    pub seller: &'a AccountInfo,
    pub escrow: &'a AccountInfo,
}

impl<'a> Refund<'a> {
    pub fn process(self) -> ProgramResult {
        let Self { buyer, seller, escrow } = self;

        // Validate seller is signer
        if !seller.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Validate escrow ownership
        if escrow.owner() != &crate::ID {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // Read and validate escrow data
        let escrow_data = escrow.try_borrow_data()?;
        
        let disc = u64::from_le_bytes(escrow_data[0..8].try_into().unwrap());
        if disc != ESCROW_DISC {
            return Err(ProgramError::InvalidAccountData);
        }

        let stored_buyer: Pubkey = escrow_data[8..40].try_into().unwrap();
        let stored_seller: Pubkey = escrow_data[40..72].try_into().unwrap();
        let amount = u64::from_le_bytes(escrow_data[72..80].try_into().unwrap());
        let status = escrow_data[80];
        drop(escrow_data);

        // Validate accounts match
        if buyer.key() != &stored_buyer {
            return Err(ProgramError::InvalidAccountData);
        }
        if seller.key() != &stored_seller {
            return Err(ProgramError::InvalidAccountData);
        }

        // Validate status is active
        if status != STATUS_ACTIVE {
            return Err(ProgramError::InvalidAccountData);
        }

        // Update status to refunded
        let mut escrow_data = escrow.try_borrow_mut_data()?;
        escrow_data[80] = STATUS_REFUNDED;
        drop(escrow_data);

        // Transfer funds from escrow back to buyer
        unsafe {
            let escrow_lamports = escrow.borrow_mut_lamports_unchecked();
            let buyer_lamports = buyer.borrow_mut_lamports_unchecked();

            *buyer_lamports = buyer_lamports
                .checked_add(amount)
                .ok_or(ProgramError::ArithmeticOverflow)?;
            *escrow_lamports = escrow_lamports
                .checked_sub(amount)
                .ok_or(ProgramError::InsufficientFunds)?;
        }

        Ok(())
    }
}

impl<'a> TryFrom<&'a [AccountInfo]> for Refund<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        if accounts.len() < 3 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }
        Ok(Self {
            buyer: &accounts[0],
            seller: &accounts[1],
            escrow: &accounts[2],
        })
    }
}
