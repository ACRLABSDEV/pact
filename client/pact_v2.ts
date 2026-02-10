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

// Program ID - UPDATE AFTER DEPLOY
export const PROGRAM_ID_V2 = new PublicKey("11111111111111111111111111111111"); // Placeholder

// Instruction discriminators
const IX_CREATE_ESCROW = 0;
const IX_MARK_DELIVERED = 1;
const IX_ACCEPT_DELIVERY = 2;
const IX_RELEASE = 3;
const IX_REFUND = 4;
const IX_DISPUTE = 5;
const IX_ARBITRATE = 6;

// Status enum
export enum EscrowStatus {
  Active = 0,
  Delivered = 1,
  Accepted = 2,
  Disputed = 3,
  Released = 4,
  Refunded = 5,
}

// Default timeout: 3 days in seconds
export const DEFAULT_TIMEOUT_SECONDS = 259200n;

// Escrow account layout offsets
const OFF_DISC = 0;
const OFF_BUYER = 8;
const OFF_SELLER = 40;
const OFF_ARBITRATOR = 72;
const OFF_MINT = 104;
const OFF_AMOUNT = 136;
const OFF_CREATED_AT = 144;
const OFF_TIMEOUT = 152;
const OFF_TERMS_HASH = 160;
const OFF_STATUS = 192;
const OFF_FLAGS = 193;
const OFF_BUMP = 194;

/**
 * Escrow account data structure
 */
export interface EscrowAccount {
  buyer: PublicKey;
  seller: PublicKey;
  arbitrator: PublicKey | null;
  amount: bigint;
  createdAt: bigint;
  timeoutSeconds: bigint;
  termsHash: Uint8Array;
  status: EscrowStatus;
  flags: number;
  bump: number;
}

/**
 * Derive escrow PDA from buyer, seller, and seed
 */
export function deriveEscrowPDA(
  buyer: PublicKey,
  seller: PublicKey,
  seed: bigint,
  programId: PublicKey = PROGRAM_ID_V2
): [PublicKey, number] {
  const seedBuffer = Buffer.alloc(8);
  seedBuffer.writeBigUInt64LE(seed);

  return PublicKey.findProgramAddressSync(
    [Buffer.from("escrow"), buyer.toBuffer(), seller.toBuffer(), seedBuffer],
    programId
  );
}

/**
 * Create an escrow - buyer deposits SOL
 */
export async function createEscrow(
  connection: Connection,
  buyer: Keypair,
  seller: PublicKey,
  arbitrator: PublicKey | null,
  amountLamports: bigint,
  seed: bigint,
  timeoutSeconds: bigint = DEFAULT_TIMEOUT_SECONDS,
  termsHash: Uint8Array = new Uint8Array(32),
  programId: PublicKey = PROGRAM_ID_V2
): Promise<string> {
  const [escrowPDA] = deriveEscrowPDA(buyer.publicKey, seller, seed, programId);
  
  // Use buyer as arbitrator if none specified (effectively no arbitrator)
  const arbPubkey = arbitrator || buyer.publicKey;

  // Instruction data: discriminator(1) + amount(8) + seed(8) + timeout(8) + terms_hash(32) = 57 bytes
  const data = Buffer.alloc(57);
  data.writeUInt8(IX_CREATE_ESCROW, 0);
  data.writeBigUInt64LE(amountLamports, 1);
  data.writeBigUInt64LE(seed, 9);
  data.writeBigUInt64LE(timeoutSeconds, 17);
  data.set(termsHash.slice(0, 32), 25);

  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: buyer.publicKey, isSigner: true, isWritable: true },
      { pubkey: seller, isSigner: false, isWritable: false },
      { pubkey: arbPubkey, isSigner: false, isWritable: false },
      { pubkey: escrowPDA, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId,
    data,
  });

  const tx = new Transaction().add(instruction);
  return sendAndConfirmTransaction(connection, tx, [buyer]);
}

/**
 * Mark work as delivered - seller only
 */
export async function markDelivered(
  connection: Connection,
  seller: Keypair,
  buyer: PublicKey,
  seed: bigint,
  programId: PublicKey = PROGRAM_ID_V2
): Promise<string> {
  const [escrowPDA] = deriveEscrowPDA(buyer, seller.publicKey, seed, programId);

  const data = Buffer.alloc(1);
  data.writeUInt8(IX_MARK_DELIVERED, 0);

  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: seller.publicKey, isSigner: true, isWritable: false },
      { pubkey: escrowPDA, isSigner: false, isWritable: true },
    ],
    programId,
    data,
  });

  const tx = new Transaction().add(instruction);
  return sendAndConfirmTransaction(connection, tx, [seller]);
}

/**
 * Accept delivery - buyer accepts and releases funds
 */
export async function acceptDelivery(
  connection: Connection,
  buyer: Keypair,
  seller: PublicKey,
  seed: bigint,
  programId: PublicKey = PROGRAM_ID_V2
): Promise<string> {
  const [escrowPDA] = deriveEscrowPDA(buyer.publicKey, seller, seed, programId);

  const data = Buffer.alloc(1);
  data.writeUInt8(IX_ACCEPT_DELIVERY, 0);

  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: buyer.publicKey, isSigner: true, isWritable: false },
      { pubkey: seller, isSigner: false, isWritable: true },
      { pubkey: escrowPDA, isSigner: false, isWritable: true },
    ],
    programId,
    data,
  });

  const tx = new Transaction().add(instruction);
  return sendAndConfirmTransaction(connection, tx, [buyer]);
}

/**
 * Release funds to seller - buyer only, skips attestation
 */
export async function releaseEscrow(
  connection: Connection,
  buyer: Keypair,
  seller: PublicKey,
  seed: bigint,
  programId: PublicKey = PROGRAM_ID_V2
): Promise<string> {
  const [escrowPDA] = deriveEscrowPDA(buyer.publicKey, seller, seed, programId);

  const data = Buffer.alloc(1);
  data.writeUInt8(IX_RELEASE, 0);

  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: buyer.publicKey, isSigner: true, isWritable: false },
      { pubkey: seller, isSigner: false, isWritable: true },
      { pubkey: escrowPDA, isSigner: false, isWritable: true },
    ],
    programId,
    data,
  });

  const tx = new Transaction().add(instruction);
  return sendAndConfirmTransaction(connection, tx, [buyer]);
}

/**
 * Refund funds to buyer
 * - Seller can refund anytime
 * - Buyer can refund if timeout expired or status is Active
 * - Arbitrator can refund if disputed
 */
export async function refundEscrow(
  connection: Connection,
  authority: Keypair,
  buyer: PublicKey,
  seller: PublicKey,
  seed: bigint,
  programId: PublicKey = PROGRAM_ID_V2
): Promise<string> {
  const [escrowPDA] = deriveEscrowPDA(buyer, seller, seed, programId);

  const data = Buffer.alloc(1);
  data.writeUInt8(IX_REFUND, 0);

  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: authority.publicKey, isSigner: true, isWritable: false },
      { pubkey: buyer, isSigner: false, isWritable: true },
      { pubkey: seller, isSigner: false, isWritable: false },
      { pubkey: escrowPDA, isSigner: false, isWritable: true },
    ],
    programId,
    data,
  });

  const tx = new Transaction().add(instruction);
  return sendAndConfirmTransaction(connection, tx, [authority]);
}

/**
 * Raise a dispute - buyer or seller
 */
export async function raiseDispute(
  connection: Connection,
  authority: Keypair,
  buyer: PublicKey,
  seller: PublicKey,
  seed: bigint,
  programId: PublicKey = PROGRAM_ID_V2
): Promise<string> {
  const [escrowPDA] = deriveEscrowPDA(buyer, seller, seed, programId);

  const data = Buffer.alloc(1);
  data.writeUInt8(IX_DISPUTE, 0);

  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: authority.publicKey, isSigner: true, isWritable: false },
      { pubkey: escrowPDA, isSigner: false, isWritable: true },
    ],
    programId,
    data,
  });

  const tx = new Transaction().add(instruction);
  return sendAndConfirmTransaction(connection, tx, [authority]);
}

/**
 * Arbitrate a dispute - arbitrator only
 * @param releaseToSeller - true = release to seller, false = refund to buyer
 */
export async function arbitrate(
  connection: Connection,
  arbitrator: Keypair,
  buyer: PublicKey,
  seller: PublicKey,
  seed: bigint,
  releaseToSeller: boolean,
  programId: PublicKey = PROGRAM_ID_V2
): Promise<string> {
  const [escrowPDA] = deriveEscrowPDA(buyer, seller, seed, programId);

  const data = Buffer.alloc(2);
  data.writeUInt8(IX_ARBITRATE, 0);
  data.writeUInt8(releaseToSeller ? 1 : 0, 1);

  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: arbitrator.publicKey, isSigner: true, isWritable: false },
      { pubkey: buyer, isSigner: false, isWritable: true },
      { pubkey: seller, isSigner: false, isWritable: true },
      { pubkey: escrowPDA, isSigner: false, isWritable: true },
    ],
    programId,
    data,
  });

  const tx = new Transaction().add(instruction);
  return sendAndConfirmTransaction(connection, tx, [arbitrator]);
}

/**
 * Read escrow account data
 */
export async function getEscrow(
  connection: Connection,
  buyer: PublicKey,
  seller: PublicKey,
  seed: bigint,
  programId: PublicKey = PROGRAM_ID_V2
): Promise<EscrowAccount | null> {
  const [escrowPDA] = deriveEscrowPDA(buyer, seller, seed, programId);

  const accountInfo = await connection.getAccountInfo(escrowPDA);
  if (!accountInfo) return null;

  const data = accountInfo.data;

  // Check discriminator
  const disc = data.readBigUInt64LE(OFF_DISC);
  if (disc !== 0x5041435445534352n) return null;

  const storedBuyer = new PublicKey(data.subarray(OFF_BUYER, OFF_BUYER + 32));
  const storedSeller = new PublicKey(data.subarray(OFF_SELLER, OFF_SELLER + 32));
  const arbBytes = data.subarray(OFF_ARBITRATOR, OFF_ARBITRATOR + 32);
  const isZeroArb = arbBytes.every(b => b === 0);
  const storedArbitrator = isZeroArb ? null : new PublicKey(arbBytes);
  
  const amount = data.readBigUInt64LE(OFF_AMOUNT);
  const createdAt = data.readBigUInt64LE(OFF_CREATED_AT);
  const timeoutSeconds = data.readBigUInt64LE(OFF_TIMEOUT);
  const termsHash = new Uint8Array(data.subarray(OFF_TERMS_HASH, OFF_TERMS_HASH + 32));
  const status = data[OFF_STATUS] as EscrowStatus;
  const flags = data[OFF_FLAGS];
  const bump = data[OFF_BUMP];

  return {
    buyer: storedBuyer,
    seller: storedSeller,
    arbitrator: storedArbitrator,
    amount,
    createdAt,
    timeoutSeconds,
    termsHash,
    status,
    flags,
    bump,
  };
}

/**
 * Check if escrow has timed out
 */
export function isTimedOut(escrow: EscrowAccount): boolean {
  if (escrow.timeoutSeconds === 0n) return false;
  const now = BigInt(Math.floor(Date.now() / 1000));
  return now >= escrow.createdAt + escrow.timeoutSeconds;
}

/**
 * Get human-readable status
 */
export function statusToString(status: EscrowStatus): string {
  const names = ["Active", "Delivered", "Accepted", "Disputed", "Released", "Refunded"];
  return names[status] || "Unknown";
}
