# Pact

On-chain escrow for AI agent payments on Solana.

**Program:** `S64L6x9bZqDewocv5MrCLeTAq1MKatLqrWfrLpdcDKM`  
**Network:** Devnet  
**Docs:** [acrlabsdev.github.io/pact](https://acrlabsdev.github.io/pact)

## What it does

Pact enables trustless payments between AI agents:

1. **Buyer** creates escrow, deposits SOL
2. **Seller** completes work
3. **Buyer** releases funds—or either party refunds

Funds are locked in a program-owned PDA until release.

## Usage

```typescript
import { createEscrow, releaseEscrow, refundEscrow } from "./pact";

// Lock 0.1 SOL in escrow
const seed = BigInt(Date.now());
await createEscrow(connection, buyer, seller, BigInt(0.1 * LAMPORTS_PER_SOL), seed);

// Release to seller
await releaseEscrow(connection, buyer, seller, seed);

// Or refund to buyer
await refundEscrow(connection, buyer.publicKey, seller, seed);
```

## Build

```bash
cargo build-sbf
solana program deploy target/deploy/pact_escrow.so --url devnet
```

## Demo

```bash
cd client
npm install
npx tsx demo.ts
```

## Structure

```
src/
  lib.rs          # Entrypoint
  instructions.rs # CreateEscrow, Release, Refund
client/
  pact.ts         # TypeScript client
  demo.ts         # Demo transaction
docs/
  index.html      # Landing page
  SKILL.md        # Agent skill file
```

## Skill

Agents can fetch the skill file:

```
curl -s https://acrlabsdev.github.io/pact/SKILL.md
```

## License

MIT
