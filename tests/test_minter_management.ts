import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SssToken } from "../target/types/sss_token";
import { expect } from "chai";

describe("Minter Management", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SssToken as Program<SssToken>;
  const authority = provider.wallet;

  let stablecoinPda: anchor.web3.PublicKey;
  let minter1: anchor.web3.Keypair;
  let minter2: anchor.web3.Keypair;
  const PRESET_SSS_1 = 1;
  const NAME = "Test Stablecoin";
  const SYMBOL = "TST";
  const URI = "https://example.com/metadata.json";
  const DECIMALS = 6;

  before(async () => {
    const [pda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("stablecoin"), authority.publicKey.toBuffer()],
      program.programId
    );
    stablecoinPda = pda;
    minter1 = anchor.web3.Keypair.generate();
    minter2 = anchor.web3.Keypair.generate();
  });

  it("Initializes stablecoin for minter tests", async () => {
    await program.methods
      .initialize(PRESET_SSS_1, NAME, SYMBOL, URI, DECIMALS)
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
        assetMint: anchor.web3.PublicKey.default,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const state = await program.account.stablecoinState.fetch(stablecoinPda);
    expect(state.preset).to.equal(PRESET_SSS_1);
  });

  it("Adds minter with quota", async () => {
    const quota = new anchor.BN(1_000_000);
    const [minterInfoPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("minter"), stablecoinPda.toBuffer(), minter1.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .addMinter(quota)
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
        minterInfo: minterInfoPda,
        minter: minter1.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const minterInfo = await program.account.minterInfo.fetch(minterInfoPda);
    expect(minterInfo.minter.toString()).to.equal(minter1.publicKey.toString());
    expect(minterInfo.quota.toNumber()).to.equal(1_000_000);
    expect(minterInfo.mintedAmount.toNumber()).to.equal(0);
  });

  it("Adds another minter with different quota", async () => {
    const quota = new anchor.BN(5_000_000);
    const [minterInfoPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("minter"), stablecoinPda.toBuffer(), minter2.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .addMinter(quota)
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
        minterInfo: minterInfoPda,
        minter: minter2.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const minterInfo = await program.account.minterInfo.fetch(minterInfoPda);
    expect(minterInfo.quota.toNumber()).to.equal(5_000_000);
  });

  it("Updates minter quota", async () => {
    const newQuota = new anchor.BN(2_000_000);
    const [minterInfoPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("minter"), stablecoinPda.toBuffer(), minter1.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .updateQuota(newQuota)
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
        minterInfo: minterInfoPda,
      })
      .rpc();

    const minterInfo = await program.account.minterInfo.fetch(minterInfoPda);
    expect(minterInfo.quota.toNumber()).to.equal(2_000_000);
  });

  it("Updates quota to zero", async () => {
    const newQuota = new anchor.BN(0);
    const [minterInfoPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("minter"), stablecoinPda.toBuffer(), minter2.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .updateQuota(newQuota)
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
        minterInfo: minterInfoPda,
      })
      .rpc();

    const minterInfo = await program.account.minterInfo.fetch(minterInfoPda);
    expect(minterInfo.quota.toNumber()).to.equal(0);
  });

  it("Removes minter", async () => {
    const [minterInfoPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("minter"), stablecoinPda.toBuffer(), minter1.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .removeMinter()
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
        minterInfo: minterInfoPda,
      })
      .rpc();

    // Verify minter was removed
    try {
      await program.account.minterInfo.fetch(minterInfoPda);
      expect.fail("MinterInfo should have been closed");
    } catch (e: any) {
      expect(e.message).to.include("Account does not exist");
    }
  });

  it("Removes second minter", async () => {
    const [minterInfoPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("minter"), stablecoinPda.toBuffer(), minter2.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .removeMinter()
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
        minterInfo: minterInfoPda,
      })
      .rpc();

    try {
      await program.account.minterInfo.fetch(minterInfoPda);
      expect.fail("MinterInfo should have been closed");
    } catch (e: any) {
      expect(e.message).to.include("Account does not exist");
    }
  });

  it("Fails to add minter with unauthorized account", async () => {
    const unauthorized = anchor.web3.Keypair.generate();
    const newMinter = anchor.web3.Keypair.generate();
    const quota = new anchor.BN(1_000_000);
    const [minterInfoPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("minter"), stablecoinPda.toBuffer(), newMinter.publicKey.toBuffer()],
      program.programId
    );

    try {
      await program.methods
        .addMinter(quota)
        .accounts({
          authority: unauthorized.publicKey,
          state: stablecoinPda,
          minterInfo: minterInfoPda,
          minter: newMinter.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([unauthorized])
        .rpc();
      expect.fail("Should have thrown Unauthorized error");
    } catch (e: any) {
      expect(e.error?.errorCode?.code).to.equal("Unauthorized");
    }
  });

  it("Fails to remove minter with unauthorized account", async () => {
    // First add a minter
    const newMinter = anchor.web3.Keypair.generate();
    const quota = new anchor.BN(1_000_000);
    const [minterInfoPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("minter"), stablecoinPda.toBuffer(), newMinter.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .addMinter(quota)
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
        minterInfo: minterInfoPda,
        minter: newMinter.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    // Try to remove with unauthorized account
    const unauthorized = anchor.web3.Keypair.generate();
    try {
      await program.methods
        .removeMinter()
        .accounts({
          authority: unauthorized.publicKey,
          state: stablecoinPda,
          minterInfo: minterInfoPda,
        })
        .signers([unauthorized])
        .rpc();
      expect.fail("Should have thrown Unauthorized error");
    } catch (e: any) {
      expect(e.error?.errorCode?.code).to.equal("Unauthorized");
    }
  });

  it("Fails to update quota with unauthorized account", async () => {
    const [minterInfoPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("minter"), stablecoinPda.toBuffer(), minter1.publicKey.toBuffer()],
      program.programId
    );

    const unauthorized = anchor.web3.Keypair.generate();
    try {
      await program.methods
        .updateQuota(new anchor.BN(10_000_000))
        .accounts({
          authority: unauthorized.publicKey,
          state: stablecoinPda,
          minterInfo: minterInfoPda,
        })
        .signers([unauthorized])
        .rpc();
      expect.fail("Should have thrown Unauthorized error");
    } catch (e: any) {
      expect(e.error?.errorCode?.code).to.equal("Unauthorized");
    }
  });

  it("Adds minter with high quota", async () => {
    const highQuota = new anchor.BN("1000000000000000"); // 1 quadrillion
    const newMinter = anchor.web3.Keypair.generate();
    const [minterInfoPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("minter"), stablecoinPda.toBuffer(), newMinter.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .addMinter(highQuota)
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
        minterInfo: minterInfoPda,
        minter: newMinter.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const minterInfo = await program.account.minterInfo.fetch(minterInfoPda);
    expect(minterInfo.quota.toString()).to.equal("1000000000000000");
  });

  it("Re-adds previously removed minter", async () => {
    const quota = new anchor.BN(3_000_000);
    const [minterInfoPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("minter"), stablecoinPda.toBuffer(), minter1.publicKey.toBuffer()],
      program.programId
    );

    // Should be able to add minter that was previously removed
    await program.methods
      .addMinter(quota)
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
        minterInfo: minterInfoPda,
        minter: minter1.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const minterInfo = await program.account.minterInfo.fetch(minterInfoPda);
    expect(minterInfo.mintedAmount.toNumber()).to.equal(0); // Should reset minted amount
  });
});
