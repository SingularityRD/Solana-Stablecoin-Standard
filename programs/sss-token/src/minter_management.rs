use crate::constants::MINTER_SEED;
use crate::error::StablecoinError;
use crate::events::*;
use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct AddMinter<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        has_one = authority @ StablecoinError::Unauthorized
    )]
    pub state: Account<'info, StablecoinState>,

    #[account(
        init,
        payer = authority,
        space = 8 + MinterInfo::INIT_SPACE,
        seeds = [MINTER_SEED, state.key().as_ref(), minter.key().as_ref()],
        bump
    )]
    pub minter_info: Account<'info, MinterInfo>,

    /// CHECK: The minter account that will be added
    pub minter: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

pub fn add_minter_handler(ctx: Context<AddMinter>, quota: u64) -> Result<()> {
    let minter_info = &mut ctx.accounts.minter_info;
    minter_info.minter = ctx.accounts.minter.key();
    minter_info.quota = quota;
    minter_info.minted_amount = 0;
    minter_info.bump = ctx.bumps.minter_info;

    emit!(MinterAdded {
        stablecoin: ctx.accounts.state.key(),
        minter: ctx.accounts.minter.key(),
        quota,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct RemoveMinter<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        has_one = authority @ StablecoinError::Unauthorized
    )]
    pub state: Account<'info, StablecoinState>,

    #[account(
        mut,
        close = authority,
        seeds = [MINTER_SEED, state.key().as_ref(), minter_info.minter.as_ref()],
        bump = minter_info.bump
    )]
    pub minter_info: Account<'info, MinterInfo>,
}

pub fn remove_minter_handler(ctx: Context<RemoveMinter>) -> Result<()> {
    let minter = ctx.accounts.minter_info.minter;

    emit!(MinterRemoved {
        stablecoin: ctx.accounts.state.key(),
        minter,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct UpdateQuota<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        has_one = authority @ StablecoinError::Unauthorized
    )]
    pub state: Account<'info, StablecoinState>,

    #[account(
        mut,
        seeds = [MINTER_SEED, state.key().as_ref(), minter_info.minter.as_ref()],
        bump = minter_info.bump
    )]
    pub minter_info: Account<'info, MinterInfo>,
}

pub fn update_quota_handler(ctx: Context<UpdateQuota>, new_quota: u64) -> Result<()> {
    let minter_info = &mut ctx.accounts.minter_info;
    let old_quota = minter_info.quota;
    minter_info.quota = new_quota;

    emit!(QuotaUpdated {
        stablecoin: ctx.accounts.state.key(),
        minter: minter_info.minter,
        old_quota,
        new_quota,
    });

    Ok(())
}
