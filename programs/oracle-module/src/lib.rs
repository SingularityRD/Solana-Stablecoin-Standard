use anchor_lang::prelude::*;

declare_id!("5ocL9qjJqjJqjJqjJqjJqjJqjJqjJqjJqjJqjJqjJqjJ");

#[program]
pub mod sss_oracle_module {
    use super::*;

    pub fn initialize_price_feed(ctx: Context<InitializePriceFeed>, decimals: u8) -> Result<()> {
        let price_feed = &mut ctx.accounts.price_feed;
        price_feed.authority = ctx.accounts.authority.key();
        price_feed.decimals = decimals;
        price_feed.last_update = Clock::get()?.unix_timestamp;
        price_feed.is_active = true;
        price_feed.bump = ctx.bumps.price_feed;
        Ok(())
    }

    pub fn update_price(ctx: Context<UpdatePrice>, price: u64, confidence: u64) -> Result<()> {
        let price_feed = &mut ctx.accounts.price_feed;
        price_feed.price = price;
        price_feed.confidence = confidence;
        price_feed.last_update = Clock::get()?.unix_timestamp;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializePriceFeed<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + PriceFeed::INIT_SPACE,
        seeds = [b"price_feed"],
        bump
    )]
    pub price_feed: Account<'info, PriceFeed>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdatePrice<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        has_one = authority @ OracleError::Unauthorized,
    )]
    pub price_feed: Account<'info, PriceFeed>,
}

#[account]
#[derive(InitSpace)]
pub struct PriceFeed {
    pub authority: Pubkey,
    pub price: u64,
    pub confidence: u64,
    pub decimals: u8,
    pub last_update: i64,
    pub is_active: bool,
    pub bump: u8,
}

#[error_code]
pub enum OracleError {
    #[msg("Not authorized to update this price feed")]
    Unauthorized,
}
