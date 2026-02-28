-- Solana Stablecoin Standard - Compliance Features Migration
-- Adds tables for compliance tracking, frozen accounts, and operation limits

--------------------------------------------------------------------------------
-- Frozen accounts table
--------------------------------------------------------------------------------
CREATE TABLE frozen_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stablecoin_id UUID NOT NULL REFERENCES stablecoins(id) ON DELETE CASCADE,
    account_pubkey VARCHAR(44) NOT NULL,
    frozen_amount BIGINT NOT NULL DEFAULT 0,
    reason TEXT NOT NULL,
    frozen_by UUID NOT NULL REFERENCES users(id),
    is_frozen BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(stablecoin_id, account_pubkey)
);

--------------------------------------------------------------------------------
-- Compliance screening history
--------------------------------------------------------------------------------
CREATE TABLE screening_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stablecoin_id UUID REFERENCES stablecoins(id) ON DELETE SET NULL,
    address VARCHAR(44) NOT NULL,
    screen_type VARCHAR(50) NOT NULL,
    result JSONB NOT NULL,
    risk_score INTEGER,
    is_flagged BOOLEAN NOT NULL DEFAULT false,
    screened_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

--------------------------------------------------------------------------------
-- Operation limits table (for minter daily/transaction limits)
--------------------------------------------------------------------------------
CREATE TABLE operation_limits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stablecoin_id UUID NOT NULL REFERENCES stablecoins(id) ON DELETE CASCADE,
    minter_pubkey VARCHAR(44) NOT NULL,
    daily_limit BIGINT,
    per_transaction_limit BIGINT,
    cooldown_seconds INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(stablecoin_id, minter_pubkey)
);

--------------------------------------------------------------------------------
-- Daily operation tracking
--------------------------------------------------------------------------------
CREATE TABLE daily_operations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stablecoin_id UUID NOT NULL REFERENCES stablecoins(id) ON DELETE CASCADE,
    minter_pubkey VARCHAR(44) NOT NULL,
    operation_type VARCHAR(20) NOT NULL,
    total_amount BIGINT NOT NULL DEFAULT 0,
    operation_count INTEGER NOT NULL DEFAULT 0,
    date DATE NOT NULL DEFAULT CURRENT_DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(stablecoin_id, minter_pubkey, operation_type, date)
);

--------------------------------------------------------------------------------
-- Indexes for new tables
--------------------------------------------------------------------------------
CREATE INDEX idx_frozen_accounts_stablecoin ON frozen_accounts(stablecoin_id);
CREATE INDEX idx_frozen_accounts_account ON frozen_accounts(account_pubkey);
CREATE INDEX idx_frozen_accounts_frozen ON frozen_accounts(is_frozen);

CREATE INDEX idx_screening_history_address ON screening_history(address);
CREATE INDEX idx_screening_history_created ON screening_history(created_at DESC);
CREATE INDEX idx_screening_history_flagged ON screening_history(is_flagged);

CREATE INDEX idx_operation_limits_stablecoin ON operation_limits(stablecoin_id);

CREATE INDEX idx_daily_operations_stablecoin ON daily_operations(stablecoin_id);
CREATE INDEX idx_daily_operations_date ON daily_operations(date);

--------------------------------------------------------------------------------
-- Triggers for updated_at
--------------------------------------------------------------------------------
CREATE TRIGGER update_frozen_accounts_updated_at BEFORE UPDATE ON frozen_accounts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_operation_limits_updated_at BEFORE UPDATE ON operation_limits
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_daily_operations_updated_at BEFORE UPDATE ON daily_operations
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
