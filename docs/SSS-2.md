# SSS-2: Compliant Stablecoin Standard

## Overview

SSS-2 is a high-assurance, compliant stablecoin configuration designed for regulated institutional issuers. It leverages advanced Solana Token-2022 extensions to enforce real-time on-chain compliance and asset recovery capabilities.

## Use Cases

- **Regulated Fiat-Backed Stablecoins**: Fully compliant USD, EUR, or other fiat-pegged tokens.
- **Institutional Asset Tokens**: Representing real-world assets (RWA) with strict ownership requirements.
- **Sanctions-Compliant Instruments**: Tokens requiring automated enforcement of OFAC or international sanctions lists.
- **Jurisdictional Settlement Layers**: Regional tokens adhering to specific regulatory frameworks like MiCA (EU).

## SSS-2 Functional Increments

SSS-2 extends the SSS-1 base with the following compliance-focused features:

- **Permanent Delegate**: Enables authorized asset seizure for regulatory compliance.
- **Transfer Hook**: Mandatory real-time validation against authorized blacklists for every transaction.
- **Blacklist Management**: PDA-based registry for restricted addresses.
- **Compliance Audit Trail**: Comprehensive on-chain event logging for all administrative and enforcement actions.
- **Graceful Degradation**: SSS-2 specific instructions revert with deterministic errors when called on SSS-1 presets.

## Access Control Matrix

The SSS-2 preset implements a granular Role-Based Access Control (RBAC) system:

| Role | Responsibility | Authority Level |
|------|----------------|-----------------|
| **Master** | Role assignment and system configuration | Administrative |
| **Blacklister** | Management of the on-chain blacklist registry | Compliance |
| **Seizer** | Execution of token seizures via Permanent Delegate | Legal/Enforcement |
| **Minter** | Token issuance within authorized quotas | Operational |
| **Burner** | Token redemption and supply reduction | Operational |
| **Pauser** | Emergency suspension of all token operations | Security |

## Technical Specification

### Program ID
```
SSSToken11111111111111111111111111111111111
```

### Preset Configuration
```rust
pub const PRESET_SSS_2: u8 = 2;
```

### State Account Structure
```rust
pub struct StablecoinState {
    pub authority: Pubkey,           // Master authority
    pub asset_mint: Pubkey,          // Token-2022 Mint address
    pub total_supply: u64,           // Aggregate supply
    pub paused: bool,                // Emergency pause state
    pub preset: u8,                  // Preset identifier (2 for SSS-2)
    pub compliance_enabled: bool,    // Compliance module toggle
    pub bump: u8,                    // PDA bump seed
    pub _reserved: [u8; 64],         // Future extensibility
}
```

## Compliance Mechanisms

### Transfer Hook (Real-time Enforcement)

SSS-2 utilizes the Token-2022 `TransferHook` extension to enforce compliance at the protocol level. Every `Transfer` or `TransferChecked` instruction invokes the SSS program's `Execute` instruction.

**Technical Mechanism:**
1. **Interface**: Implements `spl-transfer-hook-interface`.
2. **Account Resolution**: Uses an `ExtraAccountMetaList` PDA to provide the necessary `BlacklistEntry` PDAs to the hook.
3. **Validation Logic**:
   - The hook derives the `BlacklistEntry` PDA for both the `source` and `destination` owners.
   - It verifies the existence and initialization state of these PDAs.
   - If either party is found in the blacklist registry, the transaction is aborted with a `BlacklistViolation` error.
4. **Immutability**: The hook is bound to the mint at initialization and cannot be bypassed by standard client implementations.

### Permanent Delegate (Asset Seizure)

The `PermanentDelegate` extension designates the `StablecoinState` PDA as an immutable delegate for all token accounts associated with the SSS-2 mint.

**Technical Mechanism:**
1. **Signature Bypass**: Grants the `Seizer` role the authority to move tokens from any account without requiring the owner's cryptographic signature.
2. **Instruction**: The `seize` instruction executes a `transfer_checked` CPI call to the Token-2022 program.
3. **Enforcement Policy**: Seizure is restricted to accounts that have been formally added to the blacklist registry.
4. **Auditability**: Every seizure operation emits a `Seized` event containing the source, destination, amount, and timestamp.

## Technical Deep Dive

### Transfer Hook Implementation
```rust
pub fn enforce_transfer(
    ctx: Context<TransferHook>,
    amount: u64,
) -> Result<()> {
    let state = &ctx.accounts.state;
    
    // Ensure compliance module is active
    require!(state.compliance_enabled, StablecoinError::ComplianceNotEnabled);
    
    // Validate source account
    require!(
        !is_blacklisted(ctx.accounts.source_owner),
        StablecoinError::BlacklistViolation
    );
    
    // Validate destination account
    require!(
        !is_blacklisted(ctx.accounts.destination_owner),
        StablecoinError::BlacklistViolation
    );
    
    Ok(())
}
```

### Seizure Implementation
```rust
pub fn seize(
    ctx: Context<Seize>,
    amount: u64,
    destination: Pubkey,
) -> Result<()> {
    // Role-based access control check
    require_role!(ctx.accounts.authority, Role::Seizer);
    
    // Compliance state verification
    require!(ctx.accounts.state.compliance_enabled, StablecoinError::ComplianceNotEnabled);
    
    // Mandatory blacklist verification before seizure
    require!(
        is_blacklisted(ctx.accounts.from_owner),
        StablecoinError::Unauthorized
    );
    
    // Execute CPI to Token-2022 via Permanent Delegate
    token_2022::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.from.to_account_info(),
                mint: ctx.accounts.asset_mint.to_account_info(),
                to: ctx.accounts.destination.to_account_info(),
                authority: ctx.accounts.state.to_account_info(),
            },
            &[&[VAULT_SEED, ctx.accounts.asset_mint.key().as_ref(), &[ctx.accounts.state.bump]]]
        ),
        amount,
        ctx.accounts.asset_mint.decimals
    )?;
    
    Ok(())
}
```

## Applicability

### Recommended Use Cases
- **Regulated Stablecoins**: Tokens requiring strict adherence to AML/KYC and sanctions laws.
- **Institutional Finance**: Private or permissioned tokenization of financial instruments.
- **Compliance-Heavy Jurisdictions**: Markets requiring proactive on-chain enforcement capabilities.

### Non-Applicable Use Cases
- **Censorship-Resistant Protocols**: Use cases where neutral, unstoppable transfers are a core requirement.
- **Experimental Tokens**: Scenarios where the overhead of compliance checks and PDA lookups is undesirable.
- **Minimalist Deployments**: Use SSS-1 for basic token functionality without compliance extensions.

## Security Analysis

### Threat Mitigation
- **Sanctioned Actors**: Blocked in real-time via the Transfer Hook.
- **Illicit Fund Flows**: Monitored via on-chain audit trails and restricted via blacklist.
- **Administrative Abuse**: Mitigated through multi-signature requirements for `Blacklister` and `Seizer` roles.
- **Key Compromise**: Protected by hardware security modules (HSM) and role-based isolation.

### Operational Controls
- **Emergency Pause**: The `Pauser` role can suspend all transfers globally in the event of a systemic threat.
- **Immutable Hooks**: The compliance logic is hard-coded into the program and bound to the mint, preventing unauthorized bypass.

## Performance and Costs

### Compute Budget
| Operation | Base CU | Compliance Overhead | Total CU |
|-----------|---------|---------------------|----------|
| Transfer | 1,500 | 2,500 | 4,000 |
| Mint | 3,000 | 0 | 3,000 |
| Seize | 5,000 | 1,000 | 6,000 |

### Economic Costs (Mainnet-Beta)
| Item | SOL | USD (Est. @$150) |
|------|-----|------------------|
| SSS-2 Initialization | 0.08 | $12.00 |
| Blacklist Entry (PDA) | 0.003 | $0.45 |
| Seizure Transaction | 0.00006 | $0.009 |

## Legal Disclaimer

The SSS-2 preset provides technical primitives for regulatory compliance. It does not constitute legal advice or guarantee regulatory approval. Issuers are responsible for obtaining necessary licenses, performing KYC/AML procedures, and ensuring their use of the SSS-2 framework complies with all applicable local and international laws.

### Institutional Compliance Workflows

For regulated issuers, the SSS-2 standard provides a technical framework that integrates seamlessly into existing legal and compliance department workflows. This goes beyond simple blacklist management and enters the realm of institutional risk mitigation.

#### 1. Sanctions Screening Pipeline
Issuers can automate the blacklisting process by integrating the SSS Backend with major compliance providers (e.g., Chainalysis, TRM Labs). When a high-risk address is detected off-chain, the `Compliance Service` can immediately trigger an `add_to_blacklist` transaction. The SSS-2 program ensures that this restriction is applied atomically and globally across all SSS-compliant interactions.

#### 2. Suspicious Activity Reporting (SAR)
The event emission logic in SSS-2 ensures that every attempted transfer from a blacklisted account is logged. Even if the transfer is blocked, the attempted transaction footprint is available for compliance officers to extract and include in SAR filings to regulatory bodies like FinCEN.

#### 3. Asset Recovery and Legal Seizure
The combination of the `Permanent Delegate` extension and the `seize` instruction allows legal teams to act on court orders. Unlike decentralized stablecoins, SSS-2 provides the necessary "kill switch" and "confiscation" primitives required for a token to be legally classified as a digital representation of fiat in many jurisdictions.

#### 4. Tiered Compliance Access
By using the `RoleManagement` module, an institution can separate the duties of "Blacklisters" (compliance analysts) and "Seizers" (legal department heads). This prevents any single employee from having the power to confiscated assets, aligning the protocol with standard internal control requirements (e.g., SOC2).
