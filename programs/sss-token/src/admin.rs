use crate::error::StablecoinError;
use crate::events::*;
use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Admin<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = authority @ StablecoinError::Unauthorized
    )]
    pub state: Account<'info, StablecoinState>,
}

pub fn pause(ctx: Context<Admin>) -> Result<()> {
    let state = &mut ctx.accounts.state;
    require!(!state.paused, StablecoinError::VaultPaused);
    state.paused = true;

    emit!(Paused {
        stablecoin: state.key(),
        authority: ctx.accounts.authority.key(),
    });
    Ok(())
}

pub fn unpause(ctx: Context<Admin>) -> Result<()> {
    let state = &mut ctx.accounts.state;
    require!(state.paused, StablecoinError::VaultPaused);
    state.paused = false;

    emit!(Unpaused {
        stablecoin: state.key(),
        authority: ctx.accounts.authority.key(),
    });
    Ok(())
}

pub fn transfer_authority(ctx: Context<Admin>, new_authority: Pubkey) -> Result<()> {
    let state = &mut ctx.accounts.state;
    let old_authority = state.authority;
    state.authority = new_authority;

    emit!(AuthorityTransferred {
        stablecoin: state.key(),
        old_authority,
        new_authority,
    });
    Ok(())
}
