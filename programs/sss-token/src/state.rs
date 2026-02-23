use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct StablecoinConfig {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub decimals: u8,
    pub enable_permanent_delegate: bool,
    pub enable_transfer_hook: bool,
    pub default_account_frozen: bool,
}

#[account]
#[derive(InitSpace)]
pub struct StablecoinState {
    pub authority: Pubkey,
    pub asset_mint: Pubkey,
    pub total_supply: u64,
    pub paused: bool,
    pub preset: u8,
    pub compliance_enabled: bool,
    pub bump: u8,
    #[max_len(64)]
    pub _reserved: [u8; 64],
}

#[account]
#[derive(InitSpace)]
pub struct MinterInfo {
    pub minter: Pubkey,
    pub quota: u64,
    pub minted_amount: u64,
    pub bump: u8,
    #[max_len(32)]
    pub _reserved: [u8; 32],
}

#[account]
#[derive(InitSpace)]
pub struct RoleAssignment {
    pub role: Role,
    pub account: Pubkey,
    pub assigned_by: Pubkey,
    pub assigned_at: i64,
    pub bump: u8,
    #[max_len(32)]
    pub _reserved: [u8; 32],
}

#[account]
#[derive(InitSpace)]
pub struct BlacklistEntry {
    pub account: Pubkey,
    #[max_len(200)]
    pub reason: String,
    pub blacklisted_by: Pubkey,
    pub blacklisted_at: i64,
    pub bump: u8,
    #[max_len(32)]
    pub _reserved: [u8; 32],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum Role {
    Master,
    Minter,
    Burner,
    Blacklister,
    Pauser,
    Seizer,
}
