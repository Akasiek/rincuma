use std::collections::HashSet;

use axum::{
    extract::{Path, State},
    http::StatusCode,
};

use crate::{
    app_state::AppState,
    db::Task,
    web::{auth::CurrentUser, task::TaskError},
};

pub(super) async fn delete(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<i64>,
) -> Result<StatusCode, TaskError> {
    let mut db = state.db().clone();
    let mut transaction = db.transaction().await?;
    let task = Task::get_owned(&mut transaction, id, user.id)
        .await?
        .ok_or_else(TaskError::related_resource_not_found)?;

    let task_ids = collect_subtree_ids(&mut transaction, &task).await?;
    // Delete descendants before their parents.
    for task_id in task_ids.into_iter().rev() {
        Task::filter_by_id(task_id)
            .delete()
            .exec(&mut transaction)
            .await?;
    }

    transaction.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn collect_subtree_ids(
    executor: &mut dyn toasty::Executor,
    root: &Task,
) -> Result<Vec<i64>, TaskError> {
    let mut task_ids = vec![root.id];
    let mut seen = HashSet::from([root.id]);
    let mut parent_ids = vec![root.id];

    while !parent_ids.is_empty() {
        let children = Task::filter(Task::fields().parent_id().in_list(parent_ids))
            .exec(executor)
            .await?;
        parent_ids = Vec::new();
        for child in children {
            if child.owner_id != root.owner_id {
                return Err(TaskError::related_resource_not_found());
            }
            if seen.insert(child.id) {
                task_ids.push(child.id);
                parent_ids.push(child.id);
            }
        }
    }
    Ok(task_ids)
}

#[cfg(test)]
#[path = "tests/delete.rs"]
mod tests;
