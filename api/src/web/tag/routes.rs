use axum::{Router, routing::get};

use crate::{
    app_state::AppState,
    web::tag::handlers::{create, delete, get_tag, list, update},
};

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/tags", get(list).post(create))
        .route("/tags/{id}", get(get_tag).put(update).delete(delete))
}
