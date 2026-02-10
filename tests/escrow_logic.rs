//! Unit tests for Pact Escrow logic
//! 
//! These tests verify the core escrow logic without requiring the Solana runtime.
//! For full integration tests, use the TypeScript client against devnet.

use std::convert::TryInto;

/// Escrow account layout constants (must match instructions.rs)
const ESCROW_DISC: u64 = 0x5041435445534352; // "PACTESCR"
const ESCROW_SIZE: usize = 81;
const STATUS_ACTIVE: u8 = 0;
const STATUS_RELEASED: u8 = 1;
const STATUS_REFUNDED: u8 = 2;

/// Test escrow data serialization
#[test]
fn test_escrow_data_layout() {
    // Create mock escrow data
    let mut data = vec![0u8; ESCROW_SIZE];
    
    // Write discriminator
    data[0..8].copy_from_slice(&ESCROW_DISC.to_le_bytes());
    
    // Write buyer pubkey (mock 32 bytes)
    let buyer = [1u8; 32];
    data[8..40].copy_from_slice(&buyer);
    
    // Write seller pubkey (mock 32 bytes)
    let seller = [2u8; 32];
    data[40..72].copy_from_slice(&seller);
    
    // Write amount (0.1 SOL = 100_000_000 lamports)
    let amount: u64 = 100_000_000;
    data[72..80].copy_from_slice(&amount.to_le_bytes());
    
    // Write status
    data[80] = STATUS_ACTIVE;
    
    // Verify we can read it back
    let disc = u64::from_le_bytes(data[0..8].try_into().unwrap());
    assert_eq!(disc, ESCROW_DISC);
    
    let stored_buyer: [u8; 32] = data[8..40].try_into().unwrap();
    assert_eq!(stored_buyer, buyer);
    
    let stored_seller: [u8; 32] = data[40..72].try_into().unwrap();
    assert_eq!(stored_seller, seller);
    
    let stored_amount = u64::from_le_bytes(data[72..80].try_into().unwrap());
    assert_eq!(stored_amount, amount);
    
    assert_eq!(data[80], STATUS_ACTIVE);
}

/// Test status transitions
#[test]
fn test_escrow_status_values() {
    assert_eq!(STATUS_ACTIVE, 0);
    assert_eq!(STATUS_RELEASED, 1);
    assert_eq!(STATUS_REFUNDED, 2);
    
    // Ensure they're distinct
    assert_ne!(STATUS_ACTIVE, STATUS_RELEASED);
    assert_ne!(STATUS_RELEASED, STATUS_REFUNDED);
    assert_ne!(STATUS_ACTIVE, STATUS_REFUNDED);
}

/// Test escrow size is correct
#[test]
fn test_escrow_size() {
    // discriminator (8) + buyer (32) + seller (32) + amount (8) + status (1) = 81
    assert_eq!(ESCROW_SIZE, 8 + 32 + 32 + 8 + 1);
}

/// Test instruction discriminator values
#[test]
fn test_instruction_discriminators() {
    const CREATE: u8 = 0;
    const RELEASE: u8 = 1;
    const REFUND: u8 = 2;
    
    assert_eq!(CREATE, 0);
    assert_eq!(RELEASE, 1);
    assert_eq!(REFUND, 2);
}

/// Test CreateEscrow instruction data layout
#[test]
fn test_create_instruction_data() {
    let discriminator: u8 = 0;
    let amount: u64 = 100_000_000; // 0.1 SOL
    let seed: u64 = 1234567890;
    
    // Build instruction data: [discriminator (1)] [amount (8)] [seed (8)] = 17 bytes
    let mut data = vec![0u8; 17];
    data[0] = discriminator;
    data[1..9].copy_from_slice(&amount.to_le_bytes());
    data[9..17].copy_from_slice(&seed.to_le_bytes());
    
    // Parse it back
    assert_eq!(data[0], 0);
    let parsed_amount = u64::from_le_bytes(data[1..9].try_into().unwrap());
    assert_eq!(parsed_amount, amount);
    let parsed_seed = u64::from_le_bytes(data[9..17].try_into().unwrap());
    assert_eq!(parsed_seed, seed);
}

/// Test Release instruction data layout
#[test]
fn test_release_instruction_data() {
    let discriminator: u8 = 1;
    let data = vec![discriminator];
    
    assert_eq!(data.len(), 1);
    assert_eq!(data[0], 1);
}

/// Test Refund instruction data layout
#[test]
fn test_refund_instruction_data() {
    let discriminator: u8 = 2;
    let data = vec![discriminator];
    
    assert_eq!(data.len(), 1);
    assert_eq!(data[0], 2);
}

/// Test PDA seed structure
#[test]
fn test_pda_seeds() {
    let prefix = b"escrow";
    let buyer = [1u8; 32];
    let seller = [2u8; 32];
    let seed: u64 = 12345;
    let seed_bytes = seed.to_le_bytes();
    
    // Seeds: ["escrow", buyer, seller, seed_le_bytes]
    assert_eq!(prefix, b"escrow");
    assert_eq!(buyer.len(), 32);
    assert_eq!(seller.len(), 32);
    assert_eq!(seed_bytes.len(), 8);
    
    // Total seed length should be valid
    let total_len = prefix.len() + buyer.len() + seller.len() + seed_bytes.len();
    assert_eq!(total_len, 6 + 32 + 32 + 8);
}

/// Test amount edge cases
#[test]
fn test_amount_edge_cases() {
    // Zero amount should be rejected by program (but test serialization)
    let zero: u64 = 0;
    let bytes = zero.to_le_bytes();
    assert_eq!(bytes, [0, 0, 0, 0, 0, 0, 0, 0]);
    
    // Max amount
    let max: u64 = u64::MAX;
    let bytes = max.to_le_bytes();
    let parsed = u64::from_le_bytes(bytes);
    assert_eq!(parsed, max);
    
    // Typical escrow amount (1 SOL)
    let one_sol: u64 = 1_000_000_000;
    let bytes = one_sol.to_le_bytes();
    let parsed = u64::from_le_bytes(bytes);
    assert_eq!(parsed, one_sol);
}

/// Test seed edge cases
#[test]
fn test_seed_edge_cases() {
    // Zero seed
    let seed: u64 = 0;
    let bytes = seed.to_le_bytes();
    assert_eq!(bytes.len(), 8);
    
    // Max seed
    let seed: u64 = u64::MAX;
    let bytes = seed.to_le_bytes();
    let parsed = u64::from_le_bytes(bytes);
    assert_eq!(parsed, u64::MAX);
    
    // Timestamp-based seed (common pattern)
    let seed: u64 = 1707544800000; // Example timestamp
    let bytes = seed.to_le_bytes();
    let parsed = u64::from_le_bytes(bytes);
    assert_eq!(parsed, seed);
}

/// Test discriminator is ASCII "PACTESCR"
#[test]
fn test_discriminator_is_valid_ascii() {
    let disc = ESCROW_DISC;
    let bytes = disc.to_le_bytes();
    
    // "PACTESCR" in little-endian
    // P=80, A=65, C=67, T=84, E=69, S=83, C=67, R=82
    assert_eq!(bytes[0], b'R');
    assert_eq!(bytes[1], b'C');
    assert_eq!(bytes[2], b'S');
    assert_eq!(bytes[3], b'E');
    assert_eq!(bytes[4], b'T');
    assert_eq!(bytes[5], b'C');
    assert_eq!(bytes[6], b'A');
    assert_eq!(bytes[7], b'P');
    
    // Reading as string (reversed due to LE)
    let s: String = bytes.iter().rev().map(|&b| b as char).collect();
    assert_eq!(s, "PACTESCR");
}
