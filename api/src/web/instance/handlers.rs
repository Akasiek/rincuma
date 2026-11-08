use super::response::InstanceResponse;
use crate::app_state::AppState;
use axum::{Json, extract::State};

pub(super) async fn get_instance(State(state): State<AppState>) -> Json<InstanceResponse> {
    Json(state.config().into())
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
