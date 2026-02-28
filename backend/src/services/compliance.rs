use serde::{Deserialize, Serialize};
use std::sync::Arc;
use anyhow::{Context, Result};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use tracing::{info, warn};

use crate::solana::{
    SolanaService, StablecoinStateAccount, BlacklistEntryAccount, Role,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ScreeningResult {
    pub address: String,
    pub risk_score: u8,
    pub is_sanctioned: bool,
    pub is_blacklisted: bool,
    pub recommendation: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlacklistResult {
    pub address: String,
    pub signature: Option<String>,
    pub explorer_url: Option<String>,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlacklistEntry {
    pub address: String,
    pub reason: String,
    pub blacklisted_by: String,
    pub blacklisted_at: i64,
}

pub struct ComplianceService {
    provider_api_key: String,
    solana: Arc<SolanaService>,
    /// Optional authority keypair for signing transactions
    authority_keypair: Option<Keypair>,
    /// Cluster name for explorer URLs
    cluster: String,
}

impl ComplianceService {
    pub fn new(api_key: String, solana: Arc<SolanaService>) -> Self {
        Self {
            provider_api_key: api_key,
            solana,
            authority_keypair: None,
            cluster: "devnet".to_string(),
        }
    }
    
    /// Set the authority keypair for signing transactions
    pub fn set_authority_keypair(&mut self, keypair: Keypair) {
        self.authority_keypair = Some(keypair);
    }
    
    /// Set the cluster for explorer URLs
    pub fn set_cluster(&mut self, cluster: String) {
        self.cluster = cluster;
    }
    
    /// Calls external API (Chainalysis/Elliptic) to screen an address
    pub async fn screen_address(&self, address: &str, stablecoin: &Pubkey) -> Result<ScreeningResult> {
        let pubkey = address.parse::<Pubkey>()
            .with_context(|| format!("Invalid address: {}", address))?;
        
        // Check on-chain blacklist first
        let is_blacklisted = self.is_blacklisted_on_chain(stablecoin, &pubkey).await?;
        
        // Call external compliance provider (mock implementation)
        let external_result = self.screen_with_provider(address).await?;
        
        // Combine results
        let recommendation = if is_blacklisted || external_result.is_sanctioned {
            "block".to_string()
        } else if external_result.risk_score > 70 {
            "review".to_string()
        } else {
            "allow".to_string()
        };
        
        Ok(ScreeningResult {
            address: address.to_string(),
            risk_score: external_result.risk_score,
            is_sanctioned: external_result.is_sanctioned,
            is_blacklisted,
            recommendation,
        })
    }
    
    /// Screen address with external compliance provider
    async fn screen_with_provider(&self, address: &str) -> Result<ScreeningResult> {
        tracing::debug!("Screening address {} with provider API", address);
        
        // In production, this would call Chainalysis, Elliptic, or similar APIs
        // For now, we implement a mock that returns low risk for non-sanctioned addresses
        
        // Mock: check against known sanctioned address patterns
        let is_sanctioned = self.check_sanctions_list(address).await?;
        let risk_score = if is_sanctioned { 100 } else { 10 };
        
        Ok(ScreeningResult {
            address: address.to_string(),
            risk_score,
            is_sanctioned,
            is_blacklisted: false, // Will be set by caller
            recommendation: if is_sanctioned { "block" } else { "allow" }.to_string(),
        })
    }
    
    /// Check against sanctions list (mock implementation)
    async fn check_sanctions_list(&self, address: &str) -> Result<bool> {
        // In production, integrate with:
        // - OFAC sanctions list
        // - Chainalysis API
        // - Elliptic API
        // - TRM Labs API
        
        // Mock: return false for all addresses
        // Real implementation would query a sanctions database
        Ok(false)
    }
    
    /// Check if an address is blacklisted on-chain
    async fn is_blacklisted_on_chain(&self, stablecoin: &Pubkey, address: &Pubkey) -> Result<bool> {
        let blacklist_pda = self.solana.find_blacklist_pda(stablecoin, address).0;
        Ok(self.solana.account_exists(&blacklist_pda).await)
    }
    
    /// Get blacklist entry from on-chain state
    pub async fn get_blacklist_entry(&self, stablecoin: &Pubkey, address: &Pubkey) -> Result<Option<BlacklistEntry>> {
        let blacklist_pda = self.solana.find_blacklist_pda(stablecoin, address).0;
        
        if !self.solana.account_exists(&blacklist_pda).await {
            return Ok(None);
        }
        
        let data = self.solana.get_account_data(&blacklist_pda).await?;
        let entry = self.deserialize_blacklist_entry(&data)?;
        
        Ok(Some(BlacklistEntry {
            address: entry.account.to_string(),
            reason: entry.reason,
            blacklisted_by: entry.blacklisted_by.to_string(),
            blacklisted_at: entry.blacklisted_at,
        }))
    }
    
    /// Adds an address to the on-chain SSS-2 Blacklist
    pub async fn enforce_blacklist(
        &self,
        stablecoin: &Pubkey,
        address: &str,
        reason: &str,
    ) -> Result<BlacklistResult> {
        let pubkey = address.parse::<Pubkey>()
            .with_context(|| format!("Invalid address: {}", address))?;
        
        // Get authority keypair
        let authority = match &self.authority_keypair {
            Some(kp) => kp,
            None => {
                return Ok(BlacklistResult {
                    address: address.to_string(),
                    signature: None,
                    explorer_url: None,
                    success: false,
                    error: Some("Authority keypair not set".to_string()),
                });
            }
        };
        
        // Check if already blacklisted
        if self.is_blacklisted_on_chain(stablecoin, &pubkey).await? {
            warn!("Address {} is already blacklisted", address);
            return Ok(BlacklistResult {
                address: address.to_string(),
                signature: None,
                explorer_url: None,
                success: false,
                error: Some("Address is already blacklisted".to_string()),
            });
        }
        
        // Check compliance is enabled on the stablecoin
        let state_data = self.solana.get_account_data(stablecoin).await?;
        let state = self.deserialize_stablecoin_state(&state_data)?;
        
        if !state.compliance_enabled {
            return Ok(BlacklistResult {
                address: address.to_string(),
                signature: None,
                explorer_url: None,
                success: false,
                error: Some("Compliance is not enabled on this stablecoin".to_string()),
            });
        }
        
        // Find blacklist entry PDA
        let (blacklist_pda, _bump) = self.solana.find_blacklist_pda(stablecoin, &pubkey);
        
        // Build instruction
        let instruction = self.solana.build_add_blacklist_instruction(
            stablecoin,
            &authority.pubkey(),
            &pubkey,
            &blacklist_pda,
            reason.to_string(),
        );
        
        // Send transaction
        match self.solana.build_and_send_instruction(vec![instruction], &[]).await {
            Ok(signature) => {
                info!(
                    "Blacklist transaction successful: signature={}, address={}, reason={}",
                    signature, address, reason
                );
                
                Ok(BlacklistResult {
                    address: address.to_string(),
                    signature: Some(signature.to_string()),
                    explorer_url: Some(crate::solana::explorer_url(&signature.to_string(), &self.cluster)),
                    success: true,
                    error: None,
                })
            }
            Err(e) => {
                warn!("Failed to add to blacklist: {}", e);
                Ok(BlacklistResult {
                    address: address.to_string(),
                    signature: None,
                    explorer_url: None,
                    success: false,
                    error: Some(e.to_string()),
                })
            }
        }
    }
    
    /// Removes an address from the on-chain blacklist
    pub async fn remove_blacklist(
        &self,
        stablecoin: &Pubkey,
        address: &str,
    ) -> Result<BlacklistResult> {
        let pubkey = address.parse::<Pubkey>()
            .with_context(|| format!("Invalid address: {}", address))?;
        
        // Get authority keypair
        let authority = match &self.authority_keypair {
            Some(kp) => kp,
            None => {
                return Ok(BlacklistResult {
                    address: address.to_string(),
                    signature: None,
                    explorer_url: None,
                    success: false,
                    error: Some("Authority keypair not set".to_string()),
                });
            }
        };
        
        // Check if blacklisted
        if !self.is_blacklisted_on_chain(stablecoin, &pubkey).await? {
            warn!("Address {} is not blacklisted", address);
            return Ok(BlacklistResult {
                address: address.to_string(),
                signature: None,
                explorer_url: None,
                success: false,
                error: Some("Address is not blacklisted".to_string()),
            });
        }
        
        // Check compliance is enabled
        let state_data = self.solana.get_account_data(stablecoin).await?;
        let state = self.deserialize_stablecoin_state(&state_data)?;
        
        if !state.compliance_enabled {
            return Ok(BlacklistResult {
                address: address.to_string(),
                signature: None,
                explorer_url: None,
                success: false,
                error: Some("Compliance is not enabled on this stablecoin".to_string()),
            });
        }
        
        // Find blacklist entry PDA
        let (blacklist_pda, _bump) = self.solana.find_blacklist_pda(stablecoin, &pubkey);
        
        // Build instruction
        let instruction = self.solana.build_remove_blacklist_instruction(
            stablecoin,
            &authority.pubkey(),
            &pubkey,
            &blacklist_pda,
        );
        
        // Send transaction
        match self.solana.build_and_send_instruction(vec![instruction], &[]).await {
            Ok(signature) => {
                info!(
                    "Unblacklist transaction successful: signature={}, address={}",
                    signature, address
                );
                
                Ok(BlacklistResult {
                    address: address.to_string(),
                    signature: Some(signature.to_string()),
                    explorer_url: Some(crate::solana::explorer_url(&signature.to_string(), &self.cluster)),
                    success: true,
                    error: None,
                })
            }
            Err(e) => {
                warn!("Failed to remove from blacklist: {}", e);
                Ok(BlacklistResult {
                    address: address.to_string(),
                    signature: None,
                    explorer_url: None,
                    success: false,
                    error: Some(e.to_string()),
                })
            }
        }
    }
    
    /// Batch screen multiple addresses
    pub async fn batch_screen(&self, addresses: &[String], stablecoin: &Pubkey) -> Result<Vec<ScreeningResult>> {
        let mut results = Vec::with_capacity(addresses.len());
        
        for address in addresses {
            let result = self.screen_address(address, stablecoin).await?;
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// List all blacklist entries for a stablecoin (paginated)
    pub async fn list_blacklist_entries(
        &self,
        stablecoin: &Pubkey,
        addresses: &[Pubkey],
    ) -> Result<Vec<BlacklistEntry>> {
        let mut entries = Vec::new();
        
        for address in addresses {
            if let Some(entry) = self.get_blacklist_entry(stablecoin, address).await? {
                entries.push(entry);
            }
        }
        
        Ok(entries)
    }
    
    /// Deserialize stablecoin state from account data
    fn deserialize_stablecoin_state(&self, data: &[u8]) -> Result<StablecoinStateAccount> {
        if data.len() < 8 {
            anyhow::bail!("Invalid stablecoin state data length");
        }
        
        let mut slice = &data[8..];
        StablecoinStateAccount::deserialize(&mut slice)
            .context("Failed to deserialize stablecoin state")
    }
    
    /// Deserialize blacklist entry from account data
    fn deserialize_blacklist_entry(&self, data: &[u8]) -> Result<BlacklistEntryAccount> {
        if data.len() < 8 {
            anyhow::bail!("Invalid blacklist entry data length");
        }
        
        let mut slice = &data[8..];
        BlacklistEntryAccount::deserialize(&mut slice)
            .context("Failed to deserialize blacklist entry")
    }
    
    /// Verify compliance authority has required role
    pub async fn verify_blacklister_role(&self, stablecoin: &Pubkey, authority: &Pubkey) -> Result<bool> {
        // Check if authority is the master authority
        let state_data = self.solana.get_account_data(stablecoin).await?;
        let state = self.deserialize_stablecoin_state(&state_data)?;
        
        if state.authority == *authority {
            return Ok(true);
        }
        
        // Check for Blacklister role
        let role_pda = self.solana.find_role_pda(stablecoin, authority, Role::Blacklister.to_seed()).0;
        
        Ok(self.solana.account_exists(&role_pda).await)
    }
}
