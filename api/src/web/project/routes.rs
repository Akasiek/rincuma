use axum::{Router, routing::get};

use crate::{
    app_state::AppState,
    web::project::{create::create, list::list},
};

pub(crate) fn router() -> Router<AppState> {
    Router::new().route("/projects", get(list).post(create))
}
