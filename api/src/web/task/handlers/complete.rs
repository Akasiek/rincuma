use axum::{
    Json,
    extract::{Path, State},
};
use jiff::Timestamp;

use crate::{
    app_state::AppState,
    db::TaskTag,
    web::{
        auth::CurrentUser,
        task::{
            TaskError, completion::complete_locked, locking::lock_owned_task,
            reopening::reopen_locked, response::TaskResponse,
        },
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
    let mut task = lock_owned_task(&mut transaction, id, owner_id).await?;

    let tag_ids: Vec<i64> = TaskTag::filter_by_task_id(task.id)
        .exec(&mut transaction)
        .await?
        .into_iter()
        .map(|link| link.tag_id)
        .collect();

    if completed {
        complete_locked(&mut transaction, &mut task, &tag_ids, Timestamp::now()).await?;
    } else {
        reopen_locked(&mut transaction, &mut task, Timestamp::now()).await?;
    }

    transaction.commit().await?;
    Ok(Json(TaskResponse::new(task, tag_ids)))
}

#[cfg(test)]
#[path = "../tests/complete.rs"]
mod tests;
