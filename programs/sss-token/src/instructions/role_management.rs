use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::StablecoinError;
use crate::events::*;
use crate::constants::ROLE_SEED;

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
    
    let role_name = match role {
        Role::Master => "Master",
        Role::Minter => "Minter",
        Role::Burner => "Burner",
        Role::Blacklister => "Blacklister",
        Role::Pauser => "Pauser",
        Role::Seizer => "Seizer",
    };
    
    emit!(RoleAssigned {
        stablecoin: ctx.accounts.state.key(),
        role: role_name.to_string(),
        account: ctx.accounts.account.key(),
        assigned_by: ctx.accounts.authority.key(),
    });
    Ok(())
}
