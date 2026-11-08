use super::handlers::get_instance;
use crate::app_state::AppState;
use axum::{Router, routing::get};

pub(crate) fn router() -> Router<AppState> {
    Router::new().route("/instance", get(get_instance))
}
