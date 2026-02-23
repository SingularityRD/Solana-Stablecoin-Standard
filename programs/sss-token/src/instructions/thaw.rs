use crate::constants::VAULT_SEED;
use crate::error::StablecoinError;
use crate::events::*;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token_2022::{self, ThawAccount as SplThaw};
use anchor_spl::token_interface::{Mint as TokenMint, TokenAccount, TokenInterface};

#[derive(Accounts)]
pub struct ThawAccount<'info> {
    pub authority: Signer<'info>,

    #[account(
        has_one = authority @ StablecoinError::Unauthorized,
        has_one = asset_mint
    )]
    pub state: Account<'info, StablecoinState>,

    #[account(mut)]
    pub asset_mint: InterfaceAccount<'info, TokenMint>,

    #[account(mut)]
    pub account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handler(ctx: Context<ThawAccount>) -> Result<()> {
    require!(!ctx.accounts.state.paused, StablecoinError::VaultPaused);

    let state = &ctx.accounts.state;
    let asset_mint_key = state.asset_mint.key();
    let authority_seeds = &[VAULT_SEED, asset_mint_key.as_ref(), &[state.bump]];
    let signer = &[&authority_seeds[..]];

    // CPI to Thaw
    let cpi_accounts = SplThaw {
        account: ctx.accounts.account.to_account_info(),
        mint: ctx.accounts.asset_mint.to_account_info(),
        authority: state.to_account_info(),
    };

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        signer,
    );

    token_2022::thaw_account(cpi_ctx)?;

    emit!(Thawed {
        stablecoin: state.key(),
        account: ctx.accounts.account.key(),
    });

    Ok(())
}
