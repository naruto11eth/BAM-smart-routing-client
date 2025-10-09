#!/usr/bin/env bash
set -euo pipefail

# Dependencies: curl, grep, sort, awk; jq optional for pretty JSON output.

PROBE_TIMEOUT_CONNECT=${PROBE_TIMEOUT_CONNECT:-1}     # seconds
PROBE_TIMEOUT_TOTAL=${PROBE_TIMEOUT_TOTAL:-1.5}       # seconds
CONCURRENCY=${CONCURRENCY:-6}

discover_regions() {
  curl -sS https://bam.dev/validators/ \
  | grep -Eo 'https?://[a-z0-9.-]+\.testnet\.bam\.jito\.wtf' \
  | sort -u
}

probe_regions() {
  local regions="$1"
  printf "%s\n" ${regions} \
  | xargs -I{} -P "${CONCURRENCY}" sh -c '
      u="$1"
      t=$(curl -s -o /dev/null -w "%{time_connect}" \
             --connect-timeout '"${PROBE_TIMEOUT_CONNECT}"' \
             --max-time '"${PROBE_TIMEOUT_TOTAL}"' \
             "$u/")
      test -z "$t" && t=9.999
      printf "%8.3f  %s\n" "$t" "$u"
    ' _ {}
}

pick_fastest() {
  awk 'NR==1{print $2}'
}

# --- Optional: submit a signed transaction -------------------------------
# By default we submit to Solana testnet RPC (recommended).
# If you *know* the per-region host exposes public RPC on :8899, set USE_REGION_RPC=1.
submit_tx() {
  local tx_b64="$1"
  local fastest_url="$2"
  local rpc="${RPC_ENDPOINT:-https://api.testnet.solana.com}"

  if [ "${USE_REGION_RPC:-0}" = "1" ]; then
    host=$(echo "$fastest_url" | sed -E 's#https?://([^/]+)/?.*#\1#')
    rpc="http://${host}:8899"
  fi

  echo "Submitting to RPC: $rpc" >&2
  # Build JSON-RPC body (sendTransaction)
  body=$(cat <<JSON
{"jsonrpc":"2.0","id":1,"method":"sendTransaction",
 "params":["${tx_b64}", {"encoding":"base64","preflightCommitment":"confirmed","maxRetries":3}]}
JSON
)
  resp=$(curl -sS -X POST "$rpc" -H 'Content-Type: application/json' --data-binary "$body")
  if command -v jq >/dev/null 2>&1; then
    echo "$resp" | jq
  else
    echo "$resp"
  fi
}

# ------------------ main ------------------
regions=$(discover_regions)
[ -z "$regions" ] && { echo "No testnet regions found." >&2; exit 1; }

echo "Probing (connect timeout ${PROBE_TIMEOUT_CONNECT}s, total ${PROBE_TIMEOUT_TOTAL}s) ..."
results=$(probe_regions "$regions" | sort -n)
echo "$results"

fastest=$(echo "$results" | pick_fastest)
echo "Fastest region: $fastest"

# If you pass TX_B64, we will submit it:
if [ "${TX_B64:-}" != "" ]; then
  submit_tx "$TX_B64" "$fastest"
fi

