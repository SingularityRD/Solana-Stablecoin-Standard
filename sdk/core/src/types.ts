import { PublicKey } from '@solana/web3.js';
import { BN } from '@coral-xyz/anchor';

/**
 * On-chain StablecoinState account structure (matches Rust struct).
 */
export interface StablecoinState {
  authority: PublicKey;
  assetMint: PublicKey;
  totalSupply: BN;
  paused: boolean;
  preset: number;
  complianceEnabled: boolean;
  bump: number;
}

/**
 * On-chain MinterInfo account structure (matches Rust struct).
 */
export interface MinterInfoAccount {
  minter: PublicKey;
  quota: BN;
  mintedAmount: BN;
  bump: number;
}

/**
 * On-chain RoleAssignment account structure (matches Rust struct).
 */
export interface RoleAssignmentAccount {
  role: AnchorRole;
  account: PublicKey;
  assignedBy: PublicKey;
  assignedAt: BN;
  bump: number;
}

/**
 * On-chain BlacklistEntry account structure (matches Rust struct).
 */
export interface BlacklistEntryAccount {
  account: PublicKey;
  reason: string;
  blacklistedBy: PublicKey;
  blacklistedAt: BN;
  bump: number;
}

/**
 * Role enum that matches the Anchor program's Role enum.
 * Used for serialization/deserialization with the program.
 */
export enum Role {
  Master = 'Master',
  Minter = 'Minter',
  Burner = 'Burner',
  Blacklister = 'Blacklister',
  Pauser = 'Pauser',
  Seizer = 'Seizer',
}

/**
 * Anchor-compatible role object type for instruction arguments.
 * Each role is represented as an object with the lowercase role name as key.
 */
export type AnchorRole = 
  | { master: object }
  | { minter: object }
  | { burner: object }
  | { blacklister: object }
  | { pauser: object }
  | { seizer: object };

/**
 * Convert Role enum to Anchor-compatible role object.
 */
export function toAnchorRole(role: Role): AnchorRole {
  switch (role) {
    case Role.Master:
      return { master: {} };
    case Role.Minter:
      return { minter: {} };
    case Role.Burner:
      return { burner: {} };
    case Role.Blacklister:
      return { blacklister: {} };
    case Role.Pauser:
      return { pauser: {} };
    case Role.Seizer:
      return { seizer: {} };
  }
}

/**
 * SDK-friendly MinterInfo with converted types.
 */
export interface MinterInfo {
  minter: PublicKey;
  quota: number;
  mintedAmount: number;
  bump: number;
}

/**
 * SDK-friendly BlacklistEntry with converted types.
 */
export interface BlacklistEntry {
  account: PublicKey;
  reason: string;
  blacklistedBy: PublicKey;
  blacklistedAt: number;
  bump: number;
}

/**
 * SDK-friendly RoleAssignment with converted types.
 */
export interface RoleAssignment {
  role: Role;
  account: PublicKey;
  assignedBy: PublicKey;
  assignedAt: number;
  bump: number;
}
