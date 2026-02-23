use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::StablecoinError;
use crate::events::*;
use crate::constants::BLACKLIST_SEED;

#[derive(Accounts)]
pub struct Blacklist<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        has_one = authority @ StablecoinError::Unauthorized
    )]
    pub state: Account<'info, StablecoinState>,
    
    #[account(
        init_if_needed,
        payer = authority,
        space = 8 + BlacklistEntry::INIT_SPACE,
        seeds = [BLACKLIST_SEED, state.key().as_ref(), account.key().as_ref()],
        bump
    )]
    pub entry: Account<'info, BlacklistEntry>,
    
    /// CHECK: Account to blacklist
    pub account: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn add(ctx: Context<Blacklist>, reason: String) -> Result<()> {
    require!(ctx.accounts.state.compliance_enabled, StablecoinError::ComplianceNotEnabled);
    
    let entry = &mut ctx.accounts.entry;
    entry.account = ctx.accounts.account.key();
    entry.reason = reason.clone();
    entry.blacklisted_by = ctx.accounts.authority.key();
    entry.blacklisted_at = Clock::get()?.unix_timestamp;
    entry.bump = ctx.bumps.entry;
    
    emit!(BlacklistAdded {
        stablecoin: ctx.accounts.state.key(),
        account: ctx.accounts.account.key(),
        reason,
    });
    Ok(())
}

pub fn remove(ctx: Context<Blacklist>) -> Result<()> {
    require!(ctx.accounts.state.compliance_enabled, StablecoinError::ComplianceNotEnabled);
    
    let account_key = ctx.accounts.entry.account;
    ctx.accounts.entry.close(ctx.accounts.authority.to_account_info())?;
    
    emit!(BlacklistRemoved {
        stablecoin: ctx.accounts.state.key(),
        account: account_key,
    });
    Ok(())
}
