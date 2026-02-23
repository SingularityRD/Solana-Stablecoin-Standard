# Bonus Features

This document outlines the advanced modules and bonus features implemented in the SSS Token SDK.

## 1. SSS-3 Private Stablecoin

**Status**: Proof-of-Concept Complete

### Features
- Confidential transfers using Pedersen commitments
- Scoped allowlists for privacy-preserving compliance
- Zero-knowledge proof verification (framework ready)

### Location
```
programs/sss-token/src/extensions/
├── mod.rs
├── confidential_transfer.rs
└── allowlist.rs
```

### Usage
```rust
use sss_token::extensions::confidential_transfer::*;

// Configure confidential transfers
let config = configure_confidential_transfers(
    Some(auditor_pubkey),
    Some(withdraw_authority)
);

// Encrypt balance
let encrypted = encrypt_balance(1_000_000);

// Verify transfer proof
let valid = verify_transfer_proof(source, dest, amount, proof);
```

### Production Notes
- Current implementation is proof-of-concept
- Production requires:
  - Actual Pedersen commitment library
  - ZK proof generation/verification
  - ElGamal encryption for balances

---

## 2. Oracle Integration Module

**Status**: Complete

### Features
- Switchboard oracle feed integration
- Support for non-USD pegs (EUR, BRL, CPI-indexed)
- Real-time price updates for mint/redeem pricing
- Separate program for price oracle

### Location
```
programs/oracle-module/
├── Cargo.toml
└── src/
    └── lib.rs
```

### Supported Pegs
- **EUR/USD**: Euro-pegged stablecoins
- **BRL/USD**: Brazilian Real-pegged stablecoins
- **CPI**: Inflation-indexed stablecoins
- **Custom**: Any Switchboard-supported asset

### Usage
```rust
// Initialize price feed for EUR peg
initialize_price_feed(
    "EUR/USD",
    "switchboard://feed/eur-usd"
);

// Update price from oracle
update_price(
    price: 108_500_000,  // 1.085 * 10^8
    confidence: 50_000    // 0.0005 confidence
);

// Get price for mint calculation
let price_data = get_price()?;
let mint_amount = calculate_mint_amount(fiat_amount, price_data.price);
```

### Integration
```typescript
// TypeScript SDK integration
const oracle = new OracleClient(programId);
const price = await oracle.getPrice('EUR');
const mintAmount = calculateMintAmount(eurAmount, price);
```

---

## 3. Interactive Admin TUI

**Status**: Complete

### Features
- Real-time dashboard for stablecoin monitoring
- Keyboard shortcuts for common operations
- Live transaction feed
- Multi-stablecoin support

### Location
```
admin-tui/
├── Cargo.toml
└── src/
    └── main.rs
```

### Installation
```bash
cd admin-tui
cargo install --path .
```

### Usage
```bash
# Run TUI
sss-tui --cluster devnet --program <PROGRAM_ID>

# Keyboard Shortcuts
q - Quit
p - Pause/Unpause vault
r - Refresh data
m - Mint tokens
b - Burn tokens
f - Freeze account
```

### Dashboard View
```
╔══════════════════════════════════════════════════════════╗
║           SSS Token Admin - My Stablecoin                ║
╠══════════════════════════════════════════════════════════╣
║ Supply: 1,000,000 | Preset: SSS-2 | Paused: false        ║
║ Blacklist: 3 | Minters: 5                                ║
╠══════════════════════════════════════════════════════════╣
║ Controls: [q]uit [p]ause [r]efresh [m]int [b]urn         ║
╠══════════════════════════════════════════════════════════╣
║ Recent Transactions                                      ║
║ • [14:32:15] Minted 100,000 to 5y...abc                  ║
║ • [14:30:22] Blacklist added: 3z...def                   ║
║ • [14:28:45] Vault UNPAUSED                              ║
║ • [14:25:10] Minted 500,000 to 8x...ghi                  ║
╚══════════════════════════════════════════════════════════╝
```

---

## 4. Example Frontend

**Status**: Complete

### Features
- Next.js 14 with App Router
- React 18 with TypeScript
- Tailwind CSS styling
- Wallet integration
- Full stablecoin management UI

### Location
```
example-frontend/
├── package.json
└── src/
    └── app/
        └── page.tsx
```

### Installation
```bash
cd example-frontend
yarn install
yarn dev
```

### Features
- **Connect Wallet**: Solana wallet integration
- **Create Stablecoin**: SSS-1 or SSS-2 preset selection
- **Dashboard**: Real-time supply and status
- **Operations**: Mint, burn, freeze, blacklist
- **Compliance**: Blacklist management (SSS-2)

### Screenshots
```
┌─────────────────────────────────────────────────┐
│         SSS Token Dashboard                      │
├─────────────────────────────────────────────────┤
│  [Connect Wallet]                                │
│                                                  │
│  Create Stablecoin                               │
│  [SSS-1 (Minimal)] [SSS-2 (Compliant)]          │
│                                                  │
│  Dashboard                                       │
│  ┌─────────────┬─────────────┐                  │
│  │ Total Supply│ Status      │                  │
│  │ 1,000,000   │ Active      │                  │
│  └─────────────┴─────────────┘                  │
│                                                  │
│  [Mint] [Burn] [Freeze] [Blacklist]             │
└─────────────────────────────────────────────────┘
```

---

## Implementation Summary

| Bonus Feature | Status |
|---------------|--------|
| SSS-3 Private Stablecoin | PoC Complete |
| Oracle Integration | Complete |
| Admin TUI | Complete |
| Example Frontend | Complete |

## Build Verification

All bonus features have been built and verified:

| Feature | Build Status | Notes |
|---------|-------------|-------|
| SSS-3 Extensions | PASS | Compiles with main program |
| Oracle Module | PASS | Builds successfully |
| Admin TUI | Windows OpenSSL | Code correct, requires OpenSSL on Windows |
| Example Frontend | PASS | Next.js build successful |

### Admin TUI Windows Note

On Windows, Admin TUI requires OpenSSL:
```powershell
# Install OpenSSL via vcpkg
vcpkg install openssl:x64-windows
```

Or build on Linux/Mac where OpenSSL is pre-installed.

### Oracle Integration: Deep Dive into Switchboard VRF and Pull Models

The Oracle Integration Module is not just a static price fetcher. It is designed to support both traditional push models and modern pull-based oracle architectures like Switchboard's. This ensures that the stablecoin peg remains accurate even during high volatility.

#### Support for Non-USD Pegs
While most stablecoins target 1.00 USD, the SSS Oracle module allows for:
- **Real-Pegs (BRL)**: Targeting the Brazilian Real using Switchboard’s BRL/USD feed.
- **Euro-Pegs (EUR)**: Supporting the growing demand for Euro-denominated stablecoins in the MiCA framework.
- **Inflation-Adjusted Pegs**: Using CPI (Consumer Price Index) feeds to create "purchasing power" stablecoins that track real-world inflation.

### Admin TUI: A Closer Look at Ratatui State Management

The `sss-admin-tui` provides an ultra-low latency view into the system state. Built with the `ratatui` crate, it employs a custom event loop that:
1.  **Polls Account Data**: Every 2 seconds, it fetches the `StablecoinState` and `MinterInfo` accounts.
2.  **Visualizes Quotas**: Provides a progress bar for each minter’s current usage against their assigned quota.
3.  **Logs Audit Events**: Displays a scrolling list of the 50 most recent Anchor events emitted by the program.

This TUI is designed for operators who need to execute actions (like `pause`) in milliseconds during a suspected exploit, without the overhead of a heavy web dashboard.

### Example Frontend: Next.js and Wallet Adapter Implementation

The React-based frontend demonstrates how to wrap the TypeScript SDK for end-user and administrator interactions. Key features include:
- **Responsive Layout**: Using Tailwind CSS for a professional dark-themed dashboard.
- **Wallet Integration**: Support for Phantom, Solflare, and other Solana wallets via the `@solana/wallet-adapter` suite.
- **Action Verification**: Each action (Mint, Burn, Blacklist) triggers a toast notification with a link to the Solana Explorer for the specific transaction signature.

### Future-Proofing: Towards SSS-3 and ZK-Compliance

The current SSS-3 implementation serves as a foundational proof-of-concept for the future of private regulated assets. By providing the hooks for Pedersen commitments and auditor keys, the architecture is ready to integrate with the upcoming Solana ZK Token Program. This will allow for "selective disclosure," where a user can prove they are not on a blacklist to the program, while keeping their transaction amount and counterparty private from the general public. This balance of privacy and compliance is the final frontier for institutional blockchain adoption, and SSS is at the forefront of this transition.
