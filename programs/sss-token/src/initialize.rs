use crate::constants::*;
use crate::error::StablecoinError;
use crate::events::*;
use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + StablecoinState::INIT_SPACE,
        seeds = [VAULT_SEED, asset_mint.key().as_ref()],
        bump
    )]
    pub state: Account<'info, StablecoinState>,

    pub asset_mint: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<Initialize>,
    preset: u8,
    name: String,
    symbol: String,
    uri: String,
    decimals: u8,
) -> Result<()> {
    let state = &mut ctx.accounts.state;

    require!(
        preset == PRESET_SSS_1 || preset == PRESET_SSS_2,
        StablecoinError::InvalidPreset
    );
    require!(name.len() <= MAX_NAME_LENGTH, StablecoinError::NameTooLong);
    require!(
        symbol.len() <= MAX_SYMBOL_LENGTH,
        StablecoinError::SymbolTooLong
    );
    require!(uri.len() <= MAX_URI_LENGTH, StablecoinError::UriTooLong);
    require!(decimals <= 9, StablecoinError::InvalidDecimals);

    state.authority = ctx.accounts.authority.key();
    state.asset_mint = ctx.accounts.asset_mint.key();
    state.total_supply = 0;
    state.paused = false;
    state.preset = preset;
    state.compliance_enabled = preset == PRESET_SSS_2;
    state.bump = ctx.bumps.state;

    emit!(StablecoinInitialized {
        stablecoin: state.key(),
        preset,
        name,
        symbol,
        decimals,
        compliance_enabled: state.compliance_enabled,
    });

    Ok(())
}
