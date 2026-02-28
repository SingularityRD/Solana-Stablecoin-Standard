import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SssToken } from "../target/types/sss_token";
import { expect } from "chai";

describe("Role Management", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SssToken as Program<SssToken>;
  const authority = provider.wallet;

  let stablecoinPda: anchor.web3.PublicKey;
  let minter: anchor.web3.Keypair;
  let burner: anchor.web3.Keypair;
  let pauser: anchor.web3.Keypair;

  before(async () => {
    const [pda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("stablecoin"), authority.publicKey.toBuffer()],
      program.programId
    );
    stablecoinPda = pda;
    minter = anchor.web3.Keypair.generate();
    burner = anchor.web3.Keypair.generate();
    pauser = anchor.web3.Keypair.generate();
  });

  it("Initializes stablecoin", async () => {
    await program.methods
      .initialize(1, "Test", "TST", "https://example.com", 6)
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
        assetMint: anchor.web3.PublicKey.default,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
  });

  it("Assigns minter role", async () => {
    const role = { minter: {} };
    const [assignmentPda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("role"), stablecoinPda.toBuffer(), minter.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .assignRole(role)
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
        assignment: assignmentPda,
        account: minter.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const assignment = await program.account.roleAssignment.fetch(assignmentPda);
    expect(assignment.account.toString()).to.equal(minter.publicKey.toString());
  });

  it("Assigns burner role", async () => {
    const role = { burner: {} };
    const [assignmentPda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("role"), stablecoinPda.toBuffer(), burner.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .assignRole(role)
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
        assignment: assignmentPda,
        account: burner.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    expect(true).to.be.true;
  });

  it("Assigns pauser role", async () => {
    const role = { pauser: {} };
    const [assignmentPda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("role"), stablecoinPda.toBuffer(), pauser.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .assignRole(role)
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
        assignment: assignmentPda,
        account: pauser.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    expect(true).to.be.true;
  });

  it("Transfers authority", async () => {
    const newAuthority = anchor.web3.Keypair.generate();

    await program.methods
      .transferAuthority(newAuthority.publicKey)
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
      })
      .rpc();

    const state = await program.account.stablecoinState.fetch(stablecoinPda);
    expect(state.authority.toString()).to.equal(newAuthority.publicKey.toString());

    // Transfer back
    await program.methods
      .transferAuthority(authority.publicKey)
      .accounts({
        authority: newAuthority.publicKey,
        state: stablecoinPda,
      })
      .signers([newAuthority])
      .rpc();
  });

  it("Revokes minter role", async () => {
    const [assignmentPda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("role"), stablecoinPda.toBuffer(), minter.publicKey.toBuffer()],
      program.programId
    );

    // Verify role exists
    const assignmentBefore = await program.account.roleAssignment.fetch(assignmentPda);
    expect(assignmentBefore.account.toString()).to.equal(minter.publicKey.toString());

    // Revoke the role
    await program.methods
      .revokeRole()
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
        assignment: assignmentPda,
      })
      .rpc();

    // Verify role was revoked (account should be closed)
    try {
      await program.account.roleAssignment.fetch(assignmentPda);
      expect.fail("RoleAssignment should have been closed");
    } catch (e: any) {
      expect(e.message).to.include("Account does not exist");
    }
  });

  it("Revokes burner role", async () => {
    const [assignmentPda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("role"), stablecoinPda.toBuffer(), burner.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .revokeRole()
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
        assignment: assignmentPda,
      })
      .rpc();

    try {
      await program.account.roleAssignment.fetch(assignmentPda);
      expect.fail("RoleAssignment should have been closed");
    } catch (e: any) {
      expect(e.message).to.include("Account does not exist");
    }
  });

  it("Revokes pauser role", async () => {
    const [assignmentPda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("role"), stablecoinPda.toBuffer(), pauser.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .revokeRole()
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
        assignment: assignmentPda,
      })
      .rpc();

    try {
      await program.account.roleAssignment.fetch(assignmentPda);
      expect.fail("RoleAssignment should have been closed");
    } catch (e: any) {
      expect(e.message).to.include("Account does not exist");
    }
  });

  it("Fails to revoke role with unauthorized account", async () => {
    const unauthorized = anchor.web3.Keypair.generate();
    const newUser = anchor.web3.Keypair.generate();
    const role = { minter: {} };

    // First assign a role
    const [assignmentPda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("role"), stablecoinPda.toBuffer(), newUser.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .assignRole(role)
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
        assignment: assignmentPda,
        account: newUser.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    // Try to revoke with unauthorized account
    try {
      await program.methods
        .revokeRole()
        .accounts({
          authority: unauthorized.publicKey,
          state: stablecoinPda,
          assignment: assignmentPda,
        })
        .signers([unauthorized])
        .rpc();
      expect.fail("Should have thrown Unauthorized error");
    } catch (e: any) {
      expect(e.error?.errorCode?.code).to.equal("Unauthorized");
    }
  });

  it("Assigns blacklister role", async () => {
    const blacklister = anchor.web3.Keypair.generate();
    const role = { blacklister: {} };
    const [assignmentPda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("role"), stablecoinPda.toBuffer(), blacklister.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .assignRole(role)
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
        assignment: assignmentPda,
        account: blacklister.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const assignment = await program.account.roleAssignment.fetch(assignmentPda);
    expect(assignment.account.toString()).to.equal(blacklister.publicKey.toString());
  });

  it("Assigns seizer role", async () => {
    const seizer = anchor.web3.Keypair.generate();
    const role = { seizer: {} };
    const [assignmentPda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("role"), stablecoinPda.toBuffer(), seizer.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .assignRole(role)
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
        assignment: assignmentPda,
        account: seizer.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const assignment = await program.account.roleAssignment.fetch(assignmentPda);
    expect(assignment.account.toString()).to.equal(seizer.publicKey.toString());
  });

  it("Assigns master role", async () => {
    const master = anchor.web3.Keypair.generate();
    const role = { master: {} };
    const [assignmentPda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("role"), stablecoinPda.toBuffer(), master.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .assignRole(role)
      .accounts({
        authority: authority.publicKey,
        state: stablecoinPda,
        assignment: assignmentPda,
        account: master.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const assignment = await program.account.roleAssignment.fetch(assignmentPda);
    expect(assignment.account.toString()).to.equal(master.publicKey.toString());
  });
});
