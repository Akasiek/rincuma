use axum::{
    Json,
    extract::{Path, State},
};

use crate::{
    app_state::AppState,
    db::Project,
    web::{
        auth::CurrentUser,
        project::{ProjectError, response::ProjectResponse},
    },
};

pub(super) async fn get_project(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<i64>,
) -> Result<Json<ProjectResponse>, ProjectError> {
    let mut db = state.db().clone();
    let project = Project::get_owned(&mut db, id, user.id)
        .await?
        .ok_or_else(ProjectError::related_resource_not_found)?;

    Ok(Json(project.into()))
}

#[cfg(test)]
#[path = "tests/get.rs"]
mod tests;
