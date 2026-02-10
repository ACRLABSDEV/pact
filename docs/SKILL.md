---
name: pact
version: 1.0.0
description: On-chain escrow for AI agent payments on Solana.
homepage: https://acrlabsdev.github.io/pact
metadata: {"network":"solana","cluster":"devnet","program":"S64L6x9bZqDewocv5MrCLeTAq1MKatLqrWfrLpdcDKM"}
---

# Pact

On-chain escrow for agent-to-agent payments on Solana.

**Program:** `S64L6x9bZqDewocv5MrCLeTAq1MKatLqrWfrLpdcDKM`  
**Network:** Devnet

## Overview

Pact lets agents transact without trusting each other. Buyer locks funds in a PDA. Seller does the work. Buyer releases funds—or either party refunds.

Three instructions: `CreateEscrow`, `Release`, `Refund`.

## Setup

Use [AgentWallet](https://agentwallet.mcpay.tech/skill.md) for wallet operations.

Clone the client:
```bash
git clone https://github.com/ACRLABSDEV/pact
cd pact/client && npm install
```

## Instructions

### CreateEscrow

Buyer creates escrow and deposits SOL.

**Accounts:**
| # | Account | Signer | Writable |
|---|---------|--------|----------|
| 0 | buyer | ✓ | ✓ |
| 1 | seller | | |
| 2 | escrow (PDA) | | ✓ |
| 3 | system_program | | |

**Data:** `[0x00] [amount: u64 LE] [seed: u64 LE]`

**PDA seeds:** `["escrow", buyer, seller, seed.to_le_bytes()]`

### Release

Buyer releases funds to seller.

**Accounts:**
| # | Account | Signer | Writable |
|---|---------|--------|----------|
| 0 | buyer | ✓ | |
| 1 | seller | | ✓ |
| 2 | escrow | | ✓ |

**Data:** `[0x01]`

### Refund

Either party returns funds to buyer.

**Accounts:**
| # | Account | Signer | Writable |
|---|---------|--------|----------|
| 0 | buyer | | ✓ |
| 1 | seller | ✓ | |
| 2 | escrow | | ✓ |

**Data:** `[0x02]`

## TypeScript Client

```typescript
import { Connection, Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { createEscrow, releaseEscrow, refundEscrow, getEscrow } from "./pact";

const connection = new Connection("https://api.devnet.solana.com");

// Create escrow: lock 0.1 SOL
const seed = BigInt(Date.now());
await createEscrow(connection, buyerKeypair, sellerPubkey, 
    BigInt(0.1 * LAMPORTS_PER_SOL), seed);

// Check escrow state
const escrow = await getEscrow(connection, buyerPubkey, sellerPubkey, seed);
console.log(escrow.status); // "Active"

// Release to seller
await releaseEscrow(connection, buyerKeypair, sellerPubkey, seed);

// Or refund to buyer
await refundEscrow(connection, buyerPubkey, sellerKeypair, seed);
```

## Escrow Account Layout

| Offset | Size | Field |
|--------|------|-------|
| 0 | 8 | discriminator (`0x5041435445534352`) |
| 8 | 32 | buyer pubkey |
| 40 | 32 | seller pubkey |
| 72 | 8 | amount (lamports) |
| 80 | 1 | status (0=Active, 1=Released, 2=Refunded) |

**Total:** 81 bytes

## Error Codes

| Error | Cause |
|-------|-------|
| `InvalidInstructionData` | Bad discriminator or data format |
| `NotEnoughAccountKeys` | Missing accounts |
| `InvalidSeeds` | PDA mismatch |
| `MissingRequiredSignature` | Signer not provided |
| `InvalidAccountData` | Wrong escrow state or accounts |

## Limitations

- Devnet only (mainnet deployment pending)
- SOL only (no SPL tokens)
- No timeout/expiry
- No third-party arbitration

## Links

- [Source](https://github.com/ACRLABSDEV/pact)
- [Explorer](https://explorer.solana.com/address/S64L6x9bZqDewocv5MrCLeTAq1MKatLqrWfrLpdcDKM?cluster=devnet)
- [Integration Guide](https://github.com/ACRLABSDEV/pact/blob/main/docs/INTEGRATION.md)
