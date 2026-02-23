use anchor_lang::prelude::*;
use anchor_spl::token_2022::{self, TransferChecked};
use anchor_spl::token_interface::{Mint as TokenMint, TokenAccount, TokenInterface};
use crate::state::*;
use crate::error::StablecoinError;
use crate::events::*;
use crate::constants::VAULT_SEED;

#[derive(Accounts)]
pub struct Seize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        has_one = authority @ StablecoinError::Unauthorized,
        has_one = asset_mint
    )]
    pub state: Account<'info, StablecoinState>,
    
    #[account(mut)]
    pub asset_mint: InterfaceAccount<'info, TokenMint>,
    
    #[account(mut)]
    pub from: InterfaceAccount<'info, TokenAccount>,
    
    #[account(mut)]
    pub to: InterfaceAccount<'info, TokenAccount>,
    
    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handler(ctx: Context<Seize>, amount: u64) -> Result<()> {
    require!(amount > 0, StablecoinError::ZeroAmount);
    require!(!ctx.accounts.state.paused, StablecoinError::VaultPaused);
    require!(ctx.accounts.state.compliance_enabled, StablecoinError::ComplianceNotEnabled);
    
    let state = &ctx.accounts.state;
    let asset_mint_key = state.asset_mint.key();
    let authority_seeds = &[
        VAULT_SEED,
        asset_mint_key.as_ref(),
        &[state.bump],
    ];
    let signer = &[&authority_seeds[..]];

    // Use Permanent Delegate to transfer (seize) tokens
    // SSS Token PDA acts as the permanent delegate authority
    let cpi_accounts = TransferChecked {
        from: ctx.accounts.from.to_account_info(),
        mint: ctx.accounts.asset_mint.to_account_info(),
        to: ctx.accounts.to.to_account_info(),
        authority: state.to_account_info(), // The permanent delegate
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
