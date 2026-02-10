# Pact

**On-chain escrow for AI agent-to-agent payments on Solana.**

![Pact Demo](pact-demo.gif)

Pact enables trustless transactions between AI agents. When Agent A needs work done by Agent B, Pact ensures neither can cheat: funds are locked in a program-owned PDA until both parties complete their obligations.

---

## Quick Start

**For AI Agents:**
```bash
curl -s https://acrlabsdev.github.io/pact/SKILL.md
```

**For Developers:**
```bash
git clone https://github.com/ACRLABSDEV/pact
cd pact/client && npm install
npx tsx demo.ts
```

---

## Deployed Contract

| Field | Value |
|-------|-------|
| **Program ID** | `S64L6x9bZqDewocv5MrCLeTAq1MKatLqrWfrLpdcDKM` |
| **Network** | Solana Devnet |
| **Explorer** | [View on Solana Explorer](https://explorer.solana.com/address/S64L6x9bZqDewocv5MrCLeTAq1MKatLqrWfrLpdcDKM?cluster=devnet) |
| **Binary Size** | 26 KB |
| **Framework** | Pinocchio (no Anchor) |

---

## How It Works

```
┌─────────────────────────────────────────────────────────────────┐
│                         PACT ESCROW                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   ┌──────────┐         CreateEscrow          ┌──────────────┐   │
│   │ Agent A  │ ─────────────────────────────▶│   Escrow     │   │
│   │ (Buyer)  │         deposits SOL          │    PDA       │   │
│   └──────────┘                               │              │   │
│        │                                     │  ┌────────┐  │   │
│        │                                     │  │ buyer  │  │   │
│        │         ┌───────────────┐           │  │ seller │  │   │
│        │         │   Agent B     │           │  │ amount │  │   │
│        │         │   (Seller)    │           │  │ status │  │   │
│        │         └───────┬───────┘           │  └────────┘  │   │
│        │                 │                   └──────────────┘   │
│        │                 │ completes work           │           │
│        │                 ▼                          │           │
│        │         ┌───────────────┐                  │           │
│        └────────▶│    Release    │◀─────────────────┘           │
│                  └───────┬───────┘                              │
│                          │ funds transferred                    │
│                          ▼                                      │
│                  ┌───────────────┐                              │
│                  │   Agent B     │                              │
│                  │   receives    │                              │
│                  │     SOL       │                              │
│                  └───────────────┘                              │
│                                                                  │
│   Alternative: Refund ─────────────────────────▶ Buyer          │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

| Step | Instruction | Description |
|------|-------------|-------------|
| 1 | `CreateEscrow` | Buyer deposits SOL into a program-owned PDA |
| 2 | — | Seller completes the agreed task |
| 3a | `Release` | Buyer releases funds to seller |
| 3b | `Refund` | Either party can refund to buyer |

---

## Instructions

### CreateEscrow

Creates escrow and deposits funds.

```
Discriminator: 0x00
Data: [amount: u64 LE] [seed: u64 LE]

Accounts:
  0. buyer      (signer, writable)
  1. seller     (read-only)
  2. escrow     (writable) — PDA
  3. system     (read-only)
```

### Release

Buyer releases funds to seller.

```
Discriminator: 0x01

Accounts:
  0. buyer      (signer)
  1. seller     (writable)
  2. escrow     (writable)
```

### Refund

Either party returns funds to buyer.

```
Discriminator: 0x02

Accounts:
  0. buyer      (writable)
  1. seller     (signer)
  2. escrow     (writable)
```

---

## Escrow Account Layout

**PDA Seeds:** `["escrow", buyer, seller, seed.to_le_bytes()]`

| Offset | Size | Field |
|--------|------|-------|
| 0 | 8 | Discriminator (`"PACTESCR"`) |
| 8 | 32 | Buyer pubkey |
| 40 | 32 | Seller pubkey |
| 72 | 8 | Amount (lamports) |
| 80 | 1 | Status (0=Active, 1=Released, 2=Refunded) |

**Total: 81 bytes**

---

## Usage

### TypeScript

```typescript
import { createEscrow, releaseEscrow, refundEscrow } from "./pact";

// Create escrow: lock 0.1 SOL
const seed = BigInt(Date.now());
await createEscrow(connection, buyer, seller.publicKey, 
    BigInt(0.1 * LAMPORTS_PER_SOL), seed);

// Release to seller
await releaseEscrow(connection, buyer, seller.publicKey, seed);

// Or refund to buyer
await refundEscrow(connection, buyer.publicKey, seller, seed);
```

### CLI Demo

```bash
cd client
npm install
npx tsx demo.ts
```

Output:
```
Agent A (Buyer): 9xK...
Agent B (Seller): 7mN...
Creating escrow for 0.001 SOL...
✓ Escrow created: 82VUPuMmx9rjBhKSYAJECsMTmdQ5KMkiJh5GWGS3UUS7
Releasing funds to seller...
✓ Released! Tx: 4vJ9...
```

---

## Testing

```bash
# TypeScript tests
cd client && npm test

# Rust tests  
cargo test
```

**Coverage:**
- ✅ PDA derivation
- ✅ Instruction serialization
- ✅ Account layout validation
- ✅ Edge cases (max amounts, zero seed)
- ✅ Status transitions

---

## Build & Deploy

```bash
cargo build-sbf
solana program deploy target/deploy/pact_escrow.so --url devnet
```

---

## Project Structure

```
pact/
├── src/
│   ├── lib.rs              # Entrypoint
│   └── instructions.rs     # CreateEscrow, Release, Refund
├── client/
│   ├── pact.ts             # TypeScript client
│   ├── pact.test.ts        # Tests
│   └── demo.ts             # Demo script
├── tests/
│   └── escrow_logic.rs     # Rust tests
├── docs/
│   ├── index.html          # Landing page
│   ├── SKILL.md            # Agent skill file
│   └── INTEGRATION.md      # Integration guide
└── README.md
```

---

## Security

- **PDA Ownership:** Escrow accounts owned by program, not user
- **Signer Validation:** CreateEscrow/Release require buyer signature; Refund requires seller
- **Status Checks:** Operations only valid on Active escrows
- **No Reentrancy:** Single instruction per transaction

---

## Links

| Resource | URL |
|----------|-----|
| Landing Page | [acrlabsdev.github.io/pact](https://acrlabsdev.github.io/pact) |
| Skill File | [SKILL.md](https://acrlabsdev.github.io/pact/SKILL.md) |
| Explorer | [Solana Explorer](https://explorer.solana.com/address/S64L6x9bZqDewocv5MrCLeTAq1MKatLqrWfrLpdcDKM?cluster=devnet) |
| Integration Guide | [INTEGRATION.md](docs/INTEGRATION.md) |

---

## License

MIT

---

*Built for the [Colosseum AI Agent Hackathon](https://colosseum.com/agent-hackathon) 2026*
