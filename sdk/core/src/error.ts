export class StablecoinError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'StablecoinError';
  }
}

export class ZeroAmountError extends StablecoinError {
  constructor() { super('Amount must be greater than zero'); }
}

export class UnauthorizedError extends StablecoinError {
  constructor() { super('Not authorized for this action'); }
}

export class InvalidPresetError extends StablecoinError {
  constructor() { super('Invalid preset - must be 1 (SSS-1) or 2 (SSS-2)'); }
}

export class ComplianceNotEnabledError extends StablecoinError {
  constructor() { super('Compliance module not enabled - this is SSS-1'); }
}

export class BlacklistViolationError extends StablecoinError {
  constructor() { super('Transfer blocked by blacklist'); }
}

export class QuotaExceededError extends StablecoinError {
  constructor() { super('Minter exceeded quota'); }
}

export class InsufficientBalanceError extends StablecoinError {
  constructor() { super('Insufficient balance'); }
}

export class AccountFrozenError extends StablecoinError {
  constructor() { super('Account is frozen'); }
}

export class VaultPausedError extends StablecoinError {
  constructor() { super('Vault is paused'); }
}

export class MathOverflowError extends StablecoinError {
  constructor() { super('Arithmetic overflow'); }
}

export class InvalidMetadataError extends StablecoinError {
  constructor() { super('Invalid metadata'); }
}

export class RoleAlreadyExistsError extends StablecoinError {
  constructor() { super('Role already exists'); }
}

export class RoleNotFoundError extends StablecoinError {
  constructor() { super('Role not found'); }
}

export class NameTooLongError extends StablecoinError {
  constructor() { super('Name too long (max 32 chars)'); }
}

export class SymbolTooLongError extends StablecoinError {
  constructor() { super('Symbol too long (max 10 chars)'); }
}

export class UriTooLongError extends StablecoinError {
  constructor() { super('URI too long (max 200 chars)'); }
}

export class InvalidDecimalsError extends StablecoinError {
  constructor() { super('Invalid decimals - must be <= 9'); }
}

export class ReasonTooLongError extends StablecoinError {
  constructor() { super('Reason too long (max 200 chars)'); }
}
