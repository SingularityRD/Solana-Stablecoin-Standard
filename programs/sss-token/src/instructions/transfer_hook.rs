use anchor_lang::prelude::*;
use spl_transfer_hook_interface::instruction::TransferHookInstruction;
use crate::state::*;
use crate::error::StablecoinError;
use crate::constants::BLACKLIST_SEED;

/// Transfer hook enforcement for SSS-2
/// This implements the Execute instruction required by spl-transfer-hook-interface
pub fn enforce_transfer(
    ctx: Context<TransferHook>,
    amount: u64,
) -> Result<()> {
    let state = &ctx.accounts.state;
    
    // Only enforce for SSS-2
    if !state.compliance_enabled {
        return Ok(());
    }
    
    // Check sender not blacklisted
    let sender_key = ctx.accounts.source.key();
    let (sender_blacklist_pda, _) = find_blacklist_pda(state.key(), sender_key);
    let sender_blacklisted = ctx.accounts.sender_blacklist.key == &sender_blacklist_pda;
    require!(!sender_blacklisted, StablecoinError::BlacklistViolation);
    
    // Check recipient not blacklisted
    let recipient_key = ctx.accounts.destination.key();
    let (recipient_blacklist_pda, _) = find_blacklist_pda(state.key(), recipient_key);
    let recipient_blacklisted = ctx.accounts.recipient_blacklist.key == &recipient_blacklist_pda;
    require!(!recipient_blacklisted, StablecoinError::BlacklistViolation);
    
    msg!("Transfer Hook: Allowed transfer of {} tokens", amount);
    Ok(())
}

#[derive(Accounts)]
pub struct TransferHook<'info> {
    /// CHECK: The source token account
    pub source: AccountInfo<'info>,
    
    /// CHECK: The token mint
    pub mint: AccountInfo<'info>,
    
    /// CHECK: The destination token account
    pub destination: AccountInfo<'info>,
    
    /// CHECK: The source token account owner/delegate
    pub owner: AccountInfo<'info>,
    
    /// CHECK: ExtraAccountMetaList PDA
    pub extra_account_meta_list: AccountInfo<'info>,
    
    pub state: Account<'info, StablecoinState>,
    
    /// CHECK: Sender blacklist PDA
    pub sender_blacklist: AccountInfo<'info>,
    
    /// CHECK: Recipient blacklist PDA
    pub recipient_blacklist: AccountInfo<'info>,
}

fn find_blacklist_pda(stablecoin: Pubkey, account: Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[BLACKLIST_SEED, stablecoin.as_ref(), account.as_ref()],
        &crate::ID,
    )
}
