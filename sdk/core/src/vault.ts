import { Connection, PublicKey, Signer, SystemProgram } from '@solana/web3.js';
import { Program, AnchorProvider, BN } from '@coral-xyz/anchor';
import { TOKEN_2022_PROGRAM_ID } from '@solana/spl-token';

export enum Presets {
  SSS_1 = 1,
  SSS_2 = 2,
}

export interface StablecoinConfig {
  name: string;
  symbol: string;
  uri: string;
  decimals: number;
  preset?: Presets;
}

export interface StablecoinAccount {
  authority: PublicKey;
  assetMint: PublicKey;
  totalSupply: BN;
  paused: boolean;
  preset: number;
  complianceEnabled: boolean;
  bump: number;
}

export class SolanaStablecoin {
  connection: Connection;
  program: Program;
  provider: AnchorProvider;
  stablecoinPda: PublicKey;
  assetMint: PublicKey;
  config: StablecoinConfig;

  constructor(
    connection: Connection, 
    program: Program, 
    provider: AnchorProvider, 
    stablecoinPda: PublicKey, 
    assetMint: PublicKey,
    config: StablecoinConfig
  ) {
    this.connection = connection;
    this.program = program;
    this.provider = provider;
    this.stablecoinPda = stablecoinPda;
    this.assetMint = assetMint;
    this.config = config;
  }

  static async create(
    connection: Connection, 
    config: StablecoinConfig & { authority: Signer; assetMint: PublicKey },
    program: Program
  ): Promise<SolanaStablecoin> {
    const provider = program.provider as AnchorProvider;
    const [stablecoinPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('stablecoin'), config.assetMint.toBuffer()],
      program.programId
    );

    // Call initialize instruction
    await program.methods
      .initialize(
        config.preset || Presets.SSS_1,
        config.name,
        config.symbol,
        config.uri,
        config.decimals
      )
      .accounts({
        authority: config.authority.publicKey,
        state: stablecoinPda,
        assetMint: config.assetMint,
        systemProgram: SystemProgram.programId,
      })
      .signers([config.authority])
      .rpc();

    return new SolanaStablecoin(connection, program, provider, stablecoinPda, config.assetMint, config);
  }

  async mint(authority: Signer, recipient: PublicKey, amount: number): Promise<string> {
    return this.program.methods
      .mint(new BN(amount))
      .accounts({
        authority: authority.publicKey,
        state: this.stablecoinPda,
        assetMint: this.assetMint,
        recipient,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([authority])
      .rpc();
  }

  async burn(authority: Signer, from: PublicKey, amount: number): Promise<string> {
    return this.program.methods
      .burn(new BN(amount))
      .accounts({
        authority: authority.publicKey,
        state: this.stablecoinPda,
        assetMint: this.assetMint,
        from,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([authority])
      .rpc();
  }

  async freeze(authority: Signer, account: PublicKey): Promise<string> {
    return this.program.methods
      .freezeAccount()
      .accounts({
        authority: authority.publicKey,
        state: this.stablecoinPda,
        assetMint: this.assetMint,
        account,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([authority])
      .rpc();
  }

  async thaw(authority: Signer, account: PublicKey): Promise<string> {
    return this.program.methods
      .thawAccount()
      .accounts({
        authority: authority.publicKey,
        state: this.stablecoinPda,
        assetMint: this.assetMint,
        account,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([authority])
      .rpc();
  }

  async getTotalSupply(): Promise<number> {
    const account = this.program.account as unknown as { stablecoinState: { fetch: (pda: PublicKey) => Promise<StablecoinAccount> } };
    const state = await account.stablecoinState.fetch(this.stablecoinPda);
    return state.totalSupply.toNumber();
  }

  async getStatus(): Promise<{ paused: boolean; preset: number; complianceEnabled: boolean }> {
    const account = this.program.account as unknown as { stablecoinState: { fetch: (pda: PublicKey) => Promise<StablecoinAccount> } };
    const state = await account.stablecoinState.fetch(this.stablecoinPda);
    return { paused: state.paused, preset: state.preset, complianceEnabled: state.complianceEnabled };
  }

  get compliance(): ComplianceModule {
    return new ComplianceModule(this);
  }
}

export class ComplianceModule {
  private stablecoin: SolanaStablecoin;
  constructor(stablecoin: SolanaStablecoin) {
    this.stablecoin = stablecoin;
  }

  async blacklistAdd(authority: Signer, account: PublicKey, reason: string): Promise<string> {
    const [entryPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('blacklist'), this.stablecoin.stablecoinPda.toBuffer(), account.toBuffer()],
      this.stablecoin.program.programId
    );

    return this.stablecoin.program.methods
      .addToBlacklist(reason)
      .accounts({
        authority: authority.publicKey,
        state: this.stablecoin.stablecoinPda,
        entry: entryPda,
        account,
        systemProgram: SystemProgram.programId,
      })
      .signers([authority])
      .rpc();
  }

  async blacklistRemove(authority: Signer, account: PublicKey): Promise<string> {
    const [entryPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('blacklist'), this.stablecoin.stablecoinPda.toBuffer(), account.toBuffer()],
      this.stablecoin.program.programId
    );

    return this.stablecoin.program.methods
      .removeFromBlacklist()
      .accounts({
        authority: authority.publicKey,
        state: this.stablecoin.stablecoinPda,
        entry: entryPda,
        account,
        systemProgram: SystemProgram.programId, // Note: Close instructions sometimes need system_program in older anchor, but usually to_account_info() works
      })
      .signers([authority])
      .rpc();
  }

  async seize(authority: Signer, from: PublicKey, to: PublicKey, amount: number): Promise<string> {
    return this.stablecoin.program.methods
      .seize(new BN(amount))
      .accounts({
        authority: authority.publicKey,
        state: this.stablecoin.stablecoinPda,
        assetMint: this.stablecoin.assetMint,
        from,
        to,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([authority])
      .rpc();
  }
}
