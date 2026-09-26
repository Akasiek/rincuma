use axum::{Router, routing::post};

use crate::{app_state::AppState, web::task::handlers::create};

pub(crate) fn router() -> Router<AppState> {
    Router::new().route("/tasks", post(create))
}
