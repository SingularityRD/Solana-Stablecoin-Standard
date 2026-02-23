use anchor_lang::prelude::*;
use anchor_lang::solana_program::pubkey::Pubkey;
use trident::prelude::*;

#[derive(Debug, Arbitrary)]
pub struct InitializeInput {
    pub preset: u8,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub decimals: u8,
}

#[fuzz]
pub fn fuzz_initialize(input: InitializeInput) -> Result<()> {
    // Setup test environment
    let mut ctx = setup_test_environment()?;
    
    // Only test valid presets
    if input.preset != 1 && input.preset != 2 {
        return Ok(());
    }
    
    // Validate lengths
    if input.name.len() > 32 || input.symbol.len() > 16 || input.uri.len() > 200 {
        return Ok(());
    }
    
    // Validate decimals
    if input.decimals > 9 {
        return Ok(());
    }
    
    // Try to initialize
    let result = initialize(
        &mut ctx,
        input.preset,
        input.name.clone(),
        input.symbol.clone(),
        input.uri.clone(),
        input.decimals,
    );
    
    // Should succeed for valid inputs
    if input.preset == 1 || input.preset == 2 {
        assert!(result.is_ok());
    }
    
    Ok(())
}

fn setup_test_environment() -> Result<TestContext> {
    // Setup test context
    let mut test = TestContext::new();
    test.add_program("sss_token", sss_token::ID)?;
    Ok(test)
}
