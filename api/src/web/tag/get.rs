use axum::{
    Json,
    extract::{Path, State},
};

use crate::{
    app_state::AppState,
    db::Tag,
    web::{
        auth::CurrentUser,
        tag::{TagError, response::TagResponse},
    },
};

pub(super) async fn get_tag(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<i64>,
) -> Result<Json<TagResponse>, TagError> {
    let mut db = state.db().clone();
    let tag = Tag::get_owned(&mut db, id, user.id)
        .await?
        .ok_or_else(TagError::related_resource_not_found)?;

    Ok(Json(tag.into()))
}

#[cfg(test)]
#[path = "tests/get.rs"]
mod tests;
