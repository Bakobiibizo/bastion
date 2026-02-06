use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use harbor_lib::error::AppError;

use crate::captcha_solver;
use crate::error::ApiError;
use crate::state::AppState;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateRequest {
    /// The relay's HTTP auth URL (e.g. "http://52.200.206.197:4002")
    pub auth_url: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateResponse {
    pub token: String,
    pub expires_in_seconds: i64,
    pub peer_id: String,
}

/// POST /api/auth/verify-agent - Authenticate with a relay using Isnad CAPTCHA
pub async fn verify_agent(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AuthenticateRequest>,
) -> Result<Json<AuthenticateResponse>, ApiError> {
    // Need identity to get peer_id
    let identity = state
        .identity_service
        .get_identity_info()?
        .ok_or_else(|| AppError::NotFound("Identity not found. Create one first.".to_string()))?;

    let result = captcha_solver::authenticate_with_relay(&req.auth_url, &identity.peer_id)
        .await
        .map_err(|e| AppError::Network(format!("CAPTCHA auth failed: {}", e)))?;

    Ok(Json(AuthenticateResponse {
        token: result.token,
        expires_in_seconds: result.expires_in_seconds,
        peer_id: result.peer_id,
    }))
}
