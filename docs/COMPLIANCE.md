# Compliance Guide

## Regulatory Overview

SSS-2 provides on-chain compliance primitives for regulated stablecoin issuers.

## Supported Regulations

### [OFAC (Office of Foreign Assets Control)](https://ofac.treasury.gov/) Sanctions
- Add sanctioned addresses to blacklist
- Automatic transfer blocking
- Seizure of existing holdings

### [MiCA (Markets in Crypto-Assets Regulation)](https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX%3A32023R1114)
- On-chain compliance enforcement
- Audit trail for regulators
- Reserve transparency hooks

### [FATF (Financial Action Task Force) Travel Rule](https://www.fatf-gafi.org/en/publications/Fatfrecommendations/Guidance-rba-virtual-assets-2021.html)
- VASP blacklist enforcement
- Transfer screening
- Reporting capabilities

## Audit Trail Format

All compliance actions emit events:

### BlacklistAdded
```json
{
  "stablecoin": "9x...abc",
  "account": "5y...def",
  "reason": "OFAC SDN List Match",
  "timestamp": 1708473600
}
```

### BlacklistRemoved
```json
{
  "stablecoin": "9x...abc",
  "account": "5y...def",
  "timestamp": 1708560000
}
```

### Seized
```json
{
  "stablecoin": "9x...abc",
  "from": "5y...def",
  "to": "7z...ghi",
  "amount": 1000000,
  "timestamp": 1708646400
}
```

## Sanctions Screening Integration

### Manual Screening
```bash
# Add to blacklist after manual review
sss-token blacklist-add <address> \
  --reason "Manual OFAC Review"
```

### Automated Screening
Backend service integrates with:
- Chainalysis
- Elliptic
- TRM Labs
- ComplyAdvantage

```typescript
// Example integration
async function screenAndBlacklist(address: string) {
  const risk = await chainalysis.screen(address);
  if (risk.sanctions) {
    await stable.compliance.blacklistAdd(
      address,
      `Chainalysis risk score: ${risk.score}`
    );
  }
}
```

## Transaction Monitoring

### Real-time Monitoring
Backend indexer monitors:
- Large transfers (> $10,000)
- Rapid movement patterns
- Blacklist circumvention attempts
- Unusual mint/burn activity

### Alert Thresholds
```typescript
const ALERTS = {
  LARGE_TRANSFER: 10_000_000,  // $10k
  RAPID_MOVEMENT: 5,            // 5 txs in 1 min
  MINT_LIMIT: 1_000_000_000,    // $1M per minter
};
```

## Reporting

### Daily Reports
- New blacklisted addresses
- Seizure operations
- Large transfers
- Mint/burn volumes

### Regulatory Export
```bash
# Export audit trail
sss-token audit-log \
  --action blacklist \
  --from 2024-01-01 \
  --to 2024-01-31 \
  --format csv
```

## Compliance Best Practices

### Key Management
1. **Multi-sig for Blacklister**: 2-of-3 minimum
2. **Separate Seizer Key**: Legal team only
3. **Hardware Wallets**: All compliance keys
4. **Rotation Policy**: Quarterly key rotation

### Operational Security
1. **Dual Approval**: All blacklist actions
2. **Audit Review**: Weekly compliance review
3. **Incident Response**: Documented procedures
4. **Backup Procedures**: Key backup and recovery

### Documentation
1. **Blacklist Reasons**: Always document
2. **Seizure Justification**: Legal approval required
3. **Appeal Process**: For false positives
4. **Retention Policy**: 7 years minimum

## False Positive Handling

### Appeal Process
1. User submits appeal with evidence
2. Compliance team reviews (48 hours)
3. If valid: remove from blacklist
4. Document decision

### Removal from Blacklist
```bash
sss-token blacklist-remove <address>
# Event: BlacklistRemoved emitted
```

## Limitations

### What SSS-2 Cannot Do
- ❌ Prevent all circumvention attempts
- ❌ Replace legal compliance program
- ❌ Handle cross-chain transfers
- ❌ Screen before first transfer

### Recommended Additions
- ✅ Off-chain screening service
- ✅ Legal compliance team
- ✅ Law enforcement liaison
- ✅ Insurance coverage

## Legal Disclaimer

SSS-2 provides technical compliance primitives only. Issuers are responsible for:
- Maintaining required licenses
- Conducting proper KYC/AML
- Filing required reports
- Following applicable laws

Consult legal counsel before deployment.

## Technical Implementation

### Sanctions Screening Integration

#### Chainalysis Integration

```typescript
import { ChainalysisClient } from '@chainalysis/sdk';

const client = new ChainalysisClient(API_KEY);

async function screenAddress(address: string): Promise<ScreeningResult> {
  const risk = await client.getRiskScore(address);
  
  return {
    address,
    riskScore: risk.score,
    sanctions: risk.sanctions.length > 0,
    sources: risk.sanctions.map(s => s.source),
    recommendation: risk.score > 80 ? 'blacklist' : 'allow'
  };
}

async function autoBlacklist(address: string) {
  const result = await screenAddress(address);
  
  if (result.sanctions) {
    await stable.compliance.blacklistAdd(
      address,
      `Chainalysis: ${result.sources.join(', ')}`
    );
    
    // Log for compliance report
    await complianceLog({
      action: 'auto_blacklist',
      address,
      reason: result.sources.join(', '),
      timestamp: Date.now()
    });
  }
}
```

#### Elliptic Integration

```typescript
import { EllipticAPI } from 'elliptic-api';

const elliptic = new EllipticAPI(API_KEY);

async function checkWallet(walletId: string) {
  const response = await elliptic.getWalletRisk(walletId);
  
  if (response.risk_level === 'HIGH') {
    return {
      block: true,
      reason: `Elliptic Risk: ${response.risk_score}/100`,
      categories: response.exposure_categories
    };
  }
  
  return { block: false };
}
```

### Transaction Monitoring

#### Real-Time Monitoring

```typescript
import { Connection, PublicKey } from '@solana/web3.js';

class TransactionMonitor {
  private connection: Connection;
  private thresholds: MonitoringThresholds;
  
  constructor(connection: Connection, thresholds: MonitoringThresholds) {
    this.connection = connection;
    this.thresholds = thresholds;
  }
  
  async monitorTransfers(stablecoinPda: PublicKey) {
    this.connection.onProgramAccountChange(
      stablecoinPda,
      async (accountInfo) => {
        const logs = accountInfo.logs;
        const events = this.parseEvents(logs);
        
        for (const event of events) {
          if (event.name === 'Minted') {
            await this.checkMint(event);
          }
          if (event.name === 'Seized') {
            await this.checkSeizure(event);
          }
        }
      }
    );
  }
  
  private async checkMint(event: MintEvent) {
    // Check for large mints
    if (event.amount > this.thresholds.LARGE_MINT) {
      await this.alert('Large mint detected', event);
    }
    
    // Check for rapid mints
    const recentMints = await this.getRecentMints(event.minter);
    if (recentMints.length > this.thresholds.RAPID_MINT_COUNT) {
      await this.alert('Rapid minting detected', { minter: event.minter });
    }
  }
}
```

#### Alert Rules

```yaml
# monitoring-config.yaml
alerts:
  - name: LargeMint
    condition: amount > 10000000
    severity: HIGH
    channels: [slack, email]
    
  - name: RapidMinting
    condition: count(minter, 5min) > 10
    severity: MEDIUM
    channels: [slack]
    
  - name: BlacklistCircumvention
    condition: transfer_via_new_account && previously_blacklisted
    severity: CRITICAL
    channels: [slack, email, pagerduty]
    
  - name: UnusualSeizure
    condition: seizure_amount > average * 3
    severity: HIGH
    channels: [email]
```

## Reporting Framework

### Daily Compliance Report

```typescript
interface DailyReport {
  date: string;
  totalSupply: number;
  mints: { count: number; volume: number };
  burns: { count: number; volume: number };
  blacklist: {
    added: Array<{ address: string; reason: string }>;
    removed: Array<{ address: string }>;
  };
  seizures: { count: number; volume: number };
  flags: Array<{ type: string; count: number }>;
}

async function generateDailyReport(date: string): Promise<DailyReport> {
  const events = await fetchEvents(date);
  
  return {
    date,
    totalSupply: await getTotalSupply(),
    mints: countEvents(events, 'Minted'),
    burns: countEvents(events, 'Burned'),
    blacklist: {
      added: events.filter(e => e.name === 'BlacklistAdded')
        .map(e => ({ address: e.account, reason: e.reason })),
      removed: events.filter(e => e.name === 'BlacklistRemoved')
        .map(e => ({ address: e.account }))
    },
    seizures: countEvents(events, 'Seized'),
    flags: await getAlerts(date)
  };
}
```

### Monthly Regulatory Report

```markdown
# Monthly Compliance Report
## [Month Year]

### Executive Summary
- Total Supply: $X million
- Total Transactions: X,XXX
- Compliance Actions: XX

### Minting Activity
| Week | Mints | Volume | Avg Size |
|------|-------|--------|----------|
| 1 | XXX | $X.XM | $X,XXX |
| 2 | XXX | $X.XM | $X,XXX |
| 3 | XXX | $X.XM | $X,XXX |
| 4 | XXX | $X.XM | $X,XXX |

### Compliance Actions
- New Blacklist Entries: XX
- Removals: XX
- Seizures: X ($XXX,XXX)

### Regulatory Inquiries
- OFAC SDN Matches: X
- Law Enforcement Requests: X
- User Appeals: X (Approved: X, Denied: X)

### Risk Metrics
- High-Risk Transactions: X (X.X%)
- Flagged Addresses: XXX
- False Positive Rate: X.X%

### Recommendations
1. [Recommendation 1]
2. [Recommendation 2]

Compliance Officer: _________________
Date: _________________
```

## Appeal Process

### User Appeal Workflow

```typescript
interface Appeal {
  id: string;
  address: string;
  reason: string;
  evidence: string[];
  status: 'pending' | 'approved' | 'denied';
  submittedAt: number;
  reviewedAt?: number;
  reviewer?: string;
}

async function submitAppeal(address: string, reason: string, evidence: string[]) {
  const appeal: Appeal = {
    id: generateId(),
    address,
    reason,
    evidence,
    status: 'pending',
    submittedAt: Date.now()
  };
  
  await storeAppeal(appeal);
  await notifyComplianceTeam(appeal);
  
  return appeal.id;
}

async function reviewAppeal(appealId: string, decision: 'approve' | 'deny', reviewer: string) {
  const appeal = await getAppeal(appealId);
  
  if (decision === 'approve') {
    await stable.compliance.blacklistRemove(appeal.address);
    appeal.status = 'approved';
  } else {
    appeal.status = 'denied';
  }
  
  appeal.reviewedAt = Date.now();
  appeal.reviewer = reviewer;
  
  await updateAppeal(appeal);
  await notifyUser(appeal);
}
```

### Appeal Timeline

```
Day 0: User submits appeal
Day 1: Compliance team receives notification
Day 2-3: Investigation period
Day 4: Decision made
Day 5: User notified, action taken
```

## Third-Party Services

### Recommended Stack

| Function | Service | Cost |
|----------|---------|------|
| Sanctions Screening | Chainalysis | $50K+/year |
| Transaction Monitoring | TRM Labs | $30K+/year |
| Identity Verification | Sumsub | $2-5/user |
| Case Management | ComplyAdvantage | $20K+/year |
| Reporting | MetricStream | Custom |

### Integration Points

```typescript
// Unified compliance interface
interface ComplianceProvider {
  screen(address: string): Promise<ScreeningResult>;
  monitor(transaction: Transaction): Promise<Alert[]>;
  report(period: DateRange): Promise<ComplianceReport>;
}

// Adapter pattern for multiple providers
class MultiProviderCompliance implements ComplianceProvider {
  private providers: ComplianceProvider[];
  
  async screen(address: string): Promise<ScreeningResult> {
    const results = await Promise.all(
      this.providers.map(p => p.screen(address))
    );
    
    return aggregateResults(results);
  }
}
```

## Audit Requirements

### Record Retention

| Record Type | Retention Period | Format |
|-------------|------------------|--------|
| Transaction Logs | 7 years | On-chain + off-chain backup |
| Blacklist Actions | 7 years | On-chain events |
| User Appeals | 5 years | Database |
| Compliance Reports | 7 years | PDF + database |
| Screening Results | 5 years | Database |

### Audit Trail Format

```json
{
  "timestamp": "2024-02-21T12:00:00Z",
  "action": "blacklist_add",
  "actor": "compliance-officer-1",
  "target": "5y...def",
  "reason": "OFAC SDN List Match",
  "evidence": [
    "ofac-list-2024-02-21.pdf"
  ],
  "approval": [
    "manager-1",
    "manager-2"
  ],
  "txSignature": "4x...abc"
}
```
