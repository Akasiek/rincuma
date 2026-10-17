use axum::{
    Router,
    routing::{get, post, put},
};

use crate::{
    app_state::AppState,
    web::auth::{
        handlers::{login, logout, me, register},
        profile::{change_password, update_profile},
    },
};

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(me).put(update_profile))
        .route("/password", put(change_password))
}
