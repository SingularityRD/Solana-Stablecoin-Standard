use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Configuration file not found: {0}")]
    ConfigNotFound(String),
    
    #[error("Invalid configuration format: {0}")]
    InvalidConfig(String),
    
    #[error("RPC Error: {0}")]
    RpcError(#[from] solana_client::client_error::ClientError),
    
    #[error("Anchor Client Error: {0}")]
    AnchorError(#[from] anchor_client::ClientError),
    
    #[error("Invalid Pubkey format: {0}")]
    InvalidPubkey(String),
    
    #[error("Unknown Error: {0}")]
    Unknown(String),
}
