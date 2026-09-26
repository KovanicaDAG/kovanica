#!/usr/bin/env bash
# Update seed2.kovanica.online A record to point to current server IP
# Requires: CLOUDFLARE_API_TOKEN env var with Zone:Edit permission for kovanica.online

set -euo pipefail

ZONE_NAME="kovanica.online"
RECORD_NAME="seed2"
RECORD_TYPE="A"
TARGET_IP="76.13.250.65"  # Seed2 server public IPv4 (separate node)
TTL=300  # 5 minutes

if [[ -z "${CLOUDFLARE_API_TOKEN:-}" ]]; then
    echo "ERROR: CLOUDFLARE_API_TOKEN not set"
    echo "Create a token at https://dash.cloudflare.com/profile/api-tokens"
    echo "Permissions: Zone > Zone > Read, Zone > DNS > Edit"
    exit 1
fi

# Get zone ID
ZONE_ID=$(curl -s -X GET "https://api.cloudflare.com/client/v4/zones?name=${ZONE_NAME}" \
  -H "Authorization: Bearer ${CLOUDFLARE_API_TOKEN}" \
  -H "Content-Type: application/json" | jq -r '.result[0].id')

if [[ -z "$ZONE_ID" || "$ZONE_ID" == "null" ]]; then
    echo "ERROR: Could not find zone ID for ${ZONE_NAME}"
    exit 1
fi

echo "Zone ID: $ZONE_ID"

# Get existing record ID
RECORD_ID=$(curl -s -X GET "https://api.cloudflare.com/client/v4/zones/${ZONE_ID}/dns_records?name=${RECORD_NAME}.${ZONE_NAME}&type=${RECORD_TYPE}" \
  -H "Authorization: Bearer ${CLOUDFLARE_API_TOKEN}" \
  -H "Content-Type: application/json" | jq -r '.result[0].id')

if [[ -z "$RECORD_ID" || "$RECORD_ID" == "null" ]]; then
    echo "Creating new A record for ${RECORD_NAME}.${ZONE_NAME} -> ${TARGET_IP}"
    curl -s -X POST "https://api.cloudflare.com/client/v4/zones/${ZONE_ID}/dns_records" \
      -H "Authorization: Bearer ${CLOUDFLARE_API_TOKEN}" \
      -H "Content-Type: application/json" \
      --data "{\"type\":\"${RECORD_TYPE}\",\"name\":\"${RECORD_NAME}\",\"content\":\"${TARGET_IP}\",\"ttl\":${TTL},\"proxied\":false}" | jq
else
    echo "Updating existing A record (ID: $RECORD_ID) for ${RECORD_NAME}.${ZONE_NAME} -> ${TARGET_IP}"
    curl -s -X PUT "https://api.cloudflare.com/client/v4/zones/${ZONE_ID}/dns_records/${RECORD_ID}" \
      -H "Authorization: Bearer ${CLOUDFLARE_API_TOKEN}" \
      -H "Content-Type: application/json" \
      --data "{\"type\":\"${RECORD_TYPE}\",\"name\":\"${RECORD_NAME}\",\"content\":\"${TARGET_IP}\",\"ttl\":${TTL},\"proxied\":false}" | jq
fi

echo ""
echo "Verification:"
dig +short "${RECORD_NAME}.${ZONE_NAME}"