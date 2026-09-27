use axum::{Router, routing::post};

use crate::{app_state::AppState, web::project::handlers::create};

pub(crate) fn router() -> Router<AppState> {
    Router::new().route("/projects", post(create))
}
