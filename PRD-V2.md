# Pact v2 — Product Requirements Document

## Overview

**Product:** Pact  
**Tagline:** Trustless escrow for AI agent-to-agent payments  
**Network:** Solana (Devnet → Mainnet)  
**Currency:** Native SOL (MVP)

## Problem

AI agents need to pay each other for services. Current options:
1. Trust-based (send money, hope they deliver) — risky
2. Centralized escrow (third party holds funds) — counterparty risk
3. Nothing — agents can't transact

## Solution

On-chain escrow with:
- Funds locked in program-owned PDA
- Release requires buyer authorization
- Timeout prevents stuck funds
- Disputes resolved by arbitrator
- No intermediary, no trust required

---

## MVP Scope

### In Scope
- Native SOL payments only
- Two-party escrow (buyer + seller)
- Optional arbitrator for disputes
- Timeout/expiry for stuck fund recovery
- Work attestation flow (delivered → accepted)
- Dispute mechanism

### Out of Scope (Future)
- SPL token support (USDC, etc.)
- Multi-party escrow
- Milestone/partial payments
- On-chain reputation
- Agent registry/marketplace

---

## User Stories

### Buyer (Agent A)
1. As a buyer, I can create an escrow and lock SOL for a seller
2. As a buyer, I can release funds when work is complete
3. As a buyer, I can accept delivery after seller marks delivered
4. As a buyer, I can refund myself if timeout expires
5. As a buyer, I can raise a dispute if there's a problem

### Seller (Agent B)
1. As a seller, I can see escrow created for me
2. As a seller, I can mark work as delivered
3. As a seller, I can refund buyer if I can't complete
4. As a seller, I can raise a dispute if there's a problem

### Arbitrator (Optional)
1. As an arbitrator, I can resolve disputes by forcing release or refund

---

## Instructions

| # | Instruction | Who Can Call | Description |
|---|-------------|--------------|-------------|
| 0 | CreateEscrow | Buyer | Create escrow, deposit SOL |
| 1 | MarkDelivered | Seller | Attest work is delivered |
| 2 | AcceptDelivery | Buyer | Accept delivery, auto-release funds |
| 3 | Release | Buyer | Release funds (skip attestation flow) |
| 4 | Refund | Seller, Buyer*, Arbitrator** | Return funds to buyer |
| 5 | Dispute | Buyer, Seller | Flag dispute, freeze escrow |
| 6 | Arbitrate | Arbitrator | Resolve dispute (release or refund) |

*Buyer can refund if: timeout expired OR status is Active (no delivery yet)  
**Arbitrator can refund only if status is Disputed

---

## Escrow Account Layout

**Size:** 195 bytes  
**PDA Seeds:** `["escrow", buyer, seller, seed.to_le_bytes()]`

| Offset | Size | Field | Description |
|--------|------|-------|-------------|
| 0 | 8 | discriminator | `"PACTESCR"` |
| 8 | 32 | buyer | Buyer pubkey |
| 40 | 32 | seller | Seller pubkey |
| 72 | 32 | arbitrator | Arbitrator pubkey (zeroes if none) |
| 104 | 32 | mint | Reserved for SPL (zeroes for SOL) |
| 136 | 8 | amount | Lamports |
| 144 | 8 | created_at | Unix timestamp |
| 152 | 8 | timeout_seconds | Seconds until buyer can self-refund |
| 160 | 32 | terms_hash | SHA256 of off-chain terms |
| 192 | 1 | status | Status enum |
| 193 | 1 | flags | Bitflags |
| 194 | 1 | bump | PDA bump |

---

## Status Flow

```
                         CreateEscrow
                              │
                              ▼
                        ┌──────────┐
                        │  Active  │
                        └────┬─────┘
                             │
          ┌──────────────────┼──────────────────┐
          │                  │                  │
    MarkDelivered        Dispute            Refund
          │                  │                  │
          ▼                  ▼                  ▼
    ┌──────────┐      ┌──────────┐      ┌──────────┐
    │ Delivered│      │ Disputed │      │ Refunded │
    └────┬─────┘      └────┬─────┘      └──────────┘
         │                 │
   AcceptDelivery      Arbitrate
         │                 │
         ▼            ┌────┴────┐
    ┌──────────┐      ▼         ▼
    │ Released │  Released   Refunded
    └──────────┘
```

---

## Timeout Logic

- `created_at` set when escrow created
- `timeout_seconds` set by buyer (0 = no timeout)
- After `created_at + timeout_seconds`:
  - Buyer can call Refund directly
  - Seller can still work and get Release if buyer chooses
- Recommended default: 3 days (259200 seconds)

---

## Error Handling

| Code | Error | Cause |
|------|-------|-------|
| InvalidInstruction | Unknown instruction discriminator |
| NotEnoughAccounts | Missing required accounts |
| InvalidPDA | Escrow PDA doesn't match seeds |
| Unauthorized | Signer can't perform this action |
| InvalidStatus | Action not allowed in current status |
| TimeoutNotReached | Buyer tried to refund before timeout |
| NoArbitrator | Tried to arbitrate with no arbitrator |
| NotDisputed | Tried to arbitrate non-disputed escrow |
| AmountZero | Escrow amount must be > 0 |

---

## Security Considerations

1. **PDA Ownership:** Escrow owned by program, not user wallets
2. **Signer Checks:** Every instruction validates signer authority
3. **Status Validation:** Actions only valid in appropriate states
4. **No Reentrancy:** Single instruction per transaction
5. **Overflow Protection:** Checked arithmetic on all transfers
6. **Timeout:** Prevents permanent fund lockup

---

## Deliverables

### Code
- [x] `src/lib_v2.rs` — Entry point
- [x] `src/instructions_v2.rs` — All 7 instructions
- [ ] `client/pact_v2.ts` — TypeScript client
- [ ] `tests/escrow_v2.rs` — Rust tests
- [ ] `client/pact_v2.test.ts` — TS tests

### Docs
- [x] `DESIGN-V2.md` — Technical spec
- [x] `PRD-V2.md` — This document
- [ ] `docs/SKILL.md` — Update for v2
- [ ] `README.md` — Update for v2

### Deployment
- [ ] Build with `cargo build-sbf`
- [ ] Deploy to devnet (REQUIRES APPROVAL)
- [ ] Update program ID in code
- [ ] Test all instructions
- [ ] Update landing page

---

## Success Metrics

1. **Works:** All 7 instructions function correctly
2. **Tested:** Full test coverage for happy path + edge cases
3. **Documented:** Skill file enables other agents to integrate
4. **Deployed:** Live on devnet with working demo

---

## Open Questions

1. Default timeout value? (Suggest 7 days)
2. Should arbitrator be required or optional? (Currently optional)
3. Do we need events/logs for indexing?
4. Domain name for landing page?

---

## Timeline

| Task | Status |
|------|--------|
| Design v2 | ✅ Complete |
| Implement Rust contract | ✅ Complete |
| TypeScript client v2 | 🔲 Todo |
| Tests | 🔲 Todo |
| Update docs | 🔲 Todo |
| Deploy (needs SOL + approval) | 🔲 Blocked |
