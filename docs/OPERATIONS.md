# Operator Runbook

This guide provides instructions for enterprise operators managing the Solana Stablecoin Standard (SSS) tokens.

## Prerequisites

- Solana CLI installed and configured
- Admin keypair with **Master** role
- `sss-token` CLI built and available in PATH
- RPC endpoint with sufficient rate limits

## Initialization

### Initialize SSS-1 (Minimal)
Designed for internal tokens and DAO treasuries.

```bash
sss-token init \
  --preset 1 \
  --name "My Stablecoin" \
  --symbol "MYUSD" \
  --uri "https://example.com/metadata.json" \
  --decimals 6
```

### Initialize SSS-2 (Compliant)
Designed for regulated, fiat-backed stablecoins.

```bash
sss-token init \
  --preset 2 \
  --name "Compliant Stable" \
  --symbol "CUSD" \
  --uri "https://example.com/metadata.json" \
  --decimals 6
```

## Daily Operations

### Mint Tokens
Requires **Minter** role and sufficient quota.

```bash
sss-token mint <recipient_address> <amount>
```

### Burn Tokens
Requires **Burner** role.

```bash
sss-token burn <amount>
```

### Freeze Account
Prevents a specific account from transferring tokens. Requires **Master** or **Blacklister** role.

```bash
sss-token freeze <account_address>
```

### Thaw Account
Restores transfer capabilities to a frozen account.

```bash
sss-token thaw <account_address>
```

### Pause Operations (Emergency)
Global stop for all token transfers, mints, and burns. Requires **Pauser** role.

```bash
sss-token pause
```

### Unpause Operations
Resumes all token operations.

```bash
sss-token unpause
```

## SSS-2 Compliance Operations

### Blacklist Management
Enforced via transfer hooks in SSS-2.

```bash
# Add to blacklist
sss-token blacklist add <account_address> --reason "OFAC sanctions match"

# Remove from blacklist
sss-token blacklist remove <account_address>

# List all blacklisted accounts
sss-token blacklist list
```

### Seize Tokens
Confiscate tokens from a blacklisted account. Requires **Seizer** role.

```bash
sss-token seize <from_account> --to <treasury_address> <amount>
```

## Role Management

### Manage Minters
Control who can issue new tokens and their limits.

```bash
# Add a new minter with quota
sss-token minters add <minter_address> --quota 1000000

# Update minter quota
sss-token minters set-quota <minter_address> 5000000

# View minter info and remaining quota
sss-token minters info <minter_address>

# Remove minter role
sss-token minters remove <minter_address>

# List all active minters
sss-token minters list
```

### General Role Assignment
Assign specific roles to accounts.

```bash
# Assign a role (Master, Minter, Burner, Blacklister, Pauser, Seizer)
sss-token assign-role <role> <account_address>
```

## Monitoring & Reporting

### System Status
Check the current state of the stablecoin program.

```bash
# View general status
sss-token status

# Export full state to JSON for auditing
sss-token status --export state.json
```

### Supply & Holders
Monitor token distribution.

```bash
# Check total circulating supply
sss-token supply

# List holders with balance above threshold
sss-token holders --min-balance 1000
```

### Audit Logs
Review on-chain actions for compliance.

```bash
# View recent mint actions
sss-token audit-log --action mint

# Export audit trail for a specific period
sss-token audit-log \
  --from 2024-01-01 \
  --to 2024-12-31 \
  --format csv \
  --output audit-2024.csv
```

## Emergency Procedures

### Compromised Admin Key
1. **Pause immediately**: `sss-token pause`
2. **Transfer authority**: `sss-token transfer-authority <new_secure_key>`
3. **Review audit log**: `sss-token audit-log` to identify unauthorized actions.
4. **Notify stakeholders** and compliance teams.

### Blacklist Bypass Attempt
1. Verify transfer hook is enabled: `sss-token status`
2. Review failed transactions in logs.
3. Ensure attacker address is blacklisted: `sss-token blacklist add <address> --reason "Bypass attempt"`
4. Consider a temporary pause if a systematic vulnerability is suspected.

## Best Practices

- **Multi-Sig**: Always use a multi-sig (e.g., Squads) for the **Master** authority in production.
- **Quotas**: Set conservative minter quotas and increase them only as needed.
- **Monitoring**: Integrate `sss-token status` and `supply` into your daily monitoring dashboard.
- **Key Rotation**: Rotate operational keys (Minter, Burner) quarterly.
- **Testing**: Always verify complex operations on Devnet before executing on Mainnet.

## Troubleshooting

### Issue: Mint Fails with "Unauthorized"
1. Verify you have the **Minter** role: `sss-token minters list`
2. Check your remaining quota: `sss-token minters info <your-address>`
3. Ensure the vault is not paused: `sss-token status`

### Issue: Blacklist Not Enforcing
1. Verify the address is in the blacklist: `sss-token blacklist list`
2. Ensure the token is initialized with **SSS-2** preset.
3. Check if the transfer hook is correctly configured in the metadata.

### Issue: RPC Connection Errors
1. Verify your `ANCHOR_PROVIDER_URL` environment variable.
2. Check for RPC rate limiting or outages.
3. Switch to a backup RPC provider if necessary.

### Disaster Recovery and Emergency Response

In the event of a smart contract compromise, key loss, or large-scale network failure, the SSS framework includes built-in disaster recovery procedures.

#### 1. Immediate Vault Pausing
If a vulnerability is suspected, any account with the `Pauser` role should immediately execute the `sss-token pause` command. This effectively "freezes" the entire stablecoin ecosystem, preventing any further minting, burning, or transfers until the threat is neutralized.

#### 2. Authority Handover (Multi-Sig Recovery)
If an individual administrator's key is lost or compromised, the `Master` authority (ideally a multi-sig like Squads or Realms) must execute a `transfer_authority` instruction to a new, secure keypair. This ensures that the system's management functions remain accessible to the governing entity.

#### 3. State Reconstruction via Indexer
Should the on-chain data become inconsistent (e.g., due to an extreme fork or RPC failure), the `Event Indexer` service maintains a historical database of all transactions. This allows the issuer to reconstruct the balance sheet and verify current holder states against off-chain fiat reserve balances.

#### 4. Regulatory Kill-Switch
In extreme cases where a stablecoin must be decommissioned by order of a regulator, the `Seizer` role can be used to move all circulating supply back into a treasury account before the program is closed or the mint is revoked. This provides a clear, auditable path for the liquidation of a digital asset.

### Regular System Health Audits

To maintain the 100/100 quality standard in production, operators should conduct weekly system health audits. This involves running the `sss-token status` command and cross-referencing the on-chain `total_supply` with the off-chain bank statements of the fiat reserve. Any discrepancy should be investigated immediately. Additionally, the `RoleManagement` configuration should be reviewed monthly to ensure that only current, authorized employees hold operational keys. Regular rotations of the `Pauser` and `Blacklister` keys are highly recommended to prevent long-term exposure in case of a low-level device compromise.

---

## CI/CD Pipeline

The project includes a comprehensive CI/CD pipeline configured via GitHub Actions.

### Pipeline Stages

```
┌─────────────┐   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐
│    Build    │──▶│    Test     │──▶│   Security  │──▶│   Deploy    │
└─────────────┘   └─────────────┘   └─────────────┘   └─────────────┘
```

### Build Jobs

| Job | Description | Trigger |
|-----|-------------|---------|
| `build-solana-program` | Build SBF programs | All pushes |
| `build-backend` | Build Rust backend | All pushes |
| `build-cli` | Build CLI binary | All pushes |
| `build-admin-tui` | Build Admin TUI | All pushes |
| `build-sdk` | Build TypeScript SDK | All pushes |
| `build-frontend` | Build Next.js frontend | All pushes |

### Test Jobs

| Job | Description | Dependencies |
|-----|-------------|--------------|
| `test-rust` | Run Rust unit tests | `build-backend`, `build-cli` |
| `test-typescript` | Run SDK tests | `build-sdk` |
| `test-anchor` | Run Anchor integration tests | `build-solana-program` |

### Security Jobs

| Job | Description |
|-----|-------------|
| `security-cargo-audit` | Check for vulnerable dependencies |
| `security-npm-audit` | Check npm packages for vulnerabilities |

### Deploy Jobs

| Job | Environment | Trigger |
|-----|-------------|---------|
| `deploy-docker` | Docker Registry | main branch |
| `deploy-programs` | Mainnet | main branch + secrets |

### CI/CD Configuration

The pipeline is configured in `.github/workflows/ci.yml`:

```yaml
env:
  SOLANA_VERSION: '1.18.0'
  NODE_VERSION: '20'
  ANCHOR_VERSION: '0.29.0'
```

### Required Secrets

For production deployments, configure the following secrets in GitHub:

| Secret | Description |
|--------|-------------|
| `SOLANA_RPC_URL` | Mainnet RPC endpoint |
| `DEPLOY_KEYPAIR` | Keypair for program deployment |
| `GITHUB_TOKEN` | Container registry access (auto-provided) |

### Branch Strategy

```
main ──────●──────●──────●──────→ Production
           │      │      │
develop ───●──────●──────●──────→ Staging
           │
feature/   ●──────●────────────→ PR Testing
```

- **main**: Production deployments
- **develop**: Staging/preview deployments
- **feature/***: PR testing and validation

---

## Docker Deployment

### Prerequisites

- Docker 20.10+
- Docker Compose 2.0+
- At least 8GB RAM for full stack

### Quick Start (Development)

```bash
# Clone repository
git clone https://github.com/your-org/solana-stablecoin-standard.git
cd solana-stablecoin-standard

# Create .env file
cp .env.example .env
# Edit .env with your configuration

# Start services
docker-compose up -d

# Check health
curl http://localhost:3001/health/detail
```

### Production Deployment

```bash
# Set production environment
export ENVIRONMENT=production

# Deploy with production overrides
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d

# Verify deployment
docker-compose ps
docker-compose logs -f backend
```

### Container Configuration

#### Backend Service

```yaml
environment:
  - SERVER_ADDR=0.0.0.0:3001
  - DATABASE_URL=postgresql://...
  - REDIS_URL=redis://...
  - SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
  - PROGRAM_ID=SSSToken11111111111111111111111111111111111
  - JWT_SECRET=your-secret-key
  - CORS_ORIGINS=https://your-frontend.com
```

#### PostgreSQL

```yaml
environment:
  - POSTGRES_USER=sss
  - POSTGRES_PASSWORD=secure-password
  - POSTGRES_DB=sss_db
volumes:
  - postgres_data:/var/lib/postgresql/data
  - ./docker/postgres/init.sql:/docker-entrypoint-initdb.d/init.sql
```

#### Redis

```yaml
command: >
  redis-server
  --appendonly yes
  --maxmemory 512mb
  --maxmemory-policy allkeys-lru
  --requirepass ${REDIS_PASSWORD}
```

#### Nginx (Production Only)

```yaml
ports:
  - "80:80"
  - "443:443"
volumes:
  - ./docker/nginx/nginx.conf:/etc/nginx/nginx.conf:ro
  - ./docker/nginx/ssl:/etc/nginx/ssl:ro
```

### Scaling

Scale backend horizontally:

```bash
# Scale to 3 backend instances
docker-compose up -d --scale backend=3

# With production config
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d --scale backend=3
```

### Database Migrations

```bash
# Run migrations (automatic on startup)
docker-compose exec backend /app/migrate

# Manual migration
docker-compose exec backend sqlx migrate run
```

### Backup and Recovery

#### PostgreSQL Backup

```bash
# Create backup
docker-compose exec postgres pg_dump -U sss sss_db > backup_$(date +%Y%m%d).sql

# Restore from backup
cat backup_20240221.sql | docker-compose exec -T postgres psql -U sss sss_db
```

#### Redis Backup

```bash
# Trigger RDB save
docker-compose exec redis redis-cli BGSAVE

# Copy backup
docker cp sss-redis:/data/dump.rdb redis_backup_$(date +%Y%m%d).rdb
```

---

## Kubernetes Deployment

### Kubernetes Probes

The backend exposes endpoints for Kubernetes probes:

| Endpoint | Purpose | Success Criteria |
|----------|---------|------------------|
| `/health/live` | Liveness probe | Service running |
| `/health/ready` | Readiness probe | DB + RPC healthy |
| `/health/detail` | Detailed status | All components healthy |

### Probe Configuration

```yaml
livenessProbe:
  httpGet:
    path: /health/live
    port: 3001
  initialDelaySeconds: 10
  periodSeconds: 10
  
readinessProbe:
  httpGet:
    path: /health/ready
    port: 3001
  initialDelaySeconds: 30
  periodSeconds: 10
  failureThreshold: 3
```

### Horizontal Pod Autoscaler

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: sss-backend-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: sss-backend
  minReplicas: 2
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
```

---

## Monitoring and Alerting

### Prometheus Metrics

Available at `/metrics`:

```
# HTTP metrics
http_requests_total{method="GET",path="/health",status="200"} 1234
http_request_duration_seconds{method="GET",path="/api/v1/stablecoin"} 0.045

# Database metrics
db_connections_active 5
db_connections_idle 10
db_query_duration_seconds{query="get_stablecoin"} 0.002

# Business metrics
stablecoin_mints_total 1000
stablecoin_burns_total 500
stablecoin_total_supply 1000000000
```

### Grafana Dashboard

Recommended dashboards:

1. **System Overview**: CPU, memory, disk, network
2. **API Performance**: Request rates, latencies, errors
3. **Database Health**: Connections, queries, locks
4. **Business Metrics**: Supply, holders, compliance actions

### Alert Rules

```yaml
groups:
  - name: sss-alerts
    rules:
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.1
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"
          
      - alert: DatabaseConnectionPoolExhausted
        expr: db_connections_active / db_connections_max > 0.9
        for: 2m
        labels:
          severity: warning
          
      - alert: RPCHealthCheckFailed
        expr: solana_rpc_health == 0
        for: 1m
        labels:
          severity: critical
```

---

## Security Operations

### Key Management

1. **Authority Keypair**: Stored encrypted in environment
2. **JWT Secret**: Rotated quarterly
3. **Database Credentials**: Managed via secrets manager
4. **API Keys**: Scoped and rotated regularly

### Audit Logging

All administrative actions are logged:

```json
{
  "timestamp": "2024-02-21T12:00:00Z",
  "action": "stablecoin.pause",
  "actor": "admin@example.com",
  "stablecoin_id": "uuid",
  "ip_address": "192.168.1.1",
  "user_agent": "sss-cli/0.1.0"
}
```

### Incident Response

1. **Detection**: Alerts from monitoring
2. **Triage**: Identify severity and scope
3. **Containment**: Pause vault if necessary
4. **Investigation**: Review audit logs
5. **Recovery**: Execute remediation plan
6. **Post-mortem**: Document and improve
