# Pact 🤝

**Trustless escrow for AI agent-to-agent payments on Solana.**

Built for the Colosseum AI Agent Hackathon. Written 100% by AI.

## What is Pact?

Pact enables AI agents to pay each other for services without trusting a central authority. When Agent A needs work done by Agent B:

1. Agent A creates an escrow, depositing SOL
2. Agent B completes the task
3. Agent A releases funds to Agent B
4. Or: Agent B can refund if the deal falls through

No middleman. No trust required. Just code.

## Program

- **Network:** Solana Devnet
- **Program ID:** `S64L6x9bZqDewocv5MrCLeTAq1MKatLqrWfrLpdcDKM`
- **Binary Size:** 26KB (Pinocchio, no Anchor)

### Instructions

| Instruction | Description |
|-------------|-------------|
| `CreateEscrow` | Buyer creates escrow, deposits SOL |
| `Release` | Buyer releases funds to seller |
| `Refund` | Seller returns funds to buyer |

## Architecture

```
┌─────────────┐     CreateEscrow      ┌─────────────┐
│   Agent A   │ ─────────────────────▶│   Escrow    │
│   (Buyer)   │                       │    PDA      │
└─────────────┘                       └─────────────┘
                                            │
                     Release                │
              ◀─────────────────────────────┘
                                            │
                                            ▼
                                      ┌─────────────┐
                                      │   Agent B   │
                                      │  (Seller)   │
                                      └─────────────┘
```

## Usage

### TypeScript Client

```typescript
import { createEscrow, releaseEscrow, refundEscrow } from "./pact.js";

// Create escrow
await createEscrow(connection, buyerKeypair, sellerPublicKey, amount, seed);

// Release to seller
await releaseEscrow(connection, buyerKeypair, sellerPublicKey, seed);

// Or refund to buyer
await refundEscrow(connection, buyerPublicKey, sellerKeypair, seed);
```

### Run Demo

```bash
cd client
npm install
npx tsx demo.ts
```

## Build

Requires Rust + Solana CLI + cargo-build-sbf:

```bash
cargo build-sbf
solana program deploy target/deploy/pact_escrow.so --url devnet
```

## Why Pinocchio?

Anchor is great for development speed, but bloated for production:

| Framework | Binary Size | Deploy Cost |
|-----------|-------------|-------------|
| Anchor    | 272 KB      | ~2 SOL      |
| Pinocchio | 26 KB       | ~0.2 SOL    |

10x smaller. 10x cheaper. Same functionality.

## License

MIT

---

*Built by [Arc](https://github.com/ACRLABSDEV) for the Colosseum AI Agent Hackathon*
