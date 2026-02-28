//! Fuzz tests for the Burn instruction
//!
//! Tests various burn scenarios including:
//! - Valid burn amounts
//! - Zero amounts (should fail)
//! - Insufficient balance
//! - Unauthorized burning
//! - Paused state burning
//! - Overflow scenarios

use anchor_lang::prelude::*;
use anchor_lang::solana_program::pubkey::Pubkey;
use sss_token::error::StablecoinError;
use trident::prelude::*;

/// Input structure for fuzz testing the burn instruction
#[derive(Debug, Arbitrary)]
pub struct BurnInput {
    pub mint_amount: u64,
    pub burn_amount: u64,
    pub is_authorized: bool,
    pub is_paused: bool,
}

/// Fuzz test for the burn instruction
#[fuzz]
pub fn fuzz_burn(input: BurnInput) -> Result<()> {
    // Skip invalid mint amounts
    if input.mint_amount == 0 {
        return Ok(());
    }

    // Setup with initial tokens minted
    let mut ctx = setup_stablecoin_with_tokens(input.mint_amount, input.is_paused)?;

    // Determine burner
    let burner = if input.is_authorized {
        ctx.authority.clone()
    } else {
        Pubkey::new_unique()
    };

    // Attempt to burn
    let result = try_burn(&mut ctx, burner, input.burn_amount);

    // Validate result
    if input.burn_amount == 0 {
        assert!(result.is_err(), "Burn with zero amount should fail");
        let error_code = parse_anchor_error(&result.unwrap_err());
        assert_eq!(
            error_code, Some(StablecoinError::ZeroAmount as u32),
            "Expected ZeroAmount error"
        );
    } else if input.is_paused {
        assert!(result.is_err(), "Burn while paused should fail");
        let error_code = parse_anchor_error(&result.unwrap_err());
        assert_eq!(
            error_code, Some(StablecoinError::VaultPaused as u32),
            "Expected VaultPaused error"
        );
    } else if !input.is_authorized {
        assert!(result.is_err(), "Unauthorized burn should fail");
        let error_code = parse_anchor_error(&result.unwrap_err());
        assert_eq!(
            error_code, Some(StablecoinError::Unauthorized as u32),
            "Expected Unauthorized error"
        );
    } else if input.burn_amount > input.mint_amount {
        // Insufficient balance - this would fail at the token program level
        assert!(result.is_err(), "Burn more than balance should fail");
    } else {
        // Valid burn should succeed
        assert!(result.is_ok(), "Valid burn should succeed");
    }

    Ok(())
}

/// Fuzz test for burn amounts
#[fuzz]
pub fn fuzz_burn_amounts(burn_amount: u64) -> Result<()> {
    // Setup with some initial supply
    let initial_supply = 1_000_000u64;
    let mut ctx = setup_stablecoin_with_tokens(initial_supply, false)?;

    let result = try_burn(&mut ctx, ctx.authority.clone(), burn_amount);

    if burn_amount == 0 {
        assert!(result.is_err(), "Zero burn should fail");
        let error_code = parse_anchor_error(&result.unwrap_err());
        assert_eq!(
            error_code, Some(StablecoinError::ZeroAmount as u32),
            "Expected ZeroAmount error"
        );
    } else if burn_amount > initial_supply {
        // Should fail due to insufficient balance
        assert!(result.is_err(), "Burn more than supply should fail");
    } else {
        assert!(result.is_ok(), "Valid burn should succeed");
    }

    Ok(())
}

/// Fuzz test for sequential burn operations
#[fuzz]
pub struct SequentialBurnInput {
    pub initial_supply: u64,
    pub burn_amounts: Vec<u64>,
}

#[fuzz]
pub fn fuzz_sequential_burns(input: SequentialBurnInput) -> Result<()> {
    if input.burn_amounts.is_empty() || input.burn_amounts.len() > 100 {
        return Ok(());
    }

    if input.initial_supply == 0 {
        return Ok(());
    }

    let mut ctx = setup_stablecoin_with_tokens(input.initial_supply, false)?;
    let mut remaining = input.initial_supply;

    for (i, &amount) in input.burn_amounts.iter().enumerate() {
        let result = try_burn(&mut ctx, ctx.authority.clone(), amount);

        if amount == 0 {
            assert!(result.is_err(), "Burn {} with zero should fail", i);
        } else if amount > remaining {
            assert!(result.is_err(), "Burn {} exceeds remaining balance", i);
        } else {
            match result {
                Ok(_) => {
                    remaining -= amount;
                }
                Err(e) => {
                    let error_code = parse_anchor_error(&e);
                    if error_code != Some(StablecoinError::InsufficientBalance as u32)
                        && error_code != Some(StablecoinError::ZeroAmount as u32)
                    {
                        panic!("Unexpected error on burn {}: {:?}", i, e);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Fuzz test for mint then burn sequence
#[derive(Debug, Arbitrary)]
pub struct MintBurnInput {
    pub operations: Vec<MintBurnOp>,
}

#[derive(Debug, Arbitrary)]
pub enum MintBurnOp {
    Mint(u64),
    Burn(u64),
}

#[fuzz]
pub fn fuzz_mint_burn_sequence(input: MintBurnInput) -> Result<()> {
    if input.operations.is_empty() || input.operations.len() > 100 {
        return Ok(());
    }

    let mut ctx = setup_initialized_stablecoin()?;
    let mut total_supply: u64 = 0;

    for (i, op) in input.operations.iter().enumerate() {
        match op {
            MintBurnOp::Mint(amount) => {
                let result = try_mint(&mut ctx, ctx.authority.clone(), *amount);
                if *amount == 0 {
                    assert!(result.is_err(), "Mint {} with zero should fail", i);
                } else {
                    match result {
                        Ok(_) => {
                            total_supply = total_supply.saturating_add(*amount);
                        }
                        Err(e) => {
                            let error_code = parse_anchor_error(&e);
                            if error_code != Some(StablecoinError::MathOverflow as u32) {
                                panic!("Unexpected error on mint {}: {:?}", i, e);
                            }
                        }
                    }
                }
            }
            MintBurnOp::Burn(amount) => {
                if total_supply == 0 {
                    // Can't burn anything
                    continue;
                }
                let result = try_burn(&mut ctx, ctx.authority.clone(), *amount);
                if *amount == 0 {
                    assert!(result.is_err(), "Burn {} with zero should fail", i);
                } else if *amount <= total_supply {
                    match result {
                        Ok(_) => {
                            total_supply -= amount;
                        }
                        Err(e) => {
                            let error_code = parse_anchor_error(&e);
                            if error_code != Some(StablecoinError::InsufficientBalance as u32) {
                                panic!("Unexpected error on burn {}: {:?}", i, e);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Fuzz test for burn after pause/unpause
#[fuzz]
pub fn fuzz_burn_pause_sequence(actions: Vec<BurnPauseAction>) -> Result<()> {
    if actions.is_empty() || actions.len() > 50 {
        return Ok(());
    }

    let mut ctx = setup_stablecoin_with_tokens(1_000_000, false)?;
    let mut is_paused = false;

    for (i, action) in actions.iter().enumerate() {
        match action {
            BurnPauseAction::Pause => {
                try_pause(&mut ctx)?;
                is_paused = true;
            }
            BurnPauseAction::Unpause => {
                try_unpause(&mut ctx)?;
                is_paused = false;
            }
            BurnPauseAction::Burn(amount) => {
                let result = try_burn(&mut ctx, ctx.authority.clone(), *amount);

                if is_paused {
                    assert!(result.is_err(), "Burn {} should fail while paused", i);
                    if *amount != 0 {
                        let error_code = parse_anchor_error(&result.unwrap_err());
                        assert_eq!(
                            error_code, Some(StablecoinError::VaultPaused as u32),
                            "Expected VaultPaused error"
                        );
                    }
                } else if *amount == 0 {
                    assert!(result.is_err(), "Zero burn should fail");
                }
            }
        }
    }

    Ok(())
}

#[derive(Debug, Arbitrary)]
pub enum BurnPauseAction {
    Pause,
    Unpause,
    Burn(u64),
}

// ============================================================================
// Helper Functions
// ============================================================================

struct BurnTestContext {
    context: TestContext,
    authority: Pubkey,
    state_pda: Pubkey,
    asset_mint: Pubkey,
}

fn setup_initialized_stablecoin() -> Result<BurnTestContext> {
    let mut test = TestContext::new();
    test.add_program("sss_token", sss_token::ID)?;

    let authority = test.payer();
    let asset_mint = Pubkey::new_unique();

    let (state_pda, bump) = Pubkey::find_program_address(
        &[b"stablecoin", asset_mint.as_ref()],
        &sss_token::ID,
    );

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

    Ok(BurnTestContext {
        context: test,
        authority,
        state_pda,
        asset_mint,
    })
}

fn setup_stablecoin_with_tokens(initial_supply: u64, is_paused: bool) -> Result<BurnTestContext> {
    let mut ctx = setup_initialized_stablecoin()?;

    // Mint initial tokens
    if initial_supply > 0 {
        try_mint(&mut ctx, ctx.authority.clone(), initial_supply)?;
    }

    // Pause if requested
    if is_paused {
        try_pause(&mut ctx)?;
    }

    Ok(ctx)
}

fn try_mint(ctx: &mut BurnTestContext, minter: Pubkey, amount: u64) -> Result<()> {
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

fn try_burn(ctx: &mut BurnTestContext, burner: Pubkey, amount: u64) -> Result<()> {
    let from = Pubkey::new_unique();
    let burn_ix = sss_token::instruction::Burn { amount };

    ctx.context.invoke(
        &[
            AccountMeta::new(burner, true),
            AccountMeta::new(ctx.state_pda, false),
            AccountMeta::new_readonly(ctx.asset_mint, false),
            AccountMeta::new_readonly(from, false),
        ],
        burn_ix,
        None,
    )
}

fn try_pause(ctx: &mut BurnTestContext) -> Result<()> {
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

fn try_unpause(ctx: &mut BurnTestContext) -> Result<()> {
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
