# Harbor

A decentralized peer-to-peer chat application with local-first data storage, end-to-end encryption, and permission-based content sharing.

## Features

- **Decentralized Identity**: Ed25519 keypairs for signing, X25519 for key agreement
- **Local-First**: All data stored locally in SQLite, you own your data
- **P2P Networking**: Direct peer connections via libp2p (mDNS, Kademlia DHT, NAT traversal)
- **End-to-End Encryption**: AES-256-GCM with HKDF-derived conversation keys
- **Permission System**: Signed capability grants for content access (Chat, WallRead, Call)
- **Event Sourcing**: Append-only logs with lamport clocks for conflict-free sync
- **Voice Calling**: WebRTC signaling through libp2p (best-effort, works on LAN/most NATs)

## Quick Start

### Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://rustup.rs/) (stable)
- [Tauri Prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites)
  - Windows: Microsoft Visual Studio C++ Build Tools
  - macOS: Xcode Command Line Tools
  - Linux: `sudo apt install libwebkit2gtk-4.0-dev build-essential curl wget libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev`

### Installation

```bash
# Clone the repository
git clone https://github.com/Bakobiibizo/harbor.git
cd harbor

# Install frontend dependencies
npm install

# Run in development mode
npm run tauri dev
```

### Building for Production

```bash
# Build the application
npm run tauri build

# The executable will be in src-tauri/target/release/
```

## Usage Guide

### First Launch - Create Your Identity

1. When you first open Harbor, you'll be prompted to create an identity
2. Enter a **Display Name** (how others will see you)
3. Optionally add a **Bio**
4. Create a **Passphrase** (at least 8 characters) - this encrypts your private keys
5. **Important**: Store your passphrase safely! If you lose it, you cannot recover your identity

### Unlocking Your Identity

- On subsequent launches, enter your passphrase to unlock
- Your identity remains encrypted on disk when locked

### Starting the Network

1. Go to the **Network** tab
2. Click **Start Network** to connect to the P2P network
3. Peers on your local network running Harbor will be discovered automatically via mDNS
4. The status indicator shows your connection state

### Managing Contacts

1. In the **Network** tab, you'll see discovered peers
2. Click the checkmark to add a peer as a contact
3. You can search for peers by their Peer ID
4. Use the **Contacts** tab to manage your contact list

### Direct Messaging

1. Go to the **Messages** tab
2. Select a contact to open a conversation
3. Messages are end-to-end encrypted using derived conversation keys
4. Click the phone icon to initiate a voice call (if supported)

### Posting to Your Wall

1. Go to the **My Wall** tab
2. Use the composer at the top to create a post
3. You can add images and videos to your posts
4. Posts are stored locally and shared with contacts who have permission

### Viewing Your Feed

1. Go to the **Feed** tab
2. See posts from contacts who have granted you WallRead permission
3. Like and comment on posts (when implemented)

### Settings

Access settings to:
- **Profile**: Update your display name, bio, and avatar
- **Security**: Change passphrase, export/import identity
- **Network**: Configure auto-start and mDNS discovery
- **Privacy**: Control post visibility and read receipts

## Architecture

### Frontend (React + TypeScript)

```
src/
├── components/
│   ├── common/          # Button, Input, etc.
│   ├── icons/           # SVG icon components
│   ├── layout/          # MainLayout with sidebar
│   └── onboarding/      # CreateIdentity, UnlockIdentity
├── pages/
│   ├── Chat.tsx         # Direct messaging
│   ├── Wall.tsx         # Your posts
│   ├── Feed.tsx         # Posts from contacts
│   ├── Network.tsx      # Peer discovery & contacts
│   └── Settings.tsx     # App configuration
├── services/            # Tauri command wrappers
│   ├── identity.ts
│   ├── network.ts
│   ├── contacts.ts
│   ├── permissions.ts
│   ├── messaging.ts
│   ├── posts.ts
│   ├── feed.ts
│   └── calling.ts
├── stores/              # Zustand state management
│   ├── identity.ts
│   └── network.ts
├── types/               # TypeScript interfaces
└── styles/
    └── design-system.css  # CSS custom properties
```

### Backend (Rust + Tauri)

```
src-tauri/src/
├── commands/            # Tauri command handlers
│   ├── identity.rs
│   ├── network.rs
│   ├── contacts.rs
│   ├── permissions.rs
│   ├── messaging.rs
│   ├── posts.rs
│   ├── feed.rs
│   └── calling.rs
├── services/            # Business logic
│   ├── identity_service.rs    # Key management
│   ├── crypto_service.rs      # Encryption/signing
│   ├── contacts_service.rs    # Contact management
│   ├── permissions_service.rs # Capability grants
│   ├── messaging_service.rs   # Direct messages
│   ├── posts_service.rs       # Wall posts
│   ├── feed_service.rs        # Feed aggregation
│   ├── content_sync_service.rs # P2P sync
│   └── calling_service.rs     # Voice calls
├── db/
│   ├── mod.rs           # Database initialization
│   ├── migrations/      # SQL migrations
│   └── repositories/    # Data access layer
├── models/              # Data structures
└── p2p/
    ├── network.rs       # libp2p swarm
    └── protocols/       # Request-response protocols
```

### Database Schema (SQLite)

- `local_identity` - Your encrypted keypairs and profile
- `contacts` - Peer information and trust levels
- `permission_events` - Grant/revoke events (event sourced)
- `permissions_current` - Materialized permission state
- `message_events` - Message lifecycle events
- `messages` - Materialized messages for UI
- `post_events` - Post lifecycle events
- `posts` - Materialized posts
- `post_media` - Media metadata (files stored on disk)
- `call_history` - Voice call records
- `sync_state` - Per-peer sync progress
- `sync_queue` - Offline message queue
- `lamport_clock` - Logical clock for ordering

## Security Model

### Cryptography

| Purpose | Algorithm | Notes |
|---------|-----------|-------|
| Identity signing | Ed25519 | All messages signed |
| Key agreement | X25519 | Derived from Ed25519 |
| Conversation encryption | AES-256-GCM | HKDF-derived keys |
| Key encryption | Argon2id + AES-GCM | Passphrase-based |
| Content hashing | SHA-256 | Media content-addressing |

### Permission System

Permissions are signed, portable capability grants:

```rust
struct PermissionGrant {
    grant_id: Uuid,
    issuer_peer_id: PeerId,      // Who grants
    subject_peer_id: PeerId,     // Who receives
    capability: Capability,       // Chat, WallRead, Call
    issued_at: u64,
    expires_at: Option<u64>,
    signature: Vec<u8>,          // Ed25519 signature
}
```

### Protected Against
- MITM attacks (Noise protocol transport + E2E encryption)
- Message spoofing (all content signed with Ed25519)
- Replay attacks (nonce tracking, lamport clocks, message IDs)
- Unauthorized access (permission grants verified on every request)

### Known Limitations (MVP)
- No forward secrecy (no double-ratchet yet - compromise exposes history)
- No HSM/secure enclave integration
- Connection patterns visible (metadata leakage)
- Voice calls may not work behind strict NATs (no TURN server)

## Protocol Messages (CBOR)

### Identity Exchange
- `IdentityRequest` / `IdentityResponse` - Exchange peer info

### Permissions
- `PermissionRequest` - Request capability from peer
- `PermissionGrant` - Grant capability to peer
- `PermissionRevoke` - Revoke previously granted capability

### Messaging
- `DirectMessage` - Encrypted message with signature
- `MessageAck` - Delivery/read receipt

### Content Sync
- `ContentManifestRequest/Response` - List available posts
- `ContentFetchRequest` - Request specific post
- `MediaChunkRequest/Response` - Transfer media files

### Voice Calling
- `SignalingOffer/Answer` - WebRTC SDP exchange
- `SignalingIce` - ICE candidate exchange
- `SignalingHangup` - End call

## Development

### Running Tests

```bash
# Rust tests
cd src-tauri && cargo test

# TypeScript type check
npm run typecheck
```

### Code Structure

The codebase follows these patterns:
- **Event Sourcing**: All state changes are events with lamport clocks
- **CQRS**: Events stored separately from materialized views
- **Repository Pattern**: Data access abstracted behind repositories
- **Service Layer**: Business logic in services, commands are thin wrappers

## Roadmap

### Completed (Phases 1-8)
- [x] Identity system with encrypted key storage
- [x] P2P networking with libp2p
- [x] Contact management
- [x] Permission grants/revokes
- [x] Direct messaging (encrypted)
- [x] Wall/blog posts with media
- [x] Feed aggregation
- [x] Voice calling (signaling)
- [x] Modern, polished UI

### Future (Stretch Goals)
- [ ] Double-ratchet for forward secrecy
- [ ] Video calling + screen sharing
- [ ] Group chats
- [ ] Mobile app (iOS/Android via Tauri)
- [ ] TURN server for better NAT traversal
- [ ] Profile photo uploads
- [ ] Read receipts
- [ ] Typing indicators

## Contributing

Contributions are welcome! Please open an issue or PR.

## License

MIT License - see [LICENSE](LICENSE)
