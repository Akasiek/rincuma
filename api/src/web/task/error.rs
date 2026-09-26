use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub(super) enum TaskError {
    #[error("invalid task request")]
    BadRequest,
    #[error("related resource not found")]
    RelatedResourceNotFound,
    #[error("task database operation failed")]
    Database(#[from] toasty::Error),
}

impl IntoResponse for TaskError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::BadRequest => StatusCode::BAD_REQUEST,
            Self::RelatedResourceNotFound => StatusCode::NOT_FOUND,
            Self::Database(_) => {
                error!(error = ?self, "task request failed");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        status.into_response()
    }
}
