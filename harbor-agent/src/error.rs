use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use harbor_lib::error::AppError;

/// Wrapper around harbor_lib's AppError that implements axum's IntoResponse
pub struct ApiError(pub AppError);

impl From<AppError> for ApiError {
    fn from(err: AppError) -> Self {
        ApiError(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match &self.0 {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Validation(_) | AppError::InvalidData(_) => StatusCode::BAD_REQUEST,
            AppError::PermissionDenied(_) => StatusCode::FORBIDDEN,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::AlreadyExists(_) => StatusCode::CONFLICT,
            AppError::Identity(msg) if msg.contains("locked") => StatusCode::FORBIDDEN,
            AppError::Identity(msg) if msg.contains("passphrase") || msg.contains("Passphrase") => {
                StatusCode::UNAUTHORIZED
            }
            AppError::Network(_) => StatusCode::BAD_GATEWAY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = self.0.to_response();
        (status, Json(body)).into_response()
    }
}
