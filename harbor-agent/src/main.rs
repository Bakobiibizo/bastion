mod api;
mod captcha_solver;
mod error;
mod state;

use clap::Parser;
use harbor_lib::db::Database;
use harbor_lib::logging::{self, LogConfig};
use harbor_lib::services::{
    AccountsService, BoardService, ContactsService, ContentSyncService,
    FeedService, IdentityService, MessagingService, PermissionsService, PostsService,
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tracing::info;

use crate::state::{AppState, NetworkState};

#[derive(Parser)]
#[command(name = "bastion-agent", about = "Headless HTTP API daemon for autonomous agent coordination over P2P mesh")]
struct Cli {
    /// Data directory for this agent's database and identity
    #[arg(long, env = "BASTION_DATA_DIR", default_value = "~/.bastion")]
    data_dir: String,

    /// HTTP API port
    #[arg(long, default_value = "8745")]
    port: u16,

    /// Bind address
    #[arg(long, default_value = "127.0.0.1")]
    bind: String,

    /// Auto-unlock passphrase (prefer env var BASTION_PASSPHRASE)
    #[arg(long, env = "BASTION_PASSPHRASE")]
    passphrase: Option<String>,

    /// Auto-start P2P network after unlock
    #[arg(long)]
    auto_network: bool,

    /// Relay address to connect to on startup
    #[arg(long)]
    relay: Option<String>,
}

fn expand_tilde(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Some(home) = dirs_fallback() {
            return home.join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

fn dirs_fallback() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| std::env::var("USERPROFILE").ok().map(PathBuf::from))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Init logging
    logging::init_logging(LogConfig::development());

    info!("bastion-agent starting...");

    // Expand data directory and ensure it exists
    let data_dir = expand_tilde(&cli.data_dir);
    std::fs::create_dir_all(&data_dir)?;

    let db_path = data_dir.join("bastion.db");
    info!("Database path: {:?}", db_path);

    // Initialize database
    let db = Arc::new(Database::new(db_path)?);

    // Initialize services (mirrors lib.rs:112-164)
    let accounts_service = Arc::new(AccountsService::new(data_dir.clone()));
    let identity_service = Arc::new(IdentityService::new(db.clone()));
    let contacts_service = Arc::new(ContactsService::new(db.clone(), identity_service.clone()));
    let permissions_service = Arc::new(PermissionsService::new(
        db.clone(),
        identity_service.clone(),
    ));
    let messaging_service = Arc::new(MessagingService::new(
        db.clone(),
        identity_service.clone(),
        contacts_service.clone(),
        permissions_service.clone(),
    ));
    let posts_service = Arc::new(PostsService::new(
        db.clone(),
        identity_service.clone(),
        contacts_service.clone(),
        permissions_service.clone(),
    ));
    let feed_service = Arc::new(FeedService::new(
        db.clone(),
        identity_service.clone(),
        permissions_service.clone(),
        contacts_service.clone(),
    ));
    let content_sync_service = Arc::new(ContentSyncService::new(
        db.clone(),
        identity_service.clone(),
        contacts_service.clone(),
        permissions_service.clone(),
    ));
    let board_service = Arc::new(BoardService::new(db.clone(), identity_service.clone()));

    // Broadcast channel for SSE events
    let (event_tx, _) = broadcast::channel(256);

    let app_state = Arc::new(AppState {
        identity_service: identity_service.clone(),
        contacts_service,
        permissions_service,
        messaging_service,
        posts_service,
        feed_service,
        board_service,
        content_sync_service,
        accounts_service,
        network: NetworkState::new(),
        event_tx,
    });

    // Auto-unlock if passphrase provided
    if let Some(ref passphrase) = cli.passphrase {
        if identity_service.has_identity()? {
            match identity_service.unlock(passphrase) {
                Ok(info) => {
                    info!("Identity unlocked: {} ({})", info.display_name, info.peer_id);
                }
                Err(e) => {
                    tracing::error!("Failed to auto-unlock identity: {}", e);
                }
            }
        } else {
            info!("No identity exists yet. Create one via POST /api/identity");
        }
    }

    // Auto-start network if requested and identity is unlocked
    if cli.auto_network && identity_service.is_unlocked() {
        info!("Auto-starting network...");
        // Use the same logic as the network start endpoint
        let state_clone = app_state.clone();
        if let Err(e) = auto_start_network(state_clone).await {
            tracing::error!("Failed to auto-start network: {}", e);
        }
    }

    // Auto-connect to relay if specified
    if let Some(ref relay_addr) = cli.relay {
        if let Ok(handle) = app_state.network.get_handle().await {
            let addr: libp2p::Multiaddr = relay_addr.parse()?;
            if let Err(e) = handle.add_relay_server(addr).await {
                tracing::error!("Failed to connect to relay: {}", e);
            } else {
                info!("Connected to relay: {}", relay_addr);
            }
        }
    }

    // Build axum app
    let app = api::router()
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    let addr = format!("{}:{}", cli.bind, cli.port);
    info!("bastion-agent listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn auto_start_network(state: Arc<AppState>) -> Result<(), harbor_lib::error::AppError> {
    use harbor_lib::p2p::{NetworkConfig, NetworkService};

    let unlocked_keys = state.identity_service.get_unlocked_keys()?;
    let ed25519_bytes = unlocked_keys.ed25519_signing.to_bytes();
    let keypair = harbor_lib::p2p::swarm::ed25519_to_libp2p_keypair(&ed25519_bytes)?;

    let config = NetworkConfig::default();
    let identity_arc = state.identity_service.clone();
    let (mut service, handle, mut event_rx) =
        NetworkService::new(config, identity_arc, keypair)?;

    service.set_messaging_service(state.messaging_service.clone());
    service.set_contacts_service(state.contacts_service.clone());
    service.set_permissions_service(state.permissions_service.clone());
    service.set_posts_service(state.posts_service.clone());
    service.set_content_sync_service(state.content_sync_service.clone());
    service.set_board_service(state.board_service.clone());

    state.network.set_handle(handle).await;

    tokio::spawn(async move {
        info!("Network service starting in background task");
        service.run().await;
        info!("Network service stopped");
    });

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
    Ok(())
}
