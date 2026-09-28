use axum::{Json, extract::State, http::StatusCode};
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

pub(super) async fn create(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(mut request): Json<SaveProjectRequest>,
) -> Result<(StatusCode, Json<ProjectResponse>), ProjectError> {
    request.name = request.name.trim().to_owned();
    request
        .validate()
        .map_err(|_| ProjectError::bad_request())?;

    let now = Timestamp::now();
    let mut db = state.db().clone();
    let project = toasty::create!(Project {
        name: request.name,
        description: request.description,
        color: request.color,
        owner_id: user.id,
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;

    Ok((StatusCode::CREATED, Json(project.into())))
}

#[cfg(test)]
#[path = "tests/create.rs"]
mod tests;
