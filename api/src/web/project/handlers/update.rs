use axum::{
    Json,
    extract::{Path, State},
};
use garde::Validate;
use jiff::Timestamp;

use crate::{
    app_state::AppState,
    db::Project,
    web::{
        auth::CurrentUser,
        project::{ProjectError, request::SaveProjectRequest, response::ProjectResponse},
    },
};

pub(in crate::web::project) async fn update(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<i64>,
    Json(mut request): Json<SaveProjectRequest>,
) -> Result<Json<ProjectResponse>, ProjectError> {
    request.name = request.name.trim().to_owned();
    request
        .validate()
        .map_err(|_| ProjectError::bad_request())?;

    let mut db = state.db().clone();
    let mut project = Project::get_owned(&mut db, id, user.id)
        .await?
        .ok_or_else(ProjectError::related_resource_not_found)?;

    toasty::update!(project {
        name: request.name,
        description: request.description,
        color: request.color,
        updated_at: Timestamp::now(),
    })
    .exec(&mut db)
    .await?;

    Ok(Json(project.into()))
}

#[cfg(test)]
#[path = "../tests/update.rs"]
mod tests;
