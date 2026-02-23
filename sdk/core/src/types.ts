import { PublicKey } from '@solana/web3.js';

export interface StablecoinState {
  authority: PublicKey;
  assetMint: PublicKey;
  totalSupply: number;
  paused: boolean;
  preset: number;
  complianceEnabled: boolean;
  bump: number;
}

export interface MinterInfo {
  minter: PublicKey;
  quota: number;
  mintedAmount: number;
  bump: number;
}

export enum Role {
  Master = 'Master',
  Minter = 'Minter',
  Burner = 'Burner',
  Blacklister = 'Blacklister',
  Pauser = 'Pauser',
  Seizer = 'Seizer',
}

export interface BlacklistEntry {
  account: PublicKey;
  reason: string;
  blacklistedBy: PublicKey;
  blacklistedAt: number;
  bump: number;
}
