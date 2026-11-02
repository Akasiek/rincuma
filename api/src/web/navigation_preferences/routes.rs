use super::handlers::{get_preferences, save_section};
use crate::app_state::AppState;
use axum::{
    Router,
    routing::{get, put},
};

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/preferences/navigation", get(get_preferences))
        .route("/preferences/navigation/{section}", put(save_section))
}
