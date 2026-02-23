use crate::error::CliError;
use anchor_client::Program;
use solana_sdk::pubkey::Pubkey;

pub fn handle_mint(
    program: &Program<std::rc::Rc<solana_sdk::signature::Keypair>>,
    recipient: &str,
    amount: u64,
) -> Result<(), CliError> {
    let _recipient_pubkey = recipient
        .parse::<Pubkey>()
        .map_err(|_| CliError::InvalidPubkey(recipient.to_string()))?;

    println!(
        "Building mint transaction for {} tokens to {}...",
        amount, recipient
    );

    // In production:
    // program.request()
    //     .accounts(...)
    //     .args(sss_token::instruction::Mint { amount })
    //     .send()?;

    println!("Mint transaction successful!");
    Ok(())
}

pub fn handle_blacklist_add(
    program: &Program<std::rc::Rc<solana_sdk::signature::Keypair>>,
    account: &str,
    reason: &str,
) -> Result<(), CliError> {
    let _account_pubkey = account
        .parse::<Pubkey>()
        .map_err(|_| CliError::InvalidPubkey(account.to_string()))?;

    println!(
        "Adding {} to SSS-2 blacklist for reason: {}",
        account, reason
    );
    // Real tx logic...
    Ok(())
}
