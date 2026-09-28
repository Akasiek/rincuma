use axum::{Json, extract::State, http::StatusCode};
use garde::Validate;
use jiff::Timestamp;

use crate::{
    app_state::AppState,
    db::Tag,
    web::{
        auth::CurrentUser,
        tag::{TagError, request::SaveTagRequest, response::TagResponse},
    },
};

pub(super) async fn create(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(mut request): Json<SaveTagRequest>,
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
