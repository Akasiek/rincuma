use axum::{
    Json,
    extract::{Path, State},
};
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

pub(super) async fn update(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<i64>,
    Json(mut request): Json<SaveTagRequest>,
) -> Result<Json<TagResponse>, TagError> {
    request.name = request.name.trim().to_owned();
    request.validate().map_err(|_| TagError::bad_request())?;

    let mut db = state.db().clone();
    let mut tag = Tag::filter_by_id(id)
        .first()
        .exec(&mut db)
        .await?
        .filter(|tag| tag.owner_id == user.id)
        .ok_or_else(TagError::related_resource_not_found)?;

    toasty::update!(tag {
        name: request.name,
        color: request.color,
        updated_at: Timestamp::now(),
    })
    .exec(&mut db)
    .await?;

    Ok(Json(tag.into()))
}

#[cfg(test)]
#[path = "tests/update.rs"]
mod tests;
