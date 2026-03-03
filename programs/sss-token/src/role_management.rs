use crate::constants::ROLE_SEED;
use crate::error::StablecoinError;
use crate::events::*;
use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct AssignRole<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        has_one = authority @ StablecoinError::Unauthorized
    )]
    pub state: Account<'info, StablecoinState>,

    #[account(
        init,
        payer = authority,
        space = 8 + RoleAssignment::INIT_SPACE,
        seeds = [ROLE_SEED, state.key().as_ref(), account.key().as_ref()],
        bump
    )]
    pub assignment: Account<'info, RoleAssignment>,

    pub account: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AssignRole>, role: Role) -> Result<()> {
    let assignment = &mut ctx.accounts.assignment;
    assignment.role = role.clone();
    assignment.account = ctx.accounts.account.key();
    assignment.assigned_by = ctx.accounts.authority.key();
    assignment.assigned_at = Clock::get()?.unix_timestamp;
    assignment.bump = ctx.bumps.assignment;
    emit!(RoleAssigned {
        stablecoin: ctx.accounts.state.key(),
        role: role.clone(),
        account: ctx.accounts.account.key(),
        assigned_by: ctx.accounts.authority.key(),
    });
    Ok(())
}

#[derive(Accounts)]
pub struct RevokeRole<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        has_one = authority @ StablecoinError::Unauthorized
    )]
    pub state: Account<'info, StablecoinState>,

    #[account(
        mut,
        close = authority,
        seeds = [ROLE_SEED, state.key().as_ref(), assignment.account.as_ref()],
        bump = assignment.bump
    )]
    pub assignment: Account<'info, RoleAssignment>,
}

pub fn revoke_handler(ctx: Context<RevokeRole>) -> Result<()> {
    let revoked_role = ctx.accounts.assignment.role.clone();
    let revoked_account = ctx.accounts.assignment.account;
    emit!(RoleRevoked {
        stablecoin: ctx.accounts.state.key(),
        role: revoked_role,
        account: revoked_account,
    });

    Ok(())
}