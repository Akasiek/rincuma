use axum::{
    Json,
    extract::{Path, State},
};
use jiff::Timestamp;

use crate::{
    app_state::AppState,
    db::{Task, TaskTag},
    web::{
        auth::CurrentUser,
        task::{TaskError, response::TaskResponse},
    },
};

pub(in crate::web::task) async fn complete(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<i64>,
) -> Result<Json<TaskResponse>, TaskError> {
    set_completion(state, user.id, id, true).await
}

pub(in crate::web::task) async fn reopen(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<i64>,
) -> Result<Json<TaskResponse>, TaskError> {
    set_completion(state, user.id, id, false).await
}

async fn set_completion(
    state: AppState,
    owner_id: i64,
    id: i64,
    completed: bool,
) -> Result<Json<TaskResponse>, TaskError> {
    let mut db = state.db().clone();
    let mut transaction = db.transaction().await?;
    let mut task = Task::get_owned(&mut transaction, id, owner_id)
        .await?
        .ok_or_else(TaskError::related_resource_not_found)?;

    if task.completed_at.is_some() != completed {
        let now = Timestamp::now();
        toasty::update!(task {
            completed_at: completed.then_some(now),
            updated_at: now,
        })
        .exec(&mut transaction)
        .await?;
    }

    let tag_ids = TaskTag::filter_by_task_id(task.id)
        .exec(&mut transaction)
        .await?
        .into_iter()
        .map(|link| link.tag_id)
        .collect();

    transaction.commit().await?;
    Ok(Json(TaskResponse::new(task, tag_ids)))
}

#[cfg(test)]
#[path = "../tests/complete.rs"]
mod tests;
