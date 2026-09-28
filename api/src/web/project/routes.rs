use axum::{Router, routing::get};

use crate::{
    app_state::AppState,
    web::project::{create::create, delete::delete, get::get_project, list::list, update::update},
};

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/projects", get(list).post(create))
        .route(
            "/projects/{id}",
            get(get_project).put(update).delete(delete),
        )
}
