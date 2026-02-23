// SSS-3: Scoped Allowlist Extension (Proof-of-Concept)

use anchor_lang::prelude::*;

#[account]
pub struct AllowlistEntry {
    pub account: Pubkey,
    pub approved: bool,
    pub approved_by: Pubkey,
    pub approved_at: i64,
    pub scope: AllowlistScope,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum AllowlistScope {
    TransferIn,
    TransferOut,
    Both,
    Mint,
    Burn,
}

impl Default for AllowlistEntry {
    fn default() -> Self {
        Self {
            account: Pubkey::default(),
            approved: false,
            approved_by: Pubkey::default(),
            approved_at: 0,
            scope: AllowlistScope::Both,
            bump: 0,
        }
    }
}

/// Check if account is allowlisted
pub fn is_allowlisted(entry: &AllowlistEntry) -> bool {
    entry.approved
}

/// Check if transfer is allowed
pub fn can_transfer(entry: &AllowlistEntry, direction: TransferDirection) -> bool {
    if !entry.approved {
        return false;
    }
    
    match (entry.scope, direction) {
        (AllowlistScope::Both, _) => true,
        (AllowlistScope::TransferIn, TransferDirection::In) => true,
        (AllowlistScope::TransferOut, TransferDirection::Out) => true,
        _ => false,
    }
}

#[derive(Clone, Copy)]
pub enum TransferDirection {
    In,
    Out,
}
