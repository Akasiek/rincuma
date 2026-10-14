use super::handlers::{delete_settings, get_settings, save_settings};
use crate::app_state::AppState;
use axum::{Router, routing::get};

pub(crate) fn router() -> Router<AppState> {
    Router::new().route(
        "/notifications/settings",
        get(get_settings).put(save_settings).delete(delete_settings),
    )
}
