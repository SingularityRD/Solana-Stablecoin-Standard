use anyhow::{Context, Result};
use anchor_client::{
    solana_client::{
        rpc_client::RpcClient,
        rpc_config::RpcSendTransactionConfig,
    },
    solana_sdk::{
        instruction::{AccountMeta, Instruction},
        message::Message,
        pubkey::Pubkey,
        signature::{Keypair, Signature, Signer},
        system_program,
        commitment_config::CommitmentConfig,
        transaction::Transaction,
        hash::Hash,
    },
};
use anchor_lang::{AnchorDeserialize, AnchorSerialize, InstructionData};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Seed constants matching the Solana program
pub const VAULT_SEED: &[u8] = b"stablecoin";
pub const ROLE_SEED: &[u8] = b"role";
pub const BLACKLIST_SEED: &[u8] = b"blacklist";
pub const MINTER_SEED: &[u8] = b"minter";

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
    
    /// Get the latest blockhash
    pub async fn get_latest_blockhash(&self) -> Result<Hash> {
        self.rpc_client
            .get_latest_blockhash()
            .context("Failed to get latest blockhash")
    }
    
    /// Find the stablecoin PDA
    pub fn find_stablecoin_pda(&self, asset_mint: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[VAULT_SEED, asset_mint.as_ref()],
            &self.program_id,
        )
    }
    
    /// Find the role assignment PDA
    pub fn find_role_pda(&self, stablecoin: &Pubkey, account: &Pubkey, role: &[u8]) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[ROLE_SEED, stablecoin.as_ref(), account.as_ref(), role],
            &self.program_id,
        )
    }
    
    /// Find the minter info PDA
    pub fn find_minter_pda(&self, stablecoin: &Pubkey, minter: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[MINTER_SEED, stablecoin.as_ref(), minter.as_ref()],
            &self.program_id,
        )
    }
    
    /// Find the blacklist entry PDA
    pub fn find_blacklist_pda(&self, stablecoin: &Pubkey, account: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[BLACKLIST_SEED, stablecoin.as_ref(), account.as_ref()],
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
    
    /// Get account data as raw bytes
    pub async fn get_account_data(&self, pubkey: &Pubkey) -> Result<Vec<u8>> {
        self.rpc_client
            .get_account_data(pubkey)
            .context("Failed to get account data")
    }
    
    /// Check if an account exists
    pub async fn account_exists(&self, pubkey: &Pubkey) -> bool {
        self.rpc_client.get_account(pubkey).is_ok()
    }
    
    /// Get multiple accounts in a batch
    pub async fn get_multiple_accounts(&self, pubkeys: &[Pubkey]) -> Result<Vec<Option<Vec<u8>>>> {
        let accounts = self.rpc_client
            .get_multiple_accounts(pubkeys)
            .context("Failed to get multiple accounts")?;
        
        Ok(accounts.into_iter().map(|opt| opt.map(|acc| acc.data)).collect())
    }
    
    /// Send a transaction and return the signature
    pub async fn send_transaction(&self, transaction: Transaction) -> Result<Signature> {
        let signature = self.rpc_client
            .send_transaction_with_config(
                &transaction,
                RpcSendTransactionConfig {
                    skip_preflight: false,
                    preflight_commitment: Some(CommitmentConfig::confirmed().commitment),
                    ..Default::default()
                },
            )
            .context("Failed to send transaction")?;
        
        info!("Transaction sent: {}", signature);
        Ok(signature)
    }
    
    /// Send a transaction and wait for confirmation
    pub async fn send_and_confirm_transaction(&self, transaction: Transaction) -> Result<Signature> {
        let signature = self.rpc_client
            .send_and_confirm_transaction_with_spinner(&transaction)
            .context("Failed to send and confirm transaction")?;
        
        info!("Transaction confirmed: {}", signature);
        Ok(signature)
    }
    
    /// Build and send a transaction with instructions
    pub async fn build_and_send_instruction(
        &self,
        instructions: Vec<Instruction>,
        signers: &[&Keypair],
    ) -> Result<Signature> {
        let keypair_guard = self.keypair.read().await;
        let authority = keypair_guard.as_ref()
            .context("No authority keypair set")?;
        
        let latest_blockhash = self.get_latest_blockhash().await?;
        
        let mut all_signers: Vec<&Keypair> = vec![authority];
        all_signers.extend(signers);
        
        let transaction = Transaction::new(
            &all_signers,
            Message::new_with_blockhash(&instructions, Some(&authority.pubkey()), &latest_blockhash),
            latest_blockhash,
        );
        
        self.send_and_confirm_transaction(transaction).await
    }
    
    /// Build a mint instruction for the SSS token program
    pub fn build_mint_instruction(
        &self,
        stablecoin: &Pubkey,
        asset_mint: &Pubkey,
        authority: &Pubkey,
        recipient_token_account: &Pubkey,
        amount: u64,
        state_bump: u8,
        role_assignment: Option<(&Pubkey, u8)>,
        minter_info: Option<(&Pubkey, u8)>,
        token_program: &Pubkey,
    ) -> Instruction {
        let mut accounts = vec![
            AccountMeta::new(*authority, true),
            AccountMeta::new(*stablecoin, false),
            AccountMeta::new_readonly(*asset_mint, false),
            AccountMeta::new(*recipient_token_account, false),
            AccountMeta::new_readonly(*token_program, false),
        ];
        
        // Add role assignment PDA if provided
        if let Some((role_pda, _bump)) = role_assignment {
            accounts.insert(2, AccountMeta::new_readonly(*role_pda, false));
        } else {
            // Insert placeholder for optional account
            accounts.insert(2, AccountMeta::new_readonly(system_program::ID, false));
        }
        
        // Add minter info PDA if provided
        if let Some((minter_pda, _bump)) = minter_info {
            accounts.insert(3, AccountMeta::new(*minter_pda, false));
        } else {
            // Insert placeholder for optional account
            accounts.insert(3, AccountMeta::new_readonly(system_program::ID, false));
        }
        
        Instruction {
            program_id: self.program_id,
            accounts,
            data: MintInstruction { amount }.data(),
        }
    }
    
    /// Build a burn instruction for the SSS token program
    pub fn build_burn_instruction(
        &self,
        stablecoin: &Pubkey,
        asset_mint: &Pubkey,
        authority: &Pubkey,
        from_token_account: &Pubkey,
        amount: u64,
        role_assignment: Option<(&Pubkey, u8)>,
        token_program: &Pubkey,
    ) -> Instruction {
        let mut accounts = vec![
            AccountMeta::new(*authority, true),
            AccountMeta::new(*stablecoin, false),
            AccountMeta::new_readonly(*asset_mint, false),
            AccountMeta::new(*from_token_account, false),
            AccountMeta::new_readonly(*token_program, false),
        ];
        
        // Add role assignment PDA if provided
        if let Some((role_pda, _bump)) = role_assignment {
            accounts.insert(2, AccountMeta::new_readonly(*role_pda, false));
        } else {
            // Insert placeholder for optional account
            accounts.insert(2, AccountMeta::new_readonly(system_program::ID, false));
        }
        
        Instruction {
            program_id: self.program_id,
            accounts,
            data: BurnInstruction { amount }.data(),
        }
    }
    
    /// Build an add to blacklist instruction
    pub fn build_add_blacklist_instruction(
        &self,
        stablecoin: &Pubkey,
        authority: &Pubkey,
        account_to_blacklist: &Pubkey,
        blacklist_entry: &Pubkey,
        reason: String,
    ) -> Instruction {
        Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(*authority, true),
                AccountMeta::new(*stablecoin, false),
                AccountMeta::new(*blacklist_entry, false),
                AccountMeta::new_readonly(*account_to_blacklist, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: AddBlacklistInstruction { reason }.data(),
        }
    }
    
    /// Build a remove from blacklist instruction
    pub fn build_remove_blacklist_instruction(
        &self,
        stablecoin: &Pubkey,
        authority: &Pubkey,
        account_to_unblacklist: &Pubkey,
        blacklist_entry: &Pubkey,
    ) -> Instruction {
        Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(*authority, true),
                AccountMeta::new(*stablecoin, false),
                AccountMeta::new(*blacklist_entry, false),
                AccountMeta::new_readonly(*account_to_unblacklist, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: RemoveBlacklistInstruction.data(),
        }
    }
    
    /// Get token account balance (returns raw amount)
    pub async fn get_token_account_balance(&self, token_account: &Pubkey) -> Result<u64> {
        let balance = self.rpc_client
            .get_token_account_balance(token_account)
            .context("Failed to get token account balance")?;
        
        balance.amount.parse::<u64>()
            .context("Failed to parse token balance")
    }
    
    /// Confirm a transaction by signature
    pub async fn confirm_transaction(&self, signature: &Signature) -> Result<bool> {
        let result = self.rpc_client
            .get_signature_status(signature)
            .context("Failed to get transaction status")?;
        
        match result {
            Ok(_) => Ok(true),
            Err(e) => {
                warn!("Transaction failed: {:?}", e);
                Ok(false)
            }
        }
    }
    
    /// Simulate a transaction without sending it
    pub async fn simulate_transaction(&self, transaction: &Transaction) -> Result<()> {
        let result = self.rpc_client
            .simulate_transaction(transaction)
            .context("Failed to simulate transaction")?;
        
        if let Some(err) = result.value.err {
            let logs = result.value.logs.unwrap_or_default();
            anyhow::bail!("Simulation failed: {:?}, logs: {:?}", err, logs);
        }
        
        Ok(())
    }
}

/// Instruction data structures for serialization
#[derive(AnchorSerialize, AnchorDeserialize)]
struct MintInstruction {
    amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
struct BurnInstruction {
    amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
struct AddBlacklistInstruction {
    reason: String,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
struct RemoveBlacklistInstruction;

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

/// Role enum matching the Solana program
#[derive(Debug, Clone, Copy, PartialEq, Eq, AnchorSerialize, AnchorDeserialize)]
pub enum Role {
    Master,
    Minter,
    Burner,
    Blacklister,
    Pauser,
    Seizer,
}

impl Role {
    pub fn to_seed(&self) -> &'static [u8] {
        match self {
            Role::Master => b"master",
            Role::Minter => b"minter",
            Role::Burner => b"burner",
            Role::Blacklister => b"blacklister",
            Role::Pauser => b"pauser",
            Role::Seizer => b"seizer",
        }
    }
}

/// On-chain StablecoinState account structure (matches Solana program)
#[derive(Debug, Clone, AnchorDeserialize)]
pub struct StablecoinStateAccount {
    pub authority: Pubkey,
    pub asset_mint: Pubkey,
    pub total_supply: u64,
    pub paused: bool,
    pub preset: u8,
    pub compliance_enabled: bool,
    pub bump: u8,
}

/// On-chain BlacklistEntry account structure
#[derive(Debug, Clone, AnchorDeserialize)]
pub struct BlacklistEntryAccount {
    pub account: Pubkey,
    pub reason: String,
    pub blacklisted_by: Pubkey,
    pub blacklisted_at: i64,
    pub bump: u8,
}

/// On-chain MinterInfo account structure
#[derive(Debug, Clone, AnchorDeserialize)]
pub struct MinterInfoAccount {
    pub minter: Pubkey,
    pub quota: u64,
    pub minted_amount: u64,
    pub bump: u8,
}

/// On-chain RoleAssignment account structure
#[derive(Debug, Clone, AnchorDeserialize)]
pub struct RoleAssignmentAccount {
    pub role: Role,
    pub account: Pubkey,
    pub assigned_by: Pubkey,
    pub assigned_at: i64,
    pub bump: u8,
}