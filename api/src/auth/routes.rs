use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    app_state::AppState,
    auth::handlers::{login, logout, me, register},
};

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(me))
}
