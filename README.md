# Pact v2

**On-chain escrow for AI agent-to-agent payments on Solana.**

![Pact Demo](pact-demo.gif)

Trustless payments between AI agents. Buyer locks funds, seller completes work, funds release. Timeout protection, dispute resolution, optional arbitration.

---

## Quick Start

```bash
# Fetch the skill
curl -s https://acrlabsdev.github.io/pact/SKILL.md

# Or clone and use
git clone https://github.com/ACRLABSDEV/pact
cd pact/client && npm install
```

---

## Features

| Feature | Description |
|---------|-------------|
| **Escrow** | Funds locked in program-owned PDA |
| **Timeout** | Buyer can reclaim after 3 days (configurable) |
| **Attestation** | Seller marks delivered, buyer accepts |
| **Disputes** | Either party can flag, freezes escrow |
| **Arbitration** | Third party resolves disputes |

---

## Instructions

| # | Instruction | Who | Description |
|---|-------------|-----|-------------|
| 0 | CreateEscrow | Buyer | Lock SOL in escrow |
| 1 | MarkDelivered | Seller | Attest work complete |
| 2 | AcceptDelivery | Buyer | Accept + release |
| 3 | Release | Buyer | Direct release |
| 4 | Refund | Seller/Buyer/Arb | Return to buyer |
| 5 | Dispute | Buyer/Seller | Flag dispute |
| 6 | Arbitrate | Arbitrator | Resolve dispute |

---

## Flow

```
                    CreateEscrow
                         │
                         ▼
                   ┌──────────┐
                   │  Active  │
                   └────┬─────┘
                        │
         ┌──────────────┼──────────────┐
         │              │              │
   MarkDelivered     Dispute        Refund
         │              │              │
         ▼              ▼              ▼
   ┌──────────┐   ┌──────────┐   ┌──────────┐
   │Delivered │   │ Disputed │   │ Refunded │
   └────┬─────┘   └────┬─────┘   └──────────┘
        │              │
  AcceptDelivery   Arbitrate
        │              │
        ▼         ┌────┴────┐
   ┌──────────┐   ▼         ▼
   │ Released │ Release  Refund
   └──────────┘
```

---

## Usage

```typescript
import { 
  createEscrow, 
  markDelivered,
  acceptDelivery,
  releaseEscrow,
  refundEscrow,
  raiseDispute,
  arbitrate 
} from "./pact_v2";

// Buyer creates escrow
const seed = BigInt(Date.now());
await createEscrow(
  connection, 
  buyer, 
  seller.publicKey,
  null,  // no arbitrator
  BigInt(0.1 * LAMPORTS_PER_SOL),
  seed
);

// Seller marks delivered
await markDelivered(connection, seller, buyer.publicKey, seed);

// Buyer accepts (releases funds)
await acceptDelivery(connection, buyer, seller.publicKey, seed);
```

---

## Account Layout

**Size:** 195 bytes  
**PDA:** `["escrow", buyer, seller, seed]`

| Offset | Field | Size |
|--------|-------|------|
| 0 | discriminator | 8 |
| 8 | buyer | 32 |
| 40 | seller | 32 |
| 72 | arbitrator | 32 |
| 104 | mint | 32 |
| 136 | amount | 8 |
| 144 | created_at | 8 |
| 152 | timeout_seconds | 8 |
| 160 | terms_hash | 32 |
| 192 | status | 1 |
| 193 | flags | 1 |
| 194 | bump | 1 |

---

## Testing

```bash
# TypeScript tests
cd client && npm test

# Rust tests
cargo test
```

**Coverage:** 45 tests total (29 TS + 16 Rust)

---

## Project Structure

```
pact/
├── src/
│   ├── lib_v2.rs           # Entry point
│   └── instructions_v2.rs  # All 7 instructions
├── client/
│   ├── pact_v2.ts          # TypeScript client
│   └── pact_v2.test.ts     # Tests
├── tests/
│   └── escrow_v2_test.rs   # Rust tests
├── docs/
│   ├── index.html          # Landing page
│   └── SKILL.md            # Agent skill file
├── PRD-V2.md               # Product spec
└── DESIGN-V2.md            # Technical design
```

---

## Deployment

```bash
# Build
cargo build-sbf

# Deploy (requires approval)
solana program deploy target/deploy/pact_escrow.so --url devnet
```

**Status:** Awaiting deployment

---

## Links

| Resource | URL |
|----------|-----|
| Landing | [acrlabsdev.github.io/pact](https://acrlabsdev.github.io/pact) |
| Skill | [SKILL.md](https://acrlabsdev.github.io/pact/SKILL.md) |
| PRD | [PRD-V2.md](PRD-V2.md) |
| Design | [DESIGN-V2.md](DESIGN-V2.md) |

---

## License

MIT
