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
    
    #[error("Invalid argument: {0}")]
    InvalidArg(String),
    
    #[error("Keypair error: {0}")]
    KeypairError(String),
    
    #[error("Transaction failed: {0}")]
    TransactionError(String),
    
    #[error("Account not found: {0}")]
    AccountNotFound(String),
    
    #[error("IO Error: {0}")]
    IoError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Insufficient balance: required {0}, available {1}")]
    InsufficientBalance(u64, u64),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Operation not allowed: {0}")]
    NotAllowed(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Timeout: {0}")]
    Timeout(String),
    
    #[error("Unknown Error: {0}")]
    Unknown(String),
}

impl From<serde_json::Error> for CliError {
    fn from(e: serde_json::Error) -> Self {
        CliError::SerializationError(e.to_string())
    }
}

impl From<std::io::Error> for CliError {
    fn from(e: std::io::Error) -> Self {
        CliError::IoError(e.to_string())
    }
}