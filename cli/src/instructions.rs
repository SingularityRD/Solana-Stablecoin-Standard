//! Instruction types for the SSS Token CLI
//! 
//! This module provides instruction argument structs
//! for interacting with the Anchor program.

use anchor_lang::prelude::*;
use ::borsh::{BorshSerialize, BorshDeserialize};
use solana_sdk::pubkey::Pubkey;

// ==================== ROLE ENUM ====================

#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum Role {
    Master = 0,
    Minter = 1,
    Burner = 2,
    Blacklister = 3,
    Pauser = 4,
    Seizer = 5,
}

impl Role {
    pub fn to_u8(&self) -> u8 {
        *self as u8
    }
    
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Role::Master),
            1 => Some(Role::Minter),
            2 => Some(Role::Burner),
            3 => Some(Role::Blacklister),
            4 => Some(Role::Pauser),
            5 => Some(Role::Seizer),
            _ => None,
        }
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Master => write!(f, "Master"),
            Role::Minter => write!(f, "Minter"),
            Role::Burner => write!(f, "Burner"),
            Role::Blacklister => write!(f, "Blacklister"),
            Role::Pauser => write!(f, "Pauser"),
            Role::Seizer => write!(f, "Seizer"),
        }
    }
}

// ==================== INSTRUCTION DISCRIMINANTS ====================
// These match the Anchor instruction signatures

/// Instruction discriminant prefix (8 bytes)
pub fn instruction_discriminant(name: &str) -> [u8; 8] {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    name.hash(&mut hasher);
    let hash = hasher.finish();
    let mut discriminant = [0u8; 8];
    discriminant.copy_from_slice(&hash.to_le_bytes()[..8]);
    discriminant
}

// ==================== INSTRUCTION ARGS ====================
// These are serialized with borsh for instruction data

/// Args for Initialize instruction
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct InitializeArgs {
    pub preset: u8,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub decimals: u8,
}

/// Args for Mint instruction
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct MintArgs {
    pub amount: u64,
}

/// Args for Burn instruction
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct BurnArgs {
    pub amount: u64,
}

/// Args for FreezeAccount instruction (empty)
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct FreezeArgs {}

/// Args for ThawAccount instruction (empty)
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct ThawArgs {}

/// Pause instruction marker (empty args)
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct Pause {}

/// Unpause instruction marker (empty args)
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct Unpause {}

/// Args for AddToBlacklist instruction
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct AddToBlacklist {
    pub reason: String,
}

/// RemoveFromBlacklist instruction marker (empty args)
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct RemoveFromBlacklist {}

/// Args for AssignRole instruction
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct AssignRoleArgs {
    pub role: u8,
}

/// Args for RevokeRole instruction (empty)
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct RevokeRoleArgs {}

/// Args for SetQuota instruction
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct SetQuotaArgs {
    pub quota: u64,
}

/// Args for AddMinter instruction
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct AddMinterArgs {
    pub quota: u64,
}

/// Args for RemoveMinter instruction (empty)
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct RemoveMinterArgs {}

/// Args for Seize instruction
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct SeizeArgs {
    pub amount: u64,
}

/// Args for TransferAuthority instruction
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct TransferAuthority {
    pub new_authority: Pubkey,
}

// ==================== HELPER FUNCTIONS ====================

/// Build instruction data with Anchor discriminant prefix
pub fn build_instruction_data<T: BorshSerialize>(instruction_name: &str, args: T) -> std::result::Result<Vec<u8>, std::io::Error> {
    let mut data = Vec::new();
    
    // Anchor uses first 8 bytes as instruction discriminant
    // The discriminant is computed as: sha256("global:<instruction_name>")[0..8]
    let discriminant = anchor_instruction_discriminant(instruction_name);
    data.extend_from_slice(&discriminant);
    
    // Serialize the args
    args.serialize(&mut data)?;
    
    Ok(data)
}

/// Compute Anchor instruction discriminant
fn anchor_instruction_discriminant(name: &str) -> [u8; 8] {
    use sha2::{Digest, Sha256};
    let preimage = format!("global:{}", name);
    let mut hasher = Sha256::new();
    hasher.update(preimage.as_bytes());
    let hash = hasher.finalize();
    let mut discriminant = [0u8; 8];
    discriminant.copy_from_slice(&hash[..8]);
    discriminant
}

// Re-export commonly used types
pub use solana_sdk::system_program;