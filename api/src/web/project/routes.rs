use axum::{Router, routing::get};

use crate::{
    app_state::AppState,
    web::project::{create::create, get::get_project, list::list},
};

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/projects", get(list).post(create))
        .route("/projects/{id}", get(get_project))
}
