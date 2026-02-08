pub mod auth;
pub mod boards;
pub mod contacts;
pub mod events;
pub mod identity;
pub mod messaging;
pub mod network;
pub mod permissions;

use axum::routing::{delete, get, post, put};
use axum::Router;
use std::sync::Arc;

use crate::state::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        // Identity
        .route("/api/identity", get(identity::get_identity))
        .route("/api/identity", post(identity::create_identity))
        .route("/api/identity/status", get(identity::get_identity_status))
        .route("/api/identity/unlock", post(identity::unlock_identity))
        .route("/api/identity/lock", post(identity::lock_identity))
        .route(
            "/api/identity/display-name",
            put(identity::update_display_name),
        )
        .route("/api/identity/bio", put(identity::update_bio))
        // Network
        .route("/api/network/start", post(network::start_network))
        .route("/api/network/stop", post(network::stop_network))
        .route("/api/network/status", get(network::get_network_status))
        .route("/api/network/peers", get(network::get_connected_peers))
        .route("/api/network/connect", post(network::connect_to_peer))
        .route("/api/network/relay", post(network::add_relay_server))
        .route(
            "/api/network/relays/public",
            post(network::connect_to_public_relays),
        )
        .route(
            "/api/network/addresses",
            get(network::get_listening_addresses),
        )
        .route(
            "/api/network/shareable",
            get(network::get_shareable_addresses),
        )
        .route(
            "/api/network/contact-string",
            get(network::get_shareable_contact_string),
        )
        // Messaging
        .route("/api/messages/send", post(messaging::send_message))
        .route("/api/messages/unread", get(messaging::get_total_unread_count))
        .route("/api/messages/:peerId", get(messaging::get_messages))
        .route("/api/conversations", get(messaging::get_conversations))
        .route(
            "/api/conversations/:peerId/read",
            post(messaging::mark_conversation_read),
        )
        // Contacts
        .route("/api/contacts", get(contacts::get_active_contacts))
        .route("/api/contacts", post(contacts::add_contact))
        .route(
            "/api/contacts/from-string",
            post(contacts::add_contact_from_string),
        )
        .route("/api/contacts/:peerId", delete(contacts::remove_contact))
        .route(
            "/api/contacts/:peerId/block",
            post(contacts::block_contact),
        )
        // Permissions
        .route("/api/permissions/grant", post(permissions::grant_permission))
        .route(
            "/api/permissions/grant-all",
            post(permissions::grant_all_permissions),
        )
        .route(
            "/api/permissions/:grantId",
            delete(permissions::revoke_permission),
        )
        .route(
            "/api/permissions/chat-peers",
            get(permissions::get_chat_peers),
        )
        // Boards / Communities
        .route("/api/communities", get(boards::get_communities))
        .route("/api/communities/join", post(boards::join_community))
        .route(
            "/api/communities/:relayPeerId",
            delete(boards::leave_community),
        )
        .route("/api/boards/:relayPeerId", get(boards::get_boards))
        .route(
            "/api/boards/:relayPeerId/:boardId/posts",
            get(boards::get_board_posts),
        )
        .route(
            "/api/boards/:relayPeerId/:boardId/posts",
            post(boards::submit_board_post),
        )
        .route(
            "/api/boards/posts/:postId",
            delete(boards::delete_board_post),
        )
        .route(
            "/api/boards/:relayPeerId/:boardId/sync",
            post(boards::sync_board),
        )
        // Auth (Isnad CAPTCHA)
        .route("/api/auth/verify-agent", post(auth::verify_agent))
        // Events (SSE)
        .route("/api/events", get(events::event_stream))
}
