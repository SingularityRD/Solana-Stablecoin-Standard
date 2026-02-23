use anchor_lang::prelude::*;
use anchor_spl::token_2022::{self, TransferChecked};
use anchor_spl::token_interface::{Mint as TokenMint, TokenAccount, TokenInterface};
use crate::state::*;
use crate::error::StablecoinError;
use crate::events::*;
use crate::constants::{VAULT_SEED, ROLE_SEED};

#[derive(Accounts)]
pub struct Seize<'info> {
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
    
    #[account(mut)]
    pub to: InterfaceAccount<'info, TokenAccount>,
    
    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handler(ctx: Context<Seize>, amount: u64) -> Result<()> {
    let state = &ctx.accounts.state;

    // RBAC Check: Must be Master or have Seizer role
    let is_master = ctx.accounts.authority.key() == state.authority;
    let is_seizer = if let Some(assignment) = &ctx.accounts.role_assignment {
        assignment.role == Role::Seizer || assignment.role == Role::Master
    } else {
        false
    };

    require!(is_master || is_seizer, StablecoinError::Unauthorized);
    require!(amount > 0, StablecoinError::ZeroAmount);
    require!(!state.paused, StablecoinError::VaultPaused);
    require!(state.compliance_enabled, StablecoinError::ComplianceNotEnabled);
    
    let asset_mint_key = state.asset_mint.key();
    let authority_seeds = &[
        VAULT_SEED,
        asset_mint_key.as_ref(),
        &[state.bump],
    ];
    let signer = &[&authority_seeds[..]];

    let cpi_accounts = TransferChecked {
        from: ctx.accounts.from.to_account_info(),
        mint: ctx.accounts.asset_mint.to_account_info(),
        to: ctx.accounts.to.to_account_info(),
        authority: state.to_account_info(),
    };
    
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        signer,
    );
    
    token_2022::transfer_checked(cpi_ctx, amount, ctx.accounts.asset_mint.decimals)?;

    emit!(Seized {
        stablecoin: state.key(),
        from: ctx.accounts.from.key(),
        to: ctx.accounts.to.key(),
        amount,
    });
    Ok(())
}
