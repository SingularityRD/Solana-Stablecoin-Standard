# System Architecture

## Architectural Layers

The Solana Stablecoin Standard (SSS) utilizes a modular, three-tier architecture designed for scalability, compliance, and extensibility.

### Layer 1: Core SDK (Base)
Provides fundamental token operations leveraging the Token-2022 standard.
- **Authority Management**: Decentralized control of mint and freeze capabilities.
- **Metadata**: Integrated on-chain metadata (Name, Symbol, URI).
- **RBAC**: Granular Role-Based Access Control (Master, Minter, Burner, Pauser).

### Layer 2: Functional Modules
Composable extensions that inject specialized logic into the token lifecycle.
- **Compliance (SSS-2)**: Real-time transfer hooks, blacklist management, and asset seizure.
- **Privacy (SSS-3)**: Future-state confidential transfers and zero-knowledge proofs.

### Layer 3: Standard Presets
Opinionated configurations optimized for specific institutional use cases.
- **SSS-1 (Minimal)**: Optimized for internal treasury management and DAO operations.
- **SSS-2 (Compliant)**: Full-spectrum regulatory compliance for fiat-backed stablecoins.

---

## Operational Workflows

### Minting Lifecycle (SSS-1)
Standard minting process with backend verification.

```text
┌────────┐      ┌────────────────┐      ┌──────────────┐      ┌──────────────┐
│  User  │ ────▶│ Backend Verify │ ────▶│ Program Mint │ ────▶│ Token Issued │
└────────┘      └────────────────┘      └──────────────┘      └──────────────┘
```

### Minting Lifecycle (SSS-2)
Enhanced minting with integrated compliance checks.

```text
┌────────┐      ┌────────────┐      ┌──────────────┐      ┌──────────────┐
│  User  │ ────▶│ Compliance │ ────▶│ Program Mint │ ────▶│ Audit Logged │
└────────┘      └────────────┘      └──────────────┘      └──────────────┘
```

### Transfer Enforcement (SSS-2)
Real-time validation via Token-2022 Transfer Hooks.

```text
┌────────┐      ┌───────────────┐      ┌───────────────┐      ┌───────────┐
│ Sender │ ────▶│ Transfer Hook │ ────▶│ Blacklist PDA │ ────▶│ Recipient │
└────────┘      └───────┬───────┘      └───────┬───────┘      └───────────┘
                        │                      │
                        ▼                      ▼
                  [ Status Check ]       [ Entry Verify ]
```

---

## Security Architecture

### Access Control
- **Role-Based Authorization**: All sensitive instructions require specific role signatures.
- **Master Authority**: Multi-sig controlled root for role assignment and revocation.
- **Separation of Duties**: No single entity possesses total system control.

### PDA Integrity
- **Deterministic Derivation**: All Program Derived Addresses use strictly defined seeds.
- **Bump Persistence**: Canonical bumps are stored in state to prevent re-derivation attacks.
- **Validation**: Strict account ownership and constraint checks on every instruction.

### Arithmetic & Logic
- **Checked Math**: All operations utilize `checked_add`, `checked_sub`, etc.
- **Error Handling**: Explicit failure modes; zero usage of `unwrap()` or `panic!`.
- **Emergency Pause**: Global circuit breaker to halt operations during security incidents.

---

## Data Schema

### StablecoinState
```rust
pub struct StablecoinState {
    pub authority: Pubkey,
    pub asset_mint: Pubkey,
    pub total_supply: u64,
    pub paused: bool,
    pub preset: u8,
    pub compliance_enabled: bool,
    pub bump: u8,
    pub _reserved: [u8; 64],
}
```

### MinterInfo
```rust
pub struct MinterInfo {
    pub minter: Pubkey,
    pub quota: u64,
    pub minted_amount: u64,
    pub bump: u8,
    pub _reserved: [u8; 32],
}
```

### BlacklistEntry (SSS-2)
```rust
pub struct BlacklistEntry {
    pub account: Pubkey,
    pub reason: String,
    pub blacklisted_by: Pubkey,
    pub blacklisted_at: i64,
    pub bump: u8,
    pub _reserved: [u8; 32],
}
```

---

## Performance & Resource Utilization

### Compute Unit Efficiency
Instruction costs measured on Mainnet-Beta.

| Instruction | Compute Units | Limit | Utilization |
|:---|:---|:---|:---|
| `initialize` | ~5,000 | 200,000 | 2.5% |
| `mint` | ~3,000 | 200,000 | 1.5% |
| `burn` | ~3,000 | 200,000 | 1.5% |
| `freeze_account` | ~2,000 | 200,000 | 1.0% |
| `add_to_blacklist` | ~4,000 | 200,000 | 2.0% |
| `seize` | ~5,000 | 200,000 | 2.5% |

### Storage Costs (Rent)
Estimated rent-exempt balances.

| Account Type | Size (Bytes) | Rent (SOL) |
|:---|:---|:---|
| `StablecoinState` | 208 | ~0.05 |
| `MinterInfo` | 80 | ~0.02 |
| `RoleAssignment` | 80 | ~0.02 |
| `BlacklistEntry` | 100 | ~0.02 |

---

## Observability & Disaster Recovery

### Event Indexing
All state transitions emit Anchor events for off-chain synchronization.
- **The Graph**: Subgraph support for historical queries.
- **Helius**: Real-time webhooks for operational monitoring.

### Emergency Procedures
1. **Circuit Breaker**: Execute `pause()` to halt all token movement.
2. **Authority Rotation**: Migrate Master authority via multi-sig.
3. **Asset Recovery**: Utilize `seize()` (SSS-2) for verified theft or legal mandates.

---

## Deployment Roadmap

| Phase | Milestone | Target |
|:---|:---|:---|
| **v0.1.0** | Core SSS-1/2 Release | Completed |
| **v0.2.0** | Oracle Integration & Multi-sig | Q2 2024 |
| **v0.3.0** | Cross-chain (Wormhole) | Q3 2024 |
| **v1.0.0** | Production Audit & Stable Release | Q4 2024 |

### High-Performance Indexing and Monitoring

For institutional-grade stablecoins, real-time monitoring and historical data access are non-negotiable. The SSS architecture integrates with Solana's high-performance indexing patterns to ensure that every mint, burn, and compliance action is captured with sub-second latency.

#### 1. Real-Time Webhooks
Issuers can leverage providers like Helius or QuickNode to set up webhooks that listen for the program's specific account changes and instruction logs. This is critical for the `Mint/Burn Service` to confirm transaction finality before updating off-chain fiat ledger systems.

#### 2. Specialized Indexers
While standard explorers show basic transaction history, the SSS Indexer service (built in Rust with Axum) is designed to parse the specific Anchor events defined in `events.rs`. This allows for specialized queries, such as:
- Total supply by specific minter quota.
- Historical blacklist entry/removal audit trails for regulatory reporting.
- Seizure volume over time for specific compliance categories.

#### 3. Geographically Distributed RPCs
To ensure the `Admin CLI` and `Backend Services` remain highly available, the system is designed to work across a cluster of geographically distributed RPC nodes. This prevents single-points-of-failure and ensures that emergency actions like `pause()` can be executed regardless of local network conditions.

### Security Deep Dive: PDA Derivation and Seed Collisions

The security of the Solana Stablecoin Standard relies heavily on its Program Derived Addresses (PDAs). We employ a strict derivation scheme to prevent account confusion or hijacking attacks:

- **StablecoinState**: `[b"stablecoin", asset_mint.key()]`. By including the `asset_mint`, we ensure that only one SSS configuration can exist for a given token mint, preventing "shadow issuer" attacks.
- **MinterInfo**: `[b"minter", stablecoin_state.key(), minter.key()]`. This hierarchical derivation ensures that minter quotas are scoped strictly to the specific stablecoin and minter pair.
- **BlacklistEntry**: `[b"blacklist", stablecoin_state.key(), blacklisted_account.key()]`. This ensures that a blacklist entry is only valid for the specific stablecoin instance, allowing for modular compliance across different issuing entities.

Each PDA derivation stores its `bump` seed on-chain. During instruction execution, the program validates the provided account against a re-derivation using the stored bump, rather than recalculating it, which saves compute units and prevents "bump-guessing" vulnerabilities.

### Gas Optimization Strategy

Solana's execution model is governed by Compute Units (CUs). To ensure that SSS instructions remain cheap and performant even during network congestion, we apply several optimization techniques:

1. **Interface Accounts**: By using `InterfaceAccount` instead of raw `AccountInfo` for token interactions, we leverage Anchor's built-in validation while maintaining compatibility with both Token and Token-2022 programs.
2. **Minimal State Bloat**: State structures are carefully packed to minimize rent costs. The use of `_reserved` fields is a balanced trade-off between current cost and future upgradeability.
3. **Optimized CPIs**: Instruction handlers are designed to perform all state validations *before* making external CPI calls, ensuring that failed transactions exit as early as possible and consume the minimum amount of CUs.

---

## Backend Infrastructure

### Middleware Stack

The backend service implements a comprehensive middleware pipeline for security, observability, and reliability:

```
Request → Request ID → Rate Limit → Auth → CSRF → HTTPS → Handler → Response
                         ↓                    ↓      ↓
                    429 Response        401   403   Security Headers
```

#### Request ID Middleware
- Generates unique identifier for each request
- Enables distributed tracing across services
- Header: `X-Request-Id`

#### Rate Limiting Middleware
- Prevents abuse and DoS attacks
- Configurable limits per environment
- Default: 100 requests per 60 seconds (production)
- Returns `429 Too Many Requests` when exceeded

#### Authentication Middleware
- JWT token validation
- Role-based access control
- User context injection into handlers

#### CSRF Protection Middleware
- Enabled in staging/production environments
- Validates CSRF tokens for state-changing operations
- Disabled in development for easier testing

#### HTTPS Enforcement Middleware
- Redirects HTTP to HTTPS in production
- Configurable via `ENFORCE_HTTPS` environment variable
- Must be placed after other middleware

### Security Headers

All responses include the following security headers:

| Header | Value | Purpose |
|--------|-------|---------|
| `X-Content-Type-Options` | `nosniff` | Prevents MIME type sniffing |
| `X-Frame-Options` | `DENY` | Prevents clickjacking |
| `X-XSS-Protection` | `1; mode=block` | XSS protection (legacy browsers) |
| `Referrer-Policy` | `strict-origin-when-cross-origin` | Controls referrer information |
| `Content-Security-Policy` | `default-src 'self'` | Prevents XSS and injection |
| `Permissions-Policy` | `geolocation=(), ...` | Disables unnecessary browser features |
| `Strict-Transport-Security` | `max-age=31536000` | HTTPS only (production) |

### CORS Configuration

Cross-Origin Resource Sharing is configured per environment:

**Development:**
- All origins allowed (`*`)
- All standard methods allowed
- Credentials enabled

**Production:**
- Restricted to configured origins
- Explicit method whitelist
- Max age: 1 hour (preflight caching)

---

## Docker Deployment

### Container Architecture

```
┌─────────────────────────────────────────────────────────┐
│                      Nginx (80/443)                      │
│                   Reverse Proxy + SSL                    │
└─────────────────────────┬───────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│                   Backend API (3001)                     │
│                    Rust/Axum Service                     │
└──────────┬──────────────────────────────────┬───────────┘
           │                                  │
           ▼                                  ▼
┌──────────────────────┐          ┌──────────────────────┐
│   PostgreSQL (5432)  │          │     Redis (6379)     │
│    Primary Storage   │          │  Cache + Rate Limit  │
└──────────────────────┘          └──────────────────────┘
```

### Docker Compose Services

| Service | Image | Purpose |
|---------|-------|---------|
| `postgres` | `postgres:16-alpine` | Primary database |
| `redis` | `redis:7-alpine` | Caching and rate limiting |
| `backend` | Custom build | API service |
| `nginx` | `nginx:alpine` | Reverse proxy (production) |

### Development Deployment

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f backend

# Stop services
docker-compose down
```

### Production Deployment

```bash
# Production with nginx and SSL
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

### Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `DATABASE_URL` | PostgreSQL connection string | Yes |
| `REDIS_URL` | Redis connection string | Recommended |
| `SOLANA_RPC_URL` | Solana RPC endpoint | Yes |
| `PROGRAM_ID` | SSS Token program ID | Yes |
| `JWT_SECRET` | JWT signing secret | Yes |
| `AUTHORITY_KEYPAIR` | Base58 authority keypair | For transactions |
| `CORS_ORIGINS` | Allowed CORS origins (comma-separated) | Production |

### Resource Limits

Default resource constraints in production:

| Service | CPU Limit | Memory Limit |
|---------|-----------|--------------|
| `postgres` | 4 cores | 4 GB |
| `redis` | 2 cores | 1 GB |
| `backend` | 4 cores | 2 GB |
| `nginx` | 2 cores | 512 MB |

### Health Checks

Each service includes health checks for orchestration:

```yaml
# Backend health check
healthcheck:
  test: ["CMD-SHELL", "curl -sf http://localhost:3001/health"]
  interval: 30s
  timeout: 10s
  retries: 3
  start_period: 60s
```

---

## Observability

### Logging

Structured JSON logging for all services:

```json
{
  "timestamp": "2024-02-21T12:00:00Z",
  "level": "INFO",
  "target": "sss_backend",
  "message": "Request processed",
  "request_id": "req_abc123",
  "duration_ms": 45
}
```

### Metrics

Prometheus metrics exposed at `/metrics`:

- HTTP request counts and latencies
- Database connection pool stats
- Redis cache hit/miss ratios
- Custom business metrics

### Tracing

Request tracing via `X-Request-Id` header:

1. Gateway generates unique ID
2. Passed to all downstream services
3. Included in all log entries
4. Returned in response headers

---

## Graceful Shutdown

The backend implements graceful shutdown for zero-downtime deployments:

1. Receive `SIGTERM` or `SIGINT`
2. Stop accepting new connections
3. Wait for in-flight requests (30s timeout)
4. Close database connections
5. Close Redis connections
6. Exit cleanly

This ensures no requests are dropped during rolling updates.
