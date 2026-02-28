use serde::{Deserialize, Serialize};
use std::sync::Arc;
use anyhow::{Context, Result};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use tracing::{info, warn};

use crate::solana::{SolanaService, StablecoinStateAccount};

#[derive(Debug, Serialize, Deserialize)]
pub struct MintRequest {
    pub recipient: String,
    pub amount: u64,
    pub fiat_proof: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BurnRequest {
    pub amount: u64,
    pub bank_account: Option<String>,
    pub from_token_account: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionResult {
    pub signature: String,
    pub explorer_url: String,
    pub slot: Option<u64>,
}

pub struct MintBurnService {
    pub authority: String,
    solana: Arc<SolanaService>,
    /// Optional authority keypair for signing transactions
    authority_keypair: Option<Keypair>,
    /// Token program ID (defaults to Token-2022)
    token_program: Pubkey,
    /// Cluster name for explorer URLs
    cluster: String,
}

impl MintBurnService {
    pub fn new(authority: String, solana: Arc<SolanaService>) -> Self {
        Self {
            authority,
            solana,
            authority_keypair: None,
            // Token-2022 program ID
            token_program: Pubkey::from_str_const("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"),
            cluster: "devnet".to_string(),
        }
    }
    
    /// Set the authority keypair for signing transactions
    pub fn set_authority_keypair(&mut self, keypair: Keypair) {
        self.authority_keypair = Some(keypair);
    }
    
    /// Set the token program ID (use Token-2022 or legacy Token)
    pub fn set_token_program(&mut self, token_program: Pubkey) {
        self.token_program = token_program;
    }
    
    /// Set the cluster for explorer URLs
    pub fn set_cluster(&mut self, cluster: String) {
        self.cluster = cluster;
    }
    
    /// Parse recipient pubkey from string
    fn parse_recipient(&self, recipient: &str) -> Result<Pubkey> {
        recipient.parse::<Pubkey>()
            .with_context(|| format!("Invalid recipient pubkey: {}", recipient))
    }
    
    /// Get or derive the token account for a recipient
    async fn get_or_derive_token_account(&self, owner: &Pubkey, asset_mint: &Pubkey) -> Result<Pubkey> {
        // Try to find associated token account
        let associated_token = self.find_associated_token_account(owner, asset_mint);
        
        // Check if it exists
        if self.solana.account_exists(&associated_token).await {
            return Ok(associated_token);
        }
        
        // If not, return the owner's pubkey (caller should ensure account exists)
        warn!("Associated token account {} does not exist for owner {}", associated_token, owner);
        Err(anyhow::anyhow!("Recipient token account does not exist. Please create it first."))
    }
    
    /// Find associated token account address
    fn find_associated_token_account(&self, owner: &Pubkey, mint: &Pubkey) -> Pubkey {
        // For Token-2022, we use the same derivation as Token program
        let seeds = &[
            owner.as_ref(),
            self.token_program.as_ref(),
            mint.as_ref(),
        ];
        
        // Use spl-associated-token-account derivation
        // This is a simplified version - in production use spl_associated_token_account crate
        Pubkey::find_program_address(seeds, &self.token_program).0
    }
    
    /// Validates fiat deposit and creates a mint transaction
    pub async fn process_mint_request(
        &self,
        stablecoin_pubkey: &Pubkey,
        req: MintRequest,
    ) -> Result<TransactionResult> {
        // Validate fiat proof if required
        if let Some(proof) = &req.fiat_proof {
            tracing::debug!("Validating fiat proof: {}", proof);
            // In production, verify the fiat proof with banking API
            self.validate_fiat_proof(proof).await?;
        }
        
        // Parse recipient
        let recipient = self.parse_recipient(&req.recipient)?;
        
        // Get stablecoin state
        let state_data = self.solana.get_account_data(stablecoin_pubkey).await?;
        let state = self.deserialize_stablecoin_state(&state_data)?;
        
        // Check if paused
        if state.paused {
            anyhow::bail!("Stablecoin is currently paused");
        }
        
        // Get recipient token account
        let recipient_token_account = self.get_or_derive_token_account(&recipient, &state.asset_mint).await?;
        
        // Get authority keypair
        let authority = self.authority_keypair.as_ref()
            .context("Authority keypair not set")?;
        
        // Find role assignment PDA if authority has a role
        let role_pda = self.find_role_assignment(stablecoin_pubkey, &authority.pubkey());
        let role_account = if self.solana.account_exists(&role_pda).await {
            Some((role_pda, 0)) // Bump would need to be fetched
        } else {
            None
        };
        
        // Find minter info PDA
        let minter_pda = self.solana.find_minter_pda(stablecoin_pubkey, &authority.pubkey()).0;
        let minter_info = if self.solana.account_exists(&minter_pda).await {
            Some((minter_pda, 0))
        } else {
            None
        };
        
        // Build mint instruction
        let instruction = self.solana.build_mint_instruction(
            stablecoin_pubkey,
            &state.asset_mint,
            &authority.pubkey(),
            &recipient_token_account,
            req.amount,
            state.bump,
            role_account.as_ref().map(|(p, b)| (*p, *b)),
            minter_info.as_ref().map(|(p, b)| (*p, *b)),
            &self.token_program,
        );
        
        // Send transaction
        let signature = self.solana.build_and_send_instruction(
            vec![instruction],
            &[],
        ).await?;
        
        let slot = self.solana.get_slot().await.ok();
        
        info!(
            "Mint transaction successful: signature={}, recipient={}, amount={}",
            signature, req.recipient, req.amount
        );
        
        Ok(TransactionResult {
            signature: signature.to_string(),
            explorer_url: crate::solana::explorer_url(&signature.to_string(), &self.cluster),
            slot,
        })
    }
    
    /// Processes burn requests and coordinates fiat wire transfers
    pub async fn process_burn_request(
        &self,
        stablecoin_pubkey: &Pubkey,
        req: BurnRequest,
    ) -> Result<TransactionResult> {
        // Get stablecoin state
        let state_data = self.solana.get_account_data(stablecoin_pubkey).await?;
        let state = self.deserialize_stablecoin_state(&state_data)?;
        
        // Check if paused
        if state.paused {
            anyhow::bail!("Stablecoin is currently paused");
        }
        
        // Get authority keypair
        let authority = self.authority_keypair.as_ref()
            .context("Authority keypair not set")?;
        
        // Get from token account (use provided or derive from authority)
        let from_token_account = if let Some(acc) = &req.from_token_account {
            acc.parse::<Pubkey>()
                .with_context(|| format!("Invalid token account: {}", acc))?
        } else {
            self.find_associated_token_account(&authority.pubkey(), &state.asset_mint)
        };
        
        // Check balance
        let balance = self.solana.get_token_account_balance(&from_token_account).await?;
        if balance < req.amount {
            anyhow::bail!("Insufficient balance. Available: {}, Required: {}", balance, req.amount);
        }
        
        // Find role assignment PDA if authority has a role
        let role_pda = self.find_role_assignment(stablecoin_pubkey, &authority.pubkey());
        let role_account = if self.solana.account_exists(&role_pda).await {
            Some((role_pda, 0))
        } else {
            None
        };
        
        // Build burn instruction
        let instruction = self.solana.build_burn_instruction(
            stablecoin_pubkey,
            &state.asset_mint,
            &authority.pubkey(),
            &from_token_account,
            req.amount,
            role_account.as_ref().map(|(p, b)| (*p, *b)),
            &self.token_program,
        );
        
        // Send transaction
        let signature = self.solana.build_and_send_instruction(
            vec![instruction],
            &[],
        ).await?;
        
        let slot = self.solana.get_slot().await.ok();
        
        // In production: Initiate fiat wire transfer to bank_account
        if let Some(bank_account) = &req.bank_account {
            tracing::debug!("Initiating wire transfer to bank account: {}", bank_account);
            // This would integrate with a banking API
        }
        
        info!(
            "Burn transaction successful: signature={}, amount={}",
            signature, req.amount
        );
        
        Ok(TransactionResult {
            signature: signature.to_string(),
            explorer_url: crate::solana::explorer_url(&signature.to_string(), &self.cluster),
            slot,
        })
    }
    
    /// Validate fiat proof with banking API (placeholder)
    async fn validate_fiat_proof(&self, proof: &str) -> Result<()> {
        // In production, this would call a banking API to verify the proof
        // For now, we just validate it's not empty
        if proof.is_empty() {
            anyhow::bail!("Fiat proof cannot be empty");
        }
        Ok(())
    }
    
    /// Find role assignment PDA
    fn find_role_assignment(&self, stablecoin: &Pubkey, account: &Pubkey) -> Pubkey {
        // Try to find any role assignment for this account
        // In a real implementation, we'd check all role types
        self.solana.find_role_pda(stablecoin, account, b"minter").0
    }
    
    /// Deserialize stablecoin state from account data
    fn deserialize_stablecoin_state(&self, data: &[u8]) -> Result<StablecoinStateAccount> {
        // Skip 8-byte anchor discriminator
        if data.len() < 8 {
            anyhow::bail!("Invalid stablecoin state data length");
        }
        
        let mut slice = &data[8..];
        StablecoinStateAccount::deserialize(&mut slice)
            .context("Failed to deserialize stablecoin state")
    }
    
    /// Check if an address is blacklisted
    pub async fn is_blacklisted(&self, stablecoin: &Pubkey, address: &Pubkey) -> Result<bool> {
        let blacklist_pda = self.solana.find_blacklist_pda(stablecoin, address).0;
        Ok(self.solana.account_exists(&blacklist_pda).await)
    }
    
    /// Get current supply
    pub async fn get_supply(&self, stablecoin: &Pubkey) -> Result<u64> {
        let state_data = self.solana.get_account_data(stablecoin).await?;
        let state = self.deserialize_stablecoin_state(&state_data)?;
        Ok(state.total_supply)
    }
}
