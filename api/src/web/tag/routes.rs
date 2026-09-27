use axum::{Router, routing::post};

use crate::{app_state::AppState, web::tag::handlers::create};

pub(crate) fn router() -> Router<AppState> {
    Router::new().route("/tags", post(create))
}
