use harbor_lib::error::AppError;
use harbor_lib::p2p::NetworkHandle;
use harbor_lib::services::{
    AccountsService, BoardService, ContactsService, ContentSyncService, FeedService,
    IdentityService, MessagingService, PermissionsService, PostsService,
};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Network state wrapper (mirrors commands/network.rs NetworkState without Tauri deps)
pub struct NetworkState {
    pub handle: RwLock<Option<NetworkHandle>>,
}

impl NetworkState {
    pub fn new() -> Self {
        Self {
            handle: RwLock::new(None),
        }
    }

    pub async fn set_handle(&self, handle: NetworkHandle) {
        let mut guard = self.handle.write().await;
        *guard = Some(handle);
    }

    pub async fn get_handle(&self) -> Result<NetworkHandle, AppError> {
        let guard = self.handle.read().await;
        guard
            .clone()
            .ok_or_else(|| AppError::Network("Network not initialized".to_string()))
    }

    pub async fn is_running(&self) -> bool {
        self.handle.read().await.is_some()
    }
}

/// Shared application state passed to all axum handlers
pub struct AppState {
    pub identity_service: Arc<IdentityService>,
    pub contacts_service: Arc<ContactsService>,
    pub permissions_service: Arc<PermissionsService>,
    pub messaging_service: Arc<MessagingService>,
    pub posts_service: Arc<PostsService>,
    pub feed_service: Arc<FeedService>,
    pub board_service: Arc<BoardService>,
    pub content_sync_service: Arc<ContentSyncService>,
    pub accounts_service: Arc<AccountsService>,
    pub network: NetworkState,
    pub event_tx: broadcast::Sender<serde_json::Value>,
}
