use axum::{Router, routing::get};

use crate::{
    app_state::AppState,
    web::task::handlers::{create, delete, get_task, list, update},
};

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/tasks", get(list).post(create))
        .route("/tasks/{id}", get(get_task).put(update).delete(delete))
}
