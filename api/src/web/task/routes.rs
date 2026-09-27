use axum::{Router, routing::get};

use crate::{
    app_state::AppState,
    web::task::{create::create, list::list},
};

pub(crate) fn router() -> Router<AppState> {
    Router::new().route("/tasks", get(list).post(create))
}
