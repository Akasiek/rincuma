use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    app_state::AppState,
    web::task::handlers::{complete, create, delete, get_task, list, reopen, update},
};

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/tasks", get(list).post(create))
        .route("/tasks/{id}", get(get_task).put(update).delete(delete))
        .route("/tasks/{id}/complete", post(complete).delete(reopen))
}
