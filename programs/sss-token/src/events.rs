use anchor_lang::prelude::*;

#[event]
pub struct StablecoinInitialized {
    pub stablecoin: Pubkey,
    pub preset: u8,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub compliance_enabled: bool,
}

#[event]
pub struct Minted {
    pub stablecoin: Pubkey,
    pub recipient: Pubkey,
    pub amount: u64,
    pub minter: Pubkey,
}

#[event]
pub struct Burned {
    pub stablecoin: Pubkey,
    pub from: Pubkey,
    pub amount: u64,
}

#[event]
pub struct Frozen {
    pub stablecoin: Pubkey,
    pub account: Pubkey,
}

#[event]
pub struct Thawed {
    pub stablecoin: Pubkey,
    pub account: Pubkey,
}

#[event]
pub struct Paused {
    pub stablecoin: Pubkey,
    pub authority: Pubkey,
}

#[event]
pub struct Unpaused {
    pub stablecoin: Pubkey,
    pub authority: Pubkey,
}

#[event]
pub struct AuthorityTransferred {
    pub stablecoin: Pubkey,
    pub old_authority: Pubkey,
    pub new_authority: Pubkey,
}

#[event]
pub struct MinterAdded {
    pub stablecoin: Pubkey,
    pub minter: Pubkey,
    pub quota: u64,
}

#[event]
pub struct MinterRemoved {
    pub stablecoin: Pubkey,
    pub minter: Pubkey,
}

#[event]
pub struct QuotaUpdated {
    pub stablecoin: Pubkey,
    pub minter: Pubkey,
    pub old_quota: u64,
    pub new_quota: u64,
}

#[event]
pub struct BlacklistAdded {
    pub stablecoin: Pubkey,
    pub account: Pubkey,
    pub reason: String,
}

#[event]
pub struct BlacklistRemoved {
    pub stablecoin: Pubkey,
    pub account: Pubkey,
}

#[event]
pub struct Seized {
    pub stablecoin: Pubkey,
    pub from: Pubkey,
    pub to: Pubkey,
    pub amount: u64,
}

#[event]
pub struct RoleAssigned {
    pub stablecoin: Pubkey,
    pub role: String,
    pub account: Pubkey,
    pub assigned_by: Pubkey,
}

#[event]
pub struct RoleRevoked {
    pub stablecoin: Pubkey,
    pub role: String,
    pub account: Pubkey,
}
