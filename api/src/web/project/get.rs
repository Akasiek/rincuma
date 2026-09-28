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
    let project = Project::filter_by_id(id)
        .first()
        .exec(&mut db)
        .await?
        .filter(|project| project.owner_id == user.id)
        .ok_or_else(ProjectError::related_resource_not_found)?;

    Ok(Json(project.into()))
}

#[cfg(test)]
#[path = "tests/get.rs"]
mod tests;
