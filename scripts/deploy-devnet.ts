import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SssToken } from "../target/types/sss_token";
import { randomBytes } from "crypto";

async function main() {
  console.log("=== SSS Token Devnet Deployment ===\n");

  // Setup provider
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SssToken as Program<SssToken>;
  const authority = provider.wallet;

  console.log("Program ID:", program.programId.toString());
  console.log("Authority:", authority.publicKey.toString());
  console.log("Network:", provider.connection.rpcEndpoint);
  console.log();

  // Generate random stablecoin config
  const config = {
    name: "Devnet Stablecoin",
    symbol: "DUSD",
    uri: "https://example.com/metadata.json",
    decimals: 6,
    preset: 2, // SSS-2
  };

  console.log("Deploying SSS-2 Stablecoin...");
  console.log("Name:", config.name);
  console.log("Symbol:", config.symbol);
  console.log("Preset: SSS-2 (Compliant)");
  console.log();

  // Find PDA
  const [stablecoinPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("stablecoin"), authority.publicKey.toBuffer()],
    program.programId
  );

  console.log("Stablecoin PDA:", stablecoinPda.toString());
  console.log();

  // Initialize
  console.log("Initializing...");
  const initTx = await program.methods
    .initialize(
      config.preset,
      config.name,
      config.symbol,
      config.uri,
      config.decimals
    )
    .accounts({
      authority: authority.publicKey,
      state: stablecoinPda,
      assetMint: anchor.web3.PublicKey.default,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .rpc();

  console.log("✅ Initialization TX:", initTx);
  console.log();

  // Verify deployment
  console.log("Verifying deployment...");
  const state = await program.account.stablecoinState.fetch(stablecoinPda);
  
  console.log("State Account:");
  console.log("  - Preset:", state.preset);
  console.log("  - Compliance Enabled:", state.complianceEnabled);
  console.log("  - Paused:", state.paused);
  console.log("  - Total Supply:", state.totalSupply.toString());
  console.log("  - Authority:", state.authority.toString());
  console.log();

  // Example operation: Mint
  console.log("Testing mint operation...");
  const recipient = anchor.web3.Keypair.generate();
  const mintAmount = new anchor.BN(1_000_000);

  const mintTx = await program.methods
    .mint(recipient.publicKey, mintAmount)
    .accounts({
      authority: authority.publicKey,
      state: stablecoinPda,
    })
    .rpc();

  console.log("✅ Mint TX:", mintTx);
  console.log("  - Recipient:", recipient.publicKey.toString());
  console.log("  - Amount:", mintAmount.toString());
  console.log();

  // Example operation: Blacklist (SSS-2)
  console.log("Testing blacklist operation (SSS-2)...");
  const badActor = anchor.web3.Keypair.generate();
  const [blacklistPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("blacklist"), stablecoinPda.toBuffer(), badActor.publicKey.toBuffer()],
    program.programId
  );

  const blacklistTx = await program.methods
    .addToBlacklist("Devnet test - OFAC match")
    .accounts({
      authority: authority.publicKey,
      state: stablecoinPda,
      entry: blacklistPda,
      account: badActor.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .rpc();

  console.log("✅ Blacklist TX:", blacklistTx);
  console.log("  - Account:", badActor.publicKey.toString());
  console.log();

  // Summary
  console.log("=== Deployment Summary ===");
  console.log("Program ID:", program.programId.toString());
  console.log("Stablecoin PDA:", stablecoinPda.toString());
  console.log("Initialization TX:", initTx);
  console.log("Test Mint TX:", mintTx);
  console.log("Test Blacklist TX:", blacklistTx);
  console.log();
  console.log("✅ Devnet deployment complete!");
  console.log();
  console.log("View on Solana Explorer:");
  console.log("  https://explorer.solana.com/address/" + stablecoinPda.toString() + "?cluster=devnet");
}

main().catch((err) => {
  console.error("Deployment failed:", err);
  process.exit(1);
});
