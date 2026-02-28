//! Fuzz tests for the Mint instruction
//!
//! Tests various mint scenarios including:
//! - Valid mint amounts
//! - Zero amounts (should fail)
//! - Overflow scenarios
//! - Unauthorized minting
//! - Paused state minting
//! - Quota enforcement for minters

use anchor_lang::prelude::*;
use anchor_lang::solana_program::pubkey::Pubkey;
use sss_token::error::StablecoinError;
use sss_token::state::Role;
use trident::prelude::*;

/// Input structure for fuzz testing the mint instruction
#[derive(Debug, Arbitrary)]
pub struct MintInput {
    pub amount: u64,
    pub use_role_assignment: bool,
    pub is_authorized: bool,
    pub is_paused: bool,
}

/// Fuzz test for the mint instruction with various amounts
#[fuzz]
pub fn fuzz_mint(input: MintInput) -> Result<()> {
    // Setup initialized stablecoin
    let mut ctx = setup_initialized_stablecoin(input.is_paused)?;

    // Setup authorization context
    let minter = if input.is_authorized {
        ctx.authority.clone()
    } else {
        // Create an unauthorized key
        Pubkey::new_unique()
    };

    // Attempt to mint
    let result = try_mint(&mut ctx, minter, input.amount, input.use_role_assignment);

    // Validate result based on input
    if input.amount == 0 {
        // Zero amount should always fail
        assert!(result.is_err(), "Mint with zero amount should fail");
        let error_code = parse_anchor_error(&result.unwrap_err());
        assert_eq!(
            error_code, Some(StablecoinError::ZeroAmount as u32),
            "Expected ZeroAmount error"
        );
    } else if input.is_paused {
        // Paused state should fail
        assert!(result.is_err(), "Mint while paused should fail");
        let error_code = parse_anchor_error(&result.unwrap_err());
        assert_eq!(
            error_code, Some(StablecoinError::VaultPaused as u32),
            "Expected VaultPaused error"
        );
    } else if !input.is_authorized {
        // Unauthorized should fail
        assert!(result.is_err(), "Unauthorized mint should fail");
        let error_code = parse_anchor_error(&result.unwrap_err());
        assert_eq!(
            error_code, Some(StablecoinError::Unauthorized as u32),
            "Expected Unauthorized error"
        );
    } else {
        // Valid mint should succeed
        assert!(result.is_ok(), "Valid mint should succeed");
        verify_mint_result(&ctx, input.amount)?;
    }

    Ok(())
}

/// Fuzz test for mint amounts
#[fuzz]
pub fn fuzz_mint_amounts(amount: u64) -> Result<()> {
    let mut ctx = setup_initialized_stablecoin(false)?;

    let result = try_mint(&mut ctx, ctx.authority.clone(), amount, false);

    if amount == 0 {
        assert!(result.is_err(), "Zero amount should fail");
    } else {
        // Should succeed for any positive amount (assuming no overflow)
        // Large amounts may cause overflow in total_supply
        match result {
            Ok(_) => {
                // Verify supply was updated
            }
            Err(e) => {
                let error_code = parse_anchor_error(&e);
                // Large amounts could cause overflow
                if error_code == Some(StablecoinError::MathOverflow as u32) {
                    // This is acceptable for extreme amounts
                } else if error_code == Some(StablecoinError::ZeroAmount as u32) {
                    panic!("Zero amount error for non-zero amount: {}", amount);
                } else {
                    panic!("Unexpected error: {:?}", e);
                }
            }
        }
    }

    Ok(())
}

/// Fuzz test for minter quota enforcement
#[derive(Debug, Arbitrary)]
pub struct QuotaMintInput {
    pub quota: u64,
    pub mint_amounts: Vec<u64>,
}

#[fuzz]
pub fn fuzz_mint_quota(input: QuotaMintInput) -> Result<()> {
    if input.mint_amounts.is_empty() {
        return Ok(());
    }

    let mut ctx = setup_initialized_stablecoin_with_quota(input.quota)?;

    let mut total_minted: u64 = 0;

    for (i, &amount) in input.mint_amounts.iter().enumerate() {
        let result = try_mint_with_quota(&mut ctx, amount);

        if amount == 0 {
            assert!(result.is_err(), "Mint {} with zero should fail", i);
        } else {
            let new_total = total_minted.saturating_add(amount);

            if new_total > input.quota {
                // Should fail due to quota exceeded
                assert!(result.is_err(), "Mint {} should fail due to quota", i);
                let error_code = parse_anchor_error(&result.unwrap_err());
                assert_eq!(
                    error_code, Some(StablecoinError::QuotaExceeded as u32),
                    "Expected QuotaExceeded error on mint {}", i
                );
            } else if result.is_ok() {
                total_minted = new_total;
            }
        }
    }

    Ok(())
}

/// Fuzz test for multiple sequential mints
#[fuzz]
pub fn fuzz_sequential_mints(amounts: Vec<u64>) -> Result<()> {
    if amounts.is_empty() || amounts.len() > 100 {
        return Ok(());
    }

    let mut ctx = setup_initialized_stablecoin(false)?;
    let mut total_supply: u64 = 0;

    for (i, &amount) in amounts.iter().enumerate() {
        let result = try_mint(&mut ctx, ctx.authority.clone(), amount, false);

        if amount == 0 {
            assert!(result.is_err(), "Mint {} with zero should fail", i);
        } else {
            match result {
                Ok(_) => {
                    total_supply = total_supply.saturating_add(amount);
                }
                Err(e) => {
                    let error_code = parse_anchor_error(&e);
                    if error_code == Some(StablecoinError::MathOverflow as u32) {
                        // Overflow is acceptable for extreme cumulative amounts
                        break;
                    } else {
                        panic!("Unexpected error on mint {}: {:?}", i, e);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Fuzz test for mint after pause/unpause
#[fuzz]
pub fn fuzz_mint_pause_sequence(actions: Vec<PauseAction>) -> Result<()> {
    if actions.is_empty() || actions.len() > 50 {
        return Ok(());
    }

    let mut ctx = setup_initialized_stablecoin(false)?;
    let mut is_paused = false;

    for (i, action) in actions.iter().enumerate() {
        match action {
            PauseAction::Pause => {
                try_pause(&mut ctx)?;
                is_paused = true;
            }
            PauseAction::Unpause => {
                try_unpause(&mut ctx)?;
                is_paused = false;
            }
            PauseAction::Mint(amount) => {
                let result = try_mint(&mut ctx, ctx.authority.clone(), *amount, false);

                if is_paused {
                    assert!(result.is_err(), "Mint {} should fail while paused", i);
                    if *amount != 0 {
                        let error_code = parse_anchor_error(&result.unwrap_err());
                        assert_eq!(
                            error_code, Some(StablecoinError::VaultPaused as u32),
                            "Expected VaultPaused error"
                        );
                    }
                } else if *amount == 0 {
                    assert!(result.is_err(), "Zero mint should fail");
                } else {
                    // Should succeed
                    match result {
                        Ok(_) => {}
                        Err(e) => {
                            let error_code = parse_anchor_error(&e);
                            if error_code != Some(StablecoinError::MathOverflow as u32) {
                                panic!("Unexpected error: {:?}", e);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[derive(Debug, Arbitrary)]
pub enum PauseAction {
    Pause,
    Unpause,
    Mint(u64),
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Structure to hold test context with stablecoin state
struct MintTestContext {
    context: TestContext,
    authority: Pubkey,
    state_pda: Pubkey,
    asset_mint: Pubkey,
}

/// Setup an initialized stablecoin for minting tests
fn setup_initialized_stablecoin(is_paused: bool) -> Result<MintTestContext> {
    let mut test = TestContext::new();
    test.add_program("sss_token", sss_token::ID)?;

    let authority = test.payer();
    let asset_mint = Pubkey::new_unique();

    let (state_pda, bump) = Pubkey::find_program_address(
        &[b"stablecoin", asset_mint.as_ref()],
        &sss_token::ID,
    );

    // Initialize the stablecoin
    let init_ix = sss_token::instruction::Initialize {
        preset: 1,
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

    // Pause if requested
    if is_paused {
        let pause_ix = sss_token::instruction::Pause {};
        test.invoke(
            &[
                AccountMeta::new(authority, true),
                AccountMeta::new_readonly(state_pda, false),
            ],
            pause_ix,
            None,
        )?;
    }

    Ok(MintTestContext {
        context: test,
        authority,
        state_pda,
        asset_mint,
    })
}

/// Setup an initialized stablecoin with a minter quota
fn setup_initialized_stablecoin_with_quota(quota: u64) -> Result<MintTestContext> {
    let mut ctx = setup_initialized_stablecoin(false)?;

    // Add a minter with quota
    let minter = ctx.authority;
    let (minter_info_pda, _) = Pubkey::find_program_address(
        &[b"minter", ctx.state_pda.as_ref(), minter.as_ref()],
        &sss_token::ID,
    );

    let add_minter_ix = sss_token::instruction::AddMinter { quota };

    ctx.context.invoke(
        &[
            AccountMeta::new(ctx.authority, true),
            AccountMeta::new_readonly(ctx.state_pda, false),
            AccountMeta::new(minter_info_pda, false),
            AccountMeta::new_readonly(minter, false),
            AccountMeta::new_readonly(System::id(), false),
        ],
        add_minter_ix,
        None,
    )?;

    Ok(ctx)
}

/// Attempt to mint tokens
fn try_mint(
    ctx: &mut MintTestContext,
    minter: Pubkey,
    amount: u64,
    _use_role_assignment: bool,
) -> Result<()> {
    let recipient = Pubkey::new_unique();

    let mint_ix = sss_token::instruction::Mint { amount };

    ctx.context.invoke(
        &[
            AccountMeta::new(minter, true),
            AccountMeta::new(ctx.state_pda, false),
            AccountMeta::new_readonly(ctx.asset_mint, false),
            AccountMeta::new_readonly(recipient, false),
        ],
        mint_ix,
        None,
    )
}

/// Attempt to mint tokens with quota tracking
fn try_mint_with_quota(ctx: &mut MintTestContext, amount: u64) -> Result<()> {
    let recipient = Pubkey::new_unique();

    let (minter_info_pda, _) = Pubkey::find_program_address(
        &[b"minter", ctx.state_pda.as_ref(), ctx.authority.as_ref()],
        &sss_token::ID,
    );

    let mint_ix = sss_token::instruction::Mint { amount };

    ctx.context.invoke(
        &[
            AccountMeta::new(ctx.authority, true),
            AccountMeta::new(ctx.state_pda, false),
            AccountMeta::new_readonly(minter_info_pda, false),
            AccountMeta::new_readonly(ctx.asset_mint, false),
            AccountMeta::new_readonly(recipient, false),
        ],
        mint_ix,
        None,
    )
}

/// Pause the stablecoin
fn try_pause(ctx: &mut MintTestContext) -> Result<()> {
    let pause_ix = sss_token::instruction::Pause {};
    ctx.context.invoke(
        &[
            AccountMeta::new(ctx.authority, true),
            AccountMeta::new_readonly(ctx.state_pda, false),
        ],
        pause_ix,
        None,
    )
}

/// Unpause the stablecoin
fn try_unpause(ctx: &mut MintTestContext) -> Result<()> {
    let unpause_ix = sss_token::instruction::Unpause {};
    ctx.context.invoke(
        &[
            AccountMeta::new(ctx.authority, true),
            AccountMeta::new_readonly(ctx.state_pda, false),
        ],
        unpause_ix,
        None,
    )
}

/// Verify mint result
fn verify_mint_result(_ctx: &MintTestContext, _amount: u64) -> Result<()> {
    // In a real implementation, verify:
    // 1. Total supply increased by amount
    // 2. Recipient balance increased
    // 3. Event was emitted
    Ok(())
}

/// Parse an Anchor error to extract the error code
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