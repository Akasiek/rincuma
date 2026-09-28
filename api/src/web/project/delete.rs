use axum::{
    extract::{Path, State},
    http::StatusCode,
};

use crate::{
    app_state::AppState,
    db::Project,
    web::{auth::CurrentUser, project::ProjectError},
};

pub(super) async fn delete(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<i64>,
) -> Result<StatusCode, ProjectError> {
    let mut db = state.db().clone();
    let mut transaction = db.transaction().await?;
    let project = Project::get_owned(&mut transaction, id, user.id)
        .await?
        .ok_or_else(ProjectError::related_resource_not_found)?;

    Project::filter_by_id(project.id)
        .delete()
        .exec(&mut transaction)
        .await?;

    transaction.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
#[path = "tests/delete.rs"]
mod tests;
