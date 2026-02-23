use crate::error::StablecoinError;
use anchor_lang::prelude::*;

pub fn safe_add(a: u64, b: u64) -> Result<u64> {
    a.checked_add(b).ok_or(StablecoinError::MathOverflow.into())
}

pub fn safe_sub(a: u64, b: u64) -> Result<u64> {
    a.checked_sub(b).ok_or(StablecoinError::MathOverflow.into())
}

pub fn safe_mul(a: u64, b: u64) -> Result<u64> {
    a.checked_mul(b).ok_or(StablecoinError::MathOverflow.into())
}

pub fn validate_quota(minted: u64, amount: u64, quota: u64) -> Result<()> {
    let new_total = safe_add(minted, amount)?;
    require!(new_total <= quota, StablecoinError::QuotaExceeded);
    Ok(())
}

pub fn update_supply(current: u64, amount: u64, increase: bool) -> Result<u64> {
    if increase {
        safe_add(current, amount)
    } else {
        safe_sub(current, amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_add() {
        assert_eq!(safe_add(100, 200).expect("should add"), 300);
        assert!(safe_add(u64::MAX, 1).is_err());
    }

    #[test]
    fn test_safe_sub() {
        assert_eq!(safe_sub(300, 100).expect("should subtract"), 200);
        assert!(safe_sub(100, 300).is_err());
    }

    #[test]
    fn test_safe_mul() {
        assert_eq!(safe_mul(100, 5).expect("should multiply"), 500);
        assert!(safe_mul(u64::MAX, 2).is_err());
    }

    #[test]
    fn test_validate_quota() {
        assert!(validate_quota(100, 200, 500).is_ok());
        assert!(validate_quota(400, 200, 500).is_err());
    }

    #[test]
    fn test_update_supply_increase() {
        assert_eq!(update_supply(100, 50, true).expect("should increase"), 150);
    }

    #[test]
    fn test_update_supply_decrease() {
        assert_eq!(update_supply(100, 50, false).expect("should decrease"), 50);
    }
}
