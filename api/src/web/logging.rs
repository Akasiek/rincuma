use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use axum::{
    extract::MatchedPath,
    http::{Request, Response},
};
use tower_http::classify::ServerErrorsFailureClass;
use tracing::{Span, debug, error, info, info_span, warn};

static NEXT_REQUEST_ID: AtomicU64 = AtomicU64::new(1);

pub(in crate::web) fn request_span<B>(request: &Request<B>) -> Span {
    let request_id = NEXT_REQUEST_ID.fetch_add(1, Ordering::Relaxed);
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map_or("<unmatched>", MatchedPath::as_str);

    if route == "/health" {
        return info_span!("healthcheck", request_id, method = %request.method(), route);
    }

    info_span!(
        "http_request",
        request_id,
        method = %request.method(),
        route,
        owner_id = tracing::field::Empty,
    )
}

pub(in crate::web) fn log_response<B>(response: &Response<B>, latency: Duration, span: &Span) {
    let status = response.status();
    let status_code = status.as_u16();
    let latency_ms = latency.as_secs_f64() * 1000.0;

    if status.is_server_error() {
        error!(parent: span, status_code, latency_ms, "HTTP request failed");
    } else if status.is_client_error() {
        warn!(parent: span, status_code, latency_ms, "HTTP request rejected");
    } else if span
        .metadata()
        .is_some_and(|metadata| metadata.name() == "healthcheck")
    {
        debug!(parent: span, status_code, latency_ms, "Healthcheck completed");
    } else {
        info!(parent: span, status_code, latency_ms, "HTTP request completed");
    }
}

pub(in crate::web) fn log_failure(
    failure: ServerErrorsFailureClass,
    latency: Duration,
    span: &Span,
) {
    // HTTP error statuses are already logged by log_response.
    if matches!(failure, ServerErrorsFailureClass::Error(_)) {
        error!(parent: span, latency_ms = latency.as_secs_f64() * 1000.0, "HTTP response stream failed");
    }
}
