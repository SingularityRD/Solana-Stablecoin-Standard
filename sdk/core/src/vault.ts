import { Connection, PublicKey, Signer, SystemProgram } from '@solana/web3.js';
import { Program, AnchorProvider, BN } from '@coral-xyz/anchor';
import { TOKEN_2022_PROGRAM_ID } from '@solana/spl-token';
import { Role, MinterInfoAccount } from './types';

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

/**
 * Account type returned from fetching stablecoin state.
 */
export interface StablecoinAccount {
  authority: PublicKey;
  assetMint: PublicKey;
  totalSupply: BN;
  paused: boolean;
  preset: number;
  complianceEnabled: boolean;
  bump: number;
}

/**
 * SolanaStablecoin SDK class for interacting with the SSS Token program.
 * Provides methods for all stablecoin operations including mint, burn, freeze,
 * thaw, pause, unpause, seize, and authority management.
 */
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

  /**
   * Assign a role to a target account.
   * @param authority - The authority signer (must be Master role)
   * @param targetAccount - The account to assign the role to
   * @param role - The role to assign (Master, Minter, Burner, Blacklister, Pauser, Seizer)
   */
  async assignRole(authority: Signer, targetAccount: PublicKey, role: Role): Promise<string> {
    const [assignmentPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('role'), this.stablecoinPda.toBuffer(), targetAccount.toBuffer()],
      this.program.programId
    );

    const anchorRole = { [role.toLowerCase()]: {} };

    return this.program.methods
      .assignRole(anchorRole)
      .accounts({
        authority: authority.publicKey,
        state: this.stablecoinPda,
        assignment: assignmentPda,
        account: targetAccount,
        systemProgram: SystemProgram.programId,
      })
      .signers([authority])
      .rpc();
  }

  /**
   * Mint new stablecoin tokens to a recipient.
   * @param authority - The minter authority signer
   * @param recipient - The recipient token account
   * @param amount - Amount to mint (in smallest units)
   * @param roleAssignment - Optional role assignment PDA for verification
   */
  async mint(authority: Signer, recipient: PublicKey, amount: number, roleAssignment?: PublicKey): Promise<string> {
    // Using inline object for Anchor compatibility
    const accounts = {
      authority: authority.publicKey,
      state: this.stablecoinPda,
      assetMint: this.assetMint,
      recipient,
      tokenProgram: TOKEN_2022_PROGRAM_ID,
      ...(roleAssignment && { roleAssignment }),
    };

    return this.program.methods
      .mint(new BN(amount))
      .accounts(accounts)
      .signers([authority])
      .rpc();
  }

  /**
   * Burn stablecoin tokens from an account.
   * @param authority - The burner authority signer
   * @param from - The token account to burn from
   * @param amount - Amount to burn (in smallest units)
   */
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

  /**
   * Freeze a token account.
   * @param authority - The blacklister/pauser authority signer
   * @param account - The token account to freeze
   * @param roleAssignment - Optional role assignment PDA for verification
   */
  async freeze(authority: Signer, account: PublicKey, roleAssignment?: PublicKey): Promise<string> {
    const accounts = {
      authority: authority.publicKey,
      state: this.stablecoinPda,
      assetMint: this.assetMint,
      account,
      tokenProgram: TOKEN_2022_PROGRAM_ID,
      ...(roleAssignment && { roleAssignment }),
    };

    return this.program.methods
      .freezeAccount()
      .accounts(accounts)
      .signers([authority])
      .rpc();
  }

  /**
   * Thaw a frozen token account.
   * @param authority - The authority signer (must be Master or have thaw permissions)
   * @param account - The token account to thaw
   */
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

  /**
   * Seize tokens from one account to another (for compliance/enforcement).
   * @param authority - The seizer authority signer
   * @param from - The token account to seize from
   * @param to - The token account to transfer seized tokens to
   * @param amount - Amount to seize (in smallest units)
   * @param roleAssignment - Optional role assignment PDA for verification
   */
  async seize(authority: Signer, from: PublicKey, to: PublicKey, amount: number, roleAssignment?: PublicKey): Promise<string> {
    const accounts = {
      authority: authority.publicKey,
      state: this.stablecoinPda,
      assetMint: this.assetMint,
      from,
      to,
      tokenProgram: TOKEN_2022_PROGRAM_ID,
      ...(roleAssignment && { roleAssignment }),
    };

    return this.program.methods
      .seize(new BN(amount))
      .accounts(accounts)
      .signers([authority])
      .rpc();
  }

  /**
   * Pause all stablecoin operations.
   * @param authority - The master authority signer
   */
  async pause(authority: Signer): Promise<string> {
    return this.program.methods
      .pause()
      .accounts({
        authority: authority.publicKey,
        state: this.stablecoinPda,
      })
      .signers([authority])
      .rpc();
  }

  /**
   * Unpause stablecoin operations.
   * @param authority - The master authority signer
   */
  async unpause(authority: Signer): Promise<string> {
    return this.program.methods
      .unpause()
      .accounts({
        authority: authority.publicKey,
        state: this.stablecoinPda,
      })
      .signers([authority])
      .rpc();
  }

  /**
   * Transfer the master authority to a new account.
   * @param authority - The current master authority signer
   * @param newAuthority - The new authority public key
   */
  async transferAuthority(authority: Signer, newAuthority: PublicKey): Promise<string> {
    return this.program.methods
      .transferAuthority(newAuthority)
      .accounts({
        authority: authority.publicKey,
        state: this.stablecoinPda,
      })
      .signers([authority])
      .rpc();
  }

  /**
   * Add a minter with a specified quota.
   * @param authority - The authority signer (must be Master)
   * @param minter - The public key of the minter to add
   * @param quota - The minting quota for this minter (in smallest units)
   */
  async addMinter(authority: Signer, minter: PublicKey, quota: number): Promise<string> {
    const [minterInfoPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('minter'), this.stablecoinPda.toBuffer(), minter.toBuffer()],
      this.program.programId
    );

    return this.program.methods
      .addMinter(new BN(quota))
      .accounts({
        authority: authority.publicKey,
        state: this.stablecoinPda,
        minterInfo: minterInfoPda,
        minter,
        systemProgram: SystemProgram.programId,
      })
      .signers([authority])
      .rpc();
  }

  /**
   * Remove a minter from the minter list.
   * @param authority - The authority signer (must be Master)
   * @param minter - The public key of the minter to remove
   */
  async removeMinter(authority: Signer, minter: PublicKey): Promise<string> {
    const [minterInfoPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('minter'), this.stablecoinPda.toBuffer(), minter.toBuffer()],
      this.program.programId
    );

    return this.program.methods
      .removeMinter()
      .accounts({
        authority: authority.publicKey,
        state: this.stablecoinPda,
        minterInfo: minterInfoPda,
      })
      .signers([authority])
      .rpc();
  }

  /**
   * Update the quota for an existing minter.
   * @param authority - The authority signer (must be Master)
   * @param minter - The public key of the minter to update
   * @param newQuota - The new minting quota (in smallest units)
   */
  async setQuota(authority: Signer, minter: PublicKey, newQuota: number): Promise<string> {
    const [minterInfoPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('minter'), this.stablecoinPda.toBuffer(), minter.toBuffer()],
      this.program.programId
    );

    return this.program.methods
      .updateQuota(new BN(newQuota))
      .accounts({
        authority: authority.publicKey,
        state: this.stablecoinPda,
        minterInfo: minterInfoPda,
      })
      .signers([authority])
      .rpc();
  }

  /**
   * Get minter info for a specific minter.
   * @param minter - The public key of the minter
   */
  async getMinterInfo(minter: PublicKey): Promise<MinterInfoAccount | null> {
    const [minterInfoPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('minter'), this.stablecoinPda.toBuffer(), minter.toBuffer()],
      this.program.programId
    );

    try {
      const accountFetcher = this.program.account as Record<string, { fetch: (pubkey: PublicKey) => Promise<MinterInfoAccount> }>;
      return await accountFetcher['minterInfo'].fetch(minterInfoPda);
    } catch {
      return null;
    }
  }

  /**
   * Get all minters for this stablecoin.
   */
  async getAllMinters(): Promise<{ publicKey: PublicKey; account: MinterInfoAccount }[]> {
    const accountFetcher = this.program.account as Record<string, { 
      all(filters?: { memcmp?: { offset: number; bytes: string } }[]): Promise<{ publicKey: PublicKey; account: MinterInfoAccount }[]> 
    }>;
    
    return accountFetcher['minterInfo'].all([
      {
        memcmp: {
          offset: 8, // Skip discriminator
          bytes: this.stablecoinPda.toBase58(),
        },
      },
    ]);
  }

  /**
   * Revoke a role from a target account.
   * @param authority - The authority signer (must be Master)
   * @param targetAccount - The account to revoke the role from
   */
  async revokeRole(authority: Signer, targetAccount: PublicKey): Promise<string> {
    const [assignmentPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('role'), this.stablecoinPda.toBuffer(), targetAccount.toBuffer()],
      this.program.programId
    );

    return this.program.methods
      .revokeRole()
      .accounts({
        authority: authority.publicKey,
        state: this.stablecoinPda,
        assignment: assignmentPda,
        account: targetAccount,
      })
      .signers([authority])
      .rpc();
  }

  /**
   * Fetch the on-chain stablecoin state.
   */
  private async fetchState(): Promise<StablecoinAccount> {
    const accountFetcher = this.program.account as Record<string, { fetch: (pubkey: PublicKey) => Promise<StablecoinAccount> }>;
    return accountFetcher['stablecoinState'].fetch(this.stablecoinPda);
  }

  /**
   * Get the total supply of the stablecoin.
   */
  async getTotalSupply(): Promise<number> {
    const state = await this.fetchState();
    return state.totalSupply.toNumber();
  }

  /**
   * Get the current status of the stablecoin (paused, preset, compliance).
   */
  async getStatus(): Promise<{ paused: boolean; preset: number; complianceEnabled: boolean }> {
    const state = await this.fetchState();
    return { paused: state.paused, preset: state.preset, complianceEnabled: state.complianceEnabled };
  }

  /**
   * Get the full state of the stablecoin.
   */
  async getState(): Promise<StablecoinAccount> {
    return this.fetchState();
  }

  /**
   * Access the compliance module for blacklist operations.
   */
  get compliance(): ComplianceModule {
    return new ComplianceModule(this);
  }
}

/**
 * Compliance module for blacklist operations.
 */
export class ComplianceModule {
  private stablecoin: SolanaStablecoin;
  
  constructor(stablecoin: SolanaStablecoin) {
    this.stablecoin = stablecoin;
  }

  /**
   * Add an account to the blacklist.
   * @param authority - The blacklister authority signer
   * @param account - The account to blacklist
   * @param reason - Reason for blacklisting
   */
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

  /**
   * Remove an account from the blacklist.
   * @param authority - The blacklister authority signer
   * @param account - The account to remove from blacklist
   */
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
      })
      .signers([authority])
      .rpc();
  }
}