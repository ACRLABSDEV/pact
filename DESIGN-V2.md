# Pact v2 Design

## Overview

Trustless escrow for AI agent-to-agent payments on Solana with timeout, disputes, arbitration, and SPL token support.

## Features

### Core (v1)
- ✅ CreateEscrow - buyer deposits funds
- ✅ Release - buyer releases to seller
- ✅ Refund - return funds to buyer

### New in v2
- ⏰ **Timeout** - auto-unlock after expiry
- 🛡️ **Buyer Refund** - buyer can cancel before work starts or after timeout
- 📝 **Work Attestation** - seller marks "delivered", buyer marks "accepted"
- ⚠️ **Dispute Flag** - either party can flag dispute
- ⚖️ **Arbitrator** - third party can resolve disputes
- 🪙 **SPL Tokens** - support USDC and other tokens
- 📋 **Metadata** - hash of terms/description

---

## Account Layout

### Escrow Account (v2)

**PDA Seeds:** `["escrow", buyer, seller, seed.to_le_bytes()]`

| Offset | Size | Field | Description |
|--------|------|-------|-------------|
| 0 | 8 | discriminator | `"PACTESCR"` (0x5041435445534352) |
| 8 | 32 | buyer | Buyer pubkey |
| 40 | 32 | seller | Seller pubkey |
| 72 | 32 | arbitrator | Arbitrator pubkey (or zeroes if none) |
| 104 | 32 | mint | Token mint (or SOL_MINT for native SOL) |
| 136 | 8 | amount | Amount in smallest units |
| 144 | 8 | created_at | Unix timestamp (seconds) |
| 152 | 8 | timeout_seconds | Seconds until auto-refundable (0 = no timeout) |
| 160 | 32 | terms_hash | SHA256 of off-chain terms (optional) |
| 192 | 1 | status | See status enum |
| 193 | 1 | flags | Bitflags for attestations |
| 194 | 1 | bump | PDA bump |

**Total: 195 bytes**

### Status Enum

| Value | Status | Description |
|-------|--------|-------------|
| 0 | Active | Escrow created, funds locked |
| 1 | Delivered | Seller attested delivery |
| 2 | Accepted | Buyer accepted delivery |
| 3 | Disputed | Either party flagged dispute |
| 4 | Released | Funds released to seller |
| 5 | Refunded | Funds returned to buyer |

### Flags Bitfield

| Bit | Flag | Description |
|-----|------|-------------|
| 0 | seller_delivered | Seller marked as delivered |
| 1 | buyer_accepted | Buyer accepted delivery |
| 2 | buyer_disputed | Buyer flagged dispute |
| 3 | seller_disputed | Seller flagged dispute |

---

## Instructions

### 1. CreateEscrow

Creates escrow and deposits funds.

**Accounts:**
| # | Account | Signer | Writable | Description |
|---|---------|--------|----------|-------------|
| 0 | buyer | ✓ | ✓ | Funds source |
| 1 | seller | | | Recipient |
| 2 | arbitrator | | | Optional arbitrator (can be buyer for none) |
| 3 | escrow | | ✓ | PDA to create |
| 4 | mint | | | Token mint (native SOL mint for SOL) |
| 5 | buyer_token_account | | ✓ | Buyer's token account (if SPL) |
| 6 | escrow_token_account | | ✓ | Escrow's token account (if SPL) |
| 7 | system_program | | | |
| 8 | token_program | | | (if SPL) |

**Data:**
```
[0x00] [amount: u64] [seed: u64] [timeout_seconds: u64] [terms_hash: [u8; 32]]
```

### 2. MarkDelivered

Seller attests that work is delivered.

**Accounts:**
| # | Account | Signer | Writable |
|---|---------|--------|----------|
| 0 | seller | ✓ | |
| 1 | escrow | | ✓ |

**Data:** `[0x01]`

### 3. AcceptDelivery

Buyer accepts delivery (auto-releases funds).

**Accounts:**
| # | Account | Signer | Writable |
|---|---------|--------|----------|
| 0 | buyer | ✓ | |
| 1 | seller | | ✓ |
| 2 | escrow | | ✓ |
| 3+ | (token accounts if SPL) |

**Data:** `[0x02]`

### 4. Release

Buyer releases funds without attestation flow.

**Accounts:**
| # | Account | Signer | Writable |
|---|---------|--------|----------|
| 0 | buyer | ✓ | |
| 1 | seller | | ✓ |
| 2 | escrow | | ✓ |
| 3+ | (token accounts if SPL) |

**Data:** `[0x03]`

### 5. Refund

Return funds to buyer. Allowed when:
- Seller initiates (any time before release)
- Buyer initiates AND (timeout expired OR status is Active with no delivery)
- Arbitrator initiates (if dispute)

**Accounts:**
| # | Account | Signer | Writable |
|---|---------|--------|----------|
| 0 | authority | ✓ | | (buyer, seller, or arbitrator) |
| 1 | buyer | | ✓ |
| 2 | seller | | |
| 3 | escrow | | ✓ |
| 4+ | (token accounts if SPL) |

**Data:** `[0x04]`

### 6. Dispute

Either party flags a dispute. Freezes escrow until arbitrator resolves.

**Accounts:**
| # | Account | Signer | Writable |
|---|---------|--------|----------|
| 0 | authority | ✓ | | (buyer or seller) |
| 1 | escrow | | ✓ |

**Data:** `[0x05]`

### 7. Arbitrate

Arbitrator resolves dispute by forcing release or refund.

**Accounts:**
| # | Account | Signer | Writable |
|---|---------|--------|----------|
| 0 | arbitrator | ✓ | |
| 1 | buyer | | ✓ |
| 2 | seller | | ✓ |
| 3 | escrow | | ✓ |
| 4+ | (token accounts if SPL) |

**Data:** `[0x06] [decision: u8]` (0 = refund, 1 = release)

---

## State Machine

```
                    ┌─────────────┐
                    │   Active    │
                    └──────┬──────┘
                           │
            ┌──────────────┼──────────────┐
            │              │              │
            ▼              ▼              ▼
     ┌──────────┐   ┌──────────┐   ┌──────────┐
     │ Delivered│   │ Disputed │   │ Refunded │
     └────┬─────┘   └────┬─────┘   └──────────┘
          │              │
          ▼              ▼
     ┌──────────┐   ┌──────────┐
     │ Accepted │   │Arbitrate │
     └────┬─────┘   └────┬─────┘
          │              │
          ▼         ┌────┴────┐
     ┌──────────┐   ▼         ▼
     │ Released │ Released  Refunded
     └──────────┘
```

---

## Timeout Logic

- `created_at` set on CreateEscrow
- `timeout_seconds` configurable (0 = no timeout)
- After `created_at + timeout_seconds`:
  - Buyer can call Refund directly
  - Seller can still Release if not refunded
- Clock checked via Sysvar

---

## SPL Token Support

For SPL tokens:
- `mint` is the token mint address
- Escrow PDA has an associated token account
- Transfers use Token Program CPI
- For native SOL: use special SOL_MINT constant

---

## Error Codes

| Code | Error | Description |
|------|-------|-------------|
| 0 | InvalidInstruction | Unknown discriminator |
| 1 | NotEnoughAccounts | Missing required accounts |
| 2 | InvalidPDA | PDA mismatch |
| 3 | Unauthorized | Signer not authorized for action |
| 4 | InvalidStatus | Action not allowed in current status |
| 5 | TimeoutNotReached | Tried to timeout-refund too early |
| 6 | NoArbitrator | Tried to arbitrate with no arbitrator set |
| 7 | NotDisputed | Tried to arbitrate non-disputed escrow |
| 8 | AmountZero | Amount must be > 0 |

---

## Migration from v1

v1 escrows (81 bytes) are not compatible with v2 (195 bytes). 

Options:
1. Deploy as new program (recommended)
2. Add version byte and support both layouts

For simplicity: deploy as new program ID.
