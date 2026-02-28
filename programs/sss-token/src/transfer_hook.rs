use crate::constants::BLACKLIST_SEED;
use crate::error::StablecoinError;
use crate::state::*;
use anchor_lang::prelude::*;

pub fn enforce_transfer(ctx: Context<TransferHook>, _amount: u64) -> Result<()> {
    let state = &ctx.accounts.state;

    if !state.compliance_enabled {
        return Ok(());
    }

    let (sender_blacklist_pda, _) = find_blacklist_pda(state.key(), ctx.accounts.source.key());
    if ctx.accounts.sender_blacklist.key == &sender_blacklist_pda {
        require!(
            ctx.accounts.sender_blacklist.data_is_empty(),
            StablecoinError::BlacklistViolation
        );
    }

    let (recipient_blacklist_pda, _) =
        find_blacklist_pda(state.key(), ctx.accounts.destination.key());
    if ctx.accounts.recipient_blacklist.key == &recipient_blacklist_pda {
        require!(
            ctx.accounts.recipient_blacklist.data_is_empty(),
            StablecoinError::BlacklistViolation
        );
    }

    Ok(())
}

/// TransferHook accounts for SPL Token-2022 transfer hook interface.
/// Note: This uses manual account validation because the SPL transfer hook
/// interface requires specific account ordering that doesn't fit Anchor's
/// standard Accounts derive pattern.
#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct TransferHook<'info> {
    /// CHECK: Source token account
    pub source: AccountInfo<'info>,
    /// CHECK: Mint account
    pub mint: AccountInfo<'info>,
    /// CHECK: Destination token account
    pub destination: AccountInfo<'info>,
    /// CHECK: Owner of source account
    pub owner: AccountInfo<'info>,
    /// CHECK: Extra account meta list for additional accounts
    pub extra_account_meta_list: AccountInfo<'info>,
    pub state: Account<'info, StablecoinState>,
    /// CHECK: Sender blacklist entry (may not exist)
    pub sender_blacklist: AccountInfo<'info>,
    /// CHECK: Recipient blacklist entry (may not exist)
    pub recipient_blacklist: AccountInfo<'info>,
}

fn find_blacklist_pda(stablecoin: Pubkey, account: Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[BLACKLIST_SEED, stablecoin.as_ref(), account.as_ref()],
        &crate::ID,
    )
}