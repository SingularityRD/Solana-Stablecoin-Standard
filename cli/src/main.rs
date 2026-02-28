use anchor_client::{Client, Cluster, Program};
use clap::{Parser, Subcommand};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair},
    signer::Signer,
    commitment_config::CommitmentConfig,
};
use std::rc::Rc;
use std::time::Duration;

mod commands;
mod config;
mod error;
mod instructions;

use config::SssConfig;
use error::CliError;

/// Program ID for the SSS Token program
const PROGRAM_ID: &str = "SSSToken11111111111111111111111111111111111";

/// PDA seeds
const STABLECOIN_SEED: &[u8] = b"stablecoin";
const ROLE_SEED: &[u8] = b"role";
const MINTER_SEED: &[u8] = b"minter";
const BLACKLIST_SEED: &[u8] = b"blacklist";

#[derive(Parser)]
#[command(name = "sss-token")]
#[command(about = "Solana Stablecoin Standard CLI - Production Ready", version)]
struct Cli {
    /// Solana RPC URL (or set SSS_RPC_URL env var)
    #[arg(long, env = "SSS_RPC_URL", default_value = "https://api.devnet.solana.com")]
    url: String,

    /// Path to keypair file (or set SSS_KEYPAIR_PATH env var)
    #[arg(long, env = "SSS_KEYPAIR_PATH", default_value = "~/.config/solana/id.json")]
    keypair: String,

    /// Commitment level
    #[arg(long, default_value = "confirmed")]
    commitment: String,

    /// Path to config file
    #[arg(long, default_value = "sss-config.toml")]
    config: String,

    /// The administrative command to execute
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new stablecoin instance
    Init {
        #[arg(long, default_value = "1")]
        preset: u8,
        #[arg(long)]
        name: String,
        #[arg(long)]
        symbol: String,
        #[arg(long)]
        uri: String,
        #[arg(long, default_value = "6")]
        decimals: u8,
        #[arg(long)]
        asset_mint: Option<String>,
    },

    /// Mint tokens to a recipient
    Mint {
        recipient: String,
        amount: u64,
        #[arg(long)]
        stablecoin: Option<String>,
    },

    /// Burn tokens
    Burn {
        amount: u64,
        #[arg(long)]
        from: Option<String>,
        #[arg(long)]
        stablecoin: Option<String>,
    },

    /// Freeze a token account
    Freeze {
        account: String,
        #[arg(long)]
        stablecoin: Option<String>,
    },

    /// Thaw a frozen account
    Thaw {
        account: String,
        #[arg(long)]
        stablecoin: Option<String>,
    },

    /// Pause all operations
    Pause {
        #[arg(long)]
        stablecoin: Option<String>,
    },

    /// Unpause operations
    Unpause {
        #[arg(long)]
        stablecoin: Option<String>,
    },

    /// Manage blacklist
    Blacklist {
        #[command(subcommand)]
        command: BlacklistCommands,
    },

    /// Manage minters
    Minters {
        #[command(subcommand)]
        command: MinterCommands,
    },

    /// Seize tokens from blacklisted account
    Seize {
        account: String,
        #[arg(long)]
        to: String,
        amount: u64,
        #[arg(long)]
        stablecoin: Option<String>,
    },

    /// Transfer master authority
    TransferAuthority {
        new_authority: String,
        #[arg(long)]
        stablecoin: Option<String>,
    },

    /// Assign a role to an account
    AssignRole {
        role: String,
        account: String,
        #[arg(long)]
        stablecoin: Option<String>,
    },

    /// Revoke a role from an account
    RevokeRole {
        account: String,
        #[arg(long)]
        stablecoin: Option<String>,
    },

    /// Display stablecoin status
    Status {
        #[arg(long)]
        stablecoin: Option<String>,
        #[arg(long)]
        export: Option<String>,
    },

    /// Display total supply
    Supply {
        #[arg(long)]
        stablecoin: Option<String>,
    },

    /// List token holders
    Holders {
        #[arg(long, default_value = "0")]
        min_balance: u64,
        #[arg(long)]
        stablecoin: Option<String>,
    },

    /// View audit logs
    AuditLog {
        #[arg(long)]
        action: Option<String>,
        #[arg(long)]
        from: Option<String>,
        #[arg(long)]
        to: Option<String>,
        #[arg(long, default_value = "text")]
        format: String,
        #[arg(long)]
        output: Option<String>,
    },

    /// Derive PDAs for a stablecoin
    Derive {
        #[arg(long)]
        stablecoin: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum BlacklistCommands {
    Add {
        account: String,
        #[arg(long)]
        reason: String,
        #[arg(long)]
        stablecoin: Option<String>,
    },
    Remove {
        account: String,
        #[arg(long)]
        stablecoin: Option<String>,
    },
    List {
        #[arg(long)]
        stablecoin: Option<String>,
    },
    Check {
        account: String,
        #[arg(long)]
        stablecoin: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum MinterCommands {
    Add {
        account: String,
        #[arg(long, default_value = "0")]
        quota: u64,
        #[arg(long)]
        stablecoin: Option<String>,
    },
    Remove {
        account: String,
        #[arg(long)]
        stablecoin: Option<String>,
    },
    List {
        #[arg(long)]
        stablecoin: Option<String>,
    },
    Info {
        account: String,
        #[arg(long)]
        stablecoin: Option<String>,
    },
    SetQuota {
        account: String,
        quota: u64,
        #[arg(long)]
        stablecoin: Option<String>,
    },
}

fn expand_tilde(path: &str) -> String {
    if path.starts_with('~') {
        if let Some(home) = std::env::var("HOME").ok().or_else(|| std::env::var("USERPROFILE").ok()) {
            return path.replacen('~', &home, 1);
        }
    }
    path.to_string()
}

fn parse_pubkey(s: &str) -> Result<Pubkey, CliError> {
    s.parse::<Pubkey>()
        .map_err(|_| CliError::InvalidPubkey(s.to_string()))
}

fn get_commitment(s: &str) -> CommitmentConfig {
    match s.to_lowercase().as_str() {
        "processed" => CommitmentConfig::processed(),
        "confirmed" => CommitmentConfig::confirmed(),
        "finalized" => CommitmentConfig::finalized(),
        _ => CommitmentConfig::confirmed(),
    }
}

fn derive_stablecoin_pda(asset_mint: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[STABLECOIN_SEED, asset_mint.to_bytes().as_ref()],
        program_id,
    )
}

fn derive_role_pda(
    stablecoin: &Pubkey,
    account: &Pubkey,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[ROLE_SEED, stablecoin.to_bytes().as_ref(), account.to_bytes().as_ref()],
        program_id,
    )
}

fn derive_minter_pda(
    stablecoin: &Pubkey,
    minter: &Pubkey,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[MINTER_SEED, stablecoin.to_bytes().as_ref(), minter.to_bytes().as_ref()],
        program_id,
    )
}

fn derive_blacklist_pda(
    stablecoin: &Pubkey,
    account: &Pubkey,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[BLACKLIST_SEED, stablecoin.to_bytes().as_ref(), account.to_bytes().as_ref()],
        program_id,
    )
}

fn setup_client(
    url: &str,
    keypair_path: &str,
    commitment: &str,
) -> Result<(Program<Rc<Keypair>>, Pubkey, Pubkey), CliError> {
    let expanded_path = expand_tilde(keypair_path);
    let keypair = read_keypair_file(&expanded_path)
        .map_err(|e| CliError::KeypairError(format!("Failed to read keypair {}: {}", expanded_path, e)))?;
    
    let authority = keypair.pubkey();
    let commitment_config = get_commitment(commitment);
    
    let client = Client::new_with_options(
        Cluster::Custom(url.to_string(), url.to_string()),
        Rc::new(keypair),
        commitment_config,
    );
    
    let program_id = Pubkey::try_from(PROGRAM_ID)
        .map_err(|e| CliError::InvalidPubkey(e.to_string()))?;
    
    let program = client.program(program_id)
        .map_err(|e| CliError::AnchorError(e))?;
    
    Ok((program, program_id, authority))
}

fn parse_role(role_str: &str) -> Result<commands::Role, CliError> {
    match role_str.to_lowercase().as_str() {
        "master" => Ok(commands::Role::Master),
        "minter" => Ok(commands::Role::Minter),
        "burner" => Ok(commands::Role::Burner),
        "blacklister" => Ok(commands::Role::Blacklister),
        "pauser" => Ok(commands::Role::Pauser),
        "seizer" => Ok(commands::Role::Seizer),
        _ => Err(CliError::InvalidArg(format!(
            "Invalid role '{}'. Valid roles: master, minter, burner, blacklister, pauser, seizer",
            role_str
        ))),
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    // Load optional config file
    let _config = config::load_config(&cli.config).unwrap_or_default();
    
    // Setup client
    let (program, program_id, authority) = match setup_client(&cli.url, &cli.keypair, &cli.commitment) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("❌ Error setting up client: {}", e);
            std::process::exit(1);
        }
    };
    
    let result = match cli.command {
        Commands::Init { preset, name, symbol, uri, decimals, asset_mint } => {
            commands::handle_init(&program, &authority, preset, name, symbol, uri, decimals, asset_mint)
        }
        Commands::Mint { recipient, amount, stablecoin } => {
            let stablecoin_pubkey = stablecoin
                .map(|s| parse_pubkey(&s))
                .transpose()?;
            commands::handle_mint(&program, &authority, &recipient, amount, stablecoin_pubkey.as_ref())
        }
        Commands::Burn { amount, from, stablecoin } => {
            let from_pubkey = from
                .map(|s| parse_pubkey(&s))
                .transpose()?;
            let stablecoin_pubkey = stablecoin
                .map(|s| parse_pubkey(&s))
                .transpose()?;
            commands::handle_burn(&program, &authority, amount, from_pubkey.as_ref(), stablecoin_pubkey.as_ref())
        }
        Commands::Freeze { account, stablecoin } => {
            let stablecoin_pubkey = stablecoin
                .map(|s| parse_pubkey(&s))
                .transpose()?;
            commands::handle_freeze(&program, &authority, &account, stablecoin_pubkey.as_ref())
        }
        Commands::Thaw { account, stablecoin } => {
            let stablecoin_pubkey = stablecoin
                .map(|s| parse_pubkey(&s))
                .transpose()?;
            commands::handle_thaw(&program, &authority, &account, stablecoin_pubkey.as_ref())
        }
        Commands::Pause { stablecoin } => {
            let stablecoin_pubkey = stablecoin
                .map(|s| parse_pubkey(&s))
                .transpose()?;
            commands::handle_pause(&program, &authority, stablecoin_pubkey.as_ref())
        }
        Commands::Unpause { stablecoin } => {
            let stablecoin_pubkey = stablecoin
                .map(|s| parse_pubkey(&s))
                .transpose()?;
            commands::handle_unpause(&program, &authority, stablecoin_pubkey.as_ref())
        }
        Commands::Blacklist { command } => match command {
            BlacklistCommands::Add { account, reason, stablecoin } => {
                let stablecoin_pubkey = stablecoin
                    .map(|s| parse_pubkey(&s))
                    .transpose()?;
                commands::handle_blacklist_add(&program, &authority, &account, &reason, stablecoin_pubkey.as_ref())
            }
            BlacklistCommands::Remove { account, stablecoin } => {
                let stablecoin_pubkey = stablecoin
                    .map(|s| parse_pubkey(&s))
                    .transpose()?;
                commands::handle_blacklist_remove(&program, &authority, &account, stablecoin_pubkey.as_ref())
            }
            BlacklistCommands::List { stablecoin } => {
                let stablecoin_pubkey = stablecoin
                    .map(|s| parse_pubkey(&s))
                    .transpose()?;
                commands::handle_blacklist_list(&program, &authority, stablecoin_pubkey.as_ref())
            }
            BlacklistCommands::Check { account, stablecoin } => {
                let stablecoin_pubkey = stablecoin
                    .map(|s| parse_pubkey(&s))
                    .transpose()?;
                commands::handle_blacklist_check(&program, &authority, &account, stablecoin_pubkey.as_ref())
            }
        },
        Commands::Minters { command } => match command {
            MinterCommands::Add { account, quota, stablecoin } => {
                let stablecoin_pubkey = stablecoin
                    .map(|s| parse_pubkey(&s))
                    .transpose()?;
                commands::handle_minter_add(&program, &authority, &account, quota, stablecoin_pubkey.as_ref())
            }
            MinterCommands::Remove { account, stablecoin } => {
                let stablecoin_pubkey = stablecoin
                    .map(|s| parse_pubkey(&s))
                    .transpose()?;
                commands::handle_minter_remove(&program, &authority, &account, stablecoin_pubkey.as_ref())
            }
            MinterCommands::List { stablecoin } => {
                let stablecoin_pubkey = stablecoin
                    .map(|s| parse_pubkey(&s))
                    .transpose()?;
                commands::handle_minter_list(&program, &authority, stablecoin_pubkey.as_ref())
            }
            MinterCommands::Info { account, stablecoin } => {
                let stablecoin_pubkey = stablecoin
                    .map(|s| parse_pubkey(&s))
                    .transpose()?;
                commands::handle_minter_info(&program, &authority, &account, stablecoin_pubkey.as_ref())
            }
            MinterCommands::SetQuota { account, quota, stablecoin } => {
                let stablecoin_pubkey = stablecoin
                    .map(|s| parse_pubkey(&s))
                    .transpose()?;
                commands::handle_minter_set_quota(&program, &authority, &account, quota, stablecoin_pubkey.as_ref())
            }
        },
        Commands::Seize { account, to, amount, stablecoin } => {
            let stablecoin_pubkey = stablecoin
                .map(|s| parse_pubkey(&s))
                .transpose()?;
            commands::handle_seize(&program, &authority, &account, &to, amount, stablecoin_pubkey.as_ref())
        }
        Commands::TransferAuthority { new_authority, stablecoin } => {
            let stablecoin_pubkey = stablecoin
                .map(|s| parse_pubkey(&s))
                .transpose()?;
            commands::handle_transfer_authority(&program, &authority, &new_authority, stablecoin_pubkey.as_ref())
        }
        Commands::AssignRole { role, account, stablecoin } => {
            let role_enum = parse_role(&role)?;
            let stablecoin_pubkey = stablecoin
                .map(|s| parse_pubkey(&s))
                .transpose()?;
            commands::handle_assign_role(&program, &authority, role_enum, &account, stablecoin_pubkey.as_ref())
        }
        Commands::RevokeRole { account, stablecoin } => {
            let stablecoin_pubkey = stablecoin
                .map(|s| parse_pubkey(&s))
                .transpose()?;
            commands::handle_revoke_role(&program, &authority, &account, stablecoin_pubkey.as_ref())
        }
        Commands::Status { stablecoin, export } => {
            let stablecoin_pubkey = stablecoin
                .map(|s| parse_pubkey(&s))
                .transpose()?;
            commands::handle_status(&program, &authority, stablecoin_pubkey.as_ref(), export.as_deref())
        }
        Commands::Supply { stablecoin } => {
            let stablecoin_pubkey = stablecoin
                .map(|s| parse_pubkey(&s))
                .transpose()?;
            commands::handle_supply(&program, &authority, stablecoin_pubkey.as_ref())
        }
        Commands::Holders { min_balance, stablecoin } => {
            let stablecoin_pubkey = stablecoin
                .map(|s| parse_pubkey(&s))
                .transpose()?;
            commands::handle_holders(&program, &authority, min_balance, stablecoin_pubkey.as_ref())
        }
        Commands::AuditLog { action, from, to, format, output } => {
            let from_pubkey = from
                .map(|s| parse_pubkey(&s))
                .transpose()?;
            let to_pubkey = to
                .map(|s| parse_pubkey(&s))
                .transpose()?;
            commands::handle_audit_log(&program, &authority, action.as_deref(), from_pubkey.as_ref(), to_pubkey.as_ref(), &format, output.as_deref())
        }
        Commands::Derive { stablecoin } => {
            let stablecoin_pubkey = stablecoin
                .map(|s| parse_pubkey(&s))
                .transpose()?;
            commands::handle_derive(&program, &authority, stablecoin_pubkey.as_ref())
        }
    };
    
    if let Err(e) = result {
        eprintln!("❌ Error: {}", e);
        std::process::exit(1);
    }
    
    Ok(())
}