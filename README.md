# Solana Stablecoin Standard (SSS)

The Solana Stablecoin Standard (SSS) is a production-grade, modular framework and software development kit (SDK) designed for the deployment of institutional-grade stablecoins on the Solana blockchain. Developed for Superteam Brazil, this framework facilitates the creation of fully compliant, customizable stablecoins utilizing the Solana Token-2022 standard.

## Overview

The SSS provides a comprehensive foundation for stablecoin issuance, abstracting the complexities associated with Token-2022 extensions and regulatory compliance into a streamlined, configurable architecture.

The framework is organized into three primary layers:
1. **Core SDK**: Facilitates token initialization, authority management (mint/freeze), metadata integration, and Role-Based Access Control (RBAC).
2. **Modular Extensions**: Composable components providing specialized capabilities, including compliance and privacy modules.
3. **Standardized Presets**: Pre-configured deployment profiles (SSS-1 and SSS-2) optimized for immediate institutional use.

## Standard Presets

### SSS-1: Minimal Stablecoin
Optimized for internal ecosystem tokens, DAO treasuries, and settlement layers.
- **Core Capabilities**: Mint authority, freeze authority, and on-chain metadata management.
- **Compliance Model**: Reactive enforcement via manual account freezing.
- **Technical Implementation**: Minimal Token-2022 extensions to maximize compatibility and minimize computational overhead.

### SSS-2: Compliant Stablecoin
Engineered for regulated, fiat-backed stablecoins requiring strict adherence to compliance standards.
- **Core Capabilities**: Inherits all SSS-1 functionality.
- **Compliance Model**: Proactive, real-time on-chain enforcement.
- **Technical Implementation**:
  - **Transfer Hook**: Mandatory real-time validation against authorized blacklists for every transaction.
  - **Permanent Delegate**: Authorized asset seizure capabilities for regulatory compliance.
  - **Default Account State**: Configurable initial frozen state to facilitate mandatory KYC/AML workflows.

## SDK Usage

The `@stbr/sss-token` TypeScript SDK provides a high-level interface for interacting with SSS-compliant programs.

### Installation

```bash
npm install @stbr/sss-token
# or
yarn add @stbr/sss-token
```

### Initialization

Initialize the SDK client by providing a Solana connection, configuration parameters, and the Anchor program instance.

```typescript
import { SolanaStablecoin, Presets } from "@stbr/sss-token";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { Program } from "@coral-xyz/anchor";

const connection = new Connection("https://api.mainnet-beta.solana.com");
const authority = Keypair.generate();
const assetMint = Keypair.generate(); // New mint to be initialized

// Initialize the stablecoin state
const stablecoin = await SolanaStablecoin.create(
  connection,
  {
    authority,
    assetMint: assetMint.publicKey,
    name: "Institutional USD",
    symbol: "iUSD",
    uri: "https://example.com/metadata.json",
    decimals: 6,
    preset: Presets.SSS_2,
  },
  program // Anchor Program instance
);
```

### Supply Management

Execute supply management operations via authorized roles.

```typescript
// Mint tokens to a recipient
await stablecoin.mint(
  authority,
  recipientPublicKey,
  1000000 // 1.00 iUSD (assuming 6 decimals)
);

// Burn tokens from an account
await stablecoin.burn(
  authority,
  sourcePublicKey,
  500000 // 0.50 iUSD
);
```

### Account Management

Manage account states for SSS-compliant tokens.

```typescript
// Freeze an account
await stablecoin.freeze(authority, targetAccount);

// Thaw an account
await stablecoin.thaw(authority, targetAccount);
```

### Compliance Operations

Manage blacklists and execute asset seizures for SSS-2 tokens.

```typescript
// Add an account to the blacklist
await stablecoin.compliance.blacklistAdd(
  authority,
  maliciousAccount,
  "Regulatory requirement"
);

// Remove an account from the blacklist
await stablecoin.compliance.blacklistRemove(
  authority,
  rehabilitatedAccount
);

// Seize tokens from a blacklisted account
await stablecoin.compliance.seize(
  authority,
  blacklistedAccount,
  recoveryAccount,
  1000000
);
```

## Architecture and Deliverables

This repository adheres to the structural and quality benchmarks established by the Solana Vault Standard (SVS).

### 1. On-Chain Programs (Anchor)
- Unified Anchor program supporting multiple presets via initialization parameters.
- Granular Role-Based Access Control (Master, Minter, Burner, Blacklister, Pauser, Seizer).
- Deterministic failure modes: SSS-2 specific instructions revert gracefully if the compliance module is not initialized.

### 2. TypeScript SDK
- Fully typed interface for seamless integration with frontend and backend services.
- Deterministic PDA derivation utilities.
- Comprehensive error mapping between on-chain programs and client-side logic.

### 3. Backend Infrastructure
Rust-based services utilizing the Axum framework, designed for containerized orchestration.
- **Issuance Service**: Coordinates the fiat-to-stablecoin lifecycle.
- **Event Indexer**: High-performance monitoring of on-chain events for state synchronization.
- **Compliance Service**: Manages global blacklist states and integrates with external sanctions screening providers.
- **Notification Service**: Configurable webhook delivery for system events.

### 4. Administrative CLI
A high-performance Rust CLI for rapid operational execution.
- Token initialization via presets or custom TOML configurations.
- Lifecycle management (mint, burn, freeze, seize).
- Role administration and real-time system telemetry.

## Advanced Modules

The SSS architecture supports extensible modules for specialized use cases:

1. **SSS-3 Private Stablecoin (PoC)**: Implementation of confidential transfers utilizing Pedersen commitments and scoped allowlists.
2. **Oracle Integration**: Dedicated Anchor program for Switchboard oracle integration, supporting non-USD pegs (e.g., EUR, BRL, CPI-indexed).
3. **Operator TUI**: A Ratatui-based terminal interface for real-time monitoring of supply metrics and transaction logs.
4. **Reference Implementation**: A Next.js application demonstrating end-to-end stablecoin management.

## Repository Structure

```text
solana-stablecoin-standard/
├── programs/
│   ├── sss-token/          # Primary stablecoin Anchor program
│   └── oracle-module/      # Oracle integration program
├── sdk/
│   └── core/               # TypeScript SDK (@stbr/sss-token)
├── cli/                    # Rust Administrative CLI
├── backend/                # Rust/Axum backend services
├── admin-tui/              # Terminal User Interface for operators
├── example-frontend/       # Next.js reference implementation
├── docs/                   # Technical documentation
├── tests/                  # Integration test suite
└── trident-tests/          # Fuzz testing suite
```

## Documentation

Detailed technical documentation is available in the `/docs` directory:
- `ARCHITECTURE.md`: System design, data flow, and security architecture.
- `SDK.md`: SDK reference and implementation examples.
- `OPERATIONS.md`: Operational procedures and emergency protocols.
- `SSS-1.md`: Technical specification for SSS-1.
- `SSS-2.md`: Technical specification for SSS-2.
- `COMPLIANCE.md`: Regulatory framework and audit logging standards.
- `API.md`: Backend service API specifications.
- `BONUS.md`: Documentation for advanced modules.

## Getting Started

### Prerequisites
- Rust 1.82.0 or higher
- Node.js 20+ and Yarn
- Solana CLI 1.18.0+
- Anchor CLI 0.29.0+
- Docker and Docker Compose (for containerized deployment)

### Build Instructions

```bash
# Install dependencies
yarn install

# Compile Anchor programs
anchor build

# Build TypeScript SDK
cd sdk/core && yarn build

# Build CLI and Backend services
cargo build --workspace
```

### Testing

```bash
# Execute program unit tests
cargo test -p sss-token

# Execute integration tests
anchor test

# Execute fuzz tests
cd trident-tests && cargo fuzz run fuzz_initialize
```

---

## CI/CD Pipeline

The project includes a comprehensive CI/CD pipeline via GitHub Actions that runs on every push and pull request.

### Pipeline Overview

| Stage | Jobs | Description |
|-------|------|-------------|
| Build | 6 jobs | Build all components (programs, backend, CLI, SDK, frontend) |
| Test | 3 jobs | Run Rust, TypeScript, and Anchor tests |
| Security | 2 jobs | Cargo audit and NPM audit |
| Deploy | 2 jobs | Docker images and program deployment (main only) |

### Build Jobs

- `build-solana-program`: Compiles SBF programs for Solana
- `build-backend`: Builds the Rust/Axum backend
- `build-cli`: Builds the CLI binary
- `build-admin-tui`: Builds the Admin TUI
- `build-sdk`: Builds the TypeScript SDK
- `build-frontend`: Builds the Next.js frontend

### Security Scanning

- **Cargo Audit**: Scans Rust dependencies for vulnerabilities
- **NPM Audit**: Scans Node.js dependencies for vulnerabilities

### Deployment

On push to `main` branch:
1. Docker images are built and pushed to GitHub Container Registry
2. Solana programs can be deployed to mainnet (requires secrets configuration)

### Required Secrets

For production deployments, configure these secrets in GitHub:

| Secret | Description |
|--------|-------------|
| `SOLANA_RPC_URL` | Mainnet RPC endpoint |
| `DEPLOY_KEYPAIR` | Keypair for program deployment |

---

## Docker Deployment

### Quick Start (Development)

```bash
# Clone and configure
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
# Production with nginx reverse proxy
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

### Services

| Service | Port | Description |
|---------|------|-------------|
| `postgres` | 5432 | PostgreSQL database |
| `redis` | 6379 | Redis cache |
| `backend` | 3001 | API server |
| `nginx` | 80/443 | Reverse proxy (production) |

### Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `DATABASE_URL` | Yes | PostgreSQL connection string |
| `REDIS_URL` | Recommended | Redis connection string |
| `SOLANA_RPC_URL` | Yes | Solana RPC endpoint |
| `PROGRAM_ID` | Yes | SSS Token program ID |
| `JWT_SECRET` | Yes | JWT signing secret |
| `AUTHORITY_KEYPAIR` | For transactions | Base58 authority keypair |
| `CORS_ORIGINS` | Production | Allowed CORS origins |

### Health Endpoints

| Endpoint | Purpose |
|----------|---------|
| `/health` | Basic health check |
| `/health/detail` | Detailed component status |
| `/health/ready` | Kubernetes readiness probe |
| `/health/live` | Kubernetes liveness probe |
| `/metrics` | Prometheus metrics |

## Security

- **Explicit Error Handling**: The codebase avoids `unwrap()` and `expect()`, utilizing `Result<()>` for all fallible operations.
- **Arithmetic Safety**: All mathematical operations utilize checked arithmetic to prevent overflow and underflow vulnerabilities.
- **Validation Logic**: Strict account validation, including `has_one` constraints and deterministic PDA verification, is enforced across all instructions.
- **Audit Disclaimer**: This repository is a submission for a technical bounty. It has not undergone a formal external security audit. Production deployment is at the discretion and risk of the user.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to contribute to this project.

All contributors are expected to adhere to the [Code of Conduct](CODE_OF_CONDUCT.md).

## License

This project is licensed under the MIT License.
