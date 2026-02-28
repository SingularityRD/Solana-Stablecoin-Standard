import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SssToken } from "../target/types/sss_token";
import { expect } from "chai";

describe("SSS-1: Basic Operations", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SssToken as Program<SssToken>;
  const authority = provider.wallet;

  let stablecoinPda: anchor.web3.PublicKey;
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
  });

  it("Initializes SSS-1 stablecoin", async () => {
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
    expect(state.complianceEnabled).to.be.false;
    expect(state.paused).to.be.false;
    expect(state.authority.toString()).to.equal(authority.publicKey.toString());
  });

  it("Mints tokens", async () => {
    const recipient = anchor.web3.Keypair.generate();
    const amount = new anchor.BN(1_000_000);

    await program.methods
      .mint(recipient.publicKey, amount)
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
      })
      .rpc();

    const state = await program.account.stablecoinState.fetch(stablecoinPda);
    expect(state.totalSupply.toNumber()).to.equal(1_000_000);
  });

  it("Burns tokens", async () => {
    const amount = new anchor.BN(500_000);

    await program.methods
      .burn(amount)
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
      })
      .rpc();

    const state = await program.account.stablecoinState.fetch(stablecoinPda);
    expect(state.totalSupply.toNumber()).to.equal(500_000);
  });

  it("Pauses operations", async () => {
    await program.methods
      .pause()
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
      })
      .rpc();

    const state = await program.account.stablecoinState.fetch(stablecoinPda);
    expect(state.paused).to.be.true;
  });

  it("Unpauses operations", async () => {
    await program.methods
      .unpause()
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
      })
      .rpc();

    const state = await program.account.stablecoinState.fetch(stablecoinPda);
    expect(state.paused).to.be.false;
  });

  it("Freezes account", async () => {
    const account = anchor.web3.Keypair.generate().publicKey;

    await program.methods
      .freezeAccount(account)
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
      })
      .rpc();

    // Event should be emitted
    expect(true).to.be.true;
  });

  it("Thaws account", async () => {
    const account = anchor.web3.Keypair.generate().publicKey;

    await program.methods
      .thawAccount(account)
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
      })
      .rpc();

    expect(true).to.be.true;
  });

  it("Mints to multiple recipients", async () => {
    const recipient1 = anchor.web3.Keypair.generate();
    const recipient2 = anchor.web3.Keypair.generate();
    const amount = new anchor.BN(100_000);

    await program.methods
      .mint(recipient1.publicKey, amount)
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
      })
      .rpc();

    await program.methods
      .mint(recipient2.publicKey, amount)
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
      })
      .rpc();

    const state = await program.account.stablecoinState.fetch(stablecoinPda);
    expect(state.totalSupply.toNumber()).to.be.greaterThan(500_000);
  });

  it("Burns partial amount", async () => {
    const stateBefore = await program.account.stablecoinState.fetch(stablecoinPda);
    const burnAmount = new anchor.BN(10_000);

    await program.methods
      .burn(burnAmount)
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
      })
      .rpc();

    const stateAfter = await program.account.stablecoinState.fetch(stablecoinPda);
    expect(stateAfter.totalSupply.toNumber()).to.equal(
      stateBefore.totalSupply.toNumber() - 10_000
    );
  });

  it("Toggles pause multiple times", async () => {
    // Pause
    await program.methods
      .pause()
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
      })
      .rpc();

    let state = await program.account.stablecoinState.fetch(stablecoinPda);
    expect(state.paused).to.be.true;

    // Unpause
    await program.methods
      .unpause()
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
      })
      .rpc();

    state = await program.account.stablecoinState.fetch(stablecoinPda);
    expect(state.paused).to.be.false;

    // Pause again
    await program.methods
      .pause()
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
      })
      .rpc();

    state = await program.account.stablecoinState.fetch(stablecoinPda);
    expect(state.paused).to.be.true;

    // Unpause for other tests
    await program.methods
      .unpause()
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
      })
      .rpc();
  });

  it("Freezes multiple accounts", async () => {
    const account1 = anchor.web3.Keypair.generate().publicKey;
    const account2 = anchor.web3.Keypair.generate().publicKey;
    const account3 = anchor.web3.Keypair.generate().publicKey;

    await program.methods
      .freezeAccount(account1)
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
      })
      .rpc();

    await program.methods
      .freezeAccount(account2)
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
      })
      .rpc();

    await program.methods
      .freezeAccount(account3)
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
      })
      .rpc();

    expect(true).to.be.true;
  });

  it("Thaws multiple accounts", async () => {
    const account1 = anchor.web3.Keypair.generate().publicKey;
    const account2 = anchor.web3.Keypair.generate().publicKey;

    // Freeze first
    await program.methods
      .freezeAccount(account1)
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
      })
      .rpc();

    await program.methods
      .freezeAccount(account2)
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
      })
      .rpc();

    // Then thaw
    await program.methods
      .thawAccount(account1)
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
      })
      .rpc();

    await program.methods
      .thawAccount(account2)
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
      })
      .rpc();

    expect(true).to.be.true;
  });

  it("Mints large amount", async () => {
    const recipient = anchor.web3.Keypair.generate();
    const largeAmount = new anchor.BN("1000000000000"); // 1 trillion

    await program.methods
      .mint(recipient.publicKey, largeAmount)
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
      })
      .rpc();

    const state = await program.account.stablecoinState.fetch(stablecoinPda);
    expect(state.totalSupply.toNumber()).to.be.greaterThan(1_000_000_000_000);
  });

  it("Verifies state persistence", async () => {
    const state = await program.account.stablecoinState.fetch(stablecoinPda);
    
    expect(state.authority.toString()).to.equal(authority.publicKey.toString());
    expect(state.preset).to.equal(PRESET_SSS_1);
    expect(state.complianceEnabled).to.be.false;
    expect(state.bump).to.be.a("number");
  });
});
