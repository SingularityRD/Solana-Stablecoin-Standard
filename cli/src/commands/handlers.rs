//! Command handlers for the SSS Token CLI
//!
//! This module implements the actual logic for each CLI command,
//! interacting with the Solana program via the Anchor client.

use anchor_client::Program;
use anchor_lang::prelude::*;
use solana_sdk::{
    account::Account as SolanaAccount,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    system_program,
};
use std::rc::Rc;

use crate::error::CliError;
use crate::instructions::*;
use crate::{BLACKLIST_SEED, MINTER_SEED, ROLE_SEED, STABLECOIN_SEED};

// Define a custom Result type to avoid conflict with anchor_lang::prelude::Result
type CliResult<T> = std::result::Result<T, CliError>;

// Role enum re-export for convenience
pub use crate::instructions::Role;

// PDA derivation helpers matching the program's constants
fn derive_stablecoin_pda(asset_mint: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[STABLECOIN_SEED, asset_mint.to_bytes().as_ref()],
        program_id,
    )
}

fn derive_role_pda(stablecoin: &Pubkey, account: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            ROLE_SEED,
            stablecoin.to_bytes().as_ref(),
            account.to_bytes().as_ref(),
        ],
        program_id,
    )
}

fn derive_minter_pda(stablecoin: &Pubkey, minter: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            MINTER_SEED,
            stablecoin.to_bytes().as_ref(),
            minter.to_bytes().as_ref(),
        ],
        program_id,
    )
}

fn derive_blacklist_pda(
    stablecoin: &Pubkey,
    account: &Pubkey,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            BLACKLIST_SEED,
            stablecoin.to_bytes().as_ref(),
            account.to_bytes().as_ref(),
        ],
        program_id,
    )
}

fn fetch_asset_mint(program: &Program<Rc<Keypair>>, stablecoin_pda: &Pubkey) -> CliResult<Pubkey> {
    let data = program
        .rpc()
        .get_account_data(stablecoin_pda)
        .map_err(|e| {
            CliError::TransactionError(format!("Failed to fetch stablecoin state: {}", e))
        })?;
    if data.len() < 72 {
        return Err(CliError::AccountNotFound(
            "Stablecoin state not initialized or too short".to_string(),
        ));
    }
    match StablecoinStateData::try_from_slice(&data[8..]) {
        Ok(state) => Ok(state.asset_mint),
        Err(e) => Err(CliError::Unknown(format!(
            "Failed to parse stablecoin state: {}",
            e
        ))),
    }
}

fn parse_pubkey(s: &str) -> CliResult<Pubkey> {
    s.parse::<Pubkey>()
        .map_err(|_| CliError::InvalidPubkey(s.to_string()))
}

fn print_tx_success(signature: &str, action: &str) {
    println!("✅ {} successful!", action);
    println!("   Transaction: {}", signature);
    println!("   Explorer: https://explorer.solana.com/tx/{}", signature);
}

// ==================== INIT ====================
pub fn handle_init(
    program: &Program<Rc<Keypair>>,
    authority: &Pubkey,
    preset: u8,
    name: String,
    symbol: String,
    uri: String,
    decimals: u8,
    asset_mint: Option<String>,
) -> CliResult<()> {
    println!("🚀 Initializing stablecoin...");
    println!("   Preset: SSS-{}", preset);
    println!("   Name: {}", name);
    println!("   Symbol: {}", symbol);
    println!("   Decimals: {}", decimals);

    // Validate preset
    if preset != 1 && preset != 2 {
        return Err(CliError::InvalidArg(
            "Preset must be 1 (SSS-1) or 2 (SSS-2)".to_string(),
        ));
    }

    // Validate lengths
    if name.len() > 32 {
        return Err(CliError::InvalidArg(
            "Name too long (max 32 chars)".to_string(),
        ));
    }
    if symbol.len() > 10 {
        return Err(CliError::InvalidArg(
            "Symbol too long (max 10 chars)".to_string(),
        ));
    }
    if uri.len() > 200 {
        return Err(CliError::InvalidArg(
            "URI too long (max 200 chars)".to_string(),
        ));
    }
    if decimals > 9 {
        return Err(CliError::InvalidArg("Decimals must be <= 9".to_string()));
    }

    let program_id = program.id();

    // Create or use provided asset mint
    let asset_mint_pubkey = match asset_mint {
        Some(mint) => parse_pubkey(&mint)?,
        None => {
            println!("   Note: No asset_mint provided, using authority as asset_mint reference");
            *authority
        }
    };

    let (stablecoin_pda, bump) = derive_stablecoin_pda(&asset_mint_pubkey, &program_id);

    println!("   Stablecoin PDA: {}", stablecoin_pda);
    println!("   Bump: {}", bump);

    // Build accounts for Initialize instruction
    let accounts = vec![
        AccountMeta::new(*authority, true),      // authority (signer, mut)
        AccountMeta::new(stablecoin_pda, false), // state (PDA, init)
        AccountMeta::new_readonly(asset_mint_pubkey, false), // asset_mint
        AccountMeta::new_readonly(system_program::id(), false), // system_program
    ];

    // Build instruction data
    let ix_data = build_instruction_data(
        "initialize",
        InitializeArgs {
            preset,
            name,
            symbol,
            uri,
            decimals,
        },
    )
    .map_err(|e| CliError::SerializationError(e.to_string()))?;

    // Create instruction
    let ix = Instruction {
        program_id,
        accounts,
        data: ix_data,
    };

    // Send transaction
    let signature = program
        .request()
        .instruction(ix)
        .send()
        .map_err(|e| CliError::TransactionError(e.to_string()))?;

    print_tx_success(&signature.to_string(), "Initialization");

    println!("\n💡 Save this stablecoin address for future commands:");
    println!("   --stablecoin {}", stablecoin_pda);

    Ok(())
}

// ==================== MINT ====================
pub fn handle_mint(
    program: &Program<Rc<Keypair>>,
    authority: &Pubkey,
    recipient: &str,
    amount: u64,
    stablecoin: Option<&Pubkey>,
) -> CliResult<()> {
    let recipient_pubkey = parse_pubkey(recipient)?;

    println!("铸造 Minting {} tokens to {}", amount, recipient_pubkey);

    if amount == 0 {
        return Err(CliError::InvalidArg(
            "Amount must be greater than zero".to_string(),
        ));
    }

    let program_id = program.id();

    // Stablecoin PDA must be provided or derived from asset_mint
    let stablecoin_pda = match stablecoin {
        Some(s) => *s,
        None => {
            return Err(CliError::InvalidArg(
                "Stablecoin PDA is required. Use --stablecoin <address>".to_string(),
            ));
        }
    };

    // Derive role PDA for the authority
    let (role_pda, _) = derive_role_pda(&stablecoin_pda, authority, &program_id);
    let (minter_pda, _) = derive_minter_pda(&stablecoin_pda, authority, &program_id);
    let asset_mint = fetch_asset_mint(program, &stablecoin_pda)?;

    // Build accounts for Mint instruction
    let accounts = vec![
        AccountMeta::new(*authority, true),      // authority (signer, mut)
        AccountMeta::new(stablecoin_pda, false), // state (PDA, mut)
        AccountMeta::new_readonly(role_pda, false), // role_assignment (optional)
        AccountMeta::new(minter_pda, false),
        AccountMeta::new(asset_mint, false), // asset_mint (mut)
        AccountMeta::new(recipient_pubkey, false), // recipient (mut)
        AccountMeta::new_readonly(spl_token_2022::id(), false), // token_program
    ];

    let ix_data = build_instruction_data("mint", MintArgs { amount })
        .map_err(|e| CliError::SerializationError(e.to_string()))?;

    let ix = Instruction {
        program_id,
        accounts,
        data: ix_data,
    };

    let signature = program
        .request()
        .instruction(ix)
        .send()
        .map_err(|e| CliError::TransactionError(e.to_string()))?;

    print_tx_success(&signature.to_string(), "Mint");
    Ok(())
}

// ==================== BURN ====================
pub fn handle_burn(
    program: &Program<Rc<Keypair>>,
    authority: &Pubkey,
    amount: u64,
    from: Option<&Pubkey>,
    stablecoin: Option<&Pubkey>,
) -> CliResult<()> {
    println!("🔥 Burning {} tokens", amount);

    if amount == 0 {
        return Err(CliError::InvalidArg(
            "Amount must be greater than zero".to_string(),
        ));
    }

    let program_id = program.id();
    let stablecoin_pda = match stablecoin {
        Some(s) => *s,
        None => {
            return Err(CliError::InvalidArg(
                "Stablecoin PDA is required. Use --stablecoin <address>".to_string(),
            ));
        }
    };

    let from_pubkey = from.unwrap_or(authority);
    let (role_pda, _) = derive_role_pda(&stablecoin_pda, authority, &program_id);
    let asset_mint = fetch_asset_mint(program, &stablecoin_pda)?;

    let accounts = vec![
        AccountMeta::new(*authority, true),      // authority (signer, mut)
        AccountMeta::new(stablecoin_pda, false), // state (PDA, mut)
        AccountMeta::new_readonly(role_pda, false), // role_assignment (optional)
        AccountMeta::new(asset_mint, false),     // asset_mint (mut)
        AccountMeta::new(*from_pubkey, false),   // from (token account)
        AccountMeta::new_readonly(spl_token_2022::id(), false), // token_program
    ];

    let ix_data = build_instruction_data("burn", BurnArgs { amount })
        .map_err(|e| CliError::SerializationError(e.to_string()))?;

    let ix = Instruction {
        program_id,
        accounts,
        data: ix_data,
    };

    let signature = program
        .request()
        .instruction(ix)
        .send()
        .map_err(|e| CliError::TransactionError(e.to_string()))?;

    print_tx_success(&signature.to_string(), "Burn");
    Ok(())
}

// ==================== FREEZE ====================
pub fn handle_freeze(
    program: &Program<Rc<Keypair>>,
    authority: &Pubkey,
    account: &str,
    stablecoin: Option<&Pubkey>,
) -> CliResult<()> {
    let account_pubkey = parse_pubkey(account)?;

    println!("❄️ Freezing account: {}", account_pubkey);

    let program_id = program.id();
    let stablecoin_pda = match stablecoin {
        Some(s) => *s,
        None => {
            return Err(CliError::InvalidArg(
                "Stablecoin PDA is required. Use --stablecoin <address>".to_string(),
            ));
        }
    };
    let (role_pda, _) = derive_role_pda(&stablecoin_pda, authority, &program_id);
    let asset_mint = fetch_asset_mint(program, &stablecoin_pda)?;

    let accounts = vec![
        AccountMeta::new(*authority, true), // authority (signer, mut)
        AccountMeta::new_readonly(stablecoin_pda, false), // state (PDA)
        AccountMeta::new_readonly(role_pda, false), // role_assignment (optional)
        AccountMeta::new(asset_mint, false), // asset_mint (mut)
        AccountMeta::new(account_pubkey, false), // account to freeze
        AccountMeta::new_readonly(spl_token_2022::id(), false), // token_program
    ];

    let ix_data = build_instruction_data("freeze_account", FreezeArgs {})
        .map_err(|e| CliError::SerializationError(e.to_string()))?;

    let ix = Instruction {
        program_id,
        accounts,
        data: ix_data,
    };

    let signature = program
        .request()
        .instruction(ix)
        .send()
        .map_err(|e| CliError::TransactionError(e.to_string()))?;

    print_tx_success(&signature.to_string(), "Freeze");
    Ok(())
}

// ==================== THAW ====================
pub fn handle_thaw(
    program: &Program<Rc<Keypair>>,
    authority: &Pubkey,
    account: &str,
    stablecoin: Option<&Pubkey>,
) -> CliResult<()> {
    let account_pubkey = parse_pubkey(account)?;

    println!("🔥 Thawing account: {}", account_pubkey);

    let program_id = program.id();
    let stablecoin_pda = match stablecoin {
        Some(s) => *s,
        None => {
            return Err(CliError::InvalidArg(
                "Stablecoin PDA is required. Use --stablecoin <address>".to_string(),
            ));
        }
    };
    let (role_pda, _) = derive_role_pda(&stablecoin_pda, authority, &program_id);
    let asset_mint = fetch_asset_mint(program, &stablecoin_pda)?;

    let accounts = vec![
        AccountMeta::new_readonly(*authority, true),
        AccountMeta::new_readonly(stablecoin_pda, false), // state (PDA)
        AccountMeta::new_readonly(role_pda, false),       // role_assignment (optional)
        AccountMeta::new(asset_mint, false),              // asset_mint (mut)
        AccountMeta::new(account_pubkey, false),          // account to thaw
        AccountMeta::new_readonly(spl_token_2022::id(), false), // token_program
    ];

    let ix_data = build_instruction_data("thaw_account", ThawArgs {})
        .map_err(|e| CliError::SerializationError(e.to_string()))?;

    let ix = Instruction {
        program_id,
        accounts,
        data: ix_data,
    };

    let signature = program
        .request()
        .instruction(ix)
        .send()
        .map_err(|e| CliError::TransactionError(e.to_string()))?;

    print_tx_success(&signature.to_string(), "Thaw");
    Ok(())
}

// ==================== PAUSE ====================
pub fn handle_pause(
    program: &Program<Rc<Keypair>>,
    authority: &Pubkey,
    stablecoin: Option<&Pubkey>,
) -> CliResult<()> {
    println!("⏸️ Pausing stablecoin operations...");

    let program_id = program.id();
    let stablecoin_pda = match stablecoin {
        Some(s) => *s,
        None => {
            return Err(CliError::InvalidArg(
                "Stablecoin PDA is required. Use --stablecoin <address>".to_string(),
            ));
        }
    };

    let accounts = vec![
        AccountMeta::new(*authority, true),      // authority (signer, mut)
        AccountMeta::new(stablecoin_pda, false), // state (PDA)
    ];

    let ix_data = build_instruction_data("pause", Pause {})
        .map_err(|e| CliError::SerializationError(e.to_string()))?;

    let ix = Instruction {
        program_id,
        accounts,
        data: ix_data,
    };

    let signature = program
        .request()
        .instruction(ix)
        .send()
        .map_err(|e| CliError::TransactionError(e.to_string()))?;

    print_tx_success(&signature.to_string(), "Pause");
    Ok(())
}

// ==================== UNPAUSE ====================
pub fn handle_unpause(
    program: &Program<Rc<Keypair>>,
    authority: &Pubkey,
    stablecoin: Option<&Pubkey>,
) -> CliResult<()> {
    println!("▶️ Unpausing stablecoin operations...");

    let program_id = program.id();
    let stablecoin_pda = match stablecoin {
        Some(s) => *s,
        None => {
            return Err(CliError::InvalidArg(
                "Stablecoin PDA is required. Use --stablecoin <address>".to_string(),
            ));
        }
    };

    let accounts = vec![
        AccountMeta::new(*authority, true),      // authority (signer, mut)
        AccountMeta::new(stablecoin_pda, false), // state (PDA)
    ];

    let ix_data = build_instruction_data("unpause", Unpause {})
        .map_err(|e| CliError::SerializationError(e.to_string()))?;

    let ix = Instruction {
        program_id,
        accounts,
        data: ix_data,
    };

    let signature = program
        .request()
        .instruction(ix)
        .send()
        .map_err(|e| CliError::TransactionError(e.to_string()))?;

    print_tx_success(&signature.to_string(), "Unpause");
    Ok(())
}

// ==================== BLACKLIST ====================
pub fn handle_blacklist_add(
    program: &Program<Rc<Keypair>>,
    authority: &Pubkey,
    account: &str,
    reason: &str,
    stablecoin: Option<&Pubkey>,
) -> CliResult<()> {
    let account_pubkey = parse_pubkey(account)?;

    println!("🚫 Adding {} to blacklist", account_pubkey);
    println!("   Reason: {}", reason);

    let program_id = program.id();
    let stablecoin_pda = match stablecoin {
        Some(s) => *s,
        None => {
            return Err(CliError::InvalidArg(
                "Stablecoin PDA is required. Use --stablecoin <address>".to_string(),
            ));
        }
    };

    let (entry_pda, _) = derive_blacklist_pda(&stablecoin_pda, &account_pubkey, &program_id);
    let (role_pda, _) = derive_role_pda(&stablecoin_pda, authority, &program_id);

    let accounts = vec![
        AccountMeta::new(*authority, true),      // authority (signer, mut)
        AccountMeta::new(stablecoin_pda, false), // state (PDA)
        AccountMeta::new_readonly(role_pda, false), // role_assignment (optional)
        AccountMeta::new(entry_pda, false),      // entry (PDA)
        AccountMeta::new_readonly(account_pubkey, false), // account to blacklist
        AccountMeta::new_readonly(system_program::id(), false), // system_program
    ];

    let ix_data = build_instruction_data(
        "add_to_blacklist",
        AddToBlacklist {
            reason: reason.to_string(),
        },
    )
    .map_err(|e| CliError::SerializationError(e.to_string()))?;

    let ix = Instruction {
        program_id,
        accounts,
        data: ix_data,
    };

    let signature = program
        .request()
        .instruction(ix)
        .send()
        .map_err(|e| CliError::TransactionError(e.to_string()))?;

    print_tx_success(&signature.to_string(), "Blacklist add");
    Ok(())
}

pub fn handle_blacklist_remove(
    program: &Program<Rc<Keypair>>,
    authority: &Pubkey,
    account: &str,
    stablecoin: Option<&Pubkey>,
) -> CliResult<()> {
    let account_pubkey = parse_pubkey(account)?;

    println!("✅ Removing {} from blacklist", account_pubkey);

    let program_id = program.id();
    let stablecoin_pda = match stablecoin {
        Some(s) => *s,
        None => {
            return Err(CliError::InvalidArg(
                "Stablecoin PDA is required. Use --stablecoin <address>".to_string(),
            ));
        }
    };

    let (entry_pda, _) = derive_blacklist_pda(&stablecoin_pda, &account_pubkey, &program_id);
    let (role_pda, _) = derive_role_pda(&stablecoin_pda, authority, &program_id);

    let accounts = vec![
        AccountMeta::new(*authority, true),      // authority (signer, mut)
        AccountMeta::new(stablecoin_pda, false), // state (PDA)
        AccountMeta::new_readonly(role_pda, false), // role_assignment (optional)
        AccountMeta::new(entry_pda, false),      // entry (PDA)
        AccountMeta::new_readonly(account_pubkey, false), // account to unblacklist
        AccountMeta::new_readonly(system_program::id(), false), // system_program
    ];

    let ix_data = build_instruction_data("remove_from_blacklist", RemoveFromBlacklist {})
        .map_err(|e| CliError::SerializationError(e.to_string()))?;

    let ix = Instruction {
        program_id,
        accounts,
        data: ix_data,
    };

    let signature = program
        .request()
        .instruction(ix)
        .send()
        .map_err(|e| CliError::TransactionError(e.to_string()))?;

    print_tx_success(&signature.to_string(), "Blacklist remove");
    Ok(())
}

pub fn handle_blacklist_list(
    _program: &Program<Rc<Keypair>>,
    authority: &Pubkey,
    stablecoin: Option<&Pubkey>,
) -> CliResult<()> {
    println!("📋 Listing blacklisted accounts...");

    let program_id = _program.id();
    let stablecoin_pda = stablecoin
        .copied()
        .unwrap_or_else(|| derive_stablecoin_pda(authority, &program_id).0);

    println!("   Stablecoin: {}", stablecoin_pda);
    println!("   Note: Use an indexer service to list all blacklist entries");
    println!("   Or check individual accounts with: sss-token blacklist check <account>");

    Ok(())
}

pub fn handle_blacklist_check(
    program: &Program<Rc<Keypair>>,
    authority: &Pubkey,
    account: &str,
    stablecoin: Option<&Pubkey>,
) -> CliResult<()> {
    let account_pubkey = parse_pubkey(account)?;

    println!("🔍 Checking blacklist status for {}", account_pubkey);

    let program_id = program.id();
    let stablecoin_pda = match stablecoin {
        Some(s) => *s,
        None => {
            return Err(CliError::InvalidArg(
                "Stablecoin PDA is required. Use --stablecoin <address>".to_string(),
            ));
        }
    };

    let (entry_pda, _bump) = derive_blacklist_pda(&stablecoin_pda, &account_pubkey, &program_id);

    // Try to fetch the blacklist entry account using RPC
    let account_data = program.rpc().get_account_data(&entry_pda);
    match account_data {
        Ok(data) => {
            // Skip 8-byte discriminator
            if data.len() > 8 {
                match BlacklistEntryData::try_from_slice(&data[8..]) {
                    Ok(entry) => {
                        println!("🚫 Account IS blacklisted");
                        println!("   Reason: {}", entry.reason);
                        println!("   Blacklisted by: {}", entry.blacklisted_by);
                        println!("   At: {}", entry.blacklisted_at);
                    }
                    Err(_) => {
                        println!("⚠️ Could not parse blacklist entry");
                    }
                }
            } else {
                println!("✅ Account is NOT blacklisted");
            }
        }
        Err(_) => {
            println!("✅ Account is NOT blacklisted");
        }
    }

    Ok(())
}

// BlacklistEntryData for deserialization
#[derive(Debug, ::borsh::BorshDeserialize)]
struct BlacklistEntryData {
    account: Pubkey,
    reason: String,
    blacklisted_by: Pubkey,
    blacklisted_at: i64,
    bump: u8,
}

// ==================== MINTERS ====================
pub fn handle_minter_add(
    program: &Program<Rc<Keypair>>,
    authority: &Pubkey,
    account: &str,
    quota: u64,
    stablecoin: Option<&Pubkey>,
) -> CliResult<()> {
    let account_pubkey = parse_pubkey(account)?;

    println!("➕ Adding minter: {}", account_pubkey);
    if quota > 0 {
        println!("   Quota: {} tokens", quota);
    } else {
        println!("   Quota: Unlimited");
    }

    let program_id = program.id();
    let stablecoin_pda = match stablecoin {
        Some(s) => *s,
        None => {
            return Err(CliError::InvalidArg(
                "Stablecoin PDA is required. Use --stablecoin <address>".to_string(),
            ));
        }
    };

    let (minter_pda, _) = derive_minter_pda(&stablecoin_pda, &account_pubkey, &program_id);

    let accounts = vec![
        AccountMeta::new(*authority, true), // authority (signer, mut)
        AccountMeta::new_readonly(stablecoin_pda, false), // state (PDA)
        AccountMeta::new(minter_pda, false), // minter_info (PDA)
        AccountMeta::new_readonly(account_pubkey, false), // minter account
        AccountMeta::new_readonly(system_program::id(), false), // system_program
    ];

    let ix_data = build_instruction_data("add_minter", AddMinterArgs { quota })
        .map_err(|e| CliError::SerializationError(e.to_string()))?;

    let ix = Instruction {
        program_id,
        accounts,
        data: ix_data,
    };

    let signature = program
        .request()
        .instruction(ix)
        .send()
        .map_err(|e| CliError::TransactionError(e.to_string()))?;

    print_tx_success(&signature.to_string(), "Minter add");
    Ok(())
}

pub fn handle_minter_remove(
    program: &Program<Rc<Keypair>>,
    authority: &Pubkey,
    account: &str,
    stablecoin: Option<&Pubkey>,
) -> CliResult<()> {
    let account_pubkey = parse_pubkey(account)?;

    println!("➖ Removing minter: {}", account_pubkey);

    let program_id = program.id();
    let stablecoin_pda = match stablecoin {
        Some(s) => *s,
        None => {
            return Err(CliError::InvalidArg(
                "Stablecoin PDA is required. Use --stablecoin <address>".to_string(),
            ));
        }
    };

    let (minter_pda, _) = derive_minter_pda(&stablecoin_pda, &account_pubkey, &program_id);

    let accounts = vec![
        AccountMeta::new(*authority, true), // authority (signer, mut)
        AccountMeta::new_readonly(stablecoin_pda, false), // state (PDA)
        AccountMeta::new(minter_pda, false), // minter_info (PDA)
    ];

    let ix_data = build_instruction_data("remove_minter", RemoveMinterArgs {})
        .map_err(|e| CliError::SerializationError(e.to_string()))?;

    let ix = Instruction {
        program_id,
        accounts,
        data: ix_data,
    };

    let signature = program
        .request()
        .instruction(ix)
        .send()
        .map_err(|e| CliError::TransactionError(e.to_string()))?;

    print_tx_success(&signature.to_string(), "Minter removal");
    Ok(())
}

pub fn handle_minter_list(
    _program: &Program<Rc<Keypair>>,
    authority: &Pubkey,
    stablecoin: Option<&Pubkey>,
) -> CliResult<()> {
    println!("📋 Listing authorized minters...");

    let program_id = _program.id();
    let stablecoin_pda = stablecoin
        .copied()
        .unwrap_or_else(|| derive_stablecoin_pda(authority, &program_id).0);

    println!("   Stablecoin: {}", stablecoin_pda);
    println!("   Note: Use an indexer service to list all minters");

    Ok(())
}

pub fn handle_minter_info(
    program: &Program<Rc<Keypair>>,
    authority: &Pubkey,
    account: &str,
    stablecoin: Option<&Pubkey>,
) -> CliResult<()> {
    let account_pubkey = parse_pubkey(account)?;

    println!("ℹ️ Minter info for {}", account_pubkey);

    let program_id = program.id();
    let stablecoin_pda = match stablecoin {
        Some(s) => *s,
        None => {
            return Err(CliError::InvalidArg(
                "Stablecoin PDA is required. Use --stablecoin <address>".to_string(),
            ));
        }
    };

    let (role_pda, _bump) = derive_role_pda(&stablecoin_pda, &account_pubkey, &program_id);
    let (minter_pda, _bump) = derive_minter_pda(&stablecoin_pda, &account_pubkey, &program_id);

    // Check role using RPC
    let role_data = program.rpc().get_account_data(&role_pda);
    match role_data {
        Ok(data) if data.len() > 8 => match RoleAssignmentData::try_from_slice(&data[8..]) {
            Ok(assignment) => {
                println!("   Role: {:?}", assignment.role);
                println!("   Assigned by: {}", assignment.assigned_by);
                println!("   Assigned at: {}", assignment.assigned_at);
            }
            Err(_) => {
                println!("   Status: Could not parse role data");
            }
        },
        _ => {
            println!("   Status: Not a minter");
        }
    }

    // Check quota using RPC
    let minter_data = program.rpc().get_account_data(&minter_pda);
    match minter_data {
        Ok(data) if data.len() > 8 => match MinterInfoData::try_from_slice(&data[8..]) {
            Ok(info) => {
                println!("   Quota: {}", info.quota);
                println!("   Minted: {}", info.minted_amount);
                println!(
                    "   Remaining: {}",
                    if info.quota > 0 {
                        info.quota.saturating_sub(info.minted_amount)
                    } else {
                        u64::MAX
                    }
                );
            }
            Err(_) => {
                println!("   Quota: Could not parse minter data");
            }
        },
        _ => {
            println!("   Quota: Not set (unlimited)");
        }
    }

    Ok(())
}

#[derive(Debug, ::borsh::BorshDeserialize)]
struct RoleAssignmentData {
    role: u8,
    account: Pubkey,
    assigned_by: Pubkey,
    assigned_at: i64,
    bump: u8,
}

#[derive(Debug, ::borsh::BorshDeserialize)]
struct MinterInfoData {
    minter: Pubkey,
    quota: u64,
    minted_amount: u64,
    bump: u8,
}

pub fn handle_minter_set_quota(
    program: &Program<Rc<Keypair>>,
    authority: &Pubkey,
    account: &str,
    quota: u64,
    stablecoin: Option<&Pubkey>,
) -> CliResult<()> {
    let account_pubkey = parse_pubkey(account)?;

    println!("📝 Setting quota for {}: {} tokens", account_pubkey, quota);

    let program_id = program.id();
    let stablecoin_pda = match stablecoin {
        Some(s) => *s,
        None => {
            return Err(CliError::InvalidArg(
                "Stablecoin PDA is required. Use --stablecoin <address>".to_string(),
            ));
        }
    };

    let (minter_pda, _) = derive_minter_pda(&stablecoin_pda, &account_pubkey, &program_id);

    let accounts = vec![
        AccountMeta::new(*authority, true), // authority (signer, mut)
        AccountMeta::new_readonly(stablecoin_pda, false), // state (PDA)
        AccountMeta::new(minter_pda, false), // minter_info (PDA)
    ];

    let ix_data = build_instruction_data("update_quota", SetQuotaArgs { quota })
        .map_err(|e| CliError::SerializationError(e.to_string()))?;

    let ix = Instruction {
        program_id,
        accounts,
        data: ix_data,
    };

    let signature = program
        .request()
        .instruction(ix)
        .send()
        .map_err(|e| CliError::TransactionError(e.to_string()))?;

    print_tx_success(&signature.to_string(), "Quota update");
    Ok(())
}

// ==================== SEIZE ====================
pub fn handle_seize(
    program: &Program<Rc<Keypair>>,
    authority: &Pubkey,
    account: &str,
    to: &str,
    amount: u64,
    stablecoin: Option<&Pubkey>,
) -> CliResult<()> {
    let account_pubkey = parse_pubkey(account)?;
    let to_pubkey = parse_pubkey(to)?;

    println!("🔒 Seizing {} tokens from {}", amount, account_pubkey);
    println!("   Transfer to: {}", to_pubkey);

    let program_id = program.id();
    let stablecoin_pda = match stablecoin {
        Some(s) => *s,
        None => {
            return Err(CliError::InvalidArg(
                "Stablecoin PDA is required. Use --stablecoin <address>".to_string(),
            ));
        }
    };
    let (role_pda, _) = derive_role_pda(&stablecoin_pda, authority, &program_id);
    let asset_mint = fetch_asset_mint(program, &stablecoin_pda)?;

    let accounts = vec![
        AccountMeta::new(*authority, true),      // authority (signer, mut)
        AccountMeta::new(stablecoin_pda, false), // state (PDA)
        AccountMeta::new_readonly(role_pda, false), // role_assignment (optional)
        AccountMeta::new(asset_mint, false),     // asset_mint (mut)
        AccountMeta::new(account_pubkey, false), // from (token account)
        AccountMeta::new(to_pubkey, false),      // to (token account)
        AccountMeta::new_readonly(spl_token_2022::id(), false), // token_program
    ];

    let ix_data = build_instruction_data("seize", SeizeArgs { amount })
        .map_err(|e| CliError::SerializationError(e.to_string()))?;

    let ix = Instruction {
        program_id,
        accounts,
        data: ix_data,
    };

    let signature = program
        .request()
        .instruction(ix)
        .send()
        .map_err(|e| CliError::TransactionError(e.to_string()))?;

    print_tx_success(&signature.to_string(), "Seize");
    Ok(())
}

// ==================== TRANSFER AUTHORITY ====================
pub fn handle_transfer_authority(
    program: &Program<Rc<Keypair>>,
    authority: &Pubkey,
    new_authority: &str,
    stablecoin: Option<&Pubkey>,
) -> CliResult<()> {
    let new_authority_pubkey = parse_pubkey(new_authority)?;

    println!("🔑 Transferring authority to {}", new_authority_pubkey);
    println!("   Current authority: {}", authority);
    println!("   ⚠️  WARNING: This action is irreversible!");

    let program_id = program.id();
    let stablecoin_pda = match stablecoin {
        Some(s) => *s,
        None => {
            return Err(CliError::InvalidArg(
                "Stablecoin PDA is required. Use --stablecoin <address>".to_string(),
            ));
        }
    };

    let accounts = vec![
        AccountMeta::new(*authority, true),      // authority (signer, mut)
        AccountMeta::new(stablecoin_pda, false), // state (PDA)
    ];

    let ix_data = build_instruction_data(
        "transfer_authority",
        TransferAuthority {
            new_authority: new_authority_pubkey,
        },
    )
    .map_err(|e| CliError::SerializationError(e.to_string()))?;

    let ix = Instruction {
        program_id,
        accounts,
        data: ix_data,
    };

    let signature = program
        .request()
        .instruction(ix)
        .send()
        .map_err(|e| CliError::TransactionError(e.to_string()))?;

    print_tx_success(&signature.to_string(), "Authority transfer");
    Ok(())
}

// ==================== ASSIGN ROLE ====================
pub fn handle_assign_role(
    program: &Program<Rc<Keypair>>,
    authority: &Pubkey,
    role: Role,
    account: &str,
    stablecoin: Option<&Pubkey>,
) -> CliResult<()> {
    let account_pubkey = parse_pubkey(account)?;

    println!("👤 Assigning role {:?} to {}", role, account_pubkey);

    let program_id = program.id();
    let stablecoin_pda = match stablecoin {
        Some(s) => *s,
        None => {
            return Err(CliError::InvalidArg(
                "Stablecoin PDA is required. Use --stablecoin <address>".to_string(),
            ));
        }
    };

    let (role_pda, _) = derive_role_pda(&stablecoin_pda, &account_pubkey, &program_id);

    let accounts = vec![
        AccountMeta::new(*authority, true),      // authority (signer, mut)
        AccountMeta::new(stablecoin_pda, false), // state (PDA)
        AccountMeta::new(role_pda, false),       // assignment (PDA)
        AccountMeta::new_readonly(account_pubkey, false), // account to assign role
        AccountMeta::new_readonly(system_program::id(), false), // system_program
    ];

    let ix_data = build_instruction_data("assign_role", AssignRoleArgs { role: role.to_u8() })
        .map_err(|e| CliError::SerializationError(e.to_string()))?;

    let ix = Instruction {
        program_id,
        accounts,
        data: ix_data,
    };

    let signature = program
        .request()
        .instruction(ix)
        .send()
        .map_err(|e| CliError::TransactionError(e.to_string()))?;

    print_tx_success(&signature.to_string(), "Role assignment");
    Ok(())
}

// ==================== REVOKE ROLE ====================
pub fn handle_revoke_role(
    program: &Program<Rc<Keypair>>,
    authority: &Pubkey,
    account: &str,
    stablecoin: Option<&Pubkey>,
) -> CliResult<()> {
    let account_pubkey = parse_pubkey(account)?;

    println!("🚫 Revoking all roles from {}", account_pubkey);

    let program_id = program.id();
    let stablecoin_pda = match stablecoin {
        Some(s) => *s,
        None => {
            return Err(CliError::InvalidArg(
                "Stablecoin PDA is required. Use --stablecoin <address>".to_string(),
            ));
        }
    };

    let (role_pda, _) = derive_role_pda(&stablecoin_pda, &account_pubkey, &program_id);

    let accounts = vec![
        AccountMeta::new(*authority, true), // authority (signer, mut)
        AccountMeta::new_readonly(stablecoin_pda, false), // state (PDA)
        AccountMeta::new(role_pda, false),  // assignment (PDA)
    ];

    let ix_data = build_instruction_data("revoke_role", RevokeRoleArgs {})
        .map_err(|e| CliError::SerializationError(e.to_string()))?;

    let ix = Instruction {
        program_id,
        accounts,
        data: ix_data,
    };

    let signature = program
        .request()
        .instruction(ix)
        .send()
        .map_err(|e| CliError::TransactionError(e.to_string()))?;

    print_tx_success(&signature.to_string(), "Role revocation");
    Ok(())
}

// ==================== STATUS ====================
pub fn handle_status(
    program: &Program<Rc<Keypair>>,
    authority: &Pubkey,
    stablecoin: Option<&Pubkey>,
    export_path: Option<&str>,
) -> CliResult<()> {
    println!("📊 Stablecoin Status");

    let program_id = program.id();
    let stablecoin_pda = match stablecoin {
        Some(s) => *s,
        None => {
            return Err(CliError::InvalidArg(
                "Stablecoin PDA is required. Use --stablecoin <address>".to_string(),
            ));
        }
    };

    println!("   Stablecoin PDA: {}", stablecoin_pda);

    // Fetch state using RPC
    let state_data = program.rpc().get_account_data(&stablecoin_pda);
    match state_data {
        Ok(data) if data.len() > 8 => match StablecoinStateData::try_from_slice(&data[8..]) {
            Ok(state) => {
                println!("\n┌─────────────────────────────────────────┐");
                println!("│ STABLECOIN STATE                        │");
                println!("├─────────────────────────────────────────┤");
                println!("│ Authority:    {:<25}│", state.authority);
                println!("│ Asset Mint:   {:<25}│", state.asset_mint);
                println!("│ Total Supply: {:<25}│", state.total_supply);
                println!(
                    "│ Paused:       {:<25}│",
                    if state.paused { "YES" } else { "NO" }
                );
                println!("│ Preset:       SSS-{:<22}│", state.preset);
                println!(
                    "│ Compliance:   {:<25}│",
                    if state.compliance_enabled {
                        "ENABLED"
                    } else {
                        "DISABLED"
                    }
                );
                println!("│ Bump:         {:<25}│", state.bump);
                println!("└─────────────────────────────────────────┘");

                if let Some(path) = export_path {
                    let json = serde_json::json!({
                        "stablecoin_pda": stablecoin_pda.to_string(),
                        "authority": state.authority.to_string(),
                        "asset_mint": state.asset_mint.to_string(),
                        "total_supply": state.total_supply,
                        "paused": state.paused,
                        "preset": state.preset,
                        "compliance_enabled": state.compliance_enabled,
                        "bump": state.bump,
                    });
                    std::fs::write(path, serde_json::to_string_pretty(&json)?)
                        .map_err(|e| CliError::IoError(e.to_string()))?;
                    println!("\n💾 Status exported to {}", path);
                }
            }
            Err(e) => {
                println!("❌ Failed to parse state: {}", e);
            }
        },
        Ok(_) => {
            println!("❌ Account data too short");
        }
        Err(e) => {
            println!("❌ Failed to fetch state: {}", e);
            println!("   The stablecoin may not be initialized yet.");
        }
    }

    Ok(())
}

#[derive(Debug, ::borsh::BorshDeserialize)]
struct StablecoinStateData {
    authority: Pubkey,
    asset_mint: Pubkey,
    total_supply: u64,
    paused: bool,
    preset: u8,
    compliance_enabled: bool,
    bump: u8,
}

// ==================== SUPPLY ====================
pub fn handle_supply(
    program: &Program<Rc<Keypair>>,
    authority: &Pubkey,
    stablecoin: Option<&Pubkey>,
) -> CliResult<()> {
    let program_id = program.id();
    let stablecoin_pda = match stablecoin {
        Some(s) => *s,
        None => {
            return Err(CliError::InvalidArg(
                "Stablecoin PDA is required. Use --stablecoin <address>".to_string(),
            ));
        }
    };

    match program.rpc().get_account_data(&stablecoin_pda) {
        Ok(data) if data.len() > 8 => match StablecoinStateData::try_from_slice(&data[8..]) {
            Ok(state) => {
                println!("💰 Total Supply: {} tokens", state.total_supply);
            }
            Err(_) => {
                println!("❌ Could not parse supply data.");
            }
        },
        _ => {
            println!("❌ Could not fetch supply. Stablecoin may not be initialized.");
        }
    }

    Ok(())
}

// ==================== HOLDERS ====================
pub fn handle_holders(
    _program: &Program<Rc<Keypair>>,
    authority: &Pubkey,
    min_balance: u64,
    stablecoin: Option<&Pubkey>,
) -> CliResult<()> {
    let program_id = _program.id();
    let stablecoin_pda = stablecoin
        .copied()
        .unwrap_or_else(|| derive_stablecoin_pda(authority, &program_id).0);

    println!("👥 Token Holders (min balance: {})", min_balance);
    println!("   Stablecoin: {}", stablecoin_pda);
    println!("\n   Note: Holder list requires an indexer service.");
    println!("   Consider using:");
    println!("   - Helius API");
    println!("   - Solana RPC's getTokenAccountsByDelegate");
    println!("   - Custom indexer");

    Ok(())
}

// ==================== AUDIT LOG ====================
pub fn handle_audit_log(
    _program: &Program<Rc<Keypair>>,
    _authority: &Pubkey,
    action: Option<&str>,
    from: Option<&Pubkey>,
    to: Option<&Pubkey>,
    _format: &str,
    output_path: Option<&str>,
) -> CliResult<()> {
    println!("📜 Audit Log");

    if let Some(a) = action {
        println!("   Filter action: {}", a);
    }
    if let Some(f) = from {
        println!("   From: {}", f);
    }
    if let Some(t) = to {
        println!("   To: {}", t);
    }

    println!("\n   Note: Audit logs require event indexing.");
    println!("   The SSS Token emits events that can be indexed:");
    println!("   - Minted");
    println!("   - Burned");
    println!("   - Frozen");
    println!("   - Thawed");
    println!("   - Paused");
    println!("   - Blacklisted");
    println!("   - Seized");
    println!("   - RoleAssigned");

    if let Some(path) = output_path {
        println!("\n   Output would be saved to: {}", path);
    }

    Ok(())
}

// ==================== DERIVE ====================
pub fn handle_derive(
    program: &Program<Rc<Keypair>>,
    authority: &Pubkey,
    stablecoin: Option<&Pubkey>,
) -> CliResult<()> {
    let program_id = program.id();
    let stablecoin_pda = match stablecoin {
        Some(s) => *s,
        None => {
            return Err(CliError::InvalidArg(
                "Stablecoin PDA is required. Use --stablecoin <address>".to_string(),
            ));
        }
    };

    println!("🔑 PDA Derivations");
    println!("\n   Program ID: {}", program_id);
    println!("   Stablecoin: {}", stablecoin_pda);

    println!("\n┌─────────────────────────────────────────────────────────┐");
    println!("│ PDA Type         │ Public Key                           │");
    println!("├─────────────────────────────────────────────────────────┤");

    // Role PDAs for authority
    let (role_pda, bump) = derive_role_pda(&stablecoin_pda, authority, &program_id);
    println!("│ Role (authority) │ {} (bump: {})│", role_pda, bump);

    // Minter PDA for authority
    let (minter_pda, bump) = derive_minter_pda(&stablecoin_pda, authority, &program_id);
    println!("│ Minter (auth)    │ {} (bump: {})│", minter_pda, bump);

    // Blacklist PDA for authority
    let (blacklist_pda, bump) = derive_blacklist_pda(&stablecoin_pda, authority, &program_id);
    println!("│ Blacklist (auth) │ {} (bump: {})│", blacklist_pda, bump);

    println!("└─────────────────────────────────────────────────────────┘");

    println!("\n💡 Use these PDAs when calling program instructions");

    Ok(())
}
