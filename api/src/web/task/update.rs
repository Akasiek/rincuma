use std::collections::HashSet;

use axum::{
    Json,
    extract::{Path, State},
};
use garde::Validate;
use jiff::Timestamp;

use crate::{
    app_state::AppState,
    db::{Task, TaskPriority, TaskTag},
    web::{
        auth::CurrentUser,
        task::{
            TaskError,
            relations::{get_owned_task, validate_active_project, validate_owned_tag},
            request::SaveTaskRequest,
            response::TaskResponse,
        },
    },
};

pub(super) async fn update(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<i64>,
    Json(mut request): Json<SaveTaskRequest>,
) -> Result<Json<TaskResponse>, TaskError> {
    request.name = request.name.trim().to_owned();
    request.validate().map_err(|_| TaskError::bad_request())?;

    let mut db = state.db().clone();
    let mut transaction = db.transaction().await?;
    let mut task = get_owned_task(&mut transaction, id, user.id).await?;

    validate_project_change(&mut transaction, &task, request.project_id).await?;
    validate_parent(&mut transaction, &task, &request).await?;
    for tag_id in &request.tag_ids {
        validate_owned_tag(&mut transaction, *tag_id, user.id).await?;
    }

    toasty::update!(task {
        name: request.name,
        description: request.description,
        due_at: request.due_at,
        priority: request.priority.unwrap_or(TaskPriority::None),
        project_id: request.project_id,
        parent_id: request.parent_id,
        updated_at: Timestamp::now(),
    })
    .exec(&mut transaction)
    .await?;

    sync_task_with_tags(&mut transaction, task.id, &request.tag_ids).await?;

    transaction.commit().await?;
    Ok(Json(TaskResponse::new(task, request.tag_ids)))
}

/// Validates a project reassignment without breaking the task hierarchy.
///
/// An unchanged project is accepted, including an archived project, so existing
/// tasks can still be edited. A newly assigned project must exist, belong to the
/// task's owner, and be active. Removing the project is also a reassignment.
///
/// Every direct child must already belong to the requested project. Otherwise,
/// the change is rejected because a task and its children must share a project;
/// this endpoint does not move the children automatically. Compatibility with
/// the requested parent is checked separately by `validate_parent`.
async fn validate_project_change(
    executor: &mut dyn toasty::Executor,
    task: &Task,
    project_id: Option<i64>,
) -> Result<(), TaskError> {
    if project_id == task.project_id {
        return Ok(());
    }
    if let Some(project_id) = project_id {
        validate_active_project(executor, project_id, task.owner_id).await?;
    }
    let children = Task::filter(Task::fields().parent_id().eq(task.id))
        .exec(executor)
        .await?;
    if children.iter().any(|child| child.project_id != project_id) {
        return Err(TaskError::bad_request());
    }
    Ok(())
}

async fn validate_parent(
    executor: &mut dyn toasty::Executor,
    task: &Task,
    request: &SaveTaskRequest,
) -> Result<(), TaskError> {
    let Some(parent_id) = request.parent_id else {
        return Ok(());
    };
    if parent_id == task.id {
        return Err(TaskError::bad_request());
    }
    let parent = get_owned_task(executor, parent_id, task.owner_id).await?;
    if parent.project_id != request.project_id {
        return Err(TaskError::bad_request());
    }

    let mut seen = HashSet::from([task.id, parent.id]);
    let mut ancestor_id = parent.parent_id;
    while let Some(id) = ancestor_id {
        if !seen.insert(id) {
            return Err(TaskError::bad_request());
        }
        let ancestor = get_owned_task(executor, id, task.owner_id).await?;
        ancestor_id = ancestor.parent_id;
    }
    Ok(())
}

async fn sync_task_with_tags(
    executor: &mut dyn toasty::Executor,
    task_id: i64,
    tag_ids: &[i64],
) -> Result<(), TaskError> {
    TaskTag::filter_by_task_id(task_id)
        .delete()
        .exec(executor)
        .await?;
    for tag_id in tag_ids {
        toasty::create!(TaskTag {
            task_id,
            tag_id: *tag_id,
        })
        .exec(executor)
        .await?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/update.rs"]
mod tests;
