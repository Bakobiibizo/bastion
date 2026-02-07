#!/usr/bin/env bash
# test-agent.sh — End-to-end test harness for bastion-agent
#
# Usage:
#   ./scripts/test-agent.sh [relay_multiaddr] [contact_string]
#
# Examples:
#   # Standalone — just start agent, create identity, connect to relay
#   ./scripts/test-agent.sh /ip4/35.173.176.243/tcp/4001/p2p/12D3KooWHi81G15poZuH4BL5WnifQ3b2S2uSkvsa1wAhBZNA8PW9
#
#   # With a friend — add a contact and try to message them
#   ./scripts/test-agent.sh /ip4/35.173.176.243/tcp/4001/p2p/12D3KooW... "harbor://eyJ..."
#
# The agent runs on http://127.0.0.1:${PORT} (default 8745).
# Press Ctrl+C to stop.

set -euo pipefail

RELAY="${1:-}"
CONTACT="${2:-}"
PORT="${BASTION_PORT:-8745}"
DATA_DIR="${BASTION_DATA_DIR:-/tmp/bastion-test-$$}"
PASSPHRASE="${BASTION_PASSPHRASE:-test-passphrase-$$}"
AGENT_BIN="${BASTION_BIN:-$(dirname "$0")/../harbor-agent/target/release/bastion-agent}"
BASE="http://127.0.0.1:${PORT}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

pass() { echo -e "  ${GREEN}✓${NC} $1"; }
fail() { echo -e "  ${RED}✗${NC} $1"; }
info() { echo -e "  ${CYAN}→${NC} $1"; }
header() { echo -e "\n${YELLOW}[$1]${NC}"; }

api() {
  local method="$1" path="$2"
  shift 2
  curl -sf -X "$method" "${BASE}${path}" -H 'Content-Type: application/json' "$@" 2>/dev/null
}

cleanup() {
  if [ -n "${AGENT_PID:-}" ]; then
    kill "$AGENT_PID" 2>/dev/null || true
    wait "$AGENT_PID" 2>/dev/null || true
  fi
  echo -e "\n${CYAN}Data dir: ${DATA_DIR}${NC}"
  echo -e "${CYAN}To clean up: rm -rf ${DATA_DIR}${NC}"
}
trap cleanup EXIT

# ── Preflight ──────────────────────────────────────────────
header "Preflight"

if [ ! -x "$AGENT_BIN" ]; then
  fail "Agent binary not found at $AGENT_BIN"
  echo "  Build it: cargo build --release --manifest-path harbor-agent/Cargo.toml"
  exit 1
fi
pass "Agent binary: $AGENT_BIN"

mkdir -p "$DATA_DIR"
pass "Data dir: $DATA_DIR"

# ── Start Agent ────────────────────────────────────────────
header "Starting agent on port ${PORT}"

"$AGENT_BIN" --data-dir "$DATA_DIR" --port "$PORT" &
AGENT_PID=$!
disown "$AGENT_PID" 2>/dev/null || true

# Wait for it to be ready
for i in $(seq 1 20); do
  if curl -sf "${BASE}/api/identity/status" >/dev/null 2>&1; then
    break
  fi
  sleep 0.5
done

if ! curl -sf "${BASE}/api/identity/status" >/dev/null 2>&1; then
  fail "Agent did not start within 10 seconds"
  exit 1
fi
pass "Agent running (PID ${AGENT_PID})"

# ── Identity ──────────────────────────────────────────────
header "Identity"

STATUS=$(api GET /api/identity/status)
HAS_ID=$(echo "$STATUS" | python3 -c "import sys,json; print(json.load(sys.stdin)['hasIdentity'])" 2>/dev/null)

if [ "$HAS_ID" = "True" ]; then
  info "Identity exists, unlocking..."
  IDENTITY=$(api POST /api/identity/unlock -d "{\"passphrase\":\"${PASSPHRASE}\"}")
else
  info "Creating new identity..."
  IDENTITY=$(api POST /api/identity -d "{\"displayName\":\"test-agent\",\"passphrase\":\"${PASSPHRASE}\",\"bio\":\"Automated test agent\"}")
fi

PEER_ID=$(echo "$IDENTITY" | python3 -c "import sys,json; print(json.load(sys.stdin)['peerId'])" 2>/dev/null)
DISPLAY=$(echo "$IDENTITY" | python3 -c "import sys,json; print(json.load(sys.stdin)['displayName'])" 2>/dev/null)

if [ -n "$PEER_ID" ]; then
  pass "Identity: ${DISPLAY} (${PEER_ID})"
else
  fail "Failed to create/unlock identity"
  echo "  Response: $IDENTITY"
  exit 1
fi

# ── Network ───────────────────────────────────────────────
header "Network"

api POST /api/network/start >/dev/null
sleep 2

NET_STATUS=$(api GET /api/network/status)
RUNNING=$(echo "$NET_STATUS" | python3 -c "import sys,json; print(json.load(sys.stdin)['running'])" 2>/dev/null)

if [ "$RUNNING" = "True" ]; then
  pass "Network started"
else
  fail "Network failed to start"
  exit 1
fi

# ── Relay ─────────────────────────────────────────────────
if [ -n "$RELAY" ]; then
  header "Relay"

  api POST /api/network/relay -d "{\"multiaddr\":\"${RELAY}\"}" >/dev/null
  sleep 3

  NET_STATUS=$(api GET /api/network/status)
  PEERS=$(echo "$NET_STATUS" | python3 -c "import sys,json; print(json.load(sys.stdin)['stats']['connectedPeers'])" 2>/dev/null)

  if [ "${PEERS:-0}" -gt 0 ]; then
    pass "Connected to relay (${PEERS} peers)"
  else
    fail "No peers after relay connect"
  fi

  # Show relay info
  PEER_LIST=$(api GET /api/network/peers)
  echo "$PEER_LIST" | python3 -c "
import sys, json
peers = json.load(sys.stdin)
for p in peers:
    print(f'  → {p[\"peerId\"][:20]}... ({p.get(\"protocolVersion\",\"?\")})')
" 2>/dev/null || true
fi

# ── Contact ───────────────────────────────────────────────
if [ -n "$CONTACT" ]; then
  header "Contact"

  RESULT=$(api POST /api/contacts/from-string -d "{\"contactString\":\"${CONTACT}\"}")

  if echo "$RESULT" | grep -q "12D3KooW"; then
    CONTACT_PEER=$(echo "$RESULT" | tr -d '"')
    pass "Added contact: ${CONTACT_PEER}"

    # Wait for connection
    sleep 3
    PEER_LIST=$(api GET /api/network/peers)
    if echo "$PEER_LIST" | grep -q "$CONTACT_PEER"; then
      pass "Connected to contact peer"
    else
      info "Contact added but not connected yet (may need relay circuit)"
      # Try explicit connect
      MULTIADDR=$(echo "$CONTACT" | sed 's|harbor://||' | base64 -d 2>/dev/null | python3 -c "import sys,json; print(json.load(sys.stdin)['multiaddr'])" 2>/dev/null)
      if [ -n "$MULTIADDR" ]; then
        info "Dialing: ${MULTIADDR}"
        api POST /api/network/connect -d "{\"multiaddr\":\"${MULTIADDR}\"}" >/dev/null || true
        sleep 3
        if api GET /api/network/peers | grep -q "$CONTACT_PEER"; then
          pass "Connected after explicit dial"
        else
          fail "Could not connect to contact"
        fi
      fi
    fi
  else
    fail "Failed to add contact: $RESULT"
  fi
fi

# ── Status Summary ────────────────────────────────────────
header "Summary"

NET_STATUS=$(api GET /api/network/status)
CONTACTS=$(api GET /api/contacts)
CONVERSATIONS=$(api GET /api/conversations)
ADDRESSES=$(api GET /api/network/addresses)

PEER_COUNT=$(echo "$NET_STATUS" | python3 -c "import sys,json; print(json.load(sys.stdin)['stats']['connectedPeers'])" 2>/dev/null)
CONTACT_COUNT=$(echo "$CONTACTS" | python3 -c "import sys,json; print(len(json.load(sys.stdin)))" 2>/dev/null)
CONV_COUNT=$(echo "$CONVERSATIONS" | python3 -c "import sys,json; print(len(json.load(sys.stdin)))" 2>/dev/null)
ADDR_COUNT=$(echo "$ADDRESSES" | python3 -c "import sys,json; print(len(json.load(sys.stdin)))" 2>/dev/null)

echo -e "  Peer ID:       ${CYAN}${PEER_ID}${NC}"
echo -e "  Display Name:  ${DISPLAY}"
echo -e "  Connected:     ${PEER_COUNT} peers"
echo -e "  Contacts:      ${CONTACT_COUNT}"
echo -e "  Conversations: ${CONV_COUNT}"
echo -e "  Addresses:     ${ADDR_COUNT}"
echo -e "  API:           ${BASE}"
echo -e "  Data:          ${DATA_DIR}"

# Try to get shareable contact string
CONTACT_STR=$(api GET /api/network/contact-string 2>/dev/null || echo "")
if echo "$CONTACT_STR" | grep -q "harbor://"; then
  CLEAN=$(echo "$CONTACT_STR" | tr -d '"')
  echo -e "\n  ${GREEN}Your contact string:${NC}"
  echo -e "  ${CLEAN}"
fi

# ── Interactive ───────────────────────────────────────────
header "Agent running — API reference"

cat <<'EOF'
  Identity:
    GET  /api/identity              — get identity info
    GET  /api/identity/status       — check locked/unlocked

  Network:
    GET  /api/network/status        — network stats
    GET  /api/network/peers         — connected peers
    POST /api/network/relay         — {"multiaddr":"..."}
    POST /api/network/connect       — {"multiaddr":"..."}
    GET  /api/network/contact-string — shareable harbor:// URI

  Contacts:
    GET  /api/contacts              — list contacts
    POST /api/contacts/from-string  — {"contactString":"harbor://..."}

  Messaging:
    POST /api/messages/send         — {"peerId":"...","content":"..."}
    GET  /api/messages/{peerId}     — get conversation
    GET  /api/conversations         — list conversations

  Boards:
    GET  /api/communities           — list joined communities
    POST /api/communities/join      — {"relayAddress":"..."}
    GET  /api/boards/{relay}/       — list boards
    POST /api/boards/{relay}/{id}/posts — post to board

  Events:
    GET  /api/events                — SSE event stream

EOF

echo -e "${YELLOW}Press Ctrl+C to stop the agent.${NC}"
wait "$AGENT_PID"
