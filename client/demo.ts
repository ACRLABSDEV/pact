/**
 * Pact Escrow Demo - Agent-to-Agent Payment
 * 
 * This demo simulates two AI agents using Pact to trustlessly exchange SOL for services.
 * 
 * Scenario:
 * - Agent A (buyer) wants to pay Agent B (seller) for completing a task
 * - Agent A creates an escrow with the payment amount
 * - Agent B completes the task
 * - Agent A releases the funds to Agent B
 */

import { Connection, Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { createEscrow, releaseEscrow, getEscrow, deriveEscrowPDA, PROGRAM_ID } from "./pact.js";
import * as fs from "fs";
import * as path from "path";
import { fileURLToPath } from "url";

const DEVNET_URL = "https://api.devnet.solana.com";

// Load deployer keypair
function loadDeployerKeypair(): Keypair {
  const homeDir = process.env.HOME || "/root";
  const keypairPath = path.join(homeDir, ".config", "solana", "deployer.json");
  const keypairData = JSON.parse(fs.readFileSync(keypairPath, "utf-8"));
  return Keypair.fromSecretKey(Uint8Array.from(keypairData));
}

async function main() {
  console.log("🤝 Pact Escrow Demo - Agent-to-Agent Payment\n");
  console.log(`Program ID: ${PROGRAM_ID.toBase58()}\n`);

  const connection = new Connection(DEVNET_URL, "confirmed");

  // Use deployer as Agent A (buyer), generate new keypair for Agent B (seller)
  const agentA = loadDeployerKeypair();
  const agentB = Keypair.generate(); // Seller

  console.log(`Agent A (Buyer):  ${agentA.publicKey.toBase58()}`);
  console.log(`Agent B (Seller): ${agentB.publicKey.toBase58()}\n`);

  // Check balances
  const balanceA = await connection.getBalance(agentA.publicKey);
  const balanceB = await connection.getBalance(agentB.publicKey);
  console.log(`📊 Initial Balances:`);
  console.log(`   Agent A: ${balanceA / LAMPORTS_PER_SOL} SOL`);
  console.log(`   Agent B: ${balanceB / LAMPORTS_PER_SOL} SOL`);

  if (balanceA < 0.002 * LAMPORTS_PER_SOL) {
    console.log("\n❌ Agent A has insufficient funds.");
    process.exit(1);
  }

  // Create escrow with small amount
  const escrowAmount = BigInt(Math.floor(0.001 * LAMPORTS_PER_SOL)); // 0.001 SOL
  const seed = BigInt(Date.now()); // Unique seed based on timestamp

  console.log(`\n📝 Creating Escrow...`);
  console.log(`   Amount: ${Number(escrowAmount) / LAMPORTS_PER_SOL} SOL`);
  console.log(`   Seed: ${seed}`);

  const [escrowPDA] = deriveEscrowPDA(agentA.publicKey, agentB.publicKey, seed);
  console.log(`   Escrow PDA: ${escrowPDA.toBase58()}`);

  try {
    const createSig = await createEscrow(
      connection,
      agentA,
      agentB.publicKey,
      escrowAmount,
      seed
    );
    console.log(`   ✅ Created! Tx: ${createSig.slice(0, 32)}...`);
  } catch (e: any) {
    console.log(`   ❌ Failed: ${e.message}`);
    console.log(e);
    process.exit(1);
  }

  // Read escrow state
  const escrowState = await getEscrow(connection, agentA.publicKey, agentB.publicKey, seed);
  if (escrowState) {
    console.log(`\n📋 Escrow State:`);
    console.log(`   Status: ${escrowState.status}`);
    console.log(`   Amount: ${Number(escrowState.amount) / LAMPORTS_PER_SOL} SOL`);
  }

  // Simulate task completion, then release
  console.log(`\n⏳ [Agent B completes task...]`);

  console.log(`\n💸 Releasing Funds to Seller...`);
  try {
    const releaseSig = await releaseEscrow(
      connection,
      agentA,
      agentB.publicKey,
      seed
    );
    console.log(`   ✅ Released! Tx: ${releaseSig.slice(0, 32)}...`);
  } catch (e: any) {
    console.log(`   ❌ Failed: ${e.message}`);
    console.log(e);
    process.exit(1);
  }

  // Final balances
  const finalBalanceA = await connection.getBalance(agentA.publicKey);
  const finalBalanceB = await connection.getBalance(agentB.publicKey);
  console.log(`\n📊 Final Balances:`);
  console.log(`   Agent A: ${finalBalanceA / LAMPORTS_PER_SOL} SOL`);
  console.log(`   Agent B: ${finalBalanceB / LAMPORTS_PER_SOL} SOL`);

  // Read final escrow state
  const finalEscrowState = await getEscrow(connection, agentA.publicKey, agentB.publicKey, seed);
  if (finalEscrowState) {
    console.log(`\n📋 Final Escrow State:`);
    console.log(`   Status: ${finalEscrowState.status}`);
  }

  console.log(`\n✨ Demo complete! Pact enabled trustless agent-to-agent payment.`);
  console.log(`\n🔗 View on explorer:`);
  console.log(`   https://explorer.solana.com/address/${escrowPDA.toBase58()}?cluster=devnet`);
}

main().catch(console.error);
