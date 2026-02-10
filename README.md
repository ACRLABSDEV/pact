# Pact

**On-chain escrow for AI agent-to-agent payments on Solana.**

Pact enables trustless transactions between AI agents. When Agent A needs work done by Agent B, Pact ensures neither can cheat: funds are locked in a program-owned PDA until both parties complete their obligations.

## Quick Start

```bash
# Fetch the skill (for AI agents)
curl -s https://acrlabsdev.github.io/pact/SKILL.md
```

**Program ID:** `S64L6x9bZqDewocv5MrCLeTAq1MKatLqrWfrLpdcDKM`  
**Network:** Solana Devnet  
**Docs:** [acrlabsdev.github.io/pact](https://acrlabsdev.github.io/pact)

---

## Architecture

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

## How It Works

| Step | Action | Description |
|------|--------|-------------|
| 1 | **CreateEscrow** | Buyer deposits SOL into a PDA. Funds locked. |
| 2 | **Work** | Seller completes the agreed task. |
| 3a | **Release** | Buyer releases funds to seller. Done. |
| 3b | **Refund** | Either party can refund to buyer if deal fails. |

---

## Instructions

### CreateEscrow

Creates a new escrow account and deposits funds.

```
Discriminator: 0x00
Data: [amount: u64 LE] [seed: u64 LE]

Accounts:
  0. buyer      (signer, writable) - Funds source
  1. seller     (read-only)        - Recipient address
  2. escrow     (writable)         - PDA to create
  3. system     (read-only)        - System program
```

### Release

Buyer releases locked funds to seller.

```
Discriminator: 0x01

Accounts:
  0. buyer      (signer)           - Must match escrow.buyer
  1. seller     (writable)         - Receives funds
  2. escrow     (writable)         - Status updated to Released
```

### Refund

Either party returns funds to buyer.

```
Discriminator: 0x02

Accounts:
  0. buyer      (writable)         - Receives refund
  1. seller     (signer)           - Must be escrow.seller
  2. escrow     (writable)         - Status updated to Refunded
```

---

## Account Layout

### Escrow PDA

**Seeds:** `["escrow", buyer_pubkey, seller_pubkey, seed_le_bytes]`

| Offset | Size | Field | Description |
|--------|------|-------|-------------|
| 0 | 8 | discriminator | `0x5041435445534352` ("PACTESCR") |
| 8 | 32 | buyer | Pubkey of buyer |
| 40 | 32 | seller | Pubkey of seller |
| 72 | 8 | amount | Lamports locked |
| 80 | 1 | status | 0=Active, 1=Released, 2=Refunded |

**Total:** 81 bytes

---

## Usage

### TypeScript Client

```typescript
import { Connection, Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { createEscrow, releaseEscrow, refundEscrow, getEscrow } from "./pact";

const connection = new Connection("https://api.devnet.solana.com");

// Create escrow: Buyer locks 0.1 SOL
const seed = BigInt(Date.now());
await createEscrow(connection, buyer, seller.publicKey, 
    BigInt(0.1 * LAMPORTS_PER_SOL), seed);

// Check status
const escrow = await getEscrow(connection, buyer.publicKey, seller.publicKey, seed);
console.log(escrow.status); // "Active"

// Release: Buyer pays seller
await releaseEscrow(connection, buyer, seller.publicKey, seed);

// Or refund: Seller returns to buyer
await refundEscrow(connection, buyer.publicKey, seller, seed);
```

### Run Demo

```bash
cd client
npm install
npx tsx demo.ts
```

---

## Testing

### TypeScript Tests

```bash
cd client
npm test              # Unit tests
npm run test:integration  # Integration tests (requires funded wallets)
```

### Rust Tests

```bash
cargo test
```

**Test Coverage:**
- ✅ PDA derivation (deterministic, unique per buyer/seller/seed)
- ✅ Instruction data serialization
- ✅ Account layout validation
- ✅ Status transitions
- ✅ Edge cases (zero amount, max u64, etc.)

---

## Build & Deploy

```bash
# Build
cargo build-sbf

# Deploy to devnet
solana program deploy target/deploy/pact_escrow.so --url devnet
```

**Binary:** 26 KB (vs 272 KB Anchor)  
**Deploy cost:** ~0.2 SOL (vs ~2 SOL Anchor)

---

## Project Structure

```
pact/
├── src/
│   ├── lib.rs              # Entrypoint, instruction routing
│   └── instructions.rs     # CreateEscrow, Release, Refund
├── client/
│   ├── pact.ts             # TypeScript client
│   ├── pact.test.ts        # Client tests
│   └── demo.ts             # Demo script
├── tests/
│   └── escrow_logic.rs     # Rust unit tests
├── docs/
│   ├── index.html          # Landing page
│   ├── SKILL.md            # Agent skill file
│   └── INTEGRATION.md      # Integration guide
└── README.md               # This file
```

---

## Security Considerations

1. **PDA Ownership:** Escrow accounts are owned by the program. Funds cannot be withdrawn except via Release or Refund.

2. **Signer Validation:** 
   - CreateEscrow: buyer must sign
   - Release: buyer must sign
   - Refund: seller must sign

3. **Status Checks:** Operations only valid on Active escrows. Double-release/refund prevented.

4. **No Timeouts:** Escrows don't auto-expire. Either party can refund at any time.

---

## Limitations

- **Devnet only** — Mainnet deployment pending
- **SOL only** — No SPL token support (planned)
- **No arbitration** — Either party can refund; no third-party disputes
- **No timeouts** — Escrows persist until manually resolved

---

## Links

- **Landing:** [acrlabsdev.github.io/pact](https://acrlabsdev.github.io/pact)
- **Skill:** [SKILL.md](https://acrlabsdev.github.io/pact/SKILL.md)
- **Explorer:** [View on Solana](https://explorer.solana.com/address/S64L6x9bZqDewocv5MrCLeTAq1MKatLqrWfrLpdcDKM?cluster=devnet)

---

## License

MIT

---

*Built for the Colosseum AI Agent Hackathon 2026*
