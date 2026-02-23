use anchor_lang::prelude::*;
use trident::prelude::*;

#[derive(Debug, Arbitrary)]
pub struct MintInput {
    pub amount: u64,
    pub recipient_idx: u8,
}

#[fuzz]
pub fn fuzz_mint(input: MintInput) -> Result<()> {
    // Setup
    let mut ctx = setup_initialized_stablecoin()?;
    
    // Skip zero amounts
    if input.amount == 0 {
        return Ok(());
    }
    
    // Skip overflow amounts
    if input.amount > u64::MAX / 2 {
        return Ok(());
    }
    
    // Try mint
    let recipient = ctx.accounts[input.recipient_idx as usize % ctx.accounts.len()];
    let result = mint(&mut ctx, recipient.pubkey(), input.amount);
    
    // Should succeed for valid amounts
    assert!(result.is_ok());
    
    // Verify supply increased
    let state = get_stablecoin_state(&ctx)?;
    assert!(state.total_supply >= input.amount);
    
    Ok(())
}

fn setup_initialized_stablecoin() -> Result<TestContext> {
    let mut test = TestContext::new();
    test.add_program("sss_token", sss_token::ID)?;
    
    // Initialize stablecoin
    initialize(&mut test, 1, "Test".to_string(), "TST".to_string(), "uri".to_string(), 6)?;
    
    Ok(test)
}
