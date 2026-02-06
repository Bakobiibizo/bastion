use axum::extract::{Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use harbor_lib::error::AppError;

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommunityInfo {
    pub relay_peer_id: String,
    pub relay_address: String,
    pub community_name: Option<String>,
    pub joined_at: i64,
    pub last_sync_at: Option<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardInfo {
    pub board_id: String,
    pub relay_peer_id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_default: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardPostInfo {
    pub post_id: String,
    pub board_id: String,
    pub relay_peer_id: String,
    pub author_peer_id: String,
    pub author_display_name: Option<String>,
    pub content_type: String,
    pub content_text: Option<String>,
    pub lamport_clock: i64,
    pub created_at: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinCommunityRequest {
    pub relay_address: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitPostRequest {
    pub content_text: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardPostsQuery {
    pub limit: Option<i64>,
    pub before: Option<i64>,
}

/// GET /api/communities
pub async fn get_communities(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<CommunityInfo>>, ApiError> {
    let communities = state.board_service.get_communities()?;
    Ok(Json(
        communities
            .into_iter()
            .map(|c| CommunityInfo {
                relay_peer_id: c.relay_peer_id,
                relay_address: c.relay_address,
                community_name: c.community_name,
                joined_at: c.joined_at,
                last_sync_at: c.last_sync_at,
            })
            .collect(),
    ))
}

/// POST /api/communities/join
pub async fn join_community(
    State(state): State<Arc<AppState>>,
    Json(req): Json<JoinCommunityRequest>,
) -> Result<Json<()>, ApiError> {
    let handle = state.network.get_handle().await?;

    let addr: libp2p::Multiaddr = req
        .relay_address
        .parse()
        .map_err(|e| AppError::Network(format!("Invalid address: {}", e)))?;

    let relay_peer_id = addr
        .iter()
        .find_map(|proto| {
            if let libp2p::multiaddr::Protocol::P2p(peer_id) = proto {
                Some(peer_id)
            } else {
                None
            }
        })
        .ok_or_else(|| {
            AppError::Network("Address must contain peer ID (/p2p/...)".to_string())
        })?;

    // Dial the relay first
    handle.dial(relay_peer_id, vec![addr.clone()]).await.ok();

    // Join the community
    handle
        .join_community(relay_peer_id, req.relay_address)
        .await?;

    Ok(Json(()))
}

/// DELETE /api/communities/:relayPeerId
pub async fn leave_community(
    State(state): State<Arc<AppState>>,
    Path(relay_peer_id): Path<String>,
) -> Result<Json<()>, ApiError> {
    state.board_service.leave_community(&relay_peer_id)?;
    Ok(Json(()))
}

/// GET /api/boards/:relayPeerId
pub async fn get_boards(
    State(state): State<Arc<AppState>>,
    Path(relay_peer_id): Path<String>,
) -> Result<Json<Vec<BoardInfo>>, ApiError> {
    let boards = state.board_service.get_boards(&relay_peer_id)?;
    Ok(Json(
        boards
            .into_iter()
            .map(|b| BoardInfo {
                board_id: b.board_id,
                relay_peer_id: b.relay_peer_id,
                name: b.name,
                description: b.description,
                is_default: b.is_default,
            })
            .collect(),
    ))
}

/// GET /api/boards/:relayPeerId/:boardId/posts
pub async fn get_board_posts(
    State(state): State<Arc<AppState>>,
    Path((relay_peer_id, board_id)): Path<(String, String)>,
    Query(query): Query<BoardPostsQuery>,
) -> Result<Json<Vec<BoardPostInfo>>, ApiError> {
    let limit = query.limit.unwrap_or(50);
    let posts =
        state
            .board_service
            .get_board_posts(&relay_peer_id, &board_id, limit, query.before)?;
    Ok(Json(
        posts
            .into_iter()
            .map(|p| BoardPostInfo {
                post_id: p.post_id,
                board_id: p.board_id,
                relay_peer_id: p.relay_peer_id,
                author_peer_id: p.author_peer_id,
                author_display_name: p.author_display_name,
                content_type: p.content_type,
                content_text: p.content_text,
                lamport_clock: p.lamport_clock,
                created_at: p.created_at,
            })
            .collect(),
    ))
}

/// POST /api/boards/:relayPeerId/:boardId/posts
pub async fn submit_board_post(
    State(state): State<Arc<AppState>>,
    Path((relay_peer_id, board_id)): Path<(String, String)>,
    Json(req): Json<SubmitPostRequest>,
) -> Result<Json<()>, ApiError> {
    let handle = state.network.get_handle().await?;

    let peer_id: libp2p::PeerId = relay_peer_id
        .parse()
        .map_err(|e| AppError::Network(format!("Invalid peer ID: {}", e)))?;

    handle
        .submit_board_post(peer_id, board_id, req.content_text)
        .await?;

    Ok(Json(()))
}

/// DELETE /api/boards/posts/:postId
pub async fn delete_board_post(
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<String>,
    Json(body): Json<DeleteBoardPostRequest>,
) -> Result<Json<()>, ApiError> {
    let handle = state.network.get_handle().await?;

    let peer_id: libp2p::PeerId = body
        .relay_peer_id
        .parse()
        .map_err(|e| AppError::Network(format!("Invalid peer ID: {}", e)))?;

    handle.delete_board_post(peer_id, post_id).await?;
    Ok(Json(()))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteBoardPostRequest {
    pub relay_peer_id: String,
}

/// POST /api/boards/:relayPeerId/:boardId/sync
pub async fn sync_board(
    State(state): State<Arc<AppState>>,
    Path((relay_peer_id, board_id)): Path<(String, String)>,
) -> Result<Json<()>, ApiError> {
    let handle = state.network.get_handle().await?;

    let peer_id: libp2p::PeerId = relay_peer_id
        .parse()
        .map_err(|e| AppError::Network(format!("Invalid peer ID: {}", e)))?;

    handle.get_board_posts(peer_id, board_id, None, 50).await?;
    Ok(Json(()))
}
