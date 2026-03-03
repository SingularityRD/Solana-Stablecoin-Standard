#!/bin/bash
# =============================================================================
# Solana Stablecoin Standard — Let's Encrypt SSL Certificate Init Script
# =============================================================================
# Run this ONCE on your GCP VM to obtain initial SSL certificates.
# After this, the certbot container handles automatic renewal.
#
# Usage:
#   chmod +x docker/scripts/init-letsencrypt.sh
#   ./docker/scripts/init-letsencrypt.sh
#
# Prerequisites:
#   - Docker and Docker Compose installed
#   - Domain DNS A record pointing to this VM's external IP
#   - Ports 80 and 443 open in GCP firewall
#   - .env file configured with DOMAIN_NAME and CERTBOT_EMAIL
# =============================================================================

set -euo pipefail

# --- Configuration -----------------------------------------------------------
# Read from .env file or environment, with fallback prompts
if [ -f .env ]; then
    echo "Loading configuration from .env file..."
    export $(grep -v '^#' .env | grep -v '^\s*$' | xargs)
fi

DOMAIN="${DOMAIN_NAME:-}"
EMAIL="${CERTBOT_EMAIL:-}"
STAGING="${CERTBOT_STAGING:-0}"  # Set to 1 for testing (avoids rate limits)

if [ -z "$DOMAIN" ]; then
    read -p "Enter your domain name (e.g., api.example.com): " DOMAIN
fi

if [ -z "$EMAIL" ]; then
    read -p "Enter your email for Let's Encrypt notifications: " EMAIL
fi

if [ -z "$DOMAIN" ] || [ -z "$EMAIL" ]; then
    echo "ERROR: DOMAIN_NAME and CERTBOT_EMAIL are required."
    echo "Set them in .env or pass as environment variables."
    exit 1
fi

echo ""
echo "=== Let's Encrypt SSL Certificate Setup ==="
echo "Domain:  $DOMAIN"
echo "Email:   $EMAIL"
echo "Staging: $([ "$STAGING" = "1" ] && echo "YES (test mode)" || echo "NO (production)")"
echo "============================================"
echo ""

# --- Paths -------------------------------------------------------------------
CERTBOT_CONF="./docker/certbot/conf"
CERTBOT_WWW="./docker/certbot/www"
CERT_NAME="sss-api"
CERT_PATH="$CERTBOT_CONF/live/$CERT_NAME"

# --- Step 1: Create directories ----------------------------------------------
echo "[1/6] Creating certificate directories..."
mkdir -p "$CERTBOT_CONF"
mkdir -p "$CERTBOT_WWW"

# --- Step 2: Download recommended TLS parameters -----------------------------
echo "[2/6] Downloading recommended TLS parameters..."

if [ ! -f "$CERTBOT_CONF/options-ssl-nginx.conf" ]; then
    curl -sSL https://raw.githubusercontent.com/certbot/certbot/master/certbot-nginx/certbot_nginx/_internal/tls_configs/options-ssl-nginx.conf \
        -o "$CERTBOT_CONF/options-ssl-nginx.conf"
    echo "  → Downloaded options-ssl-nginx.conf"
else
    echo "  → options-ssl-nginx.conf already exists, skipping"
fi

if [ ! -f "$CERTBOT_CONF/ssl-dhparams.pem" ]; then
    curl -sSL https://raw.githubusercontent.com/certbot/certbot/master/certbot/certbot/ssl-dhparams.pem \
        -o "$CERTBOT_CONF/ssl-dhparams.pem"
    echo "  → Downloaded ssl-dhparams.pem"
else
    echo "  → ssl-dhparams.pem already exists, skipping"
fi

# --- Step 3: Create temporary self-signed certificate ------------------------
# Nginx needs a certificate to start. We create a temporary one, then replace it.
echo "[3/6] Creating temporary self-signed certificate..."
mkdir -p "$CERT_PATH"

if [ ! -f "$CERT_PATH/fullchain.pem" ]; then
    openssl req -x509 -nodes -newkey rsa:4096 -days 1 \
        -keyout "$CERT_PATH/privkey.pem" \
        -out "$CERT_PATH/fullchain.pem" \
        -subj "/CN=localhost" \
        2>/dev/null
    echo "  → Temporary certificate created"
else
    echo "  → Certificate already exists, skipping temporary creation"
fi

# --- Step 4: Start nginx with temporary cert ---------------------------------
echo "[4/6] Starting nginx with temporary certificate..."
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d nginx
echo "  → Waiting for nginx to be ready..."
sleep 5

# Verify nginx is responding
if curl -sf http://localhost/health > /dev/null 2>&1; then
    echo "  → Nginx is responding on port 80"
else
    echo "  → WARNING: Nginx health check failed, continuing anyway..."
fi

# --- Step 5: Delete temporary cert and request real one ----------------------
echo "[5/6] Requesting Let's Encrypt certificate..."

# Remove temporary certificate
rm -rf "$CERT_PATH"

# Build certbot command
STAGING_FLAG=""
if [ "$STAGING" = "1" ]; then
    STAGING_FLAG="--staging"
fi

# Request real certificate via certbot
docker-compose -f docker-compose.yml -f docker-compose.prod.yml run --rm certbot certonly \
    --webroot \
    -w /var/www/certbot \
    --cert-name "$CERT_NAME" \
    -d "$DOMAIN" \
    --email "$EMAIL" \
    --agree-tos \
    --no-eff-email \
    --force-renewal \
    $STAGING_FLAG

echo "  → Certificate obtained successfully!"

# --- Step 6: Reload nginx with real certificate ------------------------------
echo "[6/6] Reloading nginx with Let's Encrypt certificate..."
docker-compose -f docker-compose.yml -f docker-compose.prod.yml exec nginx nginx -s reload

echo ""
echo "=== SSL Setup Complete! ==="
echo ""
echo "Your site is now available at: https://$DOMAIN"
echo ""
echo "Certificate auto-renewal is handled by the certbot container."
echo "Certificates will be renewed automatically every 12 hours (if needed)."
echo ""
echo "To verify SSL configuration:"
echo "  curl -I https://$DOMAIN/health"
echo ""
echo "To manually renew certificates:"
echo "  docker-compose -f docker-compose.yml -f docker-compose.prod.yml run --rm certbot renew"
echo "  docker-compose -f docker-compose.yml -f docker-compose.prod.yml exec nginx nginx -s reload"
echo ""
echo "GCP Firewall reminder — make sure these rules exist:"
echo "  gcloud compute firewall-rules create allow-http  --allow tcp:80  --target-tags=http-server"
echo "  gcloud compute firewall-rules create allow-https --allow tcp:443 --target-tags=https-server"
echo ""
