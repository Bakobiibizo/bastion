use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use harbor_lib::db::Capability;
use harbor_lib::error::AppError;

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrantPermissionRequest {
    pub peer_id: String,
    pub capability: String,
    pub expires_in_seconds: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrantAllRequest {
    pub peer_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GrantResult {
    pub grant_id: String,
    pub capability: String,
    pub subject_peer_id: String,
    pub issued_at: i64,
    pub expires_at: Option<i64>,
}

fn capability_from_str(s: &str) -> Result<Capability, ApiError> {
    Capability::from_str(s)
        .ok_or_else(|| AppError::Validation(format!("Invalid capability: {}", s)).into())
}

/// POST /api/permissions/grant
pub async fn grant_permission(
    State(state): State<Arc<AppState>>,
    Json(req): Json<GrantPermissionRequest>,
) -> Result<Json<GrantResult>, ApiError> {
    let cap = capability_from_str(&req.capability)?;
    let grant = state.permissions_service.create_permission_grant(
        &req.peer_id,
        cap,
        req.expires_in_seconds,
    )?;

    Ok(Json(GrantResult {
        grant_id: grant.grant_id,
        capability: grant.capability,
        subject_peer_id: grant.subject_peer_id,
        issued_at: grant.issued_at,
        expires_at: grant.expires_at,
    }))
}

/// POST /api/permissions/grant-all
pub async fn grant_all_permissions(
    State(state): State<Arc<AppState>>,
    Json(req): Json<GrantAllRequest>,
) -> Result<Json<Vec<GrantResult>>, ApiError> {
    let mut results = Vec::new();

    for cap in [Capability::Chat, Capability::WallRead, Capability::Call] {
        let grant = state
            .permissions_service
            .create_permission_grant(&req.peer_id, cap, None)?;

        results.push(GrantResult {
            grant_id: grant.grant_id,
            capability: grant.capability,
            subject_peer_id: grant.subject_peer_id.clone(),
            issued_at: grant.issued_at,
            expires_at: grant.expires_at,
        });
    }

    Ok(Json(results))
}

/// DELETE /api/permissions/:grantId
pub async fn revoke_permission(
    State(state): State<Arc<AppState>>,
    Path(grant_id): Path<String>,
) -> Result<Json<bool>, ApiError> {
    state.permissions_service.revoke_permission(&grant_id)?;
    Ok(Json(true))
}

/// GET /api/permissions/chat-peers
pub async fn get_chat_peers(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<String>>, ApiError> {
    let peers = state.permissions_service.get_chat_peers()?;
    Ok(Json(peers))
}
