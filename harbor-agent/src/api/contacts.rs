use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

use harbor_lib::db::Capability;
use harbor_lib::error::AppError;

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactInfo {
    pub id: i64,
    pub peer_id: String,
    pub display_name: String,
    pub avatar_hash: Option<String>,
    pub bio: Option<String>,
    pub is_blocked: bool,
    pub trust_level: i32,
    pub last_seen_at: Option<i64>,
    pub added_at: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddContactRequest {
    pub peer_id: String,
    pub public_key: Vec<u8>,
    pub x25519_public: Vec<u8>,
    pub display_name: String,
    pub avatar_hash: Option<String>,
    pub bio: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddContactFromStringRequest {
    pub contact_string: String,
}

/// GET /api/contacts
pub async fn get_active_contacts(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ContactInfo>>, ApiError> {
    let contacts = state.contacts_service.get_active_contacts()?;
    Ok(Json(
        contacts
            .into_iter()
            .map(|c| ContactInfo {
                id: c.id,
                peer_id: c.peer_id,
                display_name: c.display_name,
                avatar_hash: c.avatar_hash,
                bio: c.bio,
                is_blocked: c.is_blocked,
                trust_level: c.trust_level,
                last_seen_at: c.last_seen_at,
                added_at: c.added_at,
            })
            .collect(),
    ))
}

/// POST /api/contacts
pub async fn add_contact(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddContactRequest>,
) -> Result<Json<i64>, ApiError> {
    let id = state.contacts_service.add_contact(
        &req.peer_id,
        &req.public_key,
        &req.x25519_public,
        &req.display_name,
        req.avatar_hash.as_deref(),
        req.bio.as_deref(),
    )?;
    Ok(Json(id))
}

/// POST /api/contacts/from-string
pub async fn add_contact_from_string(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddContactFromStringRequest>,
) -> Result<Json<String>, ApiError> {
    use base64::Engine;

    let encoded = req
        .contact_string
        .strip_prefix("harbor://")
        .ok_or_else(|| AppError::Validation("Invalid contact string format".to_string()))?;

    let json_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|e| AppError::Validation(format!("Invalid contact encoding: {}", e)))?;

    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct ContactBundle {
        multiaddr: String,
        display_name: String,
        public_key: String,
        x25519_public: String,
        bio: Option<String>,
        avatar_hash: Option<String>,
    }

    let bundle: ContactBundle = serde_json::from_slice(&json_bytes)
        .map_err(|e| AppError::Validation(format!("Invalid contact data: {}", e)))?;

    let public_key = base64::engine::general_purpose::STANDARD
        .decode(&bundle.public_key)
        .map_err(|e| AppError::Validation(format!("Invalid public key: {}", e)))?;

    let x25519_public = base64::engine::general_purpose::STANDARD
        .decode(&bundle.x25519_public)
        .map_err(|e| AppError::Validation(format!("Invalid x25519 key: {}", e)))?;

    let peer_id = bundle
        .multiaddr
        .split("/p2p/")
        .last()
        .ok_or_else(|| AppError::Validation("No peer ID in multiaddr".to_string()))?
        .to_string();

    state.contacts_service.add_contact(
        &peer_id,
        &public_key,
        &x25519_public,
        &bundle.display_name,
        bundle.avatar_hash.as_deref(),
        bundle.bio.as_deref(),
    )?;

    // Grant default permissions
    let _ = state
        .permissions_service
        .create_permission_grant(&peer_id, Capability::WallRead, None);
    let _ = state
        .permissions_service
        .create_permission_grant(&peer_id, Capability::Chat, None);

    // Try to connect
    let handle = state.network.get_handle().await?;
    let addr: libp2p::Multiaddr = bundle
        .multiaddr
        .parse()
        .map_err(|e| AppError::Validation(format!("Invalid multiaddress: {}", e)))?;

    let _ = handle.add_bootstrap_node(addr).await;

    info!(
        "Added contact {} ({}) from shareable string",
        bundle.display_name, peer_id
    );

    Ok(Json(peer_id))
}

/// DELETE /api/contacts/:peerId
pub async fn remove_contact(
    State(state): State<Arc<AppState>>,
    Path(peer_id): Path<String>,
) -> Result<Json<bool>, ApiError> {
    let removed = state.contacts_service.remove_contact(&peer_id)?;
    Ok(Json(removed))
}

/// POST /api/contacts/:peerId/block
pub async fn block_contact(
    State(state): State<Arc<AppState>>,
    Path(peer_id): Path<String>,
) -> Result<Json<bool>, ApiError> {
    let blocked = state.contacts_service.block_contact(&peer_id)?;
    Ok(Json(blocked))
}
