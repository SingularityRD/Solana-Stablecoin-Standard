use anchor_lang::prelude::*;

declare_id!("5ocL9qjJqjJqjJqjJqjJqjJqjJqjJqjJqjJqjJqjJqjJ");

#[program]
pub mod sss_oracle_module {
    use super::*;

    pub fn initialize_price_feed(ctx: Context<InitializePriceFeed>) -> Result<()> {
        let price_feed = &mut ctx.accounts.price_feed;
        price_feed.last_update = Clock::get()?.unix_timestamp;
        price_feed.is_active = true;
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
    #[account(mut)]
    pub price_feed: Account<'info, PriceFeed>,
}

#[account]
#[derive(InitSpace)]
pub struct PriceFeed {
    pub price: u64,
    pub confidence: u64,
    pub last_update: i64,
    pub is_active: bool,
    pub bump: u8,
}
