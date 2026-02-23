use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::StablecoinError;
use crate::constants::BLACKLIST_SEED;

pub fn enforce_transfer(
    ctx: Context<TransferHook>,
    _amount: u64,
) -> Result<()> {
    let state = &ctx.accounts.state;
    
    if !state.compliance_enabled {
        return Ok(());
    }
    
    let (sender_blacklist_pda, _) = find_blacklist_pda(state.key(), ctx.accounts.source.key());
    if ctx.accounts.sender_blacklist.key == &sender_blacklist_pda {
        require!(ctx.accounts.sender_blacklist.data_is_empty(), StablecoinError::BlacklistViolation);
    }
    
    let (recipient_blacklist_pda, _) = find_blacklist_pda(state.key(), ctx.accounts.destination.key());
    if ctx.accounts.recipient_blacklist.key == &recipient_blacklist_pda {
        require!(ctx.accounts.recipient_blacklist.data_is_empty(), StablecoinError::BlacklistViolation);
    }
    
    Ok(())
}

#[derive(Accounts)]
pub struct TransferHook<'info> {
    /// CHECK: source
    pub source: AccountInfo<'info>,
    /// CHECK: mint
    pub mint: AccountInfo<'info>,
    /// CHECK: destination
    pub destination: AccountInfo<'info>,
    /// CHECK: owner
    pub owner: AccountInfo<'info>,
    /// CHECK: extra meta
    pub extra_account_meta_list: AccountInfo<'info>,
    pub state: Account<'info, StablecoinState>,
    /// CHECK: sender blacklist
    pub sender_blacklist: AccountInfo<'info>,
    /// CHECK: recipient blacklist
    pub recipient_blacklist: AccountInfo<'info>,
}

fn find_blacklist_pda(stablecoin: Pubkey, account: Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[BLACKLIST_SEED, stablecoin.as_ref(), account.as_ref()],
        &crate::ID,
    )
}
