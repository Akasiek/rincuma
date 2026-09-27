use axum::{Router, routing::post};

use crate::{app_state::AppState, web::task::create::create};

pub(crate) fn router() -> Router<AppState> {
    Router::new().route("/tasks", post(create))
}
