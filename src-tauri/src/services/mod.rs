pub mod calling_service;
pub mod contacts_service;
pub mod content_sync_service;
pub mod crypto_service;
pub mod feed_service;
pub mod identity_service;
pub mod messaging_service;
pub mod permissions_service;
pub mod posts_service;
pub mod signing;

pub use calling_service::{CallingService, CallState, Call, OutgoingOffer, OutgoingAnswer, OutgoingIce, OutgoingHangup};
pub use contacts_service::ContactsService;
pub use content_sync_service::{ContentSyncService, OutgoingManifestRequest, OutgoingManifestResponse};
pub use crypto_service::CryptoService;
pub use feed_service::{FeedService, FeedItem};
pub use identity_service::IdentityService;
pub use messaging_service::{MessagingService, DecryptedMessage, OutgoingMessage};
pub use permissions_service::{
    PermissionsService, PermissionRequestMessage, PermissionGrantMessage, PermissionRevokeMessage,
};
pub use posts_service::{PostsService, OutgoingPost, OutgoingPostUpdate, OutgoingPostDelete};
pub use signing::{
    Signable, sign, verify,
    // Identity messages
    SignableIdentityRequest, SignableIdentityResponse,
    // Permission messages
    SignablePermissionRequest, SignablePermissionGrant, SignablePermissionRevoke,
    // Direct messages
    SignableDirectMessage, SignableMessageAck,
    // Post messages
    SignablePost, SignablePostUpdate, SignablePostDelete,
    // Signaling messages (voice calls)
    SignableSignalingOffer, SignableSignalingAnswer, SignableSignalingIce, SignableSignalingHangup,
    // Content sync
    SignableContentManifestRequest, SignableContentManifestResponse, PostSummary,
    PermissionProof,
};
