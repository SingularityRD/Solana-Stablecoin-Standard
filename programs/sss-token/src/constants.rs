use anchor_lang::prelude::*;

pub const VAULT_SEED: &[u8] = b"stablecoin";
pub const MINTER_SEED: &[u8] = b"minter";
pub const ROLE_SEED: &[u8] = b"role";
pub const BLACKLIST_SEED: &[u8] = b"blacklist";
pub const CONFIG_SEED: &[u8] = b"config";

pub const MAX_NAME_LENGTH: usize = 32;
pub const MAX_SYMBOL_LENGTH: usize = 16;
pub const MAX_URI_LENGTH: usize = 200;

pub const MIN_DEPOSIT_AMOUNT: u64 = 1;
pub const SHARES_DECIMALS: u8 = 9;

pub const PRESET_SSS_1: u8 = 1;
pub const PRESET_SSS_2: u8 = 2;
