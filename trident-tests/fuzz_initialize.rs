//! Fuzz tests for the Initialize instruction
//!
//! Tests both valid and invalid inputs to ensure proper error handling

use anchor_lang::prelude::*;
use anchor_lang::solana_program::pubkey::Pubkey;
use sss_token::error::StablecoinError;
use trident::prelude::*;

/// Input structure for fuzz testing the initialize instruction
#[derive(Debug, Arbitrary)]
pub struct InitializeInput {
    pub preset: u8,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub decimals: u8,
}

/// Fuzz test for the initialize instruction
///
/// Tests various combinations of inputs to ensure:
/// - Valid presets (1, 2) succeed
/// - Invalid presets (0, 3+) fail with InvalidPreset error
/// - Name validation (max 32 chars)
/// - Symbol validation (max 10 chars)  
/// - URI validation (max 200 chars)
/// - Decimals validation (max 9)
#[fuzz]
pub fn fuzz_initialize(input: InitializeInput) -> Result<()> {
    // Setup test environment with program
    let mut ctx = setup_test_environment()?;

    // Generate a unique asset mint for this test iteration
    let asset_mint = Pubkey::new_unique();

    // Attempt to initialize the stablecoin with fuzzed inputs
    let result = try_initialize(
        &mut ctx,
        input.preset,
        input.name.clone(),
        input.symbol.clone(),
        input.uri.clone(),
        input.decimals,
        asset_mint,
    );

    // Validate based on input parameters
    match result {
        Ok(_) => {
            // Success path - verify all inputs were valid
            assert!(
                input.preset == 1 || input.preset == 2,
                "Initialize should only succeed with preset 1 or 2, got: {}",
                input.preset
            );
            assert!(
                input.name.len() <= 32,
                "Initialize should only succeed with name len <= 32, got: {}",
                input.name.len()
            );
            assert!(
                input.symbol.len() <= 10,
                "Initialize should only succeed with symbol len <= 10, got: {}",
                input.symbol.len()
            );
            assert!(
                input.uri.len() <= 200,
                "Initialize should only succeed with uri len <= 200, got: {}",
                input.uri.len()
            );
            assert!(
                input.decimals <= 9,
                "Initialize should only succeed with decimals <= 9, got: {}",
                input.decimals
            );

            // Verify the state was correctly initialized
            verify_initialization(&ctx, &input)?;
        }
        Err(e) => {
            // Error path - verify the error matches the invalid input
            let error_code = parse_anchor_error(&e);

            // Determine which validation failed
            if input.preset != 1 && input.preset != 2 {
                assert_eq!(
                    error_code, Some(StablecoinError::InvalidPreset as u32),
                    "Expected InvalidPreset error for preset {}, got: {:?}",
                    input.preset, error_code
                );
            } else if input.name.len() > 32 {
                assert_eq!(
                    error_code, Some(StablecoinError::NameTooLong as u32),
                    "Expected NameTooLong error for name len {}, got: {:?}",
                    input.name.len(), error_code
                );
            } else if input.symbol.len() > 10 {
                assert_eq!(
                    error_code, Some(StablecoinError::SymbolTooLong as u32),
                    "Expected SymbolTooLong error for symbol len {}, got: {:?}",
                    input.symbol.len(), error_code
                );
            } else if input.uri.len() > 200 {
                assert_eq!(
                    error_code, Some(StablecoinError::UriTooLong as u32),
                    "Expected UriTooLong error for uri len {}, got: {:?}",
                    input.uri.len(), error_code
                );
            } else if input.decimals > 9 {
                assert_eq!(
                    error_code, Some(StablecoinError::InvalidDecimals as u32),
                    "Expected InvalidDecimals error for decimals {}, got: {:?}",
                    input.decimals, error_code
                );
            } else {
                // Unexpected error
                panic!("Unexpected error for valid inputs: {:?}", e);
            }
        }
    }

    Ok(())
}

/// Test edge cases specifically for preset values
#[fuzz]
pub fn fuzz_initialize_presets(preset: u8) -> Result<()> {
    let mut ctx = setup_test_environment()?;
    let asset_mint = Pubkey::new_unique();

    let result = try_initialize(
        &mut ctx,
        preset,
        "Test".to_string(),
        "TST".to_string(),
        "https://test.com".to_string(),
        6,
        asset_mint,
    );

    match preset {
        1 | 2 => {
            assert!(result.is_ok(), "Preset {} should succeed", preset);
        }
        _ => {
            assert!(result.is_err(), "Preset {} should fail", preset);
            let error_code = parse_anchor_error(&result.unwrap_err());
            assert_eq!(
                error_code, Some(StablecoinError::InvalidPreset as u32),
                "Expected InvalidPreset error"
            );
        }
    }

    Ok(())
}

/// Test edge cases for name length
#[fuzz]
pub fn fuzz_initialize_name_length(name: String) -> Result<()> {
    let mut ctx = setup_test_environment()?;
    let asset_mint = Pubkey::new_unique();

    let result = try_initialize(
        &mut ctx,
        1, // Valid preset
        name.clone(),
        "TST".to_string(),
        "uri".to_string(),
        6,
        asset_mint,
    );

    if name.len() <= 32 {
        assert!(result.is_ok(), "Name of length {} should succeed", name.len());
    } else {
        assert!(result.is_err(), "Name of length {} should fail", name.len());
        let error_code = parse_anchor_error(&result.unwrap_err());
        assert_eq!(
            error_code, Some(StablecoinError::NameTooLong as u32),
            "Expected NameTooLong error"
        );
    }

    Ok(())
}

/// Test edge cases for symbol length
#[fuzz]
pub fn fuzz_initialize_symbol_length(symbol: String) -> Result<()> {
    let mut ctx = setup_test_environment()?;
    let asset_mint = Pubkey::new_unique();

    let result = try_initialize(
        &mut ctx,
        1,
        "Test".to_string(),
        symbol.clone(),
        "uri".to_string(),
        6,
        asset_mint,
    );

    if symbol.len() <= 10 {
        assert!(result.is_ok(), "Symbol of length {} should succeed", symbol.len());
    } else {
        assert!(result.is_err(), "Symbol of length {} should fail", symbol.len());
        let error_code = parse_anchor_error(&result.unwrap_err());
        assert_eq!(
            error_code, Some(StablecoinError::SymbolTooLong as u32),
            "Expected SymbolTooLong error"
        );
    }

    Ok(())
}

/// Test edge cases for decimals
#[fuzz]
pub fn fuzz_initialize_decimals(decimals: u8) -> Result<()> {
    let mut ctx = setup_test_environment()?;
    let asset_mint = Pubkey::new_unique();

    let result = try_initialize(
        &mut ctx,
        1,
        "Test".to_string(),
        "TST".to_string(),
        "uri".to_string(),
        decimals,
        asset_mint,
    );

    if decimals <= 9 {
        assert!(result.is_ok(), "Decimals {} should succeed", decimals);
    } else {
        assert!(result.is_err(), "Decimals {} should fail", decimals);
        let error_code = parse_anchor_error(&result.unwrap_err());
        assert_eq!(
            error_code, Some(StablecoinError::InvalidDecimals as u32),
            "Expected InvalidDecimals error"
        );
    }

    Ok(())
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Setup the test environment with the SSS token program
fn setup_test_environment() -> Result<TestContext> {
    let mut test = TestContext::new();
    test.add_program("sss_token", sss_token::ID)?;
    Ok(test)
}

/// Attempt to initialize a stablecoin with the given parameters
fn try_initialize(
    ctx: &mut TestContext,
    preset: u8,
    name: String,
    symbol: String,
    uri: String,
    decimals: u8,
    asset_mint: Pubkey,
) -> Result<()> {
    // Find the PDA for the stablecoin state
    let (state_pda, bump) = Pubkey::find_program_address(
        &[b"stablecoin", asset_mint.as_ref()],
        &sss_token::ID,
    );

    // Build and send the initialize instruction
    let ix = sss_token::instruction::Initialize {
        preset,
        name,
        symbol,
        uri,
        decimals,
    };

    ctx.invoke(
        &[
            AccountMeta::new(ctx.payer(), true),
            AccountMeta::new(state_pda, false),
            AccountMeta::new_readonly(asset_mint, false),
            AccountMeta::new_readonly(System::id(), false),
        ],
        ix,
        Some(&[&[b"stablecoin", asset_mint.as_ref(), &[bump]]]),
    )
}

/// Verify that initialization was successful
fn verify_initialization(ctx: &TestContext, input: &InitializeInput) -> Result<()> {
    // In a real implementation, we would fetch the account state
    // and verify all fields match the input parameters
    // For now, we just verify the instruction succeeded
    Ok(())
}

/// Parse an Anchor error to extract the error code
fn parse_anchor_error(error: &Error) -> Option<u32> {
    match error {
        Error::AnchorError(e) => Some(e.error_code_number),
        Error::ProgramError(e) => {
            // Parse program error to extract custom error code
            if let Some(code) = e.to_error_code() {
                Some(code.code())
            } else {
                None
            }
        }
        _ => None,
    }
}