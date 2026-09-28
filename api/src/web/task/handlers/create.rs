use axum::{Json, extract::State, http::StatusCode};
use garde::Validate;
use jiff::Timestamp;

use crate::{
    app_state::AppState,
    db::{Task, TaskPriority, TaskTag},
    web::{
        auth::CurrentUser,
        task::{
            TaskError,
            relations::{validate_active_project, validate_owned_tag},
            request::SaveTaskRequest,
            response::TaskResponse,
        },
    },
};

pub(in crate::web::task) async fn create(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(mut request): Json<SaveTaskRequest>,
) -> Result<(StatusCode, Json<TaskResponse>), TaskError> {
    request.name = request.name.trim().to_owned();
    request.validate().map_err(|_| TaskError::bad_request())?;

    let tag_ids = request.tag_ids;
    let now = Timestamp::now();
    let mut db = state.db().clone();
    let mut transaction = db.transaction().await?;

    if let Some(project_id) = request.project_id {
        validate_active_project(&mut transaction, project_id, user.id).await?;
    }

    if let Some(parent_id) = request.parent_id {
        let parent = Task::get_owned(&mut transaction, parent_id, user.id)
            .await?
            .ok_or_else(TaskError::related_resource_not_found)?;
        if parent.project_id != request.project_id {
            return Err(TaskError::bad_request());
        }
    }

    for tag_id in &tag_ids {
        validate_owned_tag(&mut transaction, *tag_id, user.id).await?;
    }

    let task = toasty::create!(Task {
        name: request.name,
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

    Ok((StatusCode::CREATED, Json(TaskResponse::new(task, tag_ids))))
}

#[cfg(test)]
#[path = "../tests/create.rs"]
mod tests;
