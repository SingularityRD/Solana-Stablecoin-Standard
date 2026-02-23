use anchor_lang::prelude::*;
use anchor_spl::token_2022::{self, MintTo};
use anchor_spl::token_interface::{Mint as TokenMint, TokenAccount, TokenInterface};
use crate::state::*;
use crate::error::StablecoinError;
use crate::events::*;
use crate::math::update_supply;
use crate::constants::{VAULT_SEED, ROLE_SEED};

#[derive(Accounts)]
pub struct Mint<'info> {
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
    pub recipient: InterfaceAccount<'info, TokenAccount>,
    
    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handler(ctx: Context<Mint>, amount: u64) -> Result<()> {
    let state = &mut ctx.accounts.state;
    
    // RBAC Check: Must be Master (state.authority) or have Minter role
    let is_master = ctx.accounts.authority.key() == state.authority;
    let is_minter = if let Some(assignment) = &ctx.accounts.role_assignment {
        assignment.role == Role::Minter || assignment.role == Role::Master
    } else {
        false
    };

    require!(is_master || is_minter, StablecoinError::Unauthorized);
    require!(amount > 0, StablecoinError::ZeroAmount);
    require!(!state.paused, StablecoinError::VaultPaused);
    
    state.total_supply = update_supply(state.total_supply, amount, true)?;
    
    let asset_mint_key = state.asset_mint.key();
    let authority_seeds = &[
        VAULT_SEED,
        asset_mint_key.as_ref(),
        &[state.bump],
    ];
    let signer = &[&authority_seeds[..]];

    let cpi_accounts = MintTo {
        mint: ctx.accounts.asset_mint.to_account_info(),
        to: ctx.accounts.recipient.to_account_info(),
        authority: state.to_account_info(),
    };
    
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        signer,
    );
    
    token_2022::mint_to(cpi_ctx, amount)?;
    
    emit!(Minted {
        stablecoin: state.key(),
        recipient: ctx.accounts.recipient.key(),
        amount,
        minter: ctx.accounts.authority.key(),
    });
    
    Ok(())
}
