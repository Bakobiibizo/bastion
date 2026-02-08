use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

use harbor_lib::error::AppError;
use harbor_lib::p2p::{NetworkConfig, NetworkHandle, NetworkService, NetworkStats, PeerInfo};

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkStatusResponse {
    pub running: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats: Option<NetworkStats>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectRequest {
    pub multiaddr: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayRequest {
    pub multiaddr: String,
}

/// POST /api/network/start
pub async fn start_network(
    State(state): State<Arc<AppState>>,
) -> Result<Json<()>, ApiError> {
    // Check if identity is unlocked
    if !state.identity_service.is_unlocked() {
        return Err(AppError::PermissionDenied(
            "Identity must be unlocked to start network".to_string(),
        )
        .into());
    }

    // Check if network is already running
    if state.network.is_running().await {
        return Ok(Json(()));
    }

    // Get the unlocked keys to create a libp2p keypair
    let unlocked_keys = state.identity_service.get_unlocked_keys()?;
    let ed25519_bytes = unlocked_keys.ed25519_signing.to_bytes();

    // Convert to libp2p keypair
    let keypair = harbor_lib::p2p::swarm::ed25519_to_libp2p_keypair(&ed25519_bytes)?;
    let network_peer_id = libp2p::PeerId::from(keypair.public());

    // Verify peer ID matches stored identity
    if let Ok(Some(identity_info)) = state.identity_service.get_identity_info() {
        info!(
            "PEER ID CHECK - Stored: {} vs Network: {}",
            identity_info.peer_id, network_peer_id
        );
        if identity_info.peer_id != network_peer_id.to_string() {
            tracing::error!("PEER ID MISMATCH! Stored peer ID does not match network peer ID.");
        }
    }

    // Create network service
    let config = NetworkConfig::default();
    let identity_arc = state.identity_service.clone();
    let (mut service, handle, mut event_rx) =
        NetworkService::new(config, identity_arc, keypair)?;

    // Inject services
    service.set_messaging_service(state.messaging_service.clone());
    service.set_contacts_service(state.contacts_service.clone());
    service.set_permissions_service(state.permissions_service.clone());
    service.set_posts_service(state.posts_service.clone());
    service.set_content_sync_service(state.content_sync_service.clone());
    service.set_board_service(state.board_service.clone());

    // Store the handle
    state.network.set_handle(handle).await;

    // Spawn the network service
    tokio::spawn(async move {
        info!("Network service starting in background task");
        service.run().await;
        info!("Network service stopped");
    });

    // Spawn event forwarding to SSE broadcast channel
    let event_tx = state.event_tx.clone();
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            info!("Network event: {:?}", event);
            if let Ok(value) = serde_json::to_value(&event) {
                let _ = event_tx.send(value);
            }
        }
    });

    info!("Network started successfully");
    Ok(Json(()))
}

/// POST /api/network/stop
pub async fn stop_network(
    State(state): State<Arc<AppState>>,
) -> Result<Json<()>, ApiError> {
    let maybe_handle: Option<NetworkHandle> = {
        let mut guard = state.network.handle.write().await;
        guard.take()
    };

    if let Some(handle) = maybe_handle {
        handle.shutdown().await?;
        info!("Network stopped");
    }

    Ok(Json(()))
}

/// GET /api/network/status
pub async fn get_network_status(
    State(state): State<Arc<AppState>>,
) -> Result<Json<NetworkStatusResponse>, ApiError> {
    let running = state.network.is_running().await;
    let stats = if running {
        let handle = state.network.get_handle().await?;
        Some(handle.get_stats().await?)
    } else {
        None
    };
    Ok(Json(NetworkStatusResponse { running, stats }))
}

/// GET /api/network/peers
pub async fn get_connected_peers(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<PeerInfo>>, ApiError> {
    let handle = state.network.get_handle().await?;
    let peers = handle.get_connected_peers().await?;
    Ok(Json(peers))
}

/// POST /api/network/connect
pub async fn connect_to_peer(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ConnectRequest>,
) -> Result<Json<()>, ApiError> {
    let handle = state.network.get_handle().await?;

    let addr: libp2p::Multiaddr = req
        .multiaddr
        .parse()
        .map_err(|e| AppError::Validation(format!("Invalid multiaddress: {}", e)))?;

    handle.add_bootstrap_node(addr).await?;
    Ok(Json(()))
}

/// POST /api/network/relay
pub async fn add_relay_server(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RelayRequest>,
) -> Result<Json<()>, ApiError> {
    let handle = state.network.get_handle().await?;

    let addr: libp2p::Multiaddr = req
        .multiaddr
        .parse()
        .map_err(|e| AppError::Validation(format!("Invalid multiaddress: {}", e)))?;

    handle.add_relay_server(addr).await?;
    Ok(Json(()))
}

/// POST /api/network/relays/public
pub async fn connect_to_public_relays(
    State(state): State<Arc<AppState>>,
) -> Result<Json<()>, ApiError> {
    let handle = state.network.get_handle().await?;
    handle.connect_to_public_relays().await?;
    Ok(Json(()))
}

/// GET /api/network/addresses
pub async fn get_listening_addresses(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<String>>, ApiError> {
    let handle = state.network.get_handle().await?;
    let addrs = handle.get_listening_addresses().await?;
    Ok(Json(addrs))
}

/// GET /api/network/shareable
pub async fn get_shareable_addresses(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<String>>, ApiError> {
    let handle = state.network.get_handle().await?;
    let stats = handle.get_stats().await?;

    let peer_id = if let Ok(Some(identity)) = state.identity_service.get_identity_info() {
        identity.peer_id
    } else {
        return Err(AppError::NotFound("Identity not found".to_string()).into());
    };

    let mut addresses = Vec::new();

    for addr in &stats.external_addresses {
        if !addr.contains("127.0.0.1") && !addr.contains("::1") {
            if addr.contains("/p2p/") {
                addresses.push(addr.clone());
            } else {
                addresses.push(format!("{}/p2p/{}", addr, peer_id));
            }
        }
    }

    if addresses.is_empty() {
        for addr in &stats.relay_addresses {
            addresses.push(addr.clone());
        }
    }

    Ok(Json(addresses))
}

/// GET /api/network/contact-string
pub async fn get_shareable_contact_string(
    State(state): State<Arc<AppState>>,
) -> Result<Json<String>, ApiError> {
    use base64::Engine;

    let handle = state.network.get_handle().await?;
    let stats = handle.get_stats().await?;

    let identity = state
        .identity_service
        .get_identity()?
        .ok_or_else(|| AppError::NotFound("Identity not found".to_string()))?;

    let keys = state
        .identity_service
        .get_identity_info()?
        .ok_or_else(|| AppError::NotFound("Identity keys not found".to_string()))?;

    let multiaddr = if !stats.relay_addresses.is_empty() {
        stats.relay_addresses[0].clone()
    } else if !stats.external_addresses.is_empty() {
        let addr = &stats.external_addresses[0];
        if addr.contains("/p2p/") {
            addr.clone()
        } else {
            format!("{}/p2p/{}", addr, identity.peer_id)
        }
    } else {
        return Err(AppError::Network(
            "No shareable address available. Please connect to a relay first.".to_string(),
        )
        .into());
    };

    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct ContactBundle {
        multiaddr: String,
        display_name: String,
        public_key: String,
        x25519_public: String,
        bio: Option<String>,
        avatar_hash: Option<String>,
    }

    let bundle = ContactBundle {
        multiaddr,
        display_name: identity.display_name,
        public_key: base64::engine::general_purpose::STANDARD.encode(&keys.public_key),
        x25519_public: base64::engine::general_purpose::STANDARD.encode(&keys.x25519_public),
        bio: identity.bio,
        avatar_hash: identity.avatar_hash,
    };

    let json = serde_json::to_string(&bundle)
        .map_err(|e| AppError::Serialization(format!("Failed to serialize contact: {}", e)))?;

    let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(json.as_bytes());

    Ok(Json(format!("harbor://{}", encoded)))
}
