//! Isnad CAPTCHA auth layer for the AI relay.
//!
//! Flow:
//! 1. Agent POSTs to /auth/challenge with their peer_id
//! 2. Relay generates an Isnad CaptchaChallenge, stores expected answers keyed by challenge_id
//! 3. Agent solves the challenge and POSTs to /auth/verify
//! 4. Relay verifies timing + correctness, issues a token
//! 5. Agent includes this token when registering with the relay via libp2p
//! 6. Relay checks token before granting relay reservation

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use isnad::{CaptchaChallenge, CaptchaResponse, CaptchaVerifier, TaskAnswer};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// How long a pending challenge stays valid (seconds)
const CHALLENGE_TTL_SECS: i64 = 60;
/// How long a verified token stays valid (seconds)
const TOKEN_TTL_SECS: i64 = 3600;

/// Shared auth state
pub struct AuthState {
    /// Pending challenges: challenge_id -> (expected_answers, peer_id, issued_at)
    pending: RwLock<HashMap<Uuid, PendingChallenge>>,
    /// Verified tokens: token -> (peer_id, verified_at)
    verified: RwLock<HashMap<String, VerifiedAgent>>,
    /// The CAPTCHA verifier
    verifier: CaptchaVerifier,
}

struct PendingChallenge {
    expected_answers: Vec<TaskAnswer>,
    challenge_json: serde_json::Value,
    peer_id: String,
    issued_at: chrono::DateTime<Utc>,
}

#[derive(Clone)]
struct VerifiedAgent {
    peer_id: String,
    verified_at: chrono::DateTime<Utc>,
}

impl AuthState {
    pub fn new() -> Self {
        Self {
            pending: RwLock::new(HashMap::new()),
            verified: RwLock::new(HashMap::new()),
            verifier: CaptchaVerifier::new(),
        }
    }

    /// Check if a peer_id has a valid auth token
    pub async fn is_peer_verified(&self, peer_id: &str) -> bool {
        let verified = self.verified.read().await;
        verified.values().any(|v| {
            v.peer_id == peer_id
                && Utc::now()
                    .signed_duration_since(v.verified_at)
                    .num_seconds()
                    < TOKEN_TTL_SECS
        })
    }

    /// Clean up expired challenges and tokens
    pub async fn cleanup(&self) {
        let now = Utc::now();

        {
            let mut pending = self.pending.write().await;
            pending.retain(|_, v| {
                now.signed_duration_since(v.issued_at).num_seconds() < CHALLENGE_TTL_SECS
            });
        }

        {
            let mut verified = self.verified.write().await;
            verified.retain(|_, v| {
                now.signed_duration_since(v.verified_at).num_seconds() < TOKEN_TTL_SECS
            });
        }
    }
}

// -- Request/Response types --

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeRequest {
    pub peer_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeResponse {
    pub challenge: serde_json::Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyRequest {
    pub peer_id: String,
    pub response: CaptchaResponse,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyResponse {
    pub token: String,
    pub expires_in_seconds: i64,
    pub peer_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthError {
    pub error: String,
}

// -- Handlers --

/// POST /auth/challenge - Request a CAPTCHA challenge
pub async fn request_challenge(
    State(auth): State<Arc<AuthState>>,
    Json(req): Json<ChallengeRequest>,
) -> impl IntoResponse {
    // Clean up expired entries
    auth.cleanup().await;

    // Generate challenge
    let (challenge, expected_answers) = auth.verifier.generate_challenge();

    let challenge_json = serde_json::to_value(&challenge).unwrap();
    let challenge_id = challenge.challenge_id;

    // Store pending challenge
    {
        let mut pending = auth.pending.write().await;
        pending.insert(
            challenge_id,
            PendingChallenge {
                expected_answers,
                challenge_json: challenge_json.clone(),
                peer_id: req.peer_id,
                issued_at: Utc::now(),
            },
        );
    }

    tracing::info!("Issued CAPTCHA challenge {} for peer", challenge_id);

    (StatusCode::OK, Json(ChallengeResponse { challenge: challenge_json }))
}

/// POST /auth/verify - Submit CAPTCHA response and get a token
pub async fn verify_challenge(
    State(auth): State<Arc<AuthState>>,
    Json(req): Json<VerifyRequest>,
) -> Result<Json<VerifyResponse>, (StatusCode, Json<AuthError>)> {
    let challenge_id = req.response.challenge_id;

    // Look up the pending challenge
    let pending_challenge = {
        let mut pending = auth.pending.write().await;
        pending.remove(&challenge_id)
    };

    let pending = match pending_challenge {
        Some(p) => p,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(AuthError {
                    error: "Challenge not found or expired".to_string(),
                }),
            ));
        }
    };

    // Verify peer_id matches
    if pending.peer_id != req.peer_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(AuthError {
                error: "Peer ID mismatch".to_string(),
            }),
        ));
    }

    // Reconstruct the challenge from stored JSON
    let challenge: CaptchaChallenge =
        serde_json::from_value(pending.challenge_json).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthError {
                    error: format!("Internal error: {}", e),
                }),
            )
        })?;

    // Verify the response
    match auth
        .verifier
        .verify(&challenge, &req.response, &pending.expected_answers)
    {
        Ok(verification) => {
            tracing::info!(
                "CAPTCHA verified for peer {} in {}ms ({}/{} correct)",
                req.peer_id,
                verification.elapsed_ms,
                verification.tasks_correct,
                verification.tasks_total
            );

            // Generate token
            let token = generate_token(&req.peer_id);

            // Store verified agent
            {
                let mut verified = auth.verified.write().await;
                verified.insert(
                    token.clone(),
                    VerifiedAgent {
                        peer_id: req.peer_id.clone(),
                        verified_at: Utc::now(),
                    },
                );
            }

            Ok(Json(VerifyResponse {
                token,
                expires_in_seconds: TOKEN_TTL_SECS,
                peer_id: req.peer_id,
            }))
        }
        Err(e) => {
            tracing::warn!("CAPTCHA verification failed for peer {}: {}", req.peer_id, e);
            Err((
                StatusCode::FORBIDDEN,
                Json(AuthError {
                    error: format!("Verification failed: {}", e),
                }),
            ))
        }
    }
}

/// POST /auth/check - Check if a token is still valid
pub async fn check_token(
    State(auth): State<Arc<AuthState>>,
    Json(req): Json<CheckTokenRequest>,
) -> Result<Json<CheckTokenResponse>, (StatusCode, Json<AuthError>)> {
    let verified = auth.verified.read().await;
    match verified.get(&req.token) {
        Some(agent) => {
            let elapsed = Utc::now()
                .signed_duration_since(agent.verified_at)
                .num_seconds();
            if elapsed < TOKEN_TTL_SECS {
                Ok(Json(CheckTokenResponse {
                    valid: true,
                    peer_id: Some(agent.peer_id.clone()),
                    remaining_seconds: TOKEN_TTL_SECS - elapsed,
                }))
            } else {
                Ok(Json(CheckTokenResponse {
                    valid: false,
                    peer_id: None,
                    remaining_seconds: 0,
                }))
            }
        }
        None => Ok(Json(CheckTokenResponse {
            valid: false,
            peer_id: None,
            remaining_seconds: 0,
        })),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckTokenRequest {
    pub token: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckTokenResponse {
    pub valid: bool,
    pub peer_id: Option<String>,
    pub remaining_seconds: i64,
}

fn generate_token(peer_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(peer_id.as_bytes());
    hasher.update(Uuid::new_v4().as_bytes());
    hasher.update(Utc::now().timestamp().to_le_bytes());
    let hash = hasher.finalize();
    format!(
        "isnad_{}",
        hash.iter().map(|b| format!("{:02x}", b)).collect::<String>()
    )
}
