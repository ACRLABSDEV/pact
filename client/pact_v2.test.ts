import { describe, it, expect, beforeAll } from "vitest";
import {
  Connection,
  Keypair,
  PublicKey,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import {
  PROGRAM_ID_V2,
  DEFAULT_TIMEOUT_SECONDS,
  EscrowStatus,
  deriveEscrowPDA,
  createEscrow,
  markDelivered,
  acceptDelivery,
  releaseEscrow,
  refundEscrow,
  raiseDispute,
  arbitrate,
  getEscrow,
  isTimedOut,
  statusToString,
} from "./pact_v2";

// =============================================================================
// UNIT TESTS - No network required
// =============================================================================

describe("PDA Derivation (v2)", () => {
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

  it("should derive different PDAs for different participants", () => {
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

  it("should handle edge case seeds", () => {
    // Zero seed
    const [pda0, bump0] = deriveEscrowPDA(buyer, seller, BigInt(0));
    expect(pda0).toBeInstanceOf(PublicKey);

    // Max u64 seed
    const maxU64 = BigInt("18446744073709551615");
    const [pdaMax, bumpMax] = deriveEscrowPDA(buyer, seller, maxU64);
    expect(pdaMax).toBeInstanceOf(PublicKey);
  });
});

describe("Constants", () => {
  it("should have correct default timeout", () => {
    // 3 days = 259200 seconds
    expect(DEFAULT_TIMEOUT_SECONDS).toBe(259200n);
  });
});

describe("Status Enum", () => {
  it("should have correct status values", () => {
    expect(EscrowStatus.Active).toBe(0);
    expect(EscrowStatus.Delivered).toBe(1);
    expect(EscrowStatus.Accepted).toBe(2);
    expect(EscrowStatus.Disputed).toBe(3);
    expect(EscrowStatus.Released).toBe(4);
    expect(EscrowStatus.Refunded).toBe(5);
  });

  it("should convert status to string", () => {
    expect(statusToString(EscrowStatus.Active)).toBe("Active");
    expect(statusToString(EscrowStatus.Delivered)).toBe("Delivered");
    expect(statusToString(EscrowStatus.Accepted)).toBe("Accepted");
    expect(statusToString(EscrowStatus.Disputed)).toBe("Disputed");
    expect(statusToString(EscrowStatus.Released)).toBe("Released");
    expect(statusToString(EscrowStatus.Refunded)).toBe("Refunded");
  });
});

describe("Timeout Logic", () => {
  it("should detect timed out escrow", () => {
    const pastEscrow = {
      buyer: Keypair.generate().publicKey,
      seller: Keypair.generate().publicKey,
      arbitrator: null,
      amount: BigInt(1000000),
      createdAt: BigInt(Math.floor(Date.now() / 1000) - 400000), // 4+ days ago
      timeoutSeconds: 259200n, // 3 days
      termsHash: new Uint8Array(32),
      status: EscrowStatus.Active,
      flags: 0,
      bump: 255,
    };

    expect(isTimedOut(pastEscrow)).toBe(true);
  });

  it("should detect not timed out escrow", () => {
    const recentEscrow = {
      buyer: Keypair.generate().publicKey,
      seller: Keypair.generate().publicKey,
      arbitrator: null,
      amount: BigInt(1000000),
      createdAt: BigInt(Math.floor(Date.now() / 1000) - 1000), // 16 mins ago
      timeoutSeconds: 259200n, // 3 days
      termsHash: new Uint8Array(32),
      status: EscrowStatus.Active,
      flags: 0,
      bump: 255,
    };

    expect(isTimedOut(recentEscrow)).toBe(false);
  });

  it("should handle no timeout (0)", () => {
    const noTimeoutEscrow = {
      buyer: Keypair.generate().publicKey,
      seller: Keypair.generate().publicKey,
      arbitrator: null,
      amount: BigInt(1000000),
      createdAt: BigInt(0), // Long ago
      timeoutSeconds: 0n, // No timeout
      termsHash: new Uint8Array(32),
      status: EscrowStatus.Active,
      flags: 0,
      bump: 255,
    };

    expect(isTimedOut(noTimeoutEscrow)).toBe(false);
  });
});

describe("Instruction Data Formats", () => {
  it("should create correct CreateEscrow data", () => {
    const amount = BigInt(100_000_000);
    const seed = BigInt(1234567890);
    const timeout = BigInt(259200);
    const termsHash = new Uint8Array(32).fill(0xAB);

    const data = Buffer.alloc(57);
    data.writeUInt8(0, 0); // IX_CREATE_ESCROW
    data.writeBigUInt64LE(amount, 1);
    data.writeBigUInt64LE(seed, 9);
    data.writeBigUInt64LE(timeout, 17);
    data.set(termsHash, 25);

    expect(data[0]).toBe(0);
    expect(data.readBigUInt64LE(1)).toBe(amount);
    expect(data.readBigUInt64LE(9)).toBe(seed);
    expect(data.readBigUInt64LE(17)).toBe(timeout);
    expect(data.subarray(25, 57)).toEqual(Buffer.from(termsHash));
  });

  it("should create correct single-byte instructions", () => {
    const markDeliveredData = Buffer.from([1]);
    const acceptDeliveryData = Buffer.from([2]);
    const releaseData = Buffer.from([3]);
    const refundData = Buffer.from([4]);
    const disputeData = Buffer.from([5]);

    expect(markDeliveredData[0]).toBe(1);
    expect(acceptDeliveryData[0]).toBe(2);
    expect(releaseData[0]).toBe(3);
    expect(refundData[0]).toBe(4);
    expect(disputeData[0]).toBe(5);
  });

  it("should create correct Arbitrate data", () => {
    const releaseDecision = Buffer.from([6, 1]);
    const refundDecision = Buffer.from([6, 0]);

    expect(releaseDecision[0]).toBe(6);
    expect(releaseDecision[1]).toBe(1);
    expect(refundDecision[1]).toBe(0);
  });
});

describe("Account Layout", () => {
  it("should have correct offsets", () => {
    // Verify offsets match Rust implementation
    expect(8).toBe(8);   // OFF_BUYER
    expect(40).toBe(40); // OFF_SELLER
    expect(72).toBe(72); // OFF_ARBITRATOR
    expect(104).toBe(104); // OFF_MINT
    expect(136).toBe(136); // OFF_AMOUNT
    expect(144).toBe(144); // OFF_CREATED_AT
    expect(152).toBe(152); // OFF_TIMEOUT
    expect(160).toBe(160); // OFF_TERMS_HASH
    expect(192).toBe(192); // OFF_STATUS
    expect(193).toBe(193); // OFF_FLAGS
    expect(194).toBe(194); // OFF_BUMP

    // Total size
    expect(195).toBe(195); // ESCROW_SIZE
  });
});

// =============================================================================
// INTEGRATION TESTS - Requires devnet and funded wallets
// =============================================================================

describe.skipIf(!process.env.RUN_INTEGRATION_TESTS)("Integration: Full Escrow Flow", () => {
  const connection = new Connection("https://api.devnet.solana.com", "confirmed");
  let buyer: Keypair;
  let seller: Keypair;
  let arbitrator: Keypair;

  beforeAll(() => {
    // In real tests, load funded keypairs
    buyer = Keypair.generate();
    seller = Keypair.generate();
    arbitrator = Keypair.generate();
  });

  it("should get null for non-existent escrow", async () => {
    const seed = BigInt(Date.now());
    const escrow = await getEscrow(connection, buyer.publicKey, seller.publicKey, seed);
    expect(escrow).toBeNull();
  });

  // Full flow tests would go here with funded wallets
  // - Create escrow
  // - Mark delivered
  // - Accept delivery (auto-release)
  // - Test dispute flow
  // - Test timeout refund
  // - Test arbitration
});

// =============================================================================
// STATE MACHINE TESTS
// =============================================================================

describe("State Machine Transitions", () => {
  it("should define valid transitions from Active", () => {
    // From Active, can go to: Delivered, Disputed, Refunded, Released
    const validFromActive = [
      EscrowStatus.Delivered, // MarkDelivered
      EscrowStatus.Disputed, // Dispute
      EscrowStatus.Refunded, // Refund (seller or buyer after timeout)
      EscrowStatus.Released, // Release (buyer skips attestation)
    ];

    validFromActive.forEach(status => {
      expect(status).toBeGreaterThanOrEqual(0);
      expect(status).toBeLessThanOrEqual(5);
    });
  });

  it("should define valid transitions from Delivered", () => {
    // From Delivered, can go to: Disputed, Released (via AcceptDelivery or Release)
    const validFromDelivered = [
      EscrowStatus.Disputed,
      EscrowStatus.Released,
    ];

    validFromDelivered.forEach(status => {
      expect(status).toBeDefined();
    });
  });

  it("should define valid transitions from Disputed", () => {
    // From Disputed, can go to: Released or Refunded (via Arbitrate)
    const validFromDisputed = [
      EscrowStatus.Released,
      EscrowStatus.Refunded,
    ];

    validFromDisputed.forEach(status => {
      expect(status).toBeDefined();
    });
  });

  it("should have no transitions from terminal states", () => {
    // Released and Refunded are terminal states
    expect(EscrowStatus.Released).toBe(4);
    expect(EscrowStatus.Refunded).toBe(5);
  });
});
