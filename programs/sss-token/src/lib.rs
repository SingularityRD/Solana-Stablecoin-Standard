use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod events;
pub mod instructions;
pub mod math;
pub mod state;

use instructions::*;
use state::Role;

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
        instructions::initialize::handler(ctx, preset, name, symbol, uri, decimals)
    }

    pub fn mint(ctx: Context<Mint>, amount: u64) -> Result<()> {
        instructions::mint::handler(ctx, amount)
    }

    pub fn burn(ctx: Context<Burn>, amount: u64) -> Result<()> {
        instructions::burn::handler(ctx, amount)
    }

    pub fn freeze_account(ctx: Context<FreezeAccount>) -> Result<()> {
        instructions::freeze::handler(ctx)
    }

    pub fn thaw_account(ctx: Context<ThawAccount>) -> Result<()> {
        instructions::thaw::handler(ctx)
    }

    pub fn pause(ctx: Context<Admin>) -> Result<()> {
        instructions::admin::pause(ctx)
    }

    pub fn unpause(ctx: Context<Admin>) -> Result<()> {
        instructions::admin::unpause(ctx)
    }

    pub fn transfer_authority(ctx: Context<Admin>, new_authority: Pubkey) -> Result<()> {
        instructions::admin::transfer_authority(ctx, new_authority)
    }

    pub fn add_to_blacklist(ctx: Context<Blacklist>, reason: String) -> Result<()> {
        instructions::blacklist::add(ctx, reason)
    }

    pub fn remove_from_blacklist(ctx: Context<Blacklist>) -> Result<()> {
        instructions::blacklist::remove(ctx)
    }

    pub fn seize(ctx: Context<Seize>, amount: u64) -> Result<()> {
        instructions::seize::handler(ctx, amount)
    }

    pub fn assign_role(ctx: Context<AssignRole>, role: Role) -> Result<()> {
        instructions::role_management::handler(ctx, role)
    }

    // Since we are using standard Anchor instruction routing, we can expose
    // the transfer hook directly as a standard instruction, or use the exact Execute
    // discriminator. To be perfectly 100% compliant with standard Anchor,
    // we use a dedicated route for the hook testing.
    // (Note: Raw SPL integration uses a fallback, but here we just expose the logic
    // so tests and the SDK can call it directly).
    pub fn execute_transfer_hook(ctx: Context<TransferHook>, amount: u64) -> Result<()> {
        instructions::transfer_hook::enforce_transfer(ctx, amount)
    }
}
