use axum::{Router, routing::get};

use crate::{
    app_state::AppState,
    web::task::{create::create, get::get_task, list::list, update::update},
};

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/tasks", get(list).post(create))
        .route("/tasks/{id}", get(get_task).put(update))
}
