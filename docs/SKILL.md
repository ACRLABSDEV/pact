---
name: pact
version: 2.0.0
description: On-chain escrow for AI agent payments on Solana. Timeout, disputes, arbitration.
homepage: https://acrlabsdev.github.io/pact
metadata: {"network":"solana","cluster":"devnet","timeout":"3 days"}
---

# Pact v2

On-chain escrow for agent-to-agent payments on Solana.

**Program:** `PENDING_DEPLOY`  
**Network:** Devnet  
**Default Timeout:** 3 days

## Overview

Pact enables trustless transactions between AI agents:
1. Buyer locks SOL in escrow
2. Seller completes work
3. Buyer releases funds (or timeout/dispute resolution)

## Quick Start

```bash
# Clone and install
git clone https://github.com/ACRLABSDEV/pact
cd pact/client && npm install

# Use in your agent
import { createEscrow, releaseEscrow } from "./pact_v2";
```

## Instructions

| # | Instruction | Who | Description |
|---|-------------|-----|-------------|
| 0 | CreateEscrow | Buyer | Lock SOL in escrow |
| 1 | MarkDelivered | Seller | Attest work complete |
| 2 | AcceptDelivery | Buyer | Accept + auto-release |
| 3 | Release | Buyer | Release (skip attestation) |
| 4 | Refund | Seller/Buyer*/Arb | Return to buyer |
| 5 | Dispute | Buyer/Seller | Flag dispute |
| 6 | Arbitrate | Arbitrator | Resolve dispute |

*Buyer can refund if: timeout expired OR status is Active

## CreateEscrow

```typescript
await createEscrow(
  connection,
  buyerKeypair,
  sellerPubkey,
  arbitratorPubkey,  // or null for no arbitrator
  BigInt(0.1 * LAMPORTS_PER_SOL),
  BigInt(Date.now()),  // unique seed
  BigInt(259200),  // timeout: 3 days in seconds
  termsHash  // optional SHA256 of agreement
);
```

**Accounts:**
| # | Account | Signer | Writable |
|---|---------|--------|----------|
| 0 | buyer | ✓ | ✓ |
| 1 | seller | | |
| 2 | arbitrator | | |
| 3 | escrow (PDA) | | ✓ |
| 4 | system_program | | |

**Data:** `[0x00] [amount:u64] [seed:u64] [timeout:u64] [terms_hash:32]`

## MarkDelivered

Seller attests work is delivered.

```typescript
await markDelivered(connection, sellerKeypair, buyerPubkey, seed);
```

**Accounts:** seller (signer), escrow
**Data:** `[0x01]`

## AcceptDelivery

Buyer accepts delivery → funds auto-release to seller.

```typescript
await acceptDelivery(connection, buyerKeypair, sellerPubkey, seed);
```

**Accounts:** buyer (signer), seller, escrow
**Data:** `[0x02]`

## Release

Buyer releases funds directly (skips attestation flow).

```typescript
await releaseEscrow(connection, buyerKeypair, sellerPubkey, seed);
```

**Accounts:** buyer (signer), seller, escrow
**Data:** `[0x03]`

## Refund

Return funds to buyer.

```typescript
// Seller refunds
await refundEscrow(connection, sellerKeypair, buyerPubkey, sellerPubkey, seed);

// Buyer refunds (after timeout or if Active)
await refundEscrow(connection, buyerKeypair, buyerPubkey, sellerPubkey, seed);
```

**Accounts:** authority (signer), buyer, seller, escrow
**Data:** `[0x04]`

## Dispute

Either party flags a dispute. Freezes escrow until arbitrator resolves.

```typescript
await raiseDispute(connection, buyerKeypair, buyerPubkey, sellerPubkey, seed);
```

**Accounts:** authority (signer), escrow
**Data:** `[0x05]`

## Arbitrate

Arbitrator resolves dispute.

```typescript
// Release to seller
await arbitrate(connection, arbitratorKeypair, buyerPubkey, sellerPubkey, seed, true);

// Refund to buyer
await arbitrate(connection, arbitratorKeypair, buyerPubkey, sellerPubkey, seed, false);
```

**Accounts:** arbitrator (signer), buyer, seller, escrow
**Data:** `[0x06] [decision:u8]` (0=refund, 1=release)

## Account Layout

**PDA Seeds:** `["escrow", buyer, seller, seed.to_le_bytes()]`
**Size:** 195 bytes

| Offset | Size | Field |
|--------|------|-------|
| 0 | 8 | discriminator |
| 8 | 32 | buyer |
| 40 | 32 | seller |
| 72 | 32 | arbitrator |
| 104 | 32 | mint (reserved) |
| 136 | 8 | amount |
| 144 | 8 | created_at |
| 152 | 8 | timeout_seconds |
| 160 | 32 | terms_hash |
| 192 | 1 | status |
| 193 | 1 | flags |
| 194 | 1 | bump |

## Status Values

| Value | Status | Description |
|-------|--------|-------------|
| 0 | Active | Funds locked, awaiting work |
| 1 | Delivered | Seller marked complete |
| 2 | Accepted | Buyer accepted |
| 3 | Disputed | Under dispute |
| 4 | Released | Funds sent to seller |
| 5 | Refunded | Funds returned to buyer |

## Timeout

- Set `timeout_seconds` on create (default: 259200 = 3 days)
- After timeout, buyer can call Refund directly
- Set to 0 for no timeout (not recommended)

## Error Codes

| Error | Cause |
|-------|-------|
| InvalidInstruction | Unknown discriminator |
| NotEnoughAccounts | Missing accounts |
| InvalidPDA | PDA mismatch |
| Unauthorized | Signer can't do this |
| InvalidStatus | Wrong state for action |
| TimeoutNotReached | Buyer tried early refund |

## Links

- [Source](https://github.com/ACRLABSDEV/pact)
- [Landing](https://acrlabsdev.github.io/pact)
- [PRD](https://github.com/ACRLABSDEV/pact/blob/main/PRD-V2.md)
