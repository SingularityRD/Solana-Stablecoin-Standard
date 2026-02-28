//! Fuzz tests for the Transfer Hook
//!
//! Tests various transfer scenarios including:
//! - Valid transfers (SSS-1 mode - no compliance)
//! - Blacklist enforcement (SSS-2 mode)
//! - Compliance enabled/disabled
//! - Transfer amounts

use anchor_lang::prelude::*;
use anchor_lang::solana_program::pubkey::Pubkey;
use sss_token::error::StablecoinError;
use trident::prelude::*;

/// Input structure for fuzz testing transfers
#[derive(Debug, Arbitrary)]
pub struct TransferInput {
    pub amount: u64,
    pub sender_blacklisted: bool,
    pub recipient_blacklisted: bool,
    pub compliance_enabled: bool,
}

/// Fuzz test for transfer hook
#[fuzz]
pub fn fuzz_transfer(input: TransferInput) -> Result<()> {
    let preset = if input.compliance_enabled { 2 } else { 1 };
    let mut ctx = setup_stablecoin_with_compliance(preset)?;

    let sender = Pubkey::new_unique();
    let recipient = Pubkey::new_unique();

    // Add to blacklist if requested
    if input.sender_blacklisted && input.compliance_enabled {
        try_blacklist(&mut ctx, sender)?;
    }
    if input.recipient_blacklisted && input.compliance_enabled {
        try_blacklist(&mut ctx, recipient)?;
    }

    // Execute transfer hook
    let result = try_transfer_hook(&mut ctx, sender, recipient, input.amount);

    // Validate result
    if input.compliance_enabled {
        if input.sender_blacklisted || input.recipient_blacklisted {
            // Should fail due to blacklist
            assert!(result.is_err(), "Transfer should fail with blacklisted party");
            let error_code = parse_anchor_error(&result.unwrap_err());
            assert_eq!(
                error_code, Some(StablecoinError::BlacklistViolation as u32),
                "Expected BlacklistViolation error"
            );
        } else {
            // Should succeed
            assert!(result.is_ok(), "Transfer should succeed without blacklist");
        }
    } else {
        // SSS-1 mode - no compliance checks
        assert!(result.is_ok(), "Transfer should succeed in SSS-1 mode");
    }

    Ok(())
}

/// Fuzz test for transfer amounts in SSS-1 mode
#[fuzz]
pub fn fuzz_transfer_amounts_sss1(amount: u64) -> Result<()> {
    let mut ctx = setup_stablecoin_with_compliance(1)?; // SSS-1

    let sender = Pubkey::new_unique();
    let recipient = Pubkey::new_unique();

    // Any amount should succeed in SSS-1 mode
    let result = try_transfer_hook(&mut ctx, sender, recipient, amount);
    assert!(result.is_ok(), "Transfer should always succeed in SSS-1 mode");

    Ok(())
}

/// Fuzz test for blacklist scenarios
#[derive(Debug, Arbitrary)]
pub struct BlacklistTransferInput {
    pub amounts: Vec<u64>,
    pub sender_blacklisted_from_start: bool,
}

#[fuzz]
pub fn fuzz_blacklist_transfers(input: BlacklistTransferInput) -> Result<()> {
    if input.amounts.is_empty() || input.amounts.len() > 50 {
        return Ok(());
    }

    let mut ctx = setup_stablecoin_with_compliance(2)?; // SSS-2 with compliance

    let sender = Pubkey::new_unique();
    let recipient = Pubkey::new_unique();

    // Blacklist sender if requested
    if input.sender_blacklisted_from_start {
        try_blacklist(&mut ctx, sender)?;
    }

    for (i, &amount) in input.amounts.iter().enumerate() {
        let result = try_transfer_hook(&mut ctx, sender, recipient, amount);

        if input.sender_blacklisted_from_start {
            // All transfers should fail
            assert!(result.is_err(), "Transfer {} should fail with blacklisted sender", i);
        } else {
            // Should succeed - sender not blacklisted
            assert!(result.is_ok(), "Transfer {} should succeed", i);
        }
    }

    Ok(())
}

/// Fuzz test for dynamic blacklisting during transfers
#[derive(Debug, Arbitrary)]
pub struct DynamicBlacklistInput {
    pub operations: Vec<TransferOp>,
}

#[derive(Debug, Arbitrary)]
pub enum TransferOp {
    Transfer(u64),
    BlacklistSender,
    UnblacklistSender,
    BlacklistRecipient,
    UnblacklistRecipient,
}

#[fuzz]
pub fn fuzz_dynamic_blacklist(input: DynamicBlacklistInput) -> Result<()> {
    if input.operations.is_empty() || input.operations.len() > 100 {
        return Ok(());
    }

    let mut ctx = setup_stablecoin_with_compliance(2)?; // SSS-2

    let sender = Pubkey::new_unique();
    let recipient = Pubkey::new_unique();

    let mut sender_blacklisted = false;
    let mut recipient_blacklisted = false;

    for (i, op) in input.operations.iter().enumerate() {
        match op {
            TransferOp::Transfer(amount) => {
                let result = try_transfer_hook(&mut ctx, sender, recipient, *amount);

                if sender_blacklisted || recipient_blacklisted {
                    assert!(result.is_err(), "Transfer {} should fail", i);
                } else {
                    assert!(result.is_ok(), "Transfer {} should succeed", i);
                }
            }
            TransferOp::BlacklistSender => {
                if !sender_blacklisted {
                    try_blacklist(&mut ctx, sender)?;
                    sender_blacklisted = true;
                }
            }
            TransferOp::UnblacklistSender => {
                if sender_blacklisted {
                    try_unblacklist(&mut ctx, sender)?;
                    sender_blacklisted = false;
                }
            }
            TransferOp::BlacklistRecipient => {
                if !recipient_blacklisted {
                    try_blacklist(&mut ctx, recipient)?;
                    recipient_blacklisted = true;
                }
            }
            TransferOp::UnblacklistRecipient => {
                if recipient_blacklisted {
                    try_unblacklist(&mut ctx, recipient)?;
                    recipient_blacklisted = false;
                }
            }
        }
    }

    Ok(())
}

/// Fuzz test for multiple blacklisted accounts
#[fuzz]
pub fn fuzz_multiple_blacklisted_accounts(
    sender_blacklisted: bool,
    recipient_blacklisted: bool,
    amount: u64,
) -> Result<()> {
    let mut ctx = setup_stablecoin_with_compliance(2)?;

    let sender = Pubkey::new_unique();
    let recipient = Pubkey::new_unique();

    if sender_blacklisted {
        try_blacklist(&mut ctx, sender)?;
    }
    if recipient_blacklisted {
        try_blacklist(&mut ctx, recipient)?;
    }

    let result = try_transfer_hook(&mut ctx, sender, recipient, amount);

    if sender_blacklisted || recipient_blacklisted {
        assert!(result.is_err(), "Transfer should fail with any blacklisted party");
    } else {
        assert!(result.is_ok(), "Transfer should succeed without blacklist");
    }

    Ok(())
}

// ============================================================================
// Helper Functions
// ============================================================================

struct TransferTestContext {
    context: TestContext,
    authority: Pubkey,
    state_pda: Pubkey,
    asset_mint: Pubkey,
}

fn setup_stablecoin_with_compliance(preset: u8) -> Result<TransferTestContext> {
    let mut test = TestContext::new();
    test.add_program("sss_token", sss_token::ID)?;

    let authority = test.payer();
    let asset_mint = Pubkey::new_unique();

    let (state_pda, bump) = Pubkey::find_program_address(
        &[b"stablecoin", asset_mint.as_ref()],
        &sss_token::ID,
    );

    let init_ix = sss_token::instruction::Initialize {
        preset,
        name: "Test Stablecoin".to_string(),
        symbol: "TST".to_string(),
        uri: "https://test.com".to_string(),
        decimals: 6,
    };

    test.invoke(
        &[
            AccountMeta::new(authority, true),
            AccountMeta::new(state_pda, false),
            AccountMeta::new_readonly(asset_mint, false),
            AccountMeta::new_readonly(System::id(), false),
        ],
        init_ix,
        Some(&[&[b"stablecoin", asset_mint.as_ref(), &[bump]]]),
    )?;

    Ok(TransferTestContext {
        context: test,
        authority,
        state_pda,
        asset_mint,
    })
}

fn try_transfer_hook(
    ctx: &mut TransferTestContext,
    source: Pubkey,
    destination: Pubkey,
    amount: u64,
) -> Result<()> {
    let (sender_blacklist_pda, _) = Pubkey::find_program_address(
        &[b"blacklist", ctx.state_pda.as_ref(), source.as_ref()],
        &sss_token::ID,
    );

    let (recipient_blacklist_pda, _) = Pubkey::find_program_address(
        &[b"blacklist", ctx.state_pda.as_ref(), destination.as_ref()],
        &sss_token::ID,
    );

    let transfer_ix = sss_token::instruction::ExecuteTransferHook { amount };

    ctx.context.invoke(
        &[
            AccountMeta::new_readonly(source, false),
            AccountMeta::new_readonly(ctx.asset_mint, false),
            AccountMeta::new_readonly(destination, false),
            AccountMeta::new_readonly(ctx.state_pda, false),
            AccountMeta::new_readonly(sender_blacklist_pda, false),
            AccountMeta::new_readonly(recipient_blacklist_pda, false),
        ],
        transfer_ix,
        None,
    )
}

fn try_blacklist(ctx: &mut TransferTestContext, account: Pubkey) -> Result<()> {
    let (blacklist_pda, _) = Pubkey::find_program_address(
        &[b"blacklist", ctx.state_pda.as_ref(), account.as_ref()],
        &sss_token::ID,
    );

    let blacklist_ix = sss_token::instruction::AddToBlacklist {
        reason: "Fuzz test".to_string(),
    };

    ctx.context.invoke(
        &[
            AccountMeta::new(ctx.authority, true),
            AccountMeta::new_readonly(ctx.state_pda, false),
            AccountMeta::new(blacklist_pda, false),
            AccountMeta::new_readonly(account, false),
            AccountMeta::new_readonly(System::id(), false),
        ],
        blacklist_ix,
        None,
    )
}

fn try_unblacklist(ctx: &mut TransferTestContext, account: Pubkey) -> Result<()> {
    let (blacklist_pda, _) = Pubkey::find_program_address(
        &[b"blacklist", ctx.state_pda.as_ref(), account.as_ref()],
        &sss_token::ID,
    );

    let unblacklist_ix = sss_token::instruction::RemoveFromBlacklist {};

    ctx.context.invoke(
        &[
            AccountMeta::new(ctx.authority, true),
            AccountMeta::new_readonly(ctx.state_pda, false),
            AccountMeta::new(blacklist_pda, false),
        ],
        unblacklist_ix,
        None,
    )
}

fn parse_anchor_error(error: &Error) -> Option<u32> {
    match error {
        Error::AnchorError(e) => Some(e.error_code_number),
        Error::ProgramError(e) => {
            if let Some(code) = e.to_error_code() {
                Some(code.code())
            } else {
                None
            }
        }
        _ => None,
    }
}
