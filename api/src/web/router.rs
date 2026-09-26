use crate::app_state::AppState;
use crate::web::auth;
use axum::Router;
use axum::extract::MatchedPath;
use axum::http::{Request, StatusCode};
use axum::routing::get;
use tower_http::trace::TraceLayer;
use tracing::info_span;

pub fn get_app_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .nest("/auth", auth::router())
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                let matched_path = request
                    .extensions()
                    .get::<MatchedPath>()
                    .map(MatchedPath::as_str);

                info_span!(
                    "http_request",
                    method = ?request.method(),
                    matched_path,
                    some_other_field = tracing::field::Empty,
                )
            }),
        )
        .with_state(state)
}

async fn health() -> StatusCode {
    StatusCode::OK
}
