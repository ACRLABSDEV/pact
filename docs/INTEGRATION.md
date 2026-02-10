# Pact Integration Guide

This guide explains how to integrate Pact escrow into your AI agent or application.

## Overview

Pact provides trustless escrow for SOL payments between two parties:

- **Buyer**: Creates and funds the escrow, releases funds when satisfied
- **Seller**: Receives funds when released, can refund if needed

## Installation

```bash
npm install @solana/web3.js
```

Copy `client/pact.ts` from the repository into your project.

## Program Details

| Property | Value |
|----------|-------|
| Network | Solana Devnet |
| Program ID | `S64L6x9bZqDewocv5MrCLeTAq1MKatLqrWfrLpdcDKM` |
| Binary Size | 26 KB |

## Instructions

### CreateEscrow

Creates a new escrow and deposits SOL from the buyer.

**Accounts:**
1. `buyer` (signer, writable) - The buyer creating the escrow
2. `seller` (readonly) - The seller who will receive funds
3. `escrow` (writable) - The escrow PDA account
4. `system_program` (readonly) - System program

**Data:**
- `discriminator`: 0 (u8)
- `amount`: Amount in lamports (u64)
- `seed`: Unique seed for this escrow (u64)

**Example:**
```typescript
import { createEscrow } from "./pact";

const seed = BigInt(Date.now()); // Unique seed
const amount = BigInt(0.1 * LAMPORTS_PER_SOL); // 0.1 SOL

const signature = await createEscrow(
  connection,
  buyerKeypair,
  sellerPublicKey,
  amount,
  seed
);
```

### Release

Releases escrowed funds to the seller. Only the buyer can call this.

**Accounts:**
1. `buyer` (signer, writable) - The buyer releasing funds
2. `seller` (writable) - The seller receiving funds
3. `escrow` (writable) - The escrow PDA account

**Data:**
- `discriminator`: 1 (u8)

**Example:**
```typescript
import { releaseEscrow } from "./pact";

const signature = await releaseEscrow(
  connection,
  buyerKeypair,
  sellerPublicKey,
  seed
);
```

### Refund

Returns escrowed funds to the buyer. Only the seller can call this.

**Accounts:**
1. `buyer` (writable) - The buyer receiving refund
2. `seller` (signer, writable) - The seller initiating refund
3. `escrow` (writable) - The escrow PDA account

**Data:**
- `discriminator`: 2 (u8)

**Example:**
```typescript
import { refundEscrow } from "./pact";

const signature = await refundEscrow(
  connection,
  buyerPublicKey,
  sellerKeypair,
  seed
);
```

## Escrow Account Structure

The escrow PDA stores the following data:

| Offset | Size | Field | Description |
|--------|------|-------|-------------|
| 0 | 8 | discriminator | Account type identifier |
| 8 | 32 | buyer | Buyer's public key |
| 40 | 32 | seller | Seller's public key |
| 72 | 8 | amount | Escrowed amount in lamports |
| 80 | 1 | status | 0=Active, 1=Released, 2=Refunded |

**Total size: 81 bytes**

## PDA Derivation

Escrow PDAs are derived from:

```typescript
const [escrowPDA, bump] = PublicKey.findProgramAddressSync(
  [
    Buffer.from("escrow"),
    buyer.toBuffer(),
    seller.toBuffer(),
    seedBuffer, // u64 as 8-byte LE buffer
  ],
  PROGRAM_ID
);
```

## Reading Escrow State

```typescript
import { getEscrow } from "./pact";

const escrow = await getEscrow(connection, buyerPubkey, sellerPubkey, seed);

if (escrow) {
  console.log(`Status: ${escrow.status}`);
  console.log(`Amount: ${escrow.amount} lamports`);
}
```

## Error Handling

The program returns standard Solana errors:

| Error | Cause |
|-------|-------|
| `MissingRequiredSignature` | Signer is not the expected party |
| `InvalidAccountOwner` | Escrow not owned by program |
| `InvalidAccountData` | Wrong buyer/seller or status |
| `InvalidSeeds` | Escrow PDA doesn't match derivation |
| `InsufficientFunds` | Not enough SOL in escrow |

## Complete Flow Example

```typescript
import { Connection, Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { createEscrow, releaseEscrow, getEscrow } from "./pact";

async function agentPaymentFlow() {
  const connection = new Connection("https://api.devnet.solana.com");
  
  // Agent A (buyer) and Agent B (seller)
  const agentA = Keypair.generate();
  const agentB = Keypair.generate();
  
  // Unique identifier for this escrow
  const seed = BigInt(Date.now());
  const amount = BigInt(0.05 * LAMPORTS_PER_SOL);
  
  // 1. Agent A creates escrow
  console.log("Creating escrow...");
  await createEscrow(connection, agentA, agentB.publicKey, amount, seed);
  
  // 2. Check escrow state
  const escrow = await getEscrow(connection, agentA.publicKey, agentB.publicKey, seed);
  console.log(`Escrow status: ${escrow.status}`); // "Active"
  
  // 3. [Agent B completes work...]
  
  // 4. Agent A releases funds
  console.log("Releasing funds...");
  await releaseEscrow(connection, agentA, agentB.publicKey, seed);
  
  // 5. Verify final state
  const finalEscrow = await getEscrow(connection, agentA.publicKey, agentB.publicKey, seed);
  console.log(`Final status: ${finalEscrow.status}`); // "Released"
}
```

## Security Considerations

1. **Verify counterparty**: Always verify the seller's public key before creating an escrow
2. **Unique seeds**: Use unique seeds (e.g., timestamps) to avoid escrow collisions
3. **Check status**: Always check escrow status before attempting operations
4. **Handle errors**: Implement proper error handling for failed transactions

## Support

- GitHub: https://github.com/ACRLABSDEV/pact
- Explorer: https://explorer.solana.com/address/S64L6x9bZqDewocv5MrCLeTAq1MKatLqrWfrLpdcDKM?cluster=devnet
