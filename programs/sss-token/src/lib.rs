use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod events;
pub mod math;
pub mod state;

// Instruction modules - placed at crate root for Anchor compatibility
pub mod admin;
pub mod blacklist;
pub mod burn;
pub mod freeze;
pub mod initialize;
pub mod minter_management;
pub mod mint;
pub mod role_management;
pub mod seize;
pub mod thaw;
pub mod transfer_hook;

// Extensions
pub mod extensions;

// Re-export all instruction structs to crate root for Anchor client code generation
pub use admin::*;
pub use blacklist::*;
pub use burn::*;
pub use freeze::*;
pub use initialize::*;
pub use minter_management::*;
pub use mint::*;
pub use role_management::*;
pub use seize::*;
pub use thaw::*;
pub use transfer_hook::*;
pub use state::Role;

declare_id!("SSSToken11111111111111111111111111111111111");

#[program]
pub mod sss_token {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        preset: u8,
        name: String,
        symbol: String,
        uri: String,
        decimals: u8,
    ) -> Result<()> {
        initialize::handler(ctx, preset, name, symbol, uri, decimals)
    }

    pub fn mint(ctx: Context<Mint>, amount: u64) -> Result<()> {
        mint::handler(ctx, amount)
    }

    pub fn burn(ctx: Context<Burn>, amount: u64) -> Result<()> {
        burn::handler(ctx, amount)
    }

    pub fn freeze_account(ctx: Context<FreezeAccount>) -> Result<()> {
        freeze::handler(ctx)
    }

    pub fn thaw_account(ctx: Context<ThawAccount>) -> Result<()> {
        thaw::handler(ctx)
    }

    pub fn pause(ctx: Context<Admin>) -> Result<()> {
        admin::pause(ctx)
    }

    pub fn unpause(ctx: Context<Admin>) -> Result<()> {
        admin::unpause(ctx)
    }

    pub fn transfer_authority(ctx: Context<Admin>, new_authority: Pubkey) -> Result<()> {
        admin::transfer_authority(ctx, new_authority)
    }

    pub fn add_to_blacklist(ctx: Context<Blacklist>, reason: String) -> Result<()> {
        blacklist::add(ctx, reason)
    }

    pub fn remove_from_blacklist(ctx: Context<Blacklist>) -> Result<()> {
        blacklist::remove(ctx)
    }

    pub fn seize(ctx: Context<Seize>, amount: u64) -> Result<()> {
        seize::handler(ctx, amount)
    }

    pub fn assign_role(ctx: Context<AssignRole>, role: Role) -> Result<()> {
        role_management::handler(ctx, role)
    }

    pub fn revoke_role(ctx: Context<RevokeRole>) -> Result<()> {
        role_management::revoke_handler(ctx)
    }

    pub fn add_minter(ctx: Context<AddMinter>, quota: u64) -> Result<()> {
        minter_management::add_minter_handler(ctx, quota)
    }

    pub fn remove_minter(ctx: Context<RemoveMinter>) -> Result<()> {
        minter_management::remove_minter_handler(ctx)
    }

    pub fn update_quota(ctx: Context<UpdateQuota>, new_quota: u64) -> Result<()> {
        minter_management::update_quota_handler(ctx, new_quota)
    }

    // Transfer hook - exposed as standard instruction for testing.
    // In production, Token-2022 invokes via the SPL Transfer Hook interface
    // which uses a different discriminator. The fallback() function below
    // routes those calls to this handler.
    pub fn execute_transfer_hook(ctx: Context<TransferHook>, amount: u64) -> Result<()> {
        transfer_hook::enforce_transfer(ctx, amount)
    }

    /// Fallback handler for SPL Transfer Hook interface.
    /// Token-2022 uses the SPL discriminator (not Anchor's) when invoking
    /// transfer hooks during transfers. This catches those calls.
    pub fn fallback<'info>(
        program_id: &Pubkey,
        accounts: &'info [AccountInfo<'info>],
        data: &[u8],
    ) -> Result<()> {
        use anchor_lang::solana_program::program_error::ProgramError;
        // SPL Transfer Hook Execute discriminator:
        // SHA-256("spl-transfer-hook-interface:execute")[..8]
        const SPL_TRANSFER_HOOK_EXECUTE: [u8; 8] = [105, 37, 101, 197, 75, 251, 102, 26];

        if data.len() >= 16 && data[..8] == SPL_TRANSFER_HOOK_EXECUTE {
            return __private::__global::execute_transfer_hook(program_id, accounts, &data[8..]);
        }

        Err(ProgramError::InvalidInstructionData.into())
    }
}
