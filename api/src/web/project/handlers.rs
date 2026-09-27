use axum::{Json, extract::State, http::StatusCode};
use garde::Validate;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState,
    db::Project,
    web::{auth::CurrentUser, project::error::ProjectError, validation::validate_hex_color},
};

#[derive(Deserialize, Validate)]
#[garde(allow_unvalidated)]
pub(super) struct CreateProjectRequest {
    #[garde(length(bytes, min = 1, max = 255))]
    name: String,
    description: Option<String>,
    #[garde(inner(custom(validate_hex_color)))]
    color: Option<String>,
}

#[derive(Serialize)]
pub(super) struct ProjectResponse {
    id: i64,
    name: String,
    description: Option<String>,
    color: Option<String>,
    archived_at: Option<Timestamp>,
    owner_id: i64,
    created_at: Timestamp,
    updated_at: Timestamp,
}

pub(super) async fn create(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(mut request): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<ProjectResponse>), ProjectError> {
    request.name = request.name.trim().to_owned();
    request.validate().map_err(|_| ProjectError::BadRequest)?;

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

    Ok((
        StatusCode::CREATED,
        Json(ProjectResponse {
            id: project.id,
            name: project.name,
            description: project.description,
            color: project.color,
            archived_at: project.archived_at,
            owner_id: project.owner_id,
            created_at: project.created_at,
            updated_at: project.updated_at,
        }),
    ))
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
