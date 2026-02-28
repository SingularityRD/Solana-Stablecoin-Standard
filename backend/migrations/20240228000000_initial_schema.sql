-- Solana Stablecoin Standard - Initial Schema Migration
-- This migration creates all core tables for the SSS backend

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

--------------------------------------------------------------------------------
-- Users table
--------------------------------------------------------------------------------
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    solana_pubkey VARCHAR(44),
    role VARCHAR(50) NOT NULL DEFAULT 'user',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

--------------------------------------------------------------------------------
-- Stablecoins table
--------------------------------------------------------------------------------
CREATE TABLE stablecoins (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(64) NOT NULL,
    symbol VARCHAR(16) NOT NULL,
    decimals SMALLINT NOT NULL DEFAULT 6,
    preset SMALLINT NOT NULL,
    asset_mint VARCHAR(44) NOT NULL,
    stablecoin_pda VARCHAR(44) NOT NULL UNIQUE,
    authority_pubkey VARCHAR(44) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

--------------------------------------------------------------------------------
-- Role assignments table
--------------------------------------------------------------------------------
CREATE TABLE role_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stablecoin_id UUID NOT NULL REFERENCES stablecoins(id) ON DELETE CASCADE,
    account_pubkey VARCHAR(44) NOT NULL,
    role VARCHAR(50) NOT NULL,
    assigned_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(stablecoin_id, account_pubkey, role)
);

--------------------------------------------------------------------------------
-- Minter quotas table
--------------------------------------------------------------------------------
CREATE TABLE minter_quotas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stablecoin_id UUID NOT NULL REFERENCES stablecoins(id) ON DELETE CASCADE,
    minter_pubkey VARCHAR(44) NOT NULL,
    quota BIGINT NOT NULL DEFAULT 0,
    minted_amount BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(stablecoin_id, minter_pubkey)
);

--------------------------------------------------------------------------------
-- Blacklist entries table
--------------------------------------------------------------------------------
CREATE TABLE blacklist_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stablecoin_id UUID NOT NULL REFERENCES stablecoins(id) ON DELETE CASCADE,
    account_pubkey VARCHAR(44) NOT NULL,
    reason TEXT NOT NULL,
    blacklisted_by UUID NOT NULL REFERENCES users(id),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(stablecoin_id, account_pubkey)
);

--------------------------------------------------------------------------------
-- Audit log table
--------------------------------------------------------------------------------
CREATE TABLE audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stablecoin_id UUID REFERENCES stablecoins(id) ON DELETE SET NULL,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action VARCHAR(100) NOT NULL,
    tx_signature VARCHAR(88),
    details JSONB,
    ip_address INET,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

--------------------------------------------------------------------------------
-- API keys table
--------------------------------------------------------------------------------
CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    key_hash VARCHAR(255) NOT NULL,
    name VARCHAR(100),
    permissions JSONB,
    last_used_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

--------------------------------------------------------------------------------
-- Webhooks table
--------------------------------------------------------------------------------
CREATE TABLE webhooks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stablecoin_id UUID NOT NULL REFERENCES stablecoins(id) ON DELETE CASCADE,
    url VARCHAR(500) NOT NULL,
    events JSONB NOT NULL,
    secret VARCHAR(255),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

--------------------------------------------------------------------------------
-- Indexes for performance
--------------------------------------------------------------------------------

-- Users
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_solana_pubkey ON users(solana_pubkey);

-- Stablecoins
CREATE INDEX idx_stablecoins_owner ON stablecoins(owner_id);
CREATE INDEX idx_stablecoins_pda ON stablecoins(stablecoin_pda);
CREATE INDEX idx_stablecoins_asset_mint ON stablecoins(asset_mint);

-- Role assignments
CREATE INDEX idx_role_assignments_stablecoin ON role_assignments(stablecoin_id);
CREATE INDEX idx_role_assignments_account ON role_assignments(account_pubkey);

-- Minter quotas
CREATE INDEX idx_minter_quotas_stablecoin ON minter_quotas(stablecoin_id);
CREATE INDEX idx_minter_quotas_minter ON minter_quotas(minter_pubkey);

-- Blacklist
CREATE INDEX idx_blacklist_stablecoin ON blacklist_entries(stablecoin_id);
CREATE INDEX idx_blacklist_account ON blacklist_entries(account_pubkey);
CREATE INDEX idx_blacklist_active ON blacklist_entries(is_active);

-- Audit log
CREATE INDEX idx_audit_log_stablecoin ON audit_log(stablecoin_id);
CREATE INDEX idx_audit_log_user ON audit_log(user_id);
CREATE INDEX idx_audit_log_action ON audit_log(action);
CREATE INDEX idx_audit_log_created ON audit_log(created_at DESC);
CREATE INDEX idx_audit_log_tx_signature ON audit_log(tx_signature);

-- API keys
CREATE INDEX idx_api_keys_user ON api_keys(user_id);
CREATE INDEX idx_api_keys_active ON api_keys(is_active);

-- Webhooks
CREATE INDEX idx_webhooks_stablecoin ON webhooks(stablecoin_id);
CREATE INDEX idx_webhooks_active ON webhooks(is_active);

--------------------------------------------------------------------------------
-- Updated_at trigger function
--------------------------------------------------------------------------------
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply trigger to tables with updated_at
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_stablecoins_updated_at BEFORE UPDATE ON stablecoins
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_minter_quotas_updated_at BEFORE UPDATE ON minter_quotas
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
