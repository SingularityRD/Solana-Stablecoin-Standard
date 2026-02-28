-- Solana Stablecoin Standard - Database Initialization
-- This script runs when the PostgreSQL container is first created

-- Set up extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Create schema for better organization
CREATE SCHEMA IF NOT EXISTS sss;

-- Grant permissions to the application user
GRANT ALL PRIVILEGES ON SCHEMA sss TO sss;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA sss TO sss;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA sss TO sss;
ALTER DEFAULT PRIVILEGES IN SCHEMA sss GRANT ALL ON TABLES TO sss;
ALTER DEFAULT PRIVILEGES IN SCHEMA sss GRANT ALL ON SEQUENCES TO sss;

-- Set default search path for the application
ALTER USER sss SET search_path TO sss, public;

-- Performance optimization settings (these can be overridden in postgresql.conf)
-- These are applied at runtime for the database
ALTER SYSTEM SET shared_buffers = '256MB';
ALTER SYSTEM SET effective_cache_size = '1GB';
ALTER SYSTEM SET maintenance_work_mem = '128MB';
ALTER SYSTEM SET checkpoint_completion_target = 0.9;
ALTER SYSTEM SET wal_buffers = '16MB';
ALTER SYSTEM SET default_statistics_target = 100;
ALTER SYSTEM SET random_page_cost = 1.1;
ALTER SYSTEM SET effective_io_concurrency = 200;
ALTER SYSTEM SET work_mem = '16MB';

-- Log initialization
DO $$
BEGIN
    RAISE NOTICE 'SSS Database initialized successfully';
END
$$;
