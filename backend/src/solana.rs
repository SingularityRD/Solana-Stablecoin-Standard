use anyhow::{Context, Result};
use anchor_client::{
    Client, Cluster,
    solana_client::rpc_client::RpcClient,
    solana_sdk::{
        pubkey::Pubkey,
        signature::{Keypair, Signer},
        commitment_config::CommitmentConfig,
    },
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Solana service for interacting with the SSS token program
pub struct SolanaService {
    rpc_client: Arc<RpcClient>,
    program_id: Pubkey,
    keypair: Arc<RwLock<Option<Keypair>>>,
}

impl SolanaService {
    pub async fn new(rpc_url: &str, program_id: Pubkey) -> Result<Self> {
        let commitment = CommitmentConfig::confirmed();
        let rpc_client = Arc::new(RpcClient::new_with_commitment(
            rpc_url.to_string(),
            commitment,
        ));
        
        info!("Connected to Solana RPC: {}", rpc_url);
        info!("Program ID: {}", program_id);
        
        Ok(Self {
            rpc_client,
            program_id,
            keypair: Arc::new(RwLock::new(None)),
        })
    }
    
    /// Set the authority keypair for signing transactions
    pub async fn set_keypair(&self, keypair: Keypair) {
        let mut kp = self.keypair.write().await;
        *kp = Some(keypair);
    }
    
    /// Get the current program ID
    pub fn program_id(&self) -> &Pubkey {
        &self.program_id
    }
    
    /// Get the RPC client
    pub fn rpc_client(&self) -> &RpcClient {
        &self.rpc_client
    }
    
    /// Get the minimum balance for rent exemption
    pub async fn get_minimum_balance_for_rent_exemption(&self, data_len: usize) -> Result<u64> {
        self.rpc_client
            .get_minimum_balance_for_rent_exemption(data_len)
            .context("Failed to get minimum balance for rent exemption")
    }
    
    /// Get account balance
    pub async fn get_balance(&self, pubkey: &Pubkey) -> Result<u64> {
        self.rpc_client
            .get_balance(pubkey)
            .context("Failed to get account balance")
    }
    
    /// Get the current slot
    pub async fn get_slot(&self) -> Result<u64> {
        self.rpc_client
            .get_slot()
            .context("Failed to get current slot")
    }
    
    /// Check if the RPC is healthy
    pub async fn health_check(&self) -> Result<bool> {
        match self.rpc_client.get_health() {
            Ok(_) => Ok(true),
            Err(e) => {
                warn!("RPC health check failed: {}", e);
                Ok(false)
            }
        }
    }
    
    /// Find the stablecoin PDA
    pub fn find_stablecoin_pda(&self, asset_mint: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[b"stablecoin", asset_mint.as_ref()],
            &self.program_id,
        )
    }
    
    /// Find the role assignment PDA
    pub fn find_role_pda(&self, stablecoin: &Pubkey, account: &Pubkey, role: &[u8]) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[b"role", stablecoin.as_ref(), account.as_ref(), role],
            &self.program_id,
        )
    }
    
    /// Find the minter info PDA
    pub fn find_minter_pda(&self, stablecoin: &Pubkey, minter: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[b"minter", stablecoin.as_ref(), minter.as_ref()],
            &self.program_id,
        )
    }
    
    /// Find the blacklist entry PDA
    pub fn find_blacklist_pda(&self, stablecoin: &Pubkey, account: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[b"blacklist", stablecoin.as_ref(), account.as_ref()],
            &self.program_id,
        )
    }
    
    /// Find the freeze account PDA
    pub fn find_freeze_pda(&self, stablecoin: &Pubkey, account: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[b"freeze", stablecoin.as_ref(), account.as_ref()],
            &self.program_id,
        )
    }
    
    /// Validate a Solana pubkey format (base58, 32-44 chars)
    pub fn validate_pubkey(pubkey: &str) -> bool {
        if pubkey.len() < 32 || pubkey.len() > 44 {
            return false;
        }
        pubkey.parse::<Pubkey>().is_ok()
    }
}

/// Helper to parse a Pubkey from string
pub fn parse_pubkey(s: &str) -> Result<Pubkey> {
    s.parse::<Pubkey>()
        .with_context(|| format!("Invalid pubkey: {}", s))
}

/// Helper to parse a keypair from base58
pub fn parse_keypair(s: &str) -> Result<Keypair> {
    let bytes = bs58::decode(s)
        .into_vec()
        .context("Invalid base58 keypair")?;
    Keypair::from_bytes(&bytes)
        .context("Invalid keypair bytes")
}

/// Generate an explorer URL for a transaction
pub fn explorer_url(signature: &str, cluster: &str) -> String {
    match cluster {
        "mainnet" => format!("https://explorer.solana.com/tx/{}", signature),
        "devnet" => format!("https://explorer.solana.com/tx/{}?cluster=devnet", signature),
        "testnet" => format!("https://explorer.solana.com/tx/{}?cluster=testnet", signature),
        _ => format!("https://explorer.solana.com/tx/{}?cluster=custom&customUrl={}", signature, cluster),
    }
}

/// Validate a Solana pubkey
pub fn validate_pubkey(s: &str) -> Result<()> {
    if !SolanaService::validate_pubkey(s) {
        anyhow::bail!("Invalid Solana pubkey format")
    }
    Ok(())
}
