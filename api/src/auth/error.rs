use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;
use tracing::error;

use crate::auth::password::PasswordError;

#[derive(Debug, Error)]
pub(crate) enum AuthError {
    #[error("invalid authentication request")]
    BadRequest,
    #[error("authentication resource already exists")]
    Conflict,
    #[error("authentication required")]
    Unauthorized,
    #[error("internal authentication error")]
    Internal,
    #[error("authentication database operation failed")]
    Database(#[from] toasty::Error),
    #[error("password operation failed")]
    Password(#[source] PasswordError),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::BadRequest => StatusCode::BAD_REQUEST,
            Self::Conflict => StatusCode::CONFLICT,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Internal | Self::Database(_) | Self::Password(_) => {
                error!(error = ?self, "authentication request failed");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        status.into_response()
    }
}
