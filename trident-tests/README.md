# Trident Fuzz Tests

This folder contains fuzz tests for the Solana Stablecoin Standard program using the [Trident](https://github.com/Ackee-Blockchain/trident) fuzzing framework.

## Prerequisites

### All Platforms
- Rust 1.75+
- Solana CLI tools
- Anchor CLI

### Windows-Specific Requirements
For Windows, the Trident framework requires OpenSSL for TLS support. Install:

1. **Strawberry Perl** (required for OpenSSL build):
   ```
   winget install StrawberryPerl.StrawberryPerl
   ```

2. **Microsoft Visual Studio Build Tools** with C++ development tools:
   ```
   winget install Microsoft.VisualStudio.2022.BuildTools
   ```

3. After installing these, restart your terminal and run:
   ```
   cargo clean
   cargo build
   ```

Alternatively, use WSL2 or Docker for a Linux-based build environment.

## Test Files

| File | Description |
|------|-------------|
| `fuzz_initialize.rs` | Tests stablecoin initialization with various preset values, name/symbol/uri lengths, and decimal values |
| `fuzz_mint.rs` | Tests minting operations including authorization, pause states, and quota enforcement |
| `fuzz_burn.rs` | Tests burning operations including balance checks and authorization |
| `fuzz_transfer.rs` | Tests transfer hook compliance (blacklist) in SSS-1 and SSS-2 modes |
| `fuzz_roles.rs` | Tests role assignment, revocation, and permission-based operations |

## Running Tests

### Individual Fuzz Tests
```bash
# Run initialize fuzz test
cargo fuzz run fuzz_initialize

# Run mint fuzz test
cargo fuzz run fuzz_mint

# Run burn fuzz test
cargo fuzz run fuzz_burn

# Run transfer fuzz test
cargo fuzz run fuzz_transfer

# Run roles fuzz test
cargo fuzz run fuzz_roles
```

### With Custom Corpus
```bash
cargo fuzz run fuzz_initialize -- -max_total_time=60
```

### Debug Mode
```bash
cargo fuzz run fuzz_initialize --debug
```

## Test Structure

Each fuzz test follows this pattern:

1. **Input Structure**: A `#[derive(Debug, Arbitrary)]` struct defining fuzzable inputs
2. **Test Function**: A `#[fuzz]` function that:
   - Sets up the test environment
   - Executes the instruction with fuzzed inputs
   - Asserts expected behavior (success for valid inputs, specific errors for invalid inputs)

## Key Features Tested

### Error Handling
All tests verify that invalid inputs produce the expected error codes:
- `InvalidPreset` for preset values other than 1 or 2
- `ZeroAmount` for zero mint/burn amounts
- `Unauthorized` for missing permissions
- `VaultPaused` for operations on paused vaults
- `BlacklistViolation` for transfers involving blacklisted accounts
- `QuotaExceeded` for minters exceeding their quota

### Edge Cases
- Maximum string lengths (name: 32, symbol: 10, uri: 200)
- Maximum decimals (9)
- Overflow scenarios in arithmetic operations
- Sequential operations (mint -> burn, assign -> revoke)

### Compliance Modes
- SSS-1 (preset 1): No compliance checks, transfers always succeed
- SSS-2 (preset 2): Compliance enabled, blacklist enforced on transfers

## Integration with CI

These tests can be integrated into CI pipelines for extended fuzzing sessions:

```yaml
# GitHub Actions example
- name: Run Fuzz Tests (5 minutes each)
  run: |
    cargo fuzz run fuzz_initialize -- -max_total_time=300
    cargo fuzz run fuzz_mint -- -max_total_time=300
    cargo fuzz run fuzz_burn -- -max_total_time=300
    cargo fuzz run fuzz_transfer -- -max_total_time=300
    cargo fuzz run fuzz_roles -- -max_total_time=300
```
