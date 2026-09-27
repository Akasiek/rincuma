use axum::{Json, extract::State, http::StatusCode};
use garde::Validate;
use jiff::Timestamp;
use serde::Deserialize;

use crate::{
    app_state::AppState,
    db::Tag,
    web::{
        auth::CurrentUser,
        tag::{TagError, response::TagResponse},
        validation::validate_hex_color,
    },
};

#[derive(Deserialize, Validate)]
pub(super) struct CreateTagRequest {
    #[garde(length(bytes, min = 1, max = 255))]
    name: String,
    #[garde(inner(custom(validate_hex_color)))]
    color: Option<String>,
}

pub(super) async fn create(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(mut request): Json<CreateTagRequest>,
) -> Result<(StatusCode, Json<TagResponse>), TagError> {
    request.name = request.name.trim().to_owned();
    request.validate().map_err(|_| TagError::bad_request())?;

    let now = Timestamp::now();
    let mut db = state.db().clone();
    let tag = toasty::create!(Tag {
        name: request.name,
        color: request.color,
        owner_id: user.id,
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;

    Ok((StatusCode::CREATED, Json(tag.into())))
}

#[cfg(test)]
#[path = "tests/create.rs"]
mod tests;
