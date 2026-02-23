# TypeScript SDK

## Installation

```bash
yarn add @stbr/sss-token
```

## Quick Start

### Initialize with Preset

```typescript
import { SolanaStablecoin, Presets } from '@stbr/sss-token';
import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import { Program } from '@coral-xyz/anchor';

const connection = new Connection('https://api.devnet.solana.com');
const authority = Keypair.fromSecretKey(Uint8Array.from([...]));
const assetMint = new PublicKey('...'); // Your token mint address
const program = {} as Program; // Your Anchor program instance

// SSS-1 (Minimal)
const stable1 = await SolanaStablecoin.create(connection, {
  preset: Presets.SSS_1,
  name: "My Stablecoin",
  symbol: "MYUSD",
  uri: "https://example.com/metadata.json",
  decimals: 6,
  authority,
  assetMint,
}, program);

// SSS-2 (Compliant)
const stable2 = await SolanaStablecoin.create(connection, {
  preset: Presets.SSS_2,
  name: "Compliant Stable",
  symbol: "CUSD",
  uri: "https://example.com/metadata.json",
  decimals: 6,
  authority,
  assetMint,
}, program);
```

### Operations

```typescript
// Mint tokens
const txMint = await stable.mint(authority, recipient, 1_000_000);

// Burn tokens
const txBurn = await stable.burn(authority, holderAccount, 500_000);

// Freeze account
const txFreeze = await stable.freeze(authority, suspiciousAccount);

// Thaw account
const txThaw = await stable.thaw(authority, frozenAccount);

// Get total supply
const supply = await stable.getTotalSupply();

// Get status
const status = await stable.getStatus();
console.log(status.paused, status.preset, status.complianceEnabled);
```

### Compliance (SSS-2 Only)

```typescript
// Add to blacklist
await stable.compliance.blacklistAdd(
  authority,
  badActor,
  "OFAC sanctions match"
);

// Remove from blacklist
await stable.compliance.blacklistRemove(authority, address);

// Seize tokens
await stable.compliance.seize(
  authority,
  frozenAccount,
  treasury,
  amount
);
```

## PDA Helpers

```typescript
import { findStablecoinPda, findMinterPda, findBlacklistPda } from '@stbr/sss-token';

const [stablecoinPda] = findStablecoinPda(assetMint);
const [minterPda] = findMinterPda(stablecoin, minter);
const [blacklistPda] = findBlacklistPda(stablecoin, account);
```

## Error Handling

```typescript
import { 
  StablecoinError,
  ComplianceNotEnabledError,
  VaultPausedError,
  BlacklistViolationError,
} from '@stbr/sss-token';

try {
  await stable.compliance.blacklistAdd(authority, account, reason);
} catch (error) {
  if (error instanceof ComplianceNotEnabledError) {
    console.log("This is SSS-1, compliance not available");
  }
}
```

## Custom Configuration

```typescript
// Custom config without preset
const custom = await SolanaStablecoin.create(connection, {
  name: "Custom Stable",
  symbol: "CUSD",
  uri: "https://example.com/metadata.json",
  decimals: 6,
  authority,
  assetMint,
  // Custom extensions
  enablePermanentDelegate: true,
  enableTransferHook: false,
  defaultAccountFrozen: false,
} as any, program); // Cast to any if using non-preset options not in strict config type
```

## Advanced Usage

### Batch Operations

For high-volume operations, use batch transactions:

```typescript
import { Transaction } from '@solana/web3.js';
import { TOKEN_2022_PROGRAM_ID } from '@solana/spl-token';
import { BN } from '@coral-xyz/anchor';

// Batch multiple mints
const tx = new Transaction();
for (const recipient of recipients) {
  tx.add(
    await stable.program.methods
      .mint(new BN(amount))
      .accounts({
        authority: authority.publicKey,
        state: stable.stablecoinPda,
        assetMint: stable.assetMint,
        recipient,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .instruction()
  );
}

await stable.provider.sendAndConfirm(tx);
```

### Event Listening

Listen for program events:

```typescript
import { Connection } from '@solana/web3.js';

const connection = new Connection(rpcUrl);

// Listen for mint events
connection.onProgramAccountChange(
  stable.program.programId,
  (accountInfo) => {
    // Parse events from account data or logs
    console.log("Account changed:", accountInfo);
  }
);
```

### Error Handling

Comprehensive error handling:

```typescript
import { PublicKey } from '@solana/web3.js';
import { 
  StablecoinError,
  ComplianceNotEnabledError,
  VaultPausedError,
  BlacklistViolationError,
} from '@stbr/sss-token';

async function safeMint(recipient: PublicKey, amount: number) {
  try {
    return await stable.mint(authority, recipient, amount);
  } catch (error) {
    if (error instanceof VaultPausedError) {
      console.log("Vault is paused, try later");
      return null;
    }
    if (error instanceof ComplianceNotEnabledError) {
      console.log("This is SSS-1, compliance not available");
      return null;
    }
    if (error instanceof BlacklistViolationError) {
      console.log("Recipient is blacklisted");
      return null;
    }
    throw error;
  }
}
```

## Configuration Options

### Network Configuration

```typescript
// Devnet
const connection = new Connection('https://api.devnet.solana.com');

// Mainnet
const connection = new Connection('https://api.mainnet-beta.solana.com');

// Localnet
const connection = new Connection('http://localhost:8899');
```

### Commitment Levels

```typescript
// Processed (fastest, least secure)
const connection = new Connection(rpcUrl, 'processed');

// Confirmed (balanced)
const connection = new Connection(rpcUrl, 'confirmed');

// Finalized (slowest, most secure)
const connection = new Connection(rpcUrl, 'finalized');
```

## Testing

### Unit Tests

```typescript
import { describe, it, expect } from '@jest/globals';

describe('SolanaStablecoin', () => {
  it('should create stablecoin', async () => {
    const stable = await SolanaStablecoin.create(connection, config, program);
    expect(stable).toBeDefined();
  });

  it('should mint tokens', async () => {
    const tx = await stable.mint(authority, recipient, 1000);
    expect(tx).toBeDefined();
  });
});
```

### Integration Tests

```typescript
describe('SSS-2 Integration', () => {
  it('should enforce blacklist', async () => {
    await stable.compliance.blacklistAdd(authority, badActor, 'Test');
    
    try {
      await stable.mint(authority, badActor, 1000);
      expect.fail('Should have thrown');
    } catch (error) {
      expect(error).toBeInstanceOf(BlacklistViolationError);
    }
  });
});
```

## Performance Optimization

### Connection Pooling

```typescript
import { Connection } from '@solana/web3.js';

// Create connection pool
const connections = Array(10).fill(null).map(
  () => new Connection(rpcUrl, 'confirmed')
);

// Use round-robin for load balancing
let current = 0;
function getConnection() {
  return connections[current++ % connections.length];
}
```

### Transaction Batching

Group multiple operations in single transaction when possible to save on fees and improve throughput.

## Troubleshooting

### Common Issues

1. **Transaction Failed**: Check commitment level, increase retries
2. **Account Not Found**: Verify PDA derivation
3. **Insufficient Funds**: Check SOL balance for fees
4. **Blockhash Expired**: Use recent blockhash, retry logic

### Debug Mode

```typescript
// Enable debug logging
import { setLogLevel } from '@coral-xyz/anchor';
setLogLevel('debug');
```

## Migration Guide

### From Vanilla Token-2022

If you have an existing Token-2022 mint:

```typescript
// 1. Initialize SSS wrapper around existing mint
const stable = new SolanaStablecoin(
  connection,
  program,
  provider,
  stablecoinPda,
  existingMintPublicKey,
  config
);

// 2. Configure roles (if using RBAC module)
// await stable.assignRole(Role.Minter, minterAddress);

// 3. Start using SSS features
await stable.mint(authority, recipient, amount);
```

### From SSS-1 to SSS-2

Direct upgrade not supported. Migration required:

```typescript
// 1. Deploy new SSS-2 instance
const sss2 = await SolanaStablecoin.create(connection, {
  preset: Presets.SSS_2,
  name: "Migrated Stable",
  symbol: "MST",
  uri: "https://...",
  decimals: 6,
  authority: keypair,
  assetMint: newMint,
}, program);

// 2. Snapshot SSS-1 holders
const holders = await getHolders(sss1);

// 3. Airdrop SSS-2 tokens
for (const holder of holders) {
  await sss2.mint(authority, holder.address, holder.balance);
}

// 4. Set exchange rate
const EXCHANGE_RATE = 1; // 1:1

// 5. Allow swaps
for (const holder of holders) {
  await sss1.burn(authority, holder.address, holder.balance);
  await sss2.mint(authority, holder.address, holder.balance * EXCHANGE_RATE);
}
```

## Best Practices

### Security

1. **Multi-sig**: Always use multi-sig for admin roles
2. **Hardware Wallets**: Store keys on hardware wallets
3. **Key Rotation**: Rotate keys quarterly
4. **Monitoring**: Set up real-time monitoring
5. **Incident Response**: Have response plan ready

### Performance

1. **Batching**: Group operations when possible
2. **Priority Fees**: Use dynamic fees
3. **RPC Selection**: Choose low-latency RPC
4. **Caching**: Cache PDA derivations
5. **Error Handling**: Implement retry logic

### Compliance

1. **Documentation**: Keep detailed records
2. **Audit Trail**: Export logs regularly
3. **Screening**: Screen all counterparties
4. **Reporting**: File required reports
5. **Training**: Train all operators

## FAQ

### General

**Q: What's the difference between SSS-1 and SSS-2?**
A: SSS-1 is minimal (mint/freeze only). SSS-2 adds compliance (blacklist, seizure).

**Q: Can I upgrade from SSS-1 to SSS-2?**
A: No, deployment is immutable. Migrate by deploying new SSS-2.

**Q: What are the costs?**
A: Deployment ~2 SOL, operations ~0.00003 SOL each.

### Technical

**Q: What decimals should I use?**
A: 6 is standard for USD stablecoins (like USDC).

**Q: How do I add custom extensions?**
A: Fork the repo, modify state.rs, rebuild.

**Q: Can I use this on mainnet?**
A: Yes, but audit first. Use at your own risk.

### Compliance

**Q: Does this make me compliant?**
A: No, it provides tools. You need legal compliance.

**Q: What sanctions lists are supported?**
A: Integrate with Chainalysis, Elliptic, etc.

**Q: How do I handle appeals?**
A: Implement appeal process (see COMPLIANCE.md).

## Changelog

### v0.1.0 (2024-02-21)
- Initial release
- SSS-1 and SSS-2 support
- TypeScript SDK
- Complete documentation

### Planned

#### v0.2.0
- SSS-3 (confidential transfers)
- Oracle integration
- Enhanced SDK features

#### v1.0.0
- Production stable release
- External audit
- Long-term support

## Support

For support:
- Documentation: docs/ directory
- GitHub Issues: Bug reports
- GitHub Discussions: Questions
- Discord: Community support
