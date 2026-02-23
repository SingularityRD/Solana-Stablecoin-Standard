# Backend API Reference

The Solana Stablecoin Standard (SSS) API is organized around REST. Our API has predictable resource-oriented URLs, accepts JSON-encoded request bodies, returns JSON-encoded responses, and uses standard HTTP response codes, authentication, and verbs.

## Base URL

| Environment | URL |
|-------------|-----|
| **Devnet**  | `https://api.devnet.sss-token.io/v1` |
| **Mainnet** | `https://api.sss-token.io/v1` |

---

## Authentication

The SSS API uses API keys to authenticate requests. You can view and manage your API keys in the SSS Dashboard.

Your API keys carry many privileges, so be sure to keep them secure! Do not share your secret API keys in publicly accessible areas such as GitHub, client-side code, and so forth.

Authentication to the API is performed via HTTP Bearer Auth. Provide your API key as the bearer token in the `Authorization` header.

```bash
Authorization: Bearer sk_live_...
```

For sensitive operations, a payload signature may be required in the `X-Signature` header.

---

## Errors

SSS uses conventional HTTP response codes to indicate the success or failure of an API request. In general: Codes in the `2xx` range indicate success. Codes in the `4xx` range indicate an error that failed given the information provided (e.g., a required parameter was omitted, a charge failed, etc.). Codes in the `5xx` range indicate an error with SSS's servers (these are rare).

### Error Object

| Attribute | Type | Description |
|-----------|------|-------------|
| `code` | string | A short string identifying the error type. |
| `message` | string | A human-readable message providing more details about the error. |
| `request_id` | string | A unique identifier for the request. |

### HTTP Status Codes

| Code | Description |
|------|-------------|
| `400 - Bad Request` | The request was unacceptable, often due to missing a required parameter. |
| `401 - Unauthorized` | No valid API key provided. |
| `402 - Request Failed` | The parameters were valid but the request failed. |
| `403 - Forbidden` | The API key doesn't have permissions to perform the request. |
| `404 - Not Found` | The requested resource doesn't exist. |
| `409 - Conflict` | The request conflicts with another request (e.g., using the same idempotency key). |
| `429 - Too Many Requests` | Too many requests hit the API too quickly. |
| `500, 502, 503, 504 - Server Errors` | Something went wrong on SSS's end. |

---

## Pagination

All top-level API resources have support for bulk fetches via "list" API methods. These list API methods share a common structure, taking at least these two parameters: `limit` and `page`.

### Query Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `limit` | integer | A limit on the number of objects to be returned. Limit can range between 1 and 100, and the default is 10. |
| `page` | integer | The page number to retrieve. Default is 1. |

### Response Object

| Attribute | Type | Description |
|-----------|------|-------------|
| `data` | array | An array containing the actual response objects. |
| `pagination` | object | Pagination metadata. |

---

## Core Resources

### Minting

#### POST /mint
Request a new mint operation. This endpoint initiates the process of creating new stablecoin tokens after verifying fiat backing.

**Parameters**

| Parameter | Type | Description |
|-----------|------|-------------|
| `recipient` | string | **Required**. The Solana wallet address to receive the tokens. |
| `amount` | integer | **Required**. The amount to mint in base units (e.g., 1,000,000 for 1.00 iUSD). |
| `fiat_proof` | object | **Required**. Verification data for the fiat deposit. |
| `idempotency_key` | string | A unique string to prevent duplicate processing. |

**Request Example**
```json
{
  "recipient": "5y...def",
  "amount": 1000000,
  "fiat_proof": {
    "type": "bank_transfer",
    "reference": "WIRE-2024-02-21-001",
    "amount_usd": 1000000,
    "bank": "JPMorgan Chase"
  },
  "idempotency_key": "mint-001-2024-02-21"
}
```

**Response Example**
```json
{
  "id": "req_abc123",
  "object": "mint_request",
  "status": "pending",
  "recipient": "5y...def",
  "amount": 1000000,
  "tx_signature": "4x...abc",
  "estimated_completion": "2024-02-21T13:00:00Z",
  "steps": [
    {"name": "fiat_verification", "status": "completed"},
    {"name": "compliance_check", "status": "completed"},
    {"name": "mint_execution", "status": "pending"}
  ]
}
```

---

### Burning

#### POST /burn
Request a burn operation for fiat redemption.

**Parameters**

| Parameter | Type | Description |
|-----------|------|-------------|
| `amount` | integer | **Required**. The amount to burn in base units. |
| `bank_account` | object | **Required**. Destination bank account for fiat redemption. |
| `idempotency_key` | string | A unique string to prevent duplicate processing. |

**Request Example**
```json
{
  "amount": 500000,
  "bank_account": {
    "account_number": "****1234",
    "routing_number": "021000021",
    "account_holder": "Acme Corp"
  },
  "idempotency_key": "burn-001-2024-02-21"
}
```

**Response Example**
```json
{
  "id": "req_def456",
  "object": "burn_request",
  "status": "pending",
  "amount": 500000,
  "tx_signature": "5y...def",
  "fiat_transfer": {
    "amount_usd": 500000,
    "eta": "2024-02-22T09:00:00Z",
    "reference": "WIRE-2024-02-21-002"
  }
}
```

---

### Proofs

#### GET /proofs
Retrieve Merkle proofs for mint or burn transactions. These proofs are used for on-chain verification or transparency reports.

**Query Parameters**

| Parameter | Type | Description |
|-----------|------|-------------|
| `tx_signature` | string | **Required**. The Solana transaction signature. |
| `type` | string | **Required**. Either `mint` or `burn`. |

**Response Example**
```json
{
  "proof": ["0x...", "0x..."],
  "merkle_root": "0x...",
  "index": 42,
  "leaf": "0x..."
}
```

---

## Compliance

### Status

#### GET /compliance
Get the overall compliance status of the system.

**Response Example**
```json
{
  "compliance_enabled": true,
  "blacklist_count": 15,
  "last_updated": "2024-02-21T12:00:00Z",
  "paused": false
}
```

---

### Screening

#### POST /compliance/screen
Screen a Solana address for sanctions, PEP (Politically Exposed Persons), and general risk.

**Parameters**

| Parameter | Type | Description |
|-----------|------|-------------|
| `address` | string | **Required**. The Solana address to screen. |
| `include_history` | boolean | Whether to include transaction history analysis. Default is `false`. |

**Response Example**
```json
{
  "address": "5y...def",
  "risk_score": 15,
  "risk_level": "LOW",
  "sanctions": false,
  "pep": false,
  "sources_checked": ["OFAC", "EU", "UN", "HMT"],
  "recommendation": "allow",
  "last_updated": "2024-02-21T12:00:00Z"
}
```

---

### Blacklist

#### GET /compliance/blacklist/:address
Check if a specific address is currently blacklisted.

**Response Example**
```json
{
  "blacklisted": true,
  "reason": "OFAC SDN List",
  "added_at": "2024-02-21T12:00:00Z",
  "source": "OFAC",
  "can_appeal": true
}
```

#### POST /compliance/blacklist
Add an address to the global blacklist. This operation requires high-level permissions.

**Parameters**

| Parameter | Type | Description |
|-----------|------|-------------|
| `address` | string | **Required**. The Solana address to blacklist. |
| `reason` | string | **Required**. Reason for blacklisting. |
| `source` | string | The authority or list source (e.g., "OFAC"). |
| `evidence` | array | List of document identifiers or URLs providing evidence. |

**Response Example**
```json
{
  "success": true,
  "blacklist_id": "bl_abc123",
  "tx_signature": "6z...ghi",
  "timestamp": "2024-02-21T12:00:00Z"
}
```

---

## Webhooks

Webhooks allow you to receive real-time notifications when certain events occur in your SSS account.

### Management

#### GET /webhooks
List all configured webhooks.

**Response Example**
```json
{
  "data": [
    {
      "id": "wh_123",
      "url": "https://example.com/hook",
      "events": ["mint.completed", "burn.completed"],
      "active": true,
      "created_at": "2024-02-21T10:00:00Z"
    }
  ],
  "pagination": {
    "total": 1
  }
}
```

#### POST /webhooks
Create a new webhook subscription.

**Parameters**

| Parameter | Type | Description |
|-----------|------|-------------|
| `url` | string | **Required**. The URL where the webhook will be sent. |
| `events` | array | **Required**. List of event types to subscribe to. |
| `description` | string | An optional description for the webhook. |

**Response Example**
```json
{
  "id": "wh_123",
  "url": "https://example.com/hook",
  "events": ["mint.completed", "burn.completed"],
  "secret": "whsec_...",
  "active": true
}
```

#### DELETE /webhooks/:id
Delete a webhook subscription.

**Response Example**
```json
{
  "deleted": true,
  "id": "wh_123"
}
```

---

### Event Types

| Event | Description |
|-------|-------------|
| `mint.completed` | Occurs when a mint operation is successfully executed on-chain. |
| `burn.completed` | Occurs when a burn operation is successfully executed on-chain. |
| `blacklist.added` | Occurs when an address is added to the blacklist. |
| `blacklist.removed` | Occurs when an address is removed from the blacklist. |
| `vault.paused` | Occurs when the vault operations are paused. |

### Webhook Security

All webhook payloads are signed with HMAC-SHA256. The signature is included in the `X-SSS-Signature` header.

```typescript
import { createHmac } from 'crypto';

function verifyWebhook(payload: string, signature: string, secret: string): boolean {
  const expected = createHmac('sha256', secret)
    .update(payload)
    .digest('hex');
  
  return signature === expected;
}
```

---

## Health

#### GET /health
Check the health status of the API and its downstream dependencies.

**Response Example**
```json
{
  "status": "ok",
  "timestamp": "2024-02-21T12:00:00Z",
  "version": "1.0.0",
  "services": {
    "database": "up",
    "solana_rpc": "up",
    "compliance_engine": "up"
  }
}
```
