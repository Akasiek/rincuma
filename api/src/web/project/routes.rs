use axum::{Router, routing::get};

use crate::{
    app_state::AppState,
    web::project::handlers::{create, delete, get_project, list, update},
};

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/projects", get(list).post(create))
        .route(
            "/projects/{id}",
            get(get_project).put(update).delete(delete),
        )
}
