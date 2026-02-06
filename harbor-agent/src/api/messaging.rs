use axum::extract::{Path, Query, State};
use axum::Json;
use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use tracing::info;

use harbor_lib::error::AppError;
use harbor_lib::p2p::protocols::messaging::{DirectMessage, MessagingCodec, MessagingMessage};
use harbor_lib::services::{DecryptedMessage, OutgoingMessage};

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageRequest {
    pub peer_id: String,
    pub content: String,
    pub content_type: Option<String>,
    pub reply_to: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageResult {
    pub message_id: String,
    pub conversation_id: String,
    pub sent_at: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageInfo {
    pub message_id: String,
    pub conversation_id: String,
    pub sender_peer_id: String,
    pub recipient_peer_id: String,
    pub content: String,
    pub content_type: String,
    pub reply_to_message_id: Option<String>,
    pub sent_at: i64,
    pub delivered_at: Option<i64>,
    pub read_at: Option<i64>,
    pub status: String,
    pub is_outgoing: bool,
}

impl From<DecryptedMessage> for MessageInfo {
    fn from(msg: DecryptedMessage) -> Self {
        Self {
            message_id: msg.message_id,
            conversation_id: msg.conversation_id,
            sender_peer_id: msg.sender_peer_id,
            recipient_peer_id: msg.recipient_peer_id,
            content: msg.content,
            content_type: msg.content_type,
            reply_to_message_id: msg.reply_to_message_id,
            sent_at: msg.sent_at,
            delivered_at: msg.delivered_at,
            read_at: msg.read_at,
            status: msg.status,
            is_outgoing: msg.is_outgoing,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationInfo {
    pub conversation_id: String,
    pub peer_id: String,
    pub last_message_at: i64,
    pub unread_count: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessagesQuery {
    pub limit: Option<i64>,
    pub before: Option<i64>,
}

fn outgoing_to_direct_message(outgoing: &OutgoingMessage) -> DirectMessage {
    DirectMessage {
        message_id: outgoing.message_id.clone(),
        conversation_id: outgoing.conversation_id.clone(),
        sender_peer_id: outgoing.sender_peer_id.clone(),
        recipient_peer_id: outgoing.recipient_peer_id.clone(),
        content_encrypted: outgoing.content_encrypted.clone(),
        content_type: outgoing.content_type.clone(),
        reply_to: outgoing.reply_to.clone(),
        nonce_counter: outgoing.nonce_counter,
        lamport_clock: outgoing.lamport_clock,
        timestamp: outgoing.timestamp,
        signature: outgoing.signature.clone(),
    }
}

/// POST /api/messages/send
pub async fn send_message(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SendMessageRequest>,
) -> Result<Json<SendMessageResult>, ApiError> {
    let content_type = body.content_type.unwrap_or_else(|| "text".to_string());

    let outgoing = state.messaging_service.send_message(
        &body.peer_id,
        &body.content,
        &content_type,
        body.reply_to.as_deref(),
    )?;

    let direct_msg = outgoing_to_direct_message(&outgoing);
    let msg_wrapper = MessagingMessage::Message(direct_msg);
    let payload = MessagingCodec::encode(&msg_wrapper)
        .map_err(|e| AppError::Internal(format!("Failed to encode message: {}", e)))?;

    let libp2p_peer_id = PeerId::from_str(&body.peer_id)
        .map_err(|e| AppError::Validation(format!("Invalid peer ID: {}", e)))?;

    let handle = state.network.get_handle().await?;
    handle
        .send_message(libp2p_peer_id, "message".to_string(), payload)
        .await?;

    info!(
        "Message {} sent to peer {}",
        outgoing.message_id, body.peer_id
    );

    Ok(Json(SendMessageResult {
        message_id: outgoing.message_id,
        conversation_id: outgoing.conversation_id,
        sent_at: outgoing.timestamp,
    }))
}

/// GET /api/messages/:peerId
pub async fn get_messages(
    State(state): State<Arc<AppState>>,
    Path(peer_id): Path<String>,
    Query(query): Query<MessagesQuery>,
) -> Result<Json<Vec<MessageInfo>>, ApiError> {
    let limit = query.limit.unwrap_or(50);
    let messages =
        state
            .messaging_service
            .get_conversation_messages(&peer_id, limit, query.before)?;
    Ok(Json(messages.into_iter().map(MessageInfo::from).collect()))
}

/// GET /api/conversations
pub async fn get_conversations(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ConversationInfo>>, ApiError> {
    let conversations = state.messaging_service.get_conversations()?;
    Ok(Json(
        conversations
            .into_iter()
            .map(|c| ConversationInfo {
                conversation_id: c.conversation_id,
                peer_id: c.peer_id,
                last_message_at: c.last_message_at,
                unread_count: c.unread_count,
            })
            .collect(),
    ))
}

/// POST /api/conversations/:peerId/read
pub async fn mark_conversation_read(
    State(state): State<Arc<AppState>>,
    Path(peer_id): Path<String>,
) -> Result<Json<i64>, ApiError> {
    let count = state.messaging_service.mark_conversation_read(&peer_id)?;
    Ok(Json(count))
}

/// GET /api/messages/unread
pub async fn get_total_unread_count(
    State(state): State<Arc<AppState>>,
) -> Result<Json<i64>, ApiError> {
    let conversations = state.messaging_service.get_conversations()?;
    let total: i64 = conversations.iter().map(|c| c.unread_count).sum();
    Ok(Json(total))
}
