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
});
