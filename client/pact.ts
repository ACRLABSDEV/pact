import {
  Connection,
  PublicKey,
  Keypair,
  Transaction,
  TransactionInstruction,
  SystemProgram,
  LAMPORTS_PER_SOL,
  sendAndConfirmTransaction,
} from "@solana/web3.js";

// Program ID deployed on devnet
export const PROGRAM_ID = new PublicKey("S64L6x9bZqDewocv5MrCLeTAq1MKatLqrWfrLpdcDKM");

// Instruction discriminators
const INSTRUCTION_CREATE = 0;
const INSTRUCTION_RELEASE = 1;
const INSTRUCTION_REFUND = 2;

/**
 * Derive escrow PDA from buyer, seller, and seed
 */
export function deriveEscrowPDA(
  buyer: PublicKey,
  seller: PublicKey,
  seed: bigint
): [PublicKey, number] {
  const seedBuffer = Buffer.alloc(8);
  seedBuffer.writeBigUInt64LE(seed);

  return PublicKey.findProgramAddressSync(
    [Buffer.from("escrow"), buyer.toBuffer(), seller.toBuffer(), seedBuffer],
    PROGRAM_ID
  );
}

/**
 * Create an escrow - buyer deposits SOL to be released to seller
 */
export async function createEscrow(
  connection: Connection,
  buyer: Keypair,
  seller: PublicKey,
  amountLamports: bigint,
  seed: bigint
): Promise<string> {
  const [escrowPDA] = deriveEscrowPDA(buyer.publicKey, seller, seed);

  // Instruction data: discriminator (1) + amount (8) + seed (8)
  const data = Buffer.alloc(17);
  data.writeUInt8(INSTRUCTION_CREATE, 0);
  data.writeBigUInt64LE(amountLamports, 1);
  data.writeBigUInt64LE(seed, 9);

  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: buyer.publicKey, isSigner: true, isWritable: true },
      { pubkey: seller, isSigner: false, isWritable: false },
      { pubkey: escrowPDA, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId: PROGRAM_ID,
    data,
  });

  const tx = new Transaction().add(instruction);
  const signature = await sendAndConfirmTransaction(connection, tx, [buyer]);

  return signature;
}

/**
 * Release escrow funds to seller - only buyer can do this
 */
export async function releaseEscrow(
  connection: Connection,
  buyer: Keypair,
  seller: PublicKey,
  seed: bigint
): Promise<string> {
  const [escrowPDA] = deriveEscrowPDA(buyer.publicKey, seller, seed);

  // Instruction data: just discriminator
  const data = Buffer.alloc(1);
  data.writeUInt8(INSTRUCTION_RELEASE, 0);

  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: buyer.publicKey, isSigner: true, isWritable: true },
      { pubkey: seller, isSigner: false, isWritable: true },
      { pubkey: escrowPDA, isSigner: false, isWritable: true },
    ],
    programId: PROGRAM_ID,
    data,
  });

  const tx = new Transaction().add(instruction);
  const signature = await sendAndConfirmTransaction(connection, tx, [buyer]);

  return signature;
}

/**
 * Refund escrow to buyer - only seller can do this
 */
export async function refundEscrow(
  connection: Connection,
  buyer: PublicKey,
  seller: Keypair,
  seed: bigint
): Promise<string> {
  const [escrowPDA] = deriveEscrowPDA(buyer, seller.publicKey, seed);

  // Instruction data: just discriminator
  const data = Buffer.alloc(1);
  data.writeUInt8(INSTRUCTION_REFUND, 0);

  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: buyer, isSigner: false, isWritable: true },
      { pubkey: seller.publicKey, isSigner: true, isWritable: true },
      { pubkey: escrowPDA, isSigner: false, isWritable: true },
    ],
    programId: PROGRAM_ID,
    data,
  });

  const tx = new Transaction().add(instruction);
  const signature = await sendAndConfirmTransaction(connection, tx, [seller]);

  return signature;
}

/**
 * Read escrow account data
 */
export async function getEscrow(
  connection: Connection,
  buyer: PublicKey,
  seller: PublicKey,
  seed: bigint
): Promise<{
  buyer: PublicKey;
  seller: PublicKey;
  amount: bigint;
  status: "Active" | "Released" | "Refunded";
} | null> {
  const [escrowPDA] = deriveEscrowPDA(buyer, seller, seed);

  const accountInfo = await connection.getAccountInfo(escrowPDA);
  if (!accountInfo) return null;

  const data = accountInfo.data;

  // Parse escrow data
  // [0..8] discriminator
  // [8..40] buyer
  // [40..72] seller
  // [72..80] amount
  // [80] status

  const storedBuyer = new PublicKey(data.subarray(8, 40));
  const storedSeller = new PublicKey(data.subarray(40, 72));
  const amount = data.readBigUInt64LE(72);
  const statusByte = data[80];

  const statusMap: Record<number, "Active" | "Released" | "Refunded"> = {
    0: "Active",
    1: "Released",
    2: "Refunded",
  };

  return {
    buyer: storedBuyer,
    seller: storedSeller,
    amount,
    status: statusMap[statusByte] || "Active",
  };
}
