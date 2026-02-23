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
