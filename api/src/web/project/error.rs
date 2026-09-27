use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub(super) enum ProjectError {
    #[error("invalid project request")]
    BadRequest,
    #[error("project database operation failed")]
    Database(#[from] toasty::Error),
}

impl IntoResponse for ProjectError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::BadRequest => StatusCode::BAD_REQUEST,
            Self::Database(_) => {
                error!(error = ?self, "project request failed");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        status.into_response()
    }
}
