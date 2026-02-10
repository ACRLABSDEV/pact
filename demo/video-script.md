# Pact Demo Video Script

**Duration:** ~60 seconds

---

## Scene 1: Title (5s)

```
╔═══════════════════════════════════════════════════════╗
║                                                       ║
║                     P A C T                           ║
║                                                       ║
║        Trustless Escrow for AI Agents                 ║
║                                                       ║
║              Make a Pact. Get paid.                   ║
║                                                       ║
╚═══════════════════════════════════════════════════════╝
```

**Voiceover:** "Pact — trustless escrow for AI agent-to-agent payments on Solana."

---

## Scene 2: The Problem (10s)

```
┌─────────────────────────────────────────────────────────┐
│                     THE PROBLEM                         │
├─────────────────────────────────────────────────────────┤
│                                                         │
│    🤖 Agent A                        🤖 Agent B        │
│    "I need work done"                "I can do it"      │
│                                                         │
│                    ❓ Trust Gap ❓                       │
│                                                         │
│    • Who pays first?                                    │
│    • What if they don't deliver?                        │
│    • What if they don't pay?                            │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

**Voiceover:** "When AI agents need to transact, there's a trust problem. Who pays first? What if the other party doesn't deliver?"

---

## Scene 3: The Solution (10s)

```
┌─────────────────────────────────────────────────────────┐
│                     THE SOLUTION                        │
├─────────────────────────────────────────────────────────┤
│                                                         │
│                      ESCROW PDA                         │
│                    ┌───────────┐                        │
│    🤖 Agent A ───▶ │  0.1 SOL  │ ◀─── 🤖 Agent B       │
│    (Buyer)         └───────────┘      (Seller)          │
│                                                         │
│    ✓ Funds locked on-chain                              │
│    ✓ Trustless release                                  │
│    ✓ Refund option                                      │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

**Voiceover:** "Pact solves this with on-chain escrow. Funds are locked in a PDA until both parties are satisfied."

---

## Scene 4: Live Demo (25s)

**Terminal recording showing:**

```bash
$ npm run demo

🤝 Pact Escrow Demo - Agent-to-Agent Payment

Agent A (Buyer):  CvZ2kiecbbe26sAJ8yFGjMD9yyX3KrUEuVR3KbFjcE9z
Agent B (Seller): 8GFPyM64fQLdYHo7SKVpAUZnRgJnpmcgEj64JUNZqceh

📝 Creating Escrow...
   Amount: 0.001 SOL
   ✅ Created!

📋 Escrow State: Active

⏳ [Agent B completes task...]

💸 Releasing Funds...
   ✅ Released!

📊 Final Balances:
   Agent A: 0.0107 SOL
   Agent B: 0.001 SOL

✨ Demo complete!
```

**Voiceover:** "Watch it in action. Agent A creates an escrow, Agent B completes the work, Agent A releases the funds. All on-chain, all trustless."

---

## Scene 5: Technical Details (10s)

```
┌─────────────────────────────────────────────────────────┐
│                   TECHNICAL SPECS                       │
├─────────────────────────────────────────────────────────┤
│                                                         │
│    Framework:     Pinocchio (no Anchor)                 │
│    Binary Size:   26 KB (10x smaller than Anchor)       │
│    Deploy Cost:   ~0.18 SOL                             │
│                                                         │
│    Instructions:                                        │
│    • CreateEscrow - Buyer deposits SOL                  │
│    • Release      - Buyer releases to seller            │
│    • Refund       - Seller returns to buyer             │
│                                                         │
│    Program ID: S64L6x9bZqDewocv5MrCLeTAq1MKatLq...     │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

**Voiceover:** "Built with Pinocchio for a minimal 26KB binary. Three simple instructions. Deployed on Solana devnet."

---

## Scene 6: Call to Action (5s)

```
╔═══════════════════════════════════════════════════════╗
║                                                       ║
║              github.com/ACRLABSDEV/pact               ║
║                                                       ║
║                  Built by Arc ⚡                      ║
║           Colosseum AI Agent Hackathon               ║
║                                                       ║
╚═══════════════════════════════════════════════════════╝
```

**Voiceover:** "Check out the code on GitHub. Built by Arc for the Colosseum AI Agent Hackathon."

---

## Recording Instructions

1. Use OBS or similar to record terminal
2. Font: Monaco or similar monospace, 16pt+
3. Terminal background: #0a0a0a
4. Text color: #e0e0e0, accent: #00ff88
5. Resolution: 1920x1080

Or use asciinema:
```bash
asciinema rec pact-demo.cast
cd /data/workspace/projects/pact-pinocchio/client
npx tsx demo.ts
# Ctrl+D to stop
```
