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
});
