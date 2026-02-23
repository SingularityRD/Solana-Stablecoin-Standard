use anchor_lang::prelude::*;

#[error_code]
pub enum StablecoinError {
    #[msg("Amount must be greater than zero")]
    ZeroAmount,
    #[msg("Not authorized for this action")]
    Unauthorized,
    #[msg("Invalid preset - must be 1 (SSS-1) or 2 (SSS-2)")]
    InvalidPreset,
    #[msg("Compliance module not enabled - this is SSS-1")]
    ComplianceNotEnabled,
    #[msg("Transfer blocked by blacklist")]
    BlacklistViolation,
    #[msg("Minter exceeded quota")]
    QuotaExceeded,
    #[msg("Insufficient balance")]
    InsufficientBalance,
    #[msg("Account is frozen")]
    AccountFrozen,
    #[msg("Vault is paused")]
    VaultPaused,
    #[msg("Arithmetic overflow")]
    MathOverflow,
    #[msg("Invalid metadata")]
    InvalidMetadata,
    #[msg("Role already exists")]
    RoleAlreadyExists,
    #[msg("Role not found")]
    RoleNotFound,
    #[msg("Name too long (max 32 chars)")]
    NameTooLong,
    #[msg("Symbol too long (max 16 chars)")]
    SymbolTooLong,
    #[msg("URI too long (max 200 chars)")]
    UriTooLong,
    #[msg("Invalid decimals - must be <= 9")]
    InvalidDecimals,
}
