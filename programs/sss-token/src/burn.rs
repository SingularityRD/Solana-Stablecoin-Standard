use crate::constants::ROLE_SEED;
use crate::error::StablecoinError;
use crate::events::*;
use crate::math::update_supply;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token_2022::{self, Burn as SplBurn};
use anchor_spl::token_interface::{Mint as TokenMint, TokenAccount, TokenInterface};

#[derive(Accounts)]
pub struct Burn<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = asset_mint
    )]
    pub state: Account<'info, StablecoinState>,

    #[account(
        seeds = [ROLE_SEED, state.key().as_ref(), authority.key().as_ref()],
        bump,
    )]
    pub role_assignment: Option<Account<'info, RoleAssignment>>,

    #[account(mut)]
    pub asset_mint: InterfaceAccount<'info, TokenMint>,

    #[account(mut)]
    pub from: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handler(ctx: Context<Burn>, amount: u64) -> Result<()> {
    require!(amount > 0, StablecoinError::ZeroAmount);
    require!(!ctx.accounts.state.paused, StablecoinError::VaultPaused);

    // RBAC Check: Must be Master (state.authority) or have Burner role
    let is_master = ctx.accounts.authority.key() == ctx.accounts.state.authority;
    let is_burner = if let Some(assignment) = &ctx.accounts.role_assignment {
        assignment.role == Role::Burner || assignment.role == Role::Master
    } else {
        false
    };

    require!(is_master || is_burner, StablecoinError::Unauthorized);

    let state = &mut ctx.accounts.state;
    state.total_supply = update_supply(state.total_supply, amount, false)?;

    // CPI to SPL Token-2022 to actual burn tokens
    let cpi_accounts = SplBurn {
        mint: ctx.accounts.asset_mint.to_account_info(),
        from: ctx.accounts.from.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);

    token_2022::burn(cpi_ctx, amount)?;

    emit!(Burned {
        stablecoin: state.key(),
        from: ctx.accounts.from.key(),
        amount,
    });

    Ok(())
}