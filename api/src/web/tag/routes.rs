use axum::{Router, routing::get};

use crate::{
    app_state::AppState,
    web::tag::{create::create, list::list},
};

pub(crate) fn router() -> Router<AppState> {
    Router::new().route("/tags", get(list).post(create))
}
