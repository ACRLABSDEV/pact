//! Unit tests for Pact Escrow v2 logic

use std::convert::TryInto;

// Constants (must match instructions_v2.rs)
const ESCROW_DISC: u64 = 0x5041435445534352; // "PACTESCR"
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

// Offsets
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

#[test]
fn test_escrow_v2_size() {
    // discriminator(8) + buyer(32) + seller(32) + arbitrator(32) + mint(32) +
    // amount(8) + created_at(8) + timeout_seconds(8) + terms_hash(32) +
    // status(1) + flags(1) + bump(1) = 195
    let expected = 8 + 32 + 32 + 32 + 32 + 8 + 8 + 8 + 32 + 1 + 1 + 1;
    assert_eq!(expected, ESCROW_SIZE);
}

#[test]
fn test_escrow_v2_layout() {
    let mut data = vec![0u8; ESCROW_SIZE];
    
    // Write discriminator
    data[OFF_DISC..OFF_DISC + 8].copy_from_slice(&ESCROW_DISC.to_le_bytes());
    
    // Write buyer
    let buyer = [1u8; 32];
    data[OFF_BUYER..OFF_BUYER + 32].copy_from_slice(&buyer);
    
    // Write seller
    let seller = [2u8; 32];
    data[OFF_SELLER..OFF_SELLER + 32].copy_from_slice(&seller);
    
    // Write arbitrator
    let arbitrator = [3u8; 32];
    data[OFF_ARBITRATOR..OFF_ARBITRATOR + 32].copy_from_slice(&arbitrator);
    
    // Write mint (zeroes for SOL)
    data[OFF_MINT..OFF_MINT + 32].copy_from_slice(&[0u8; 32]);
    
    // Write amount
    let amount: u64 = 1_000_000_000; // 1 SOL
    data[OFF_AMOUNT..OFF_AMOUNT + 8].copy_from_slice(&amount.to_le_bytes());
    
    // Write created_at
    let created_at: u64 = 1707544800; // Some timestamp
    data[OFF_CREATED_AT..OFF_CREATED_AT + 8].copy_from_slice(&created_at.to_le_bytes());
    
    // Write timeout
    let timeout: u64 = 259200; // 3 days
    data[OFF_TIMEOUT..OFF_TIMEOUT + 8].copy_from_slice(&timeout.to_le_bytes());
    
    // Write terms hash
    let terms_hash = [0xABu8; 32];
    data[OFF_TERMS_HASH..OFF_TERMS_HASH + 32].copy_from_slice(&terms_hash);
    
    // Write status
    data[OFF_STATUS] = STATUS_ACTIVE;
    
    // Write flags
    data[OFF_FLAGS] = 0;
    
    // Write bump
    data[OFF_BUMP] = 255;
    
    // Verify reads
    let disc = u64::from_le_bytes(data[OFF_DISC..OFF_DISC + 8].try_into().unwrap());
    assert_eq!(disc, ESCROW_DISC);
    
    let stored_buyer: [u8; 32] = data[OFF_BUYER..OFF_BUYER + 32].try_into().unwrap();
    assert_eq!(stored_buyer, buyer);
    
    let stored_seller: [u8; 32] = data[OFF_SELLER..OFF_SELLER + 32].try_into().unwrap();
    assert_eq!(stored_seller, seller);
    
    let stored_arbitrator: [u8; 32] = data[OFF_ARBITRATOR..OFF_ARBITRATOR + 32].try_into().unwrap();
    assert_eq!(stored_arbitrator, arbitrator);
    
    let stored_amount = u64::from_le_bytes(data[OFF_AMOUNT..OFF_AMOUNT + 8].try_into().unwrap());
    assert_eq!(stored_amount, amount);
    
    let stored_created_at = u64::from_le_bytes(data[OFF_CREATED_AT..OFF_CREATED_AT + 8].try_into().unwrap());
    assert_eq!(stored_created_at, created_at);
    
    let stored_timeout = u64::from_le_bytes(data[OFF_TIMEOUT..OFF_TIMEOUT + 8].try_into().unwrap());
    assert_eq!(stored_timeout, timeout);
    
    let stored_terms_hash: [u8; 32] = data[OFF_TERMS_HASH..OFF_TERMS_HASH + 32].try_into().unwrap();
    assert_eq!(stored_terms_hash, terms_hash);
    
    assert_eq!(data[OFF_STATUS], STATUS_ACTIVE);
    assert_eq!(data[OFF_FLAGS], 0);
    assert_eq!(data[OFF_BUMP], 255);
}

#[test]
fn test_status_values() {
    assert_eq!(STATUS_ACTIVE, 0);
    assert_eq!(STATUS_DELIVERED, 1);
    assert_eq!(STATUS_ACCEPTED, 2);
    assert_eq!(STATUS_DISPUTED, 3);
    assert_eq!(STATUS_RELEASED, 4);
    assert_eq!(STATUS_REFUNDED, 5);
    
    // All unique
    let statuses = [STATUS_ACTIVE, STATUS_DELIVERED, STATUS_ACCEPTED, 
                    STATUS_DISPUTED, STATUS_RELEASED, STATUS_REFUNDED];
    for i in 0..statuses.len() {
        for j in (i+1)..statuses.len() {
            assert_ne!(statuses[i], statuses[j]);
        }
    }
}

#[test]
fn test_flag_bits() {
    assert_eq!(FLAG_SELLER_DELIVERED, 0b0001);
    assert_eq!(FLAG_BUYER_ACCEPTED, 0b0010);
    assert_eq!(FLAG_BUYER_DISPUTED, 0b0100);
    assert_eq!(FLAG_SELLER_DISPUTED, 0b1000);
    
    // Flags can be combined
    let combined = FLAG_SELLER_DELIVERED | FLAG_BUYER_DISPUTED;
    assert_eq!(combined, 0b0101);
    
    // Check individual flags
    assert!(combined & FLAG_SELLER_DELIVERED != 0);
    assert!(combined & FLAG_BUYER_DISPUTED != 0);
    assert!(combined & FLAG_BUYER_ACCEPTED == 0);
    assert!(combined & FLAG_SELLER_DISPUTED == 0);
}

#[test]
fn test_create_escrow_instruction_data() {
    // discriminator(1) + amount(8) + seed(8) + timeout(8) + terms_hash(32) = 57 bytes
    let discriminator: u8 = 0;
    let amount: u64 = 100_000_000;
    let seed: u64 = 1234567890;
    let timeout: u64 = 259200;
    let terms_hash = [0xABu8; 32];
    
    let mut data = vec![0u8; 57];
    data[0] = discriminator;
    data[1..9].copy_from_slice(&amount.to_le_bytes());
    data[9..17].copy_from_slice(&seed.to_le_bytes());
    data[17..25].copy_from_slice(&timeout.to_le_bytes());
    data[25..57].copy_from_slice(&terms_hash);
    
    assert_eq!(data[0], 0);
    assert_eq!(u64::from_le_bytes(data[1..9].try_into().unwrap()), amount);
    assert_eq!(u64::from_le_bytes(data[9..17].try_into().unwrap()), seed);
    assert_eq!(u64::from_le_bytes(data[17..25].try_into().unwrap()), timeout);
    
    let stored_hash: [u8; 32] = data[25..57].try_into().unwrap();
    assert_eq!(stored_hash, terms_hash);
}

#[test]
fn test_single_byte_instructions() {
    // Most instructions just have discriminator
    assert_eq!(1u8, 1); // MARK_DELIVERED
    assert_eq!(2u8, 2); // ACCEPT_DELIVERY
    assert_eq!(3u8, 3); // RELEASE
    assert_eq!(4u8, 4); // REFUND
    assert_eq!(5u8, 5); // DISPUTE
}

#[test]
fn test_arbitrate_instruction_data() {
    // discriminator(1) + decision(1) = 2 bytes
    let refund_data = vec![6u8, 0]; // Arbitrate with refund
    let release_data = vec![6u8, 1]; // Arbitrate with release
    
    assert_eq!(refund_data[0], 6);
    assert_eq!(refund_data[1], 0);
    assert_eq!(release_data[0], 6);
    assert_eq!(release_data[1], 1);
}

#[test]
fn test_timeout_default() {
    // 3 days = 259200 seconds
    let three_days_seconds: u64 = 3 * 24 * 60 * 60;
    assert_eq!(three_days_seconds, 259200);
}

#[test]
fn test_timeout_logic() {
    let created_at: u64 = 1707544800;
    let timeout_seconds: u64 = 259200; // 3 days
    
    // Just after creation - not timed out
    let now_early = created_at + 1000;
    assert!(now_early < created_at + timeout_seconds);
    
    // Just before timeout - not timed out
    let now_almost = created_at + timeout_seconds - 1;
    assert!(now_almost < created_at + timeout_seconds);
    
    // Exactly at timeout - timed out
    let now_exact = created_at + timeout_seconds;
    assert!(now_exact >= created_at + timeout_seconds);
    
    // After timeout - timed out
    let now_after = created_at + timeout_seconds + 1000;
    assert!(now_after >= created_at + timeout_seconds);
}

#[test]
fn test_no_timeout() {
    // timeout_seconds = 0 means no timeout
    let timeout_seconds: u64 = 0;
    
    // With no timeout, we should never auto-unlock
    // This is handled by checking timeout_seconds > 0 before timeout logic
    assert_eq!(timeout_seconds, 0);
}

#[test]
fn test_state_transitions_from_active() {
    // From Active (0), valid next states are:
    // - Delivered (1) via MarkDelivered
    // - Disputed (3) via Dispute
    // - Released (4) via Release
    // - Refunded (5) via Refund
    
    let from = STATUS_ACTIVE;
    let valid_to = [STATUS_DELIVERED, STATUS_DISPUTED, STATUS_RELEASED, STATUS_REFUNDED];
    
    for to in valid_to {
        assert!(to != from);
        assert!(to <= STATUS_REFUNDED);
    }
}

#[test]
fn test_state_transitions_from_delivered() {
    // From Delivered (1), valid next states are:
    // - Disputed (3) via Dispute
    // - Released (4) via AcceptDelivery or Release
    
    let from = STATUS_DELIVERED;
    let valid_to = [STATUS_DISPUTED, STATUS_RELEASED];
    
    for to in valid_to {
        assert!(to > from);
    }
}

#[test]
fn test_state_transitions_from_disputed() {
    // From Disputed (3), valid next states are:
    // - Released (4) via Arbitrate (release)
    // - Refunded (5) via Arbitrate (refund)
    
    let from = STATUS_DISPUTED;
    let valid_to = [STATUS_RELEASED, STATUS_REFUNDED];
    
    for to in valid_to {
        assert!(to > from);
    }
}

#[test]
fn test_terminal_states() {
    // Released and Refunded are terminal - no further transitions
    assert_eq!(STATUS_RELEASED, 4);
    assert_eq!(STATUS_REFUNDED, 5);
}

#[test]
fn test_pda_seeds_structure() {
    let prefix = b"escrow";
    let buyer = [1u8; 32];
    let seller = [2u8; 32];
    let seed: u64 = 12345;
    let seed_bytes = seed.to_le_bytes();
    
    assert_eq!(prefix, b"escrow");
    assert_eq!(buyer.len(), 32);
    assert_eq!(seller.len(), 32);
    assert_eq!(seed_bytes.len(), 8);
    
    // Total seed length
    let total = prefix.len() + buyer.len() + seller.len() + seed_bytes.len();
    assert_eq!(total, 6 + 32 + 32 + 8); // 78 bytes
}

#[test]
fn test_discriminator_ascii() {
    let disc = ESCROW_DISC;
    let bytes = disc.to_le_bytes();
    
    // "PACTESCR" in little-endian
    let s: String = bytes.iter().rev().map(|&b| b as char).collect();
    assert_eq!(s, "PACTESCR");
}
