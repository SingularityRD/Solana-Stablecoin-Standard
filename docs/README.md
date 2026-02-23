# Solana Stablecoin Standard (SSS)

> Production-ready modular stablecoin SDK for Solana

**Superteam Brazil Bounty Submission** - $5,000 USDC Prize Pool

## Overview

The Solana Stablecoin Standard (SSS) provides a complete toolkit for issuing compliant stablecoins on Solana. Built with Anchor and following Solana Vault Standard patterns, SSS offers:

- **Configurable Anchor Program** supporting SSS-1 (Minimal) and SSS-2 (Compliant) presets
- **TypeScript SDK** (@stbr/sss-token) for seamless integration
- **Admin CLI** (sss-token) for operator workflows
- **Backend Services** (Rust/Axum) for mint/burn coordination and compliance
- **Complete Documentation** with operator runbooks

## Quick Start

```bash
# Clone repository
git clone https://github.com/solanabr/solana-stablecoin-standard
cd solana-stablecoin-standard

# Install dependencies
yarn install

# Build programs
anchor build

# Run tests
anchor test

# Deploy to Devnet
anchor deploy --provider.cluster devnet
```

## Presets

| Preset | Name | Features | Use Case |
|--------|------|----------|----------|
| **SSS-1** | Minimal Stablecoin | Mint/freeze authority + metadata | Internal tokens, DAO treasuries |
| **SSS-2** | Compliant Stablecoin | SSS-1 + permanent delegate + transfer hook + blacklist | Regulated stablecoins (USDC-class) |

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│              Solana Stablecoin Standard                      │
├─────────────────────────────────────────────────────────────┤
│  Layer 1: Base SDK                                           │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Token-2022 + Mint Authority + Freeze Authority        │   │
│  │ Role Management (Master, Minter, Burner, Pauser)     │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  Layer 2: Modules                                            │
│  ┌─────────────────┐  ┌─────────────────┐                   │
│  │ Compliance      │  │ Privacy         │                   │
│  │ - Transfer Hook │  │ - Confidential  │                   │
│  │ - Blacklist     │  │   Transfers     │                   │
│  │ - Permanent Del │  │ - Allowlists    │                   │
│  └─────────────────┘  └─────────────────┘                   │
│                                                              │
│  Layer 3: Standard Presets                                   │
│  ┌─────────────────┐  ┌─────────────────┐                   │
│  │ SSS-1 (Minimal) │  │ SSS-2 (Compliant)│                  │
│  │ Layer 1 only    │  │ Layer 1 +       │                   │
│  │                 │  │ Compliance      │                   │
│  └─────────────────┘  └─────────────────┘                   │
└─────────────────────────────────────────────────────────────┘
```

## Documentation

| Document | Description |
|----------|-------------|
| [ARCHITECTURE](ARCHITECTURE.md) | Layer model, data flows, security |
| [SDK](SDK.md) | TypeScript SDK usage and examples |
| [OPERATIONS](OPERATIONS.md) | Operator runbook |
| [SSS-1](SSS-1.md) | Minimal stablecoin specification |
| [SSS-2](SSS-2.md) | Compliant stablecoin specification |
| [COMPLIANCE](COMPLIANCE.md) | Regulatory considerations |
| [API](API.md) | Backend API reference |

## Security

- Role-based access control (no single key controls everything)
- Checked arithmetic throughout
- Emergency pause mechanism
- Graceful failure for disabled features
- Audit trail for compliance actions

**Audit Status**: Not audited. Use at your own risk.

## License

MIT

## Disclaimer

This software is provided "as is" without warranty. Use at your own risk. Not audited. For Devnet testing only during bounty period.

## Detailed Features

### Token-2022 Extensions

SSS leverages Solana's Token-2022 standard with the following extensions:

1. **Mint Authority**: Control who can create new tokens
2. **Freeze Authority**: Ability to freeze individual accounts
3. **Metadata Extension**: On-chain token metadata (name, symbol, URI)
4. **Permanent Delegate** (SSS-2): Enables token seizure for compliance
5. **Transfer Hook** (SSS-2): Enforce blacklist checks on every transfer

### Role-Based Access Control

The system implements fine-grained access control with six distinct roles:

| Role | Permissions | Use Case |
|------|-------------|----------|
| Master | Full control, role assignment | Admin multi-sig |
| Minter | Mint tokens up to quota | Authorized minters |
| Burner | Burn tokens | Redemption process |
| Blacklister | Manage blacklist | Compliance team |
| Pauser | Emergency pause | Security team |
| Seizer | Seize tokens | Legal enforcement |

### Security Features

- **Checked Arithmetic**: All math operations use checked addition/subtraction
- **PDA Validation**: All PDAs derived with proper seeds and bumps stored
- **Account Validation**: All accounts validated before use
- **Reentrancy Protection**: State changes before external calls
- **Emergency Controls**: Pause mechanism for critical situations

## Getting Started

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Solana Tool Suite
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"

# Install Anchor
cargo install --git https://github.com/coral-xyz/anchor avm --force

# Install Node.js and Yarn
nvm install 20
npm install -g yarn
```

### Quick Start

```bash
# Clone repository
git clone https://github.com/solanabr/solana-stablecoin-standard
cd solana-stablecoin-standard

# Install dependencies
yarn install

# Build programs
anchor build

# Run tests
anchor test

# Deploy to Devnet
anchor deploy --provider.cluster devnet
```

## Community & Support

- **GitHub**: github.com/solanabr/solana-stablecoin-standard
- **Documentation**: docs/ directory
- **Issues**: GitHub Issues
- **Discord**: Superteam Brazil Discord

## Contributing

Contributions welcome! Please read our contributing guidelines before submitting PRs.

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit PR

## License

MIT License - see LICENSE file for details.

## Acknowledgments

- Solana Vault Standard (SVS) for architecture patterns
- Superteam Brazil for bounty support
- Anchor Framework team
- Solana Foundation

## Quick Reference

### Common Commands

```bash
# Initialize SSS-1
sss-token init --preset 1 --name "Token" --symbol "TKN" --uri "https://..."

# Initialize SSS-2
sss-token init --preset 2 --name "Token" --symbol "TKN" --uri "https://..."

# Mint tokens
sss-token mint <recipient> <amount>

# Burn tokens
sss-token burn <amount>

# Freeze account
sss-token freeze <address>

# Blacklist (SSS-2)
sss-token blacklist add <address> --reason "..."

# Check status
sss-token status
sss-token supply
```

### SDK Quick Start

```typescript
import { SolanaStablecoin, Presets } from '@stbr/sss-token';

// Create
const stable = await SolanaStablecoin.create(connection, {
  preset: Presets.SSS_2,
  name: "My Stable",
  symbol: "MYS",
  decimals: 6,
  authority: keypair,
});

// Mint
await stable.mint(recipient, 1000000);

// Compliance (SSS-2)
await stable.compliance.blacklistAdd(address, "OFAC");
```

## Support

### Getting Help

1. **Documentation**: Start with docs/ directory
2. **Examples**: Check examples/ folder
3. **Issues**: GitHub Issues for bugs
4. **Discussions**: GitHub Discussions for questions

### Response Times

| Channel | Response Time |
|---------|---------------|
| GitHub Issues | 24-48 hours |
| Discord | 1-4 hours |
| Email | 24 hours |
| Emergency | 1 hour |

## Contributing

### How to Contribute

1. **Report Bugs**: GitHub Issues
2. **Suggest Features**: GitHub Discussions
3. **Submit Code**: Pull Requests
4. **Improve Docs**: Edit markdown files

### Development Setup

```bash
# Clone
git clone https://github.com/solanabr/solana-stablecoin-standard
cd solana-stablecoin-standard

# Install
yarn install

# Build
anchor build

# Test
anchor test
```

### Pull Request Process

1. Fork repository
2. Create feature branch
3. Make changes
4. Add tests
5. Run linter
6. Submit PR
7. Address reviews
8. Merge

## License

MIT License

Copyright (c) 2024 Superteam Brazil

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

## Acknowledgments

Built with:
- Anchor Framework
- Solana Program Library
- Solana Vault Standard (reference)
- Superteam Brazil community

Special thanks to all contributors and the Solana community.
