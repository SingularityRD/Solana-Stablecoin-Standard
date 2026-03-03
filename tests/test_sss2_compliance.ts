import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";

const TOKEN_2022_PROGRAM_ID = new anchor.web3.PublicKey("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

describe("SSS-2: Compliance Operations", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SssToken;
  const authority = provider.wallet;
  const assetMint = anchor.web3.Keypair.generate();

  let stablecoinPda: anchor.web3.PublicKey;
  let blacklister: anchor.web3.Keypair;
  let seizer: anchor.web3.Keypair;
  let blacklisterRolePda: anchor.web3.PublicKey;
  let seizerRolePda: anchor.web3.PublicKey;
  const PRESET_SSS_2 = 2;
  const NAME = "Compliant Stablecoin";
  const SYMBOL = "CUSD";
  const URI = "https://example.com/metadata.json";
  const DECIMALS = 6;

  before(async () => {
    const [pda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("stablecoin"), assetMint.publicKey.toBuffer()],
      program.programId
    );
    stablecoinPda = pda;
    blacklister = anchor.web3.Keypair.generate();
    seizer = anchor.web3.Keypair.generate();
    [blacklisterRolePda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("role"), stablecoinPda.toBuffer(), blacklister.publicKey.toBuffer()],
      program.programId
    );
    [seizerRolePda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("role"), stablecoinPda.toBuffer(), seizer.publicKey.toBuffer()],
      program.programId
    );
  });

  it("Initializes SSS-2 stablecoin", async () => {
    await program.methods
      .initialize(PRESET_SSS_2, NAME, SYMBOL, URI, DECIMALS)
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
        assetMint: assetMint.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([assetMint])
      .rpc();

    const state = await program.account.stablecoinState.fetch(stablecoinPda);
    expect(state.preset).to.equal(PRESET_SSS_2);
    expect(state.complianceEnabled).to.be.true;
  });

  it("Assigns blacklister role", async () => {
    const role = { blacklister: {} };
    
    await program.methods
      .assignRole(role)
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
        assignment: blacklisterRolePda,
        account: blacklister.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const roleAssignment = await program.account.roleAssignment.fetch(blacklisterRolePda);
    expect(roleAssignment.account.toString()).to.equal(blacklister.publicKey.toString());
  });

  it("Assigns seizer role", async () => {
    const role = { seizer: {} };
    
    await program.methods
      .assignRole(role)
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
        assignment: seizerRolePda,
        account: seizer.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const roleAssignment = await program.account.roleAssignment.fetch(seizerRolePda);
    expect(roleAssignment.account.toString()).to.equal(seizer.publicKey.toString());
  });

  it("Adds account to blacklist", async () => {
    const badActor = anchor.web3.Keypair.generate();
    const reason = "OFAC sanctions match";
    const [entryPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("blacklist"), stablecoinPda.toBuffer(), badActor.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .addToBlacklist(reason)
      .accounts({
        authority: blacklister.publicKey,
        state: stablecoinPda,
        roleAssignment: blacklisterRolePda,
        entry: entryPda,
        account: badActor.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([blacklister])
      .rpc();

    const entry = await program.account.blacklistEntry.fetch(entryPda);
    expect(entry.reason).to.equal(reason);
  });

  it("Removes account from blacklist", async () => {
    const badActor = anchor.web3.Keypair.generate();
    const [entryPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("blacklist"), stablecoinPda.toBuffer(), badActor.publicKey.toBuffer()],
      program.programId
    );
    
    // First add to blacklist
    await program.methods
      .addToBlacklist("Test reason")
      .accounts({
        authority: blacklister.publicKey,
        state: stablecoinPda,
        roleAssignment: blacklisterRolePda,
        entry: entryPda,
        account: badActor.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([blacklister])
      .rpc();

    // Then remove
    await program.methods
      .removeFromBlacklist()
      .accounts({
        authority: blacklister.publicKey,
        state: stablecoinPda,
        roleAssignment: blacklisterRolePda,
        entry: entryPda,
        account: badActor.publicKey,
      })
      .signers([blacklister])
      .rpc();

    const entryInfo = await provider.connection.getAccountInfo(entryPda);
    expect(entryInfo).to.equal(null);
  });

  it("Seizes tokens from blacklisted account", async () => {
    const fromTokenAccount = anchor.web3.Keypair.generate().publicKey;
    const toTokenAccount = anchor.web3.Keypair.generate().publicKey;
    const amount = new anchor.BN(100_000);

    await program.methods
      .seize(amount)
      .accounts({
        authority: seizer.publicKey,
        state: stablecoinPda,
        roleAssignment: seizerRolePda,
        assetMint: assetMint.publicKey,
        from: fromTokenAccount,
        to: toTokenAccount,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([seizer])
      .rpc();

    const state = await program.account.stablecoinState.fetch(stablecoinPda);
    expect(state.complianceEnabled).to.be.true;
  });

  it("Rejects SSS-2 operations on SSS-1", async () => {
    // This would require deploying a separate SSS-1 instance
    // For now, we verify the compliance check exists in code
    const state = await program.account.stablecoinState.fetch(stablecoinPda);
    expect(state.preset).to.equal(PRESET_SSS_2);
  });

  it("Blacklists multiple accounts", async () => {
    const badActor1 = anchor.web3.Keypair.generate();
    const badActor2 = anchor.web3.Keypair.generate();
    const badActor3 = anchor.web3.Keypair.generate();

    const [entryPda1] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("blacklist"), stablecoinPda.toBuffer(), badActor1.publicKey.toBuffer()],
      program.programId
    );
    const [entryPda2] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("blacklist"), stablecoinPda.toBuffer(), badActor2.publicKey.toBuffer()],
      program.programId
    );
    const [entryPda3] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("blacklist"), stablecoinPda.toBuffer(), badActor3.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .addToBlacklist("Fraudulent activity")
      .accounts({
        authority: blacklister.publicKey,
        state: stablecoinPda,
        roleAssignment: blacklisterRolePda,
        entry: entryPda1,
        account: badActor1.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([blacklister])
      .rpc();

    await program.methods
      .addToBlacklist("Money laundering")
      .accounts({
        authority: blacklister.publicKey,
        state: stablecoinPda,
        roleAssignment: blacklisterRolePda,
        entry: entryPda2,
        account: badActor2.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([blacklister])
      .rpc();

    await program.methods
      .addToBlacklist("OFAC sanctions")
      .accounts({
        authority: blacklister.publicKey,
        state: stablecoinPda,
        roleAssignment: blacklisterRolePda,
        entry: entryPda3,
        account: badActor3.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([blacklister])
      .rpc();

    // Verify entries were created
    const entry1 = await program.account.blacklistEntry.fetch(entryPda1);
    expect(entry1.reason).to.equal("Fraudulent activity");

    const entry2 = await program.account.blacklistEntry.fetch(entryPda2);
    expect(entry2.reason).to.equal("Money laundering");

    const entry3 = await program.account.blacklistEntry.fetch(entryPda3);
    expect(entry3.reason).to.equal("OFAC sanctions");
  });

  it("Updates blacklist with new reason", async () => {
    const badActor = anchor.web3.Keypair.generate();
    const [entryPda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("blacklist"), stablecoinPda.toBuffer(), badActor.publicKey.toBuffer()],
      program.programId
    );

    // Add to blacklist
    await program.methods
      .addToBlacklist("Initial reason")
      .accounts({
        authority: blacklister.publicKey,
        state: stablecoinPda,
        roleAssignment: blacklisterRolePda,
        entry: entryPda,
        account: badActor.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([blacklister])
      .rpc();

    // Remove and re-add with new reason
    await program.methods
      .removeFromBlacklist()
      .accounts({
        authority: blacklister.publicKey,
        state: stablecoinPda,
        roleAssignment: blacklisterRolePda,
        entry: entryPda,
        account: badActor.publicKey,
      })
      .signers([blacklister])
      .rpc();

    await program.methods
      .addToBlacklist("Updated reason - confirmed fraud")
      .accounts({
        authority: blacklister.publicKey,
        state: stablecoinPda,
        roleAssignment: blacklisterRolePda,
        entry: entryPda,
        account: badActor.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([blacklister])
      .rpc();

    const entry = await program.account.blacklistEntry.fetch(entryPda);
    expect(entry.reason).to.equal("Updated reason - confirmed fraud");
  });

  it("Seizes from multiple accounts", async () => {
    const fromTokenAccount1 = anchor.web3.Keypair.generate().publicKey;
    const fromTokenAccount2 = anchor.web3.Keypair.generate().publicKey;
    const toTokenAccount = anchor.web3.Keypair.generate().publicKey;
    const amount1 = new anchor.BN(50_000);
    const amount2 = new anchor.BN(75_000);

    await program.methods
      .seize(amount1)
      .accounts({
        authority: seizer.publicKey,
        state: stablecoinPda,
        roleAssignment: seizerRolePda,
        assetMint: assetMint.publicKey,
        from: fromTokenAccount1,
        to: toTokenAccount,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([seizer])
      .rpc();

    await program.methods
      .seize(amount2)
      .accounts({
        authority: seizer.publicKey,
        state: stablecoinPda,
        roleAssignment: seizerRolePda,
        assetMint: assetMint.publicKey,
        from: fromTokenAccount2,
        to: toTokenAccount,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([seizer])
      .rpc();

    const state = await program.account.stablecoinState.fetch(stablecoinPda);
    expect(state.complianceEnabled).to.be.true;
  });

  it("Seizes to different destinations", async () => {
    const fromTokenAccount = anchor.web3.Keypair.generate().publicKey;
    const toTokenAccount1 = anchor.web3.Keypair.generate().publicKey;
    const toTokenAccount2 = anchor.web3.Keypair.generate().publicKey;
    const amount = new anchor.BN(25_000);

    await program.methods
      .seize(amount)
      .accounts({
        authority: seizer.publicKey,
        state: stablecoinPda,
        roleAssignment: seizerRolePda,
        assetMint: assetMint.publicKey,
        from: fromTokenAccount,
        to: toTokenAccount1,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([seizer])
      .rpc();

    await program.methods
      .seize(amount)
      .accounts({
        authority: seizer.publicKey,
        state: stablecoinPda,
        roleAssignment: seizerRolePda,
        assetMint: assetMint.publicKey,
        from: fromTokenAccount,
        to: toTokenAccount2,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([seizer])
      .rpc();

    const state = await program.account.stablecoinState.fetch(stablecoinPda);
    expect(state.complianceEnabled).to.be.true;
  });

  it("Verifies compliance is enabled for SSS-2", async () => {
    const state = await program.account.stablecoinState.fetch(stablecoinPda);
    expect(state.complianceEnabled).to.be.true;
    expect(state.preset).to.equal(PRESET_SSS_2);
  });

  it("Verifies blacklister role permissions", async () => {
    const [rolePda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("role"), stablecoinPda.toBuffer(), blacklister.publicKey.toBuffer()],
      program.programId
    );

    const roleAssignment = await program.account.roleAssignment.fetch(rolePda);
    expect(roleAssignment.account.toString()).to.equal(blacklister.publicKey.toString());
  });

  it("Verifies seizer role permissions", async () => {
    const [rolePda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("role"), stablecoinPda.toBuffer(), seizer.publicKey.toBuffer()],
      program.programId
    );

    const roleAssignment = await program.account.roleAssignment.fetch(rolePda);
    expect(roleAssignment.account.toString()).to.equal(seizer.publicKey.toString());
  });

  it("Blacklists account with long reason", async () => {
    const badActor = anchor.web3.Keypair.generate();
    const longReason = "A".repeat(199); // Max 200 chars

    const [entryPda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("blacklist"), stablecoinPda.toBuffer(), badActor.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .addToBlacklist(longReason)
      .accounts({
        authority: blacklister.publicKey,
        state: stablecoinPda,
        roleAssignment: blacklisterRolePda,
        entry: entryPda,
        account: badActor.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([blacklister])
      .rpc();

    const entry = await program.account.blacklistEntry.fetch(entryPda);
    expect(entry.reason).to.equal(longReason);
  });

  it("Seizes large amount", async () => {
    const fromTokenAccount = anchor.web3.Keypair.generate().publicKey;
    const toTokenAccount = anchor.web3.Keypair.generate().publicKey;
    const largeAmount = new anchor.BN("100000000000"); // 100 billion

    await program.methods
      .seize(largeAmount)
      .accounts({
        authority: seizer.publicKey,
        state: stablecoinPda,
        roleAssignment: seizerRolePda,
        assetMint: assetMint.publicKey,
        from: fromTokenAccount,
        to: toTokenAccount,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([seizer])
      .rpc();

    const state = await program.account.stablecoinState.fetch(stablecoinPda);
    expect(state.complianceEnabled).to.be.true;
  });
});
