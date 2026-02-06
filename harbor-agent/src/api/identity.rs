use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use harbor_lib::models::{CreateIdentityRequest, IdentityInfo};

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentityStatusResponse {
    pub has_identity: bool,
    pub is_unlocked: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockRequest {
    pub passphrase: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDisplayNameRequest {
    pub display_name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBioRequest {
    pub bio: Option<String>,
}

/// GET /api/identity
pub async fn get_identity(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Option<IdentityInfo>>, ApiError> {
    let info = state.identity_service.get_identity_info()?;
    Ok(Json(info))
}

/// GET /api/identity/status
pub async fn get_identity_status(
    State(state): State<Arc<AppState>>,
) -> Result<Json<IdentityStatusResponse>, ApiError> {
    let has_identity = state.identity_service.has_identity()?;
    let is_unlocked = state.identity_service.is_unlocked();
    Ok(Json(IdentityStatusResponse {
        has_identity,
        is_unlocked,
    }))
}

/// POST /api/identity
pub async fn create_identity(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateIdentityRequest>,
) -> Result<Json<IdentityInfo>, ApiError> {
    let display_name = request.display_name.clone();
    let bio = request.bio.clone();

    let identity = state.identity_service.create_identity(request)?;

    // Register in accounts registry
    let _ = state.accounts_service.register_account(
        identity.peer_id.clone(),
        display_name,
        bio,
        identity.avatar_hash.clone(),
    );

    Ok(Json(identity))
}

/// POST /api/identity/unlock
pub async fn unlock_identity(
    State(state): State<Arc<AppState>>,
    Json(req): Json<UnlockRequest>,
) -> Result<Json<IdentityInfo>, ApiError> {
    let info = state.identity_service.unlock(&req.passphrase)?;
    Ok(Json(info))
}

/// POST /api/identity/lock
pub async fn lock_identity(
    State(state): State<Arc<AppState>>,
) -> Result<Json<()>, ApiError> {
    state.identity_service.lock();
    Ok(Json(()))
}

/// PUT /api/identity/display-name
pub async fn update_display_name(
    State(state): State<Arc<AppState>>,
    Json(req): Json<UpdateDisplayNameRequest>,
) -> Result<Json<()>, ApiError> {
    state.identity_service.update_display_name(&req.display_name)?;
    Ok(Json(()))
}

/// PUT /api/identity/bio
pub async fn update_bio(
    State(state): State<Arc<AppState>>,
    Json(req): Json<UpdateBioRequest>,
) -> Result<Json<()>, ApiError> {
    state.identity_service.update_bio(req.bio.as_deref())?;
    Ok(Json(()))
}
