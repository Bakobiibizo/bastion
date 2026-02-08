# Harbor Fork: P2P Networking Fixes

These are three fixes to the Harbor desktop app's libp2p networking layer that resolve relay circuit instability, connection drops, and messaging crypto failures. All changes are in `src-tauri/src/`.

The Harbor fork shares the same `src-tauri/` library code as Bastion. These fixes have been tested and confirmed working on the Bastion side.

---

## Fix 1: Relay Circuit Reservation Timing

**Problem**: `swarm.listen_on(<relay>/p2p-circuit)` is called immediately after `swarm.dial()` to request a relay reservation. At that point the connection isn't fully negotiated — the relay client transport doesn't know about it yet. The reservation silently fails or returns an empty error.

**Root cause**: The relay v2 protocol requires the connection to be fully established (including the Identify exchange) before a reservation request can succeed.

**File**: `src-tauri/src/p2p/network.rs`

### Step 1: Add a pending reservations field to `NetworkService`

Add this field to the `NetworkService` struct:

```rust
/// Relay peers we've dialed but haven't yet requested a reservation for.
/// Key: relay peer ID, Value: full relay multiaddr (transport + /p2p/<id>).
/// Reservation is requested in Identify::Received after the connection is fully negotiated.
pending_relay_reservations: HashMap<PeerId, Multiaddr>,
```

Initialize it in `NetworkService::new()`:

```rust
pending_relay_reservations: HashMap::new(),
```

### Step 2: In `connect_to_relays()`, replace the `listen_on` call with queueing

Find the section after `self.swarm.dial(relay_addr.clone())` succeeds. Remove any `self.swarm.listen_on(...)` call that constructs a `/p2p-circuit` address. Replace it with:

```rust
// Queue relay reservation for after Identify completes.
// listen_on must be called AFTER the connection is fully negotiated
// (Identify::Received), not immediately after dial — otherwise the
// relay client transport doesn't know about the connection yet.
self.pending_relay_reservations.insert(relay_peer_id, relay_addr.clone());
info!("Relay reservation queued for {} (will request after identify)", relay_peer_id);
```

### Step 3: In the `AddRelayServer` command handler, same change

Find the `NetworkCommand::AddRelayServer` match arm. After the `self.swarm.dial(address.clone())` succeeds, remove any `self.swarm.listen_on(...)` call. Replace with:

```rust
// Queue relay reservation for after Identify completes.
// listen_on must be called AFTER the connection is fully negotiated
// (Identify::Received), not immediately after dial.
self.pending_relay_reservations.insert(relay_peer_id, address.clone());
info!("Relay reservation queued for {} (will request after identify)", relay_peer_id);
```

### Step 4: In the `Identify::Received` event handler, trigger the reservation

Find where `SwarmEvent::Behaviour(ChatBehaviourEvent::Identify(identify::Event::Received { peer_id, info, .. }))` is handled. After the existing address processing (adding to Kademlia, etc.), add:

```rust
// If this peer is a relay we're waiting on, request the reservation NOW.
// This is the correct timing — the connection is fully negotiated and
// the relay client transport knows about it.
if let Some(relay_addr) = self.pending_relay_reservations.remove(&peer_id) {
    let circuit_listen_addr: Multiaddr = relay_addr
        .clone()
        .with(libp2p::multiaddr::Protocol::P2pCircuit);
    info!("Requesting relay reservation on {} (post-identify)", circuit_listen_addr);
    match self.swarm.listen_on(circuit_listen_addr.clone()) {
        Ok(id) => {
            info!("Relay listener registered: {:?} on {}", id, circuit_listen_addr);
        }
        Err(e) => {
            warn!("Failed to request relay reservation {}: {}", circuit_listen_addr, e);
        }
    }
}
```

---

## Fix 2: Disable DCUtR (Direct Connection Upgrade through Relay)

**Problem**: When two peers connect through a relay circuit, DCUtR automatically attempts a hole-punch to establish a direct connection. On containerized/headless/NAT'd hosts, hole-punching always fails because the reported listen addresses are unreachable LAN IPs (172.x, 192.168.x). The failed hole-punch tears down the relay circuit, causing a disconnection. The peers reconnect, DCUtR tries again, fails again — creating a disconnect/reconnect cycle every ~2 minutes.

**Root cause**: DCUtR failure is treated as a transport error that cascades to the relay circuit.

**File**: `src-tauri/src/p2p/behaviour.rs`

### Step 1: Add Toggle import

```rust
use libp2p::{
    // ... existing imports ...
    swarm::{NetworkBehaviour, behaviour::toggle::Toggle},
};
```

### Step 2: Change the dcutr field type

In the `ChatBehaviour` struct:

```rust
/// DCUtR for direct connection upgrade through relay (disabled by default —
/// hole punching fails in most agent topologies and destabilises relay circuits)
pub dcutr: Toggle<dcutr::Behaviour>,
```

### Step 3: Initialize as disabled

In `ChatBehaviour::new()`:

```rust
// DCUtR for hole punching — disabled by default.
// When enabled, failed hole-punch attempts destabilise relay circuits
// causing disconnections every ~2 minutes.
let dcutr = Toggle::from(None::<dcutr::Behaviour>);
```

Remove the old `dcutr::Behaviour::new(local_peer_id)` call.

**Note**: `Toggle<T>` implements `NetworkBehaviour` and silently no-ops when set to `None`. The `#[derive(NetworkBehaviour)]` macro handles it correctly. No other code changes needed — match arms for `ChatBehaviourEvent::Dcutr` will simply never fire.

---

## Fix 3: Double-Base64 Key Decode in Contact Strings

**Problem**: When importing a contact from a `harbor://` contact string, the Ed25519 public key and X25519 public key are decoded only once. But the contact string format double-encodes them: `base64(base64(32 raw bytes))`. After one decode you get a 44-byte base64 string instead of the 32-byte raw key. The messaging service then fails with "Invalid X25519 key" when trying to derive the shared encryption secret.

**Root cause**: `get_shareable_contact_string` stores keys that are already base64-encoded (from the DB), then the entire JSON bundle is base64-encoded for the URI. When parsing, one decode unwraps the URI encoding but the keys are still base64 strings, not raw bytes.

**File**: `src-tauri/src/commands/network.rs` (the `add_contact_from_string` Tauri command)

### Change

Find where the public key and x25519 key are decoded from the `ContactBundle`. Replace the single decode with a double decode + fallback:

```rust
// Decode the keys — contact strings have double-base64-encoded keys:
// base64(base64(32 raw bytes)). Decode both layers, fall back to single decode
// for correctly-encoded keys.
let pk_once = base64::engine::general_purpose::STANDARD
    .decode(&bundle.public_key)
    .map_err(|e| AppError::Validation(format!("Invalid public key: {}", e)))?;
let public_key = base64::engine::general_purpose::STANDARD
    .decode(&pk_once)
    .unwrap_or(pk_once);

let x25519_once = base64::engine::general_purpose::STANDARD
    .decode(&bundle.x25519_public)
    .map_err(|e| AppError::Validation(format!("Invalid x25519 key: {}", e)))?;
let x25519_public = base64::engine::general_purpose::STANDARD
    .decode(&x25519_once)
    .unwrap_or(x25519_once);
```

The `unwrap_or(pk_once)` / `unwrap_or(x25519_once)` fallback handles the case where keys are correctly single-encoded (future-proofing).

**Important**: After applying this fix, any contacts that were previously imported with the wrong key length need to be re-added. The simplest approach: delete and re-import the contact from a fresh contact string. The DB stores raw key bytes, so once correctly imported they stay correct.

---

## Verification

After applying all three fixes:

1. `cargo check` in `src-tauri/` — should compile cleanly
2. Start the app, connect to a relay server
3. Verify relay reservation succeeds (check logs for "Relay listener registered" after "Requesting relay reservation on ... (post-identify)")
4. Verify connection stays stable for >5 minutes (no disconnect/reconnect cycle)
5. Exchange contact strings with another peer and send a message — should encrypt/decrypt correctly

## Files Summary

| File | Fix |
|------|-----|
| `src-tauri/src/p2p/network.rs` | Relay reservation timing (pending_relay_reservations HashMap + Identify::Received trigger) |
| `src-tauri/src/p2p/behaviour.rs` | Disable DCUtR (Toggle\<None\>) |
| `src-tauri/src/commands/network.rs` | Double-base64 key decode in add_contact_from_string |
