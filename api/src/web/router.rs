use super::{logging, notifications};
use crate::app_state::AppState;
use crate::web::{auth, project, tag, task};
use axum::Router;
use axum::http::StatusCode;
use axum::routing::get;
use tower_http::trace::TraceLayer;

pub fn get_app_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .nest("/auth", auth::router())
        .merge(task::router())
        .merge(project::router())
        .merge(tag::router())
        .merge(notifications::router())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(logging::request_span)
                .on_request(())
                .on_response(logging::log_response)
                .on_failure(logging::log_failure),
        )
        .with_state(state)
}

async fn health() -> StatusCode {
    StatusCode::OK
}
