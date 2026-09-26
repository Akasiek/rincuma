use std::collections::HashSet;

use axum::{Json, extract::State, http::StatusCode};
use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState,
    db::{Task, TaskPriority, TaskTag},
    web::{
        auth::CurrentUser,
        task::{
            error::TaskError,
            relations::{get_owned_task, validate_active_project, validate_owned_tag},
        },
    },
};

#[derive(Deserialize)]
pub(super) struct CreateTaskRequest {
    name: String,
    description: Option<String>,
    due_at: Option<Timestamp>,
    priority: Option<TaskPriority>,
    project_id: Option<i64>,
    parent_id: Option<i64>,
    #[serde(default)]
    tag_ids: Vec<i64>,
}

#[derive(Serialize)]
pub(super) struct TaskResponse {
    id: i64,
    name: String,
    description: Option<String>,
    due_at: Option<Timestamp>,
    completed_at: Option<Timestamp>,
    priority: TaskPriority,
    owner_id: i64,
    project_id: Option<i64>,
    parent_id: Option<i64>,
    tag_ids: Vec<i64>,
    created_at: Timestamp,
    updated_at: Timestamp,
}

pub(super) async fn create(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(request): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<TaskResponse>), TaskError> {
    let name = request.name.trim();
    if name.is_empty() || name.len() > 255 {
        return Err(TaskError::BadRequest);
    }

    let tag_ids = request.tag_ids;
    let mut seen = HashSet::new();
    if tag_ids.iter().any(|&id| id <= 0 || !seen.insert(id)) {
        return Err(TaskError::BadRequest);
    }

    let now = Timestamp::now();
    let mut db = state.db().clone();
    let mut transaction = db.transaction().await?;

    if let Some(project_id) = request.project_id {
        validate_active_project(&mut transaction, project_id, user.id).await?;
    }

    if let Some(parent_id) = request.parent_id {
        let parent = get_owned_task(&mut transaction, parent_id, user.id).await?;
        if parent.project_id != request.project_id {
            return Err(TaskError::BadRequest);
        }
    }

    for tag_id in &tag_ids {
        validate_owned_tag(&mut transaction, *tag_id, user.id).await?;
    }

    let task = toasty::create!(Task {
        name: name.to_owned(),
        description: request.description,
        due_at: request.due_at,
        priority: request.priority.unwrap_or(TaskPriority::None),
        owner_id: user.id,
        parent_id: request.parent_id,
        project_id: request.project_id,
        created_at: now,
        updated_at: now,
    })
    .exec(&mut transaction)
    .await?;

    for tag_id in &tag_ids {
        toasty::create!(TaskTag {
            task_id: task.id,
            tag_id: *tag_id,
        })
        .exec(&mut transaction)
        .await?;
    }

    transaction.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(TaskResponse {
            id: task.id,
            name: task.name,
            description: task.description,
            due_at: task.due_at,
            completed_at: task.completed_at,
            priority: task.priority,
            owner_id: task.owner_id,
            project_id: task.project_id,
            parent_id: task.parent_id,
            tag_ids,
            created_at: task.created_at,
            updated_at: task.updated_at,
        }),
    ))
}
