use axum::{
    extract::{Path, State},
    http::StatusCode,
};

use crate::{
    app_state::AppState,
    db::{Tag, TaskTag},
    web::{auth::CurrentUser, tag::TagError},
};

pub(super) async fn delete(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<i64>,
) -> Result<StatusCode, TagError> {
    let mut db = state.db().clone();
    let mut transaction = db.transaction().await?;
    let tag = Tag::get_owned(&mut transaction, id, user.id)
        .await?
        .ok_or_else(TagError::related_resource_not_found)?;

    TaskTag::filter_by_tag_id(tag.id)
        .delete()
        .exec(&mut transaction)
        .await?;
    Tag::filter_by_id(tag.id)
        .delete()
        .exec(&mut transaction)
        .await?;

    transaction.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
#[path = "tests/delete.rs"]
mod tests;
