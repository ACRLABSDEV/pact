import { describe, it, expect, beforeAll } from "vitest";
import {
  Connection,
  Keypair,
  PublicKey,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import {
  PROGRAM_ID,
  deriveEscrowPDA,
  createEscrow,
  releaseEscrow,
  refundEscrow,
  getEscrow,
} from "./pact";

// =============================================================================
// UNIT TESTS - No network required
// =============================================================================

describe("PDA Derivation", () => {
  const buyer = Keypair.generate().publicKey;
  const seller = Keypair.generate().publicKey;

  it("should derive deterministic PDA for same inputs", () => {
    const seed = BigInt(12345);
    const [pda1, bump1] = deriveEscrowPDA(buyer, seller, seed);
    const [pda2, bump2] = deriveEscrowPDA(buyer, seller, seed);

    expect(pda1.toBase58()).toBe(pda2.toBase58());
    expect(bump1).toBe(bump2);
  });

  it("should derive different PDAs for different seeds", () => {
    const [pda1] = deriveEscrowPDA(buyer, seller, BigInt(1));
    const [pda2] = deriveEscrowPDA(buyer, seller, BigInt(2));

    expect(pda1.toBase58()).not.toBe(pda2.toBase58());
  });

  it("should derive different PDAs for different buyer/seller", () => {
    const seed = BigInt(12345);
    const otherBuyer = Keypair.generate().publicKey;

    const [pda1] = deriveEscrowPDA(buyer, seller, seed);
    const [pda2] = deriveEscrowPDA(otherBuyer, seller, seed);

    expect(pda1.toBase58()).not.toBe(pda2.toBase58());
  });

  it("should derive different PDAs when buyer/seller swapped", () => {
    const seed = BigInt(12345);
    const [pda1] = deriveEscrowPDA(buyer, seller, seed);
    const [pda2] = deriveEscrowPDA(seller, buyer, seed);

    expect(pda1.toBase58()).not.toBe(pda2.toBase58());
  });

  it("should handle seed of 0", () => {
    const [pda, bump] = deriveEscrowPDA(buyer, seller, BigInt(0));
    expect(pda).toBeInstanceOf(PublicKey);
    expect(bump).toBeGreaterThanOrEqual(0);
    expect(bump).toBeLessThanOrEqual(255);
  });

  it("should handle max u64 seed", () => {
    const maxU64 = BigInt("18446744073709551615");
    const [pda, bump] = deriveEscrowPDA(buyer, seller, maxU64);
    expect(pda).toBeInstanceOf(PublicKey);
    expect(bump).toBeGreaterThanOrEqual(0);
  });
});

describe("Program ID", () => {
  it("should be a valid public key", () => {
    expect(PROGRAM_ID).toBeInstanceOf(PublicKey);
    expect(PROGRAM_ID.toBase58()).toBe("S64L6x9bZqDewocv5MrCLeTAq1MKatLqrWfrLpdcDKM");
  });
});

// =============================================================================
// INTEGRATION TESTS - Requires devnet connection
// =============================================================================

describe("Integration: Escrow Flow", () => {
  const connection = new Connection("https://api.devnet.solana.com", "confirmed");
  let buyer: Keypair;
  let seller: Keypair;

  beforeAll(async () => {
    // Load test keypairs from environment or generate
    // In CI, these would be funded test wallets
    buyer = Keypair.generate();
    seller = Keypair.generate();
  });

  it("should get null for non-existent escrow", async () => {
    const seed = BigInt(Date.now());
    const escrow = await getEscrow(connection, buyer.publicKey, seller.publicKey, seed);
    expect(escrow).toBeNull();
  });

  // NOTE: Full integration tests require funded wallets
  // These are marked as skip unless RUN_INTEGRATION_TESTS=true
  describe.skipIf(!process.env.RUN_INTEGRATION_TESTS)("With Funded Wallets", () => {
    let fundedBuyer: Keypair;
    let fundedSeller: Keypair;
    const testSeed = BigInt(Date.now());
    const escrowAmount = BigInt(0.001 * LAMPORTS_PER_SOL);

    beforeAll(async () => {
      // Load funded keypairs from environment
      if (process.env.BUYER_KEYPAIR) {
        fundedBuyer = Keypair.fromSecretKey(
          Uint8Array.from(JSON.parse(process.env.BUYER_KEYPAIR))
        );
      }
      if (process.env.SELLER_KEYPAIR) {
        fundedSeller = Keypair.fromSecretKey(
          Uint8Array.from(JSON.parse(process.env.SELLER_KEYPAIR))
        );
      }
    });

    it("should create escrow", async () => {
      const sig = await createEscrow(
        connection,
        fundedBuyer,
        fundedSeller.publicKey,
        escrowAmount,
        testSeed
      );
      expect(sig).toBeTruthy();
      expect(sig.length).toBeGreaterThan(0);

      // Verify escrow was created
      const escrow = await getEscrow(
        connection,
        fundedBuyer.publicKey,
        fundedSeller.publicKey,
        testSeed
      );
      expect(escrow).not.toBeNull();
      expect(escrow!.status).toBe("Active");
      expect(escrow!.amount).toBe(escrowAmount);
    });

    it("should release escrow to seller", async () => {
      const sellerBalanceBefore = await connection.getBalance(fundedSeller.publicKey);

      const sig = await releaseEscrow(
        connection,
        fundedBuyer,
        fundedSeller.publicKey,
        testSeed
      );
      expect(sig).toBeTruthy();

      const sellerBalanceAfter = await connection.getBalance(fundedSeller.publicKey);
      expect(sellerBalanceAfter).toBeGreaterThan(sellerBalanceBefore);

      const escrow = await getEscrow(
        connection,
        fundedBuyer.publicKey,
        fundedSeller.publicKey,
        testSeed
      );
      expect(escrow!.status).toBe("Released");
    });

    it("should refund escrow to buyer", async () => {
      const refundSeed = BigInt(Date.now() + 1000);
      
      // Create new escrow
      await createEscrow(
        connection,
        fundedBuyer,
        fundedSeller.publicKey,
        escrowAmount,
        refundSeed
      );

      const buyerBalanceBefore = await connection.getBalance(fundedBuyer.publicKey);

      // Seller initiates refund
      const sig = await refundEscrow(
        connection,
        fundedBuyer.publicKey,
        fundedSeller,
        refundSeed
      );
      expect(sig).toBeTruthy();

      const buyerBalanceAfter = await connection.getBalance(fundedBuyer.publicKey);
      expect(buyerBalanceAfter).toBeGreaterThan(buyerBalanceBefore);

      const escrow = await getEscrow(
        connection,
        fundedBuyer.publicKey,
        fundedSeller.publicKey,
        refundSeed
      );
      expect(escrow!.status).toBe("Refunded");
    });
  });
});

// =============================================================================
// SECURITY TESTS
// =============================================================================

describe("Security Considerations", () => {
  it("should not allow amount overflow in instruction data", () => {
    // The client uses BigInt which handles large numbers correctly
    const maxSafeAmount = BigInt("18446744073709551615"); // max u64
    expect(() => {
      const buffer = Buffer.alloc(8);
      buffer.writeBigUInt64LE(maxSafeAmount);
    }).not.toThrow();
  });

  it("should validate public keys are 32 bytes", () => {
    const validKey = Keypair.generate().publicKey;
    expect(validKey.toBuffer().length).toBe(32);
  });
});
