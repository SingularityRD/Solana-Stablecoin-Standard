# SSS-1: Minimal Stablecoin Standard Specification

## 1. Abstract

SSS-1 (Minimal Stablecoin Standard) defines the foundational architecture for a streamlined, high-performance stablecoin on the Solana blockchain. By leveraging the Token-2022 standard while omitting complex on-chain compliance modules, SSS-1 provides a low-latency, cost-effective solution for environments where regulatory overhead is managed off-chain or is not required. This specification outlines the core functionalities, security model, and operational parameters of the SSS-1 preset.

## 2. Rationale: The Case for Minimality

In the evolving landscape of digital assets, not every use case requires the heavy-duty compliance infrastructure of a public, fiat-backed stablecoin. SSS-1 is designed for efficiency and simplicity, offering several key advantages:

- **Reduced Compute Overhead**: By bypassing transfer hooks and complex blacklist checks, SSS-1 transactions consume significantly fewer Compute Units (CUs), leading to higher throughput and lower priority fees.
- **Operational Simplicity**: Issuers can manage their ecosystem without the complexity of maintaining on-chain blacklist PDAs or managing permanent delegates.
- **Lower Deployment Costs**: The minimal state footprint results in lower rent-exemption requirements for the program and associated accounts.
- **Predictable Performance**: Without external program calls (like transfer hooks), transaction execution is more deterministic and less prone to congestion-related failures.

SSS-1 is the ideal choice for internal treasuries, DAO governance, and closed-loop ecosystem settlements where trust is established through membership or off-chain legal frameworks rather than real-time on-chain enforcement.

## 3. Comparison: SSS-1 vs. SSS-2

The following table highlights the architectural differences between the Minimal (SSS-1) and Compliant (SSS-2) standards:

| Feature | SSS-1 (Minimal) | SSS-2 (Compliant) |
|:--- |:---:|:---:|
| **Mint/Burn Authority** | ✅ | ✅ |
| **Freeze/Thaw Authority** | ✅ | ✅ |
| **On-Chain Metadata** | ✅ | ✅ |
| **Role-Based Access Control** | ✅ | ✅ |
| **Transfer Hook Enforcement** | ❌ | ✅ |
| **Permanent Delegate (Seizure)** | ❌ | ✅ |
| **On-Chain Blacklist** | ❌ | ✅ |
| **Audit Trail Logging** | Basic | Comprehensive |
| **Compute Unit Efficiency** | Ultra-High | High |
| **Regulatory Readiness** | Reactive | Proactive |

## 4. Use Cases

SSS-1 is optimized for the following scenarios:

- **Internal Treasury Management**: Moving funds between corporate entities or business units.
- **DAO Governance & Settlement**: Distributing grants or settling internal DAO obligations.
- **Ecosystem-Specific Tokens**: Settlement assets within a specific game, marketplace, or platform.
- **Development & Prototyping**: Rapidly deploying stablecoin-like assets for testing and integration.

## 5. Core Features

### 5.1. Foundational Functionality
- **Mint Authority**: Centralized control over token issuance with per-minter quotas.
- **Freeze Authority**: The ability to halt transfers for specific accounts in emergency scenarios.
- **On-Chain Metadata**: Native support for name, symbol, and URI via Token-2022 extensions.
- **Token-2022 Compatibility**: Full support for the latest Solana token standards.

### 5.2. Role-Based Access Control (RBAC)
- **Master Authority**: The root of trust, capable of reassigning all other roles.
- **Minter Role**: Authorized to issue new tokens within defined supply constraints.
- **Burner Role**: Authorized to remove tokens from circulation.
- **Pauser Role**: Capable of triggering a global circuit breaker to halt all program operations.

### 5.3. Account Lifecycle Management
- **Granular Freezing**: Individual accounts can be frozen or thawed by the Pauser.
- **Authority Delegation**: Master authority can be transferred to a multi-sig or DAO governance contract.

## 6. Compliance Architecture

Unlike SSS-2, which enforces compliance at the protocol level, SSS-1 adopts a **Reactive Compliance Model**.

- **No Proactive Blocking**: Transfers are not checked against a blacklist in real-time.
- **Manual Intervention**: Compliance is achieved through the `freeze_account` instruction, allowing issuers to respond to legal requests or security incidents.
- **Error Handling**: Instructions specific to SSS-2 (e.g., `add_to_blacklist`) will return a `ComplianceNotEnabled` error when called against an SSS-1 instance.
- **Auditability**: While lacking a dedicated compliance log, all state-changing operations emit standard Anchor events for off-chain indexing.

## 7. Technical Specification

### 7.1. Program ID
```
SSSToken11111111111111111111111111111111111
```

### 7.2. Preset Value
```rust
pub const PRESET_SSS_1: u8 = 1;
```

### 7.3. State Account Structure
```rust
pub struct StablecoinState {
    pub authority: Pubkey,           // Master authority
    pub asset_mint: Pubkey,          // Underlying token mint
    pub total_supply: u64,           // Total tokens minted
    pub paused: bool,                // Emergency pause flag
    pub preset: u8,                  // Always 1 for SSS-1
    pub compliance_enabled: bool,    // Always false for SSS-1
    pub bump: u8,                    // PDA bump
    pub _reserved: [u8; 64],         // Future upgrades
}
```

### 7.4. Instruction Set

| Instruction | Authority Required | Description |
|-------------|-------------------|-------------|
| `initialize` | Signer | Create stablecoin with preset=1 |
| `mint` | Minter | Mint new tokens |
| `burn` | Burner | Burn tokens |
| `freeze_account` | Pauser | Freeze an account |
| `thaw_account` | Pauser | Thaw a frozen account |
| `pause` | Pauser | Pause all operations |
| `unpause` | Pauser | Resume operations |
| `transfer_authority` | Master | Transfer master authority |
| `assign_role` | Master | Assign role to account |

## 8. Security Model

### 8.1. Critical Authorities
1. **Mint Authority**: Represents the power to inflate supply. Must be protected by multi-sig or hardware security modules (HSMs).
2. **Freeze Authority**: Can censor individual users. Usage should be governed by clear legal or community guidelines.
3. **Master Authority**: The ultimate control point. Should ideally be held by a decentralized governance body or a high-threshold multi-sig.

### 8.2. Regulatory Considerations
Issuers using SSS-1 must acknowledge that the lack of on-chain enforcement (like transfer hooks) may increase regulatory risk for public-facing tokens in certain jurisdictions. SSS-1 is best suited for environments where participants are pre-vetted or where the issuer can manage compliance through off-chain means.

## 9. Deployment Guidelines

### 9.1. Optimal Use Cases
✅ **Recommended for**:
- Private or permissioned ecosystems.
- High-frequency internal settlement.
- Experimental or testnet deployments.

❌ **Not Recommended for**:
- Public, fiat-backed stablecoins requiring strict AML/KYC enforcement.
- Tokens subject to OFAC or other international sanctions lists requiring real-time blocking.
- Institutional-grade assets requiring a permanent seizure mechanism.

## 10. Upgradeability and Migration

SSS-1 is designed as a fixed-preset configuration. It **cannot** be directly upgraded to SSS-2 on-chain due to the fundamental differences in account structures and Token-2022 extensions (e.g., the permanent delegate cannot be added after mint initialization).

### 10.1. Migration Strategy
If an SSS-1 deployment must transition to a compliant SSS-2 model, the following migration path is recommended:
1. **Deploy**: Initialize a new SSS-2 stablecoin.
2. **Snapshot**: Record all SSS-1 balances at a specific slot.
3. **Swap**: Implement a migration contract allowing users to burn SSS-1 in exchange for SSS-2 at a 1:1 ratio.
4. **Decommission**: Revoke all authorities on the SSS-1 mint.

## 11. Operational Examples

### 11.1. Initialization
```bash
# Initialize SSS-1
sss-token init \
  --preset 1 \
  --name "Internal USD" \
  --symbol "IUSD" \
  --uri "https://company.com/token.json" \
  --decimals 6
```

### 11.2. Role Management
```bash
# Add minter with quota
sss-token minters add <minter_key> --quota 1000000
```

### 11.3. Issuance
```bash
# Mint initial supply
sss-token mint <treasury> 1000000
```

## 12. Implementation Details

### 12.1. Account Structures
```rust
#[account]
pub struct StablecoinState {
    pub authority: Pubkey,           // Master authority (32 bytes)
    pub asset_mint: Pubkey,          // Underlying token mint (32 bytes)
    pub total_supply: u64,           // Total tokens minted (8 bytes)
    pub paused: bool,                // Emergency pause flag (1 byte)
    pub preset: u8,                  // Always 1 for SSS-1 (1 byte)
    pub compliance_enabled: bool,    // Always false for SSS-1 (1 byte)
    pub bump: u8,                    // PDA bump (1 byte)
    pub _reserved: [u8; 64],         // Future upgrades (64 bytes)
}                                    // Total: 140 bytes + discriminator (8) = 148 bytes
```

### 12.2. Instruction Flows

#### Mint Flow
```
User → Sign Transaction → Program → Validate Authority → 
Update Supply → Emit Event → Token Transfer → Confirm
```

#### Freeze Flow
```
Pauser → Sign Transaction → Program → Validate Role → 
Mark Account Frozen → Emit Event → Confirm
```

## 13. Performance and Gas Optimization

SSS-1 is optimized for minimal compute units:

- **Initialize**: ~5,000 CU
- **Mint**: ~3,000 CU
- **Burn**: ~3,000 CU
- **Freeze**: ~2,000 CU

Average transaction cost at 50,000 CU: ~0.000005 SOL

## 14. Reference Implementations

### 14.1. DAO Treasury
```bash
# Initialize DAO token
sss-token init --preset 1 \
  --name "DAO Treasury" \
  --symbol "DAOT" \
  --uri "https://dao.org/token.json"

# Add treasury minter
sss-token minters add <treasury-address> --quota 10000000

# Mint for grants
sss-token mint <grant-recipient> 100000
```

### 14.2. Internal Settlement
```bash
# Initialize settlement token
sss-token init --preset 1 \
  --name "Internal USD" \
  --symbol "IUSD" \
  --uri "https://company.internal/token.json"

# Mint to business units
sss-token mint <unit-a> 500000
sss-token mint <unit-b> 300000
```

## 15. Limitations and Constraints

### 15.1. Functional Boundaries
1. **No On-Chain Compliance**: Cannot enforce blacklist automatically.
2. **No Token Seizure**: Cannot confiscate tokens from holders.
3. **No Transfer Restrictions**: Transfers are permissionless by default.
4. **Basic Audit Trail**: Limited to standard event logging.

### 15.2. Mitigation Strategies
For compliance needs within SSS-1:
- Implement off-chain monitoring and screening.
- Utilize the `freeze_account` instruction reactively.
- Plan for migration to SSS-2 if regulatory requirements change.

## 16. Migration Protocol (Technical)

### 16.1. SSS-1 to SSS-2 Transition
Direct upgrade is not supported. The recommended migration process is:
1. Deploy new SSS-2 stablecoin.
2. Snapshot SSS-1 holders.
3. Airdrop SSS-2 tokens or provide a swap facility.
4. Set exchange rate (typically 1:1).
5. Allow a defined swap period.
6. Burn remaining SSS-1 supply.
7. Decommission SSS-1 authorities.

### 16.2. Migration Contract Example
```typescript
async function migrate(sss1Amount: number) {
  // Burn SSS-1 tokens
  await sss1.burn(sss1Amount);
  
  // Mint SSS-2 tokens (1:1)
  await sss2.mint(userAddress, sss1Amount);
  
  // Log migration
  await logMigration(userAddress, sss1Amount);
}
```

## 17. Security Analysis

### 17.1. Threat Vectors
1. **Mint Authority Compromise**: Mitigation via Multi-sig and HSMs.
2. **Freeze Authority Abuse**: Mitigation via governance oversight and multi-sig.
3. **Front-Running**: Mitigation via priority fees and private RPC endpoints.

### 17.2. Best Practices
1. Mandatory multi-sig for all administrative keys.
2. Implementation of timelocks for critical state changes.
3. Real-time event monitoring and alerting.
4. Formal incident response protocols.
5. Periodic third-party security audits.

## 18. Economic and Cost Analysis

### 18.1. Deployment Costs
| Item | Cost (SOL) | Cost (USD @ $150) |
|------|------------|------------|
| Program Deployment | ~2.0 SOL | ~$300 |
| Stablecoin Initialization | ~0.05 SOL | ~$7.50 |
| Account Rent (per user) | ~0.002 SOL | ~$0.30 |

### 18.2. Operational Costs
| Operation | CU | Cost (SOL) |
|-----------|-----|------------|
| Mint | 3,000 | ~0.00003 |
| Burn | 3,000 | ~0.00003 |
| Freeze | 2,000 | ~0.00002 |
| Transfer | 1,500 | ~0.000015 |

## 19. Support and Resources
- **Technical Documentation**: docs/SSS-1.md
- **Code Examples**: examples/sss1/
- **Issue Tracking**: GitHub Issues
- **Community Support**: Superteam Brazil Discord
