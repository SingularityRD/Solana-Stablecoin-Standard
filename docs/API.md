# Backend API Reference

The Solana Stablecoin Standard (SSS) API is organized around REST. Our API has predictable resource-oriented URLs, accepts JSON-encoded request bodies, returns JSON-encoded responses, and uses standard HTTP response codes, authentication, and verbs.

## Base URL

| Environment | URL |
|-------------|-----|
| **Development** | `http://localhost:3001` |
| **Devnet** | `https://api.devnet.sss-token.io/v1` |
| **Mainnet** | `https://api.sss-token.io/v1` |

---

## Authentication

The SSS API uses JWT (JSON Web Tokens) for authentication. Tokens are obtained through the login/register endpoints.

### Authorization Header

All authenticated requests require the `Authorization` header:

```bash
Authorization: Bearer <access_token>
```

### Token Types

| Token Type | Description | Default Expiry |
|------------|-------------|----------------|
| `access_token` | Used for API authentication | 24 hours |
| `refresh_token` | Used to obtain new access tokens | 7 days |

---

## Errors

SSS uses conventional HTTP response codes to indicate the success or failure of an API request.

### Error Response Format

```json
{
  "error": {
    "code": "validation_error",
    "message": "Invalid input parameters",
    "details": "email: invalid email format"
  },
  "request_id": "req_abc123"
}
```

### HTTP Status Codes

| Code | Description |
|------|-------------|
| `200 - OK` | Request successful |
| `201 - Created` | Resource created successfully |
| `204 - No Content` | Successful deletion |
| `400 - Bad Request` | Invalid request parameters |
| `401 - Unauthorized` | Missing or invalid authentication |
| `403 - Forbidden` | Insufficient permissions |
| `404 - Not Found` | Resource not found |
| `409 - Conflict` | Resource already exists |
| `422 - Unprocessable Entity` | Validation error |
| `429 - Too Many Requests` | Rate limit exceeded |
| `500 - Internal Server Error` | Server error |

---

## Pagination

List endpoints support pagination via query parameters.

### Query Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `limit` | integer | 100 | Maximum items per page (max 1000) |
| `offset` | integer | 0 | Number of items to skip |

### Response Format

```json
{
  "data": [...],
  "pagination": {
    "total": 150,
    "limit": 100,
    "offset": 0
  }
}
```

---

## Health & Monitoring

### GET /health
Basic health check endpoint.

**Response**
```json
{
  "status": "ok"
}
```

### GET /health/detail
Detailed health check with component status.

**Response**
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 86400,
  "timestamp": "2024-02-21T12:00:00Z",
  "environment": "production",
  "components": {
    "database": {
      "status": "healthy",
      "latency_ms": 5
    },
    "solana_rpc": {
      "status": "healthy",
      "latency_ms": 150
    },
    "memory": {
      "status": "healthy",
      "used_mb": 256,
      "total_mb": 1024,
      "usage_percent": 25.0
    }
  }
}
```

### GET /health/ready
Kubernetes readiness probe. Returns 200 only if all dependencies are healthy.

**Response**
```json
{
  "ready": true,
  "checks": {
    "database": true,
    "solana_rpc": true
  }
}
```

### GET /health/live
Kubernetes liveness probe. Returns 200 if the service is running.

**Response**
```json
{
  "alive": true
}
```

### GET /metrics
Prometheus metrics endpoint.

**Response**
```
# HELP http_requests_total Total HTTP requests
# TYPE http_requests_total counter
http_requests_total{method="GET",path="/health",status="200"} 1234
...
```

---

## Authentication Endpoints

### POST /api/v1/auth/register
Register a new user account.

**Parameters**

| Parameter | Type | Description |
|-----------|------|-------------|
| `email` | string | **Required**. Valid email address. |
| `password` | string | **Required**. Minimum 8 characters. |
| `solana_pubkey` | string | Optional. Solana wallet public key. |

**Request Example**
```json
{
  "email": "user@example.com",
  "password": "securepassword123",
  "solana_pubkey": "5y...def"
}
```

**Response Example**
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
  "token_type": "Bearer",
  "expires_in": 86400,
  "user": {
    "id": "uuid",
    "email": "user@example.com",
    "role": "user",
    "solana_pubkey": "5y...def",
    "created_at": "2024-02-21T12:00:00Z"
  }
}
```

### POST /api/v1/auth/login
Authenticate and obtain tokens.

**Parameters**

| Parameter | Type | Description |
|-----------|------|-------------|
| `email` | string | **Required**. User email. |
| `password` | string | **Required**. User password. |

**Response Example**
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
  "token_type": "Bearer",
  "expires_in": 86400,
  "user": {
    "id": "uuid",
    "email": "user@example.com",
    "role": "user",
    "solana_pubkey": "5y...def"
  }
}
```

### POST /api/v1/auth/refresh
Refresh access token using refresh token.

**Parameters**

| Parameter | Type | Description |
|-----------|------|-------------|
| `refresh_token` | string | **Required**. Valid refresh token. |

**Response Example**
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
  "token_type": "Bearer",
  "expires_in": 86400,
  "user": {...}
}
```

---

## User Endpoints

### GET /api/v1/users/me
Get current user profile.

**Response Example**
```json
{
  "id": "uuid",
  "email": "user@example.com",
  "role": "user",
  "solana_pubkey": "5y...def",
  "created_at": "2024-02-21T12:00:00Z",
  "updated_at": "2024-02-21T12:00:00Z"
}
```

### PUT /api/v1/users/me
Update current user profile.

**Parameters**

| Parameter | Type | Description |
|-----------|------|-------------|
| `solana_pubkey` | string | Optional. Solana wallet public key. |

**Request Example**
```json
{
  "solana_pubkey": "7z...ghi"
}
```

---

## Stablecoin Endpoints

### POST /api/v1/stablecoin
Create a new stablecoin.

**Parameters**

| Parameter | Type | Description |
|-----------|------|-------------|
| `name` | string | **Required**. Token name (max 32 chars). |
| `symbol` | string | **Required**. Token symbol (max 10 chars). |
| `asset_mint` | string | **Required**. Solana pubkey for the asset mint. |
| `preset` | integer | **Required**. 1 for SSS-1, 2 for SSS-2. |
| `decimals` | integer | Token decimals (default: 6). |

**Request Example**
```json
{
  "name": "My Stablecoin",
  "symbol": "MYUSD",
  "asset_mint": "5y...def",
  "preset": 2,
  "decimals": 6
}
```

**Response Example**
```json
{
  "id": "uuid",
  "owner_id": "user_uuid",
  "name": "My Stablecoin",
  "symbol": "MYUSD",
  "decimals": 6,
  "preset": 2,
  "asset_mint": "5y...def",
  "stablecoin_pda": "7z...ghi",
  "authority_pubkey": "9x...jkl",
  "is_active": true,
  "created_at": "2024-02-21T12:00:00Z"
}
```

### GET /api/v1/stablecoin
List all stablecoins for the authenticated user (or all for admins).

**Response Example**
```json
[
  {
    "id": "uuid",
    "name": "My Stablecoin",
    "symbol": "MYUSD",
    "preset": 2,
    "is_active": true,
    ...
  }
]
```

### GET /api/v1/stablecoin/:id
Get a specific stablecoin by ID.

**Response Example**
```json
{
  "id": "uuid",
  "owner_id": "user_uuid",
  "name": "My Stablecoin",
  "symbol": "MYUSD",
  ...
}
```

### PUT /api/v1/stablecoin/:id
Update a stablecoin.

**Parameters**

| Parameter | Type | Description |
|-----------|------|-------------|
| `name` | string | Optional. New name. |
| `is_active` | boolean | Optional. Active status. |

### GET /api/v1/stablecoin/:id/status
Get on-chain status for a stablecoin.

**Response Example**
```json
{
  "stablecoin": {...},
  "total_supply": 1000000000,
  "paused": false,
  "compliance_enabled": true,
  "holder_count": 150
}
```

---

## Operations Endpoints

### POST /api/v1/stablecoin/:id/mint
Mint new tokens to a recipient.

**Parameters**

| Parameter | Type | Description |
|-----------|------|-------------|
| `recipient` | string | **Required**. Recipient wallet address. |
| `amount` | integer | **Required**. Amount to mint in base units. |

**Request Example**
```json
{
  "recipient": "5y...def",
  "amount": 1000000
}
```

**Response Example**
```json
{
  "tx_signature": "4x...abc",
  "status": "pending",
  "explorer_url": "https://explorer.solana.com/tx/4x...abc"
}
```

### POST /api/v1/stablecoin/:id/burn
Burn tokens from an account.

**Parameters**

| Parameter | Type | Description |
|-----------|------|-------------|
| `amount` | integer | **Required**. Amount to burn in base units. |
| `from_account` | string | Optional. Source token account. |

**Request Example**
```json
{
  "amount": 500000,
  "from_account": "5y...def"
}
```

### POST /api/v1/stablecoin/:id/transfer
Transfer tokens between accounts.

**Parameters**

| Parameter | Type | Description |
|-----------|------|-------------|
| `from` | string | **Required**. Source token account. |
| `to` | string | **Required**. Destination token account. |
| `amount` | integer | **Required**. Amount to transfer. |

**Request Example**
```json
{
  "from": "5y...def",
  "to": "7z...ghi",
  "amount": 100000
}
```

---

## Admin Endpoints

### POST /api/v1/stablecoin/:id/pause
Pause all stablecoin operations. Requires Pauser role.

**Response Example**
```json
{
  "tx_signature": "pause_tx_...",
  "status": "pending",
  "explorer_url": "..."
}
```

### POST /api/v1/stablecoin/:id/unpause
Resume all stablecoin operations. Requires Pauser role.

### POST /api/v1/stablecoin/:id/freeze/:account
Freeze a specific token account. Requires Blacklister/Pauser role.

**Response Example**
```json
{
  "tx_signature": "freeze_...",
  "status": "pending",
  "explorer_url": "..."
}
```

### POST /api/v1/stablecoin/:id/thaw/:account
Unfreeze a previously frozen account.

### POST /api/v1/stablecoin/:id/seize
Seize tokens from a blacklisted account (SSS-2 only). Requires Seizer role.

**Parameters**

| Parameter | Type | Description |
|-----------|------|-------------|
| `from_account` | string | **Required**. Account to seize from (must be blacklisted). |
| `to_account` | string | **Required**. Destination account for seized tokens. |
| `amount` | integer | **Required**. Amount to seize. |

**Request Example**
```json
{
  "from_account": "5y...def",
  "to_account": "7z...ghi",
  "amount": 1000000
}
```

---

## Role Management Endpoints

### POST /api/v1/stablecoin/:id/roles
Assign a role to an account. Requires Master role.

**Parameters**

| Parameter | Type | Description |
|-----------|------|-------------|
| `account` | string | **Required**. Solana pubkey to assign role to. |
| `role` | string | **Required**. Role: master, minter, burner, blacklister, pauser, seizer. |

**Request Example**
```json
{
  "account": "5y...def",
  "role": "minter"
}
```

**Response Example**
```json
{
  "id": "uuid",
  "stablecoin_id": "uuid",
  "account_pubkey": "5y...def",
  "role": "minter",
  "assigned_by": "user_uuid",
  "created_at": "2024-02-21T12:00:00Z"
}
```

### DELETE /api/v1/stablecoin/:id/roles/:account
Revoke all roles from an account. Requires Master role.

### GET /api/v1/stablecoin/:id/roles
List all role assignments for a stablecoin.

**Response Example**
```json
[
  {
    "id": "uuid",
    "account_pubkey": "5y...def",
    "role": "minter",
    "assigned_by": "user_uuid",
    "created_at": "2024-02-21T12:00:00Z"
  }
]
```

---

## Minter Management Endpoints

### POST /api/v1/stablecoin/:id/minters
Add a minter with quota. Requires Master role.

**Parameters**

| Parameter | Type | Description |
|-----------|------|-------------|
| `account` | string | **Required**. Minter's Solana pubkey. |
| `quota` | integer | Optional. Minting quota (0 = unlimited). |

**Request Example**
```json
{
  "account": "5y...def",
  "quota": 1000000000
}
```

**Response Example**
```json
{
  "id": "uuid",
  "stablecoin_id": "uuid",
  "minter_pubkey": "5y...def",
  "quota": 1000000000,
  "minted_amount": 0,
  "created_at": "2024-02-21T12:00:00Z"
}
```

### GET /api/v1/stablecoin/:id/minters
List all minters for a stablecoin.

### DELETE /api/v1/stablecoin/:id/minters/:account
Remove a minter. Requires Master role.

### PUT /api/v1/stablecoin/:id/minters/:account/quota
Update minter quota. Requires Master role.

**Parameters**

| Parameter | Type | Description |
|-----------|------|-------------|
| `quota` | integer | **Required**. New quota value. |

**Request Example**
```json
{
  "quota": 5000000000
}
```

---

## Compliance Endpoints

### GET /api/v1/stablecoin/:id/blacklist
List all blacklisted accounts.

**Response Example**
```json
[
  {
    "id": "uuid",
    "stablecoin_id": "uuid",
    "account_pubkey": "5y...def",
    "reason": "OFAC sanctions match",
    "blacklisted_by": "user_uuid",
    "is_active": true,
    "created_at": "2024-02-21T12:00:00Z"
  }
]
```

### POST /api/v1/stablecoin/:id/blacklist
Add an account to the blacklist. Requires Blacklister role.

**Parameters**

| Parameter | Type | Description |
|-----------|------|-------------|
| `account` | string | **Required**. Solana pubkey to blacklist. |
| `reason` | string | **Required**. Reason for blacklisting. |

**Request Example**
```json
{
  "account": "5y...def",
  "reason": "OFAC SDN List Match"
}
```

### DELETE /api/v1/stablecoin/:id/blacklist/:account
Remove an account from the blacklist. Requires Blacklister role.

### GET /api/v1/stablecoin/:id/screen/:address
Screen an address for compliance risk.

**Response Example**
```json
{
  "address": "5y...def",
  "risk_score": 0,
  "is_sanctioned": false,
  "is_blacklisted": false,
  "recommendation": "allow"
}
```

---

## Audit Endpoints

### GET /api/v1/stablecoin/:id/audit
List audit logs for a stablecoin.

**Query Parameters**

| Parameter | Type | Description |
|-----------|------|-------------|
| `action` | string | Filter by action type. |
| `limit` | integer | Max results (default 100, max 1000). |
| `offset` | integer | Pagination offset. |

**Response Example**
```json
[
  {
    "id": "uuid",
    "stablecoin_id": "uuid",
    "user_id": "user_uuid",
    "action": "stablecoin.mint",
    "tx_signature": "4x...abc",
    "metadata": {
      "recipient": "5y...def",
      "amount": 1000000
    },
    "created_at": "2024-02-21T12:00:00Z"
  }
]
```

### GET /api/v1/audit/:tx_signature
Get a specific audit log entry by transaction signature.

---

## Webhook Endpoints

### GET /api/v1/stablecoin/:id/webhooks
List all webhooks for a stablecoin.

**Response Example**
```json
[
  {
    "id": "uuid",
    "stablecoin_id": "uuid",
    "url": "https://example.com/webhook",
    "events": ["mint.completed", "burn.completed"],
    "is_active": true,
    "created_at": "2024-02-21T12:00:00Z"
  }
]
```

### POST /api/v1/stablecoin/:id/webhooks
Create a new webhook subscription.

**Parameters**

| Parameter | Type | Description |
|-----------|------|-------------|
| `url` | string | **Required**. Webhook endpoint URL. |
| `events` | array | **Required**. List of event types. |
| `secret` | string | Optional. Secret for signature verification. |

**Request Example**
```json
{
  "url": "https://example.com/webhook",
  "events": ["mint.completed", "burn.completed", "blacklist.added"],
  "secret": "whsec_..."
}
```

### DELETE /api/v1/stablecoin/:id/webhooks/:webhook_id
Delete a webhook subscription.

### POST /webhooks
Incoming webhook handler (for external services).

---

## Proofs Endpoint

### GET /proofs
Get Merkle proofs for transactions.

**Note**: Currently a stub endpoint. Full implementation pending.

**Response Example**
```json
{
  "proofs": []
}
```

---

## Event Types

| Event | Description |
|-------|-------------|
| `mint.completed` | Mint operation executed successfully. |
| `burn.completed` | Burn operation executed successfully. |
| `transfer.completed` | Transfer executed successfully. |
| `blacklist.added` | Address added to blacklist. |
| `blacklist.removed` | Address removed from blacklist. |
| `vault.paused` | Vault operations paused. |
| `vault.unpaused` | Vault operations resumed. |
| `account.frozen` | Token account frozen. |
| `account.thawed` | Token account unfrozen. |
| `seizure.completed` | Token seizure completed. |
| `role.assigned` | Role assigned to account. |
| `role.revoked` | Role revoked from account. |

---

## Webhook Security

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

## Rate Limiting

API requests are rate limited per IP address:

| Environment | Requests | Window |
|-------------|----------|--------|
| Development | 1000 | 60 seconds |
| Production | 100 | 60 seconds |

Rate limit headers are included in responses:

```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1708527600
```

---

## Security Headers

All API responses include security headers:

| Header | Value |
|--------|-------|
| `X-Content-Type-Options` | `nosniff` |
| `X-Frame-Options` | `DENY` |
| `X-XSS-Protection` | `1; mode=block` |
| `Referrer-Policy` | `strict-origin-when-cross-origin` |
| `Content-Security-Policy` | `default-src 'self'; frame-ancestors 'none'` |
| `Permissions-Policy` | `geolocation=(), microphone=(), camera=(), payment=()` |

In production, additional headers are included:

| Header | Value |
|--------|-------|
| `Strict-Transport-Security` | `max-age=31536000; includeSubDomains; preload` |

---

## Request ID

Each request is assigned a unique identifier for debugging:

```
X-Request-Id: req_abc123def456
```

Include this ID when reporting issues.