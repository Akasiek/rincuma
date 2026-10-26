use std::collections::{BTreeSet, HashSet};

use jiff::{SignedDuration, Timestamp};

use crate::{
    db::{Task, TaskSeries},
    web::task::{TaskError, locking::lock_subtree},
};

// Toast timers pause on hover or loss of focus. Keep a bounded Undo grace period;
// expired tasks remain soft deleted rather than being permanently purged.
const UNDO_RETENTION_SECONDS: i64 = 10 * 60;

pub(super) struct DeletionReceipt {
    pub(super) id: i64,
    pub(super) deleted_at: Timestamp,
    pub(super) expires_at: Timestamp,
}

pub(super) fn undo_expires_at(deleted_at: Timestamp) -> Result<Timestamp, TaskError> {
    deleted_at
        .checked_add(SignedDuration::from_secs(UNDO_RETENTION_SECONDS))
        .map_err(|_| TaskError::bad_request())
}

pub(super) async fn soft_delete_subtree(
    executor: &mut dyn toasty::Executor,
    id: i64,
    owner_id: i64,
    now: Timestamp,
) -> Result<DeletionReceipt, TaskError> {
    let tasks = collect_locked_subtree(executor, id, owner_id).await?;
    let stopped = stop_deleted_series(executor, &tasks, owner_id, now).await?;

    for task in &tasks {
        let stopped_series_id = task.series_id.filter(|_| stopped.contains(&task.id));
        toasty::update!(Task::filter_by_id(task.id) {
            deleted_at: Some(now),
            deletion_root_id: Some(id),
            stopped_series_id,
        })
        .exec(executor)
        .await?;
    }

    // Read the persisted timestamp: PostgreSQL stores microseconds, while
    // Timestamp::now() may contain nanoseconds. Undo must use the stored value.
    let root = Task::lock_owned_including_deleted(executor, id, owner_id)
        .await?
        .ok_or_else(TaskError::related_resource_not_found)?;
    let deleted_at = root.deleted_at.ok_or_else(TaskError::bad_request)?;

    Ok(DeletionReceipt {
        id,
        deleted_at,
        expires_at: undo_expires_at(deleted_at)?,
    })
}

/// Locks series before tasks and rechecks membership against concurrent changes.
async fn collect_locked_subtree(
    executor: &mut dyn toasty::Executor,
    id: i64,
    owner_id: i64,
) -> Result<Vec<Task>, TaskError> {
    let root = Task::get_owned(executor, id, owner_id)
        .await?
        .ok_or_else(TaskError::related_resource_not_found)?;
    let selected = collect_subtree(executor, root).await?;
    lock_subtree(executor, &selected, owner_id).await?;
    let root = Task::get_owned(executor, id, owner_id)
        .await?
        .ok_or_else(TaskError::related_resource_not_found)?;
    let current = collect_subtree(executor, root).await?;
    let selected_ids: HashSet<_> = selected.iter().map(|task| task.id).collect();
    let current_ids: HashSet<_> = current.iter().map(|task| task.id).collect();
    if current_ids != selected_ids {
        return Err(TaskError::conflict(
            "This task's subtasks changed. Please try again.",
        ));
    }
    Ok(current)
}

async fn collect_subtree(
    executor: &mut dyn toasty::Executor,
    root: Task,
) -> Result<Vec<Task>, TaskError> {
    let owner_id = root.owner_id;
    let mut seen = HashSet::from([root.id]);
    let mut parent_ids = vec![root.id];
    let mut tasks = vec![root];
    while !parent_ids.is_empty() {
        let children = Task::not_deleted()
            .filter(Task::fields().parent_id().in_list(parent_ids))
            .exec(executor)
            .await?;
        parent_ids = Vec::new();
        for child in children {
            if child.owner_id != owner_id {
                return Err(TaskError::related_resource_not_found());
            }
            if seen.insert(child.id) {
                parent_ids.push(child.id);
                tasks.push(child);
            }
        }
    }
    Ok(tasks)
}

async fn stop_deleted_series(
    executor: &mut dyn toasty::Executor,
    tasks: &[Task],
    owner_id: i64,
    now: Timestamp,
) -> Result<BTreeSet<i64>, TaskError> {
    let active: Vec<_> = tasks
        .iter()
        .filter(|task| task.completed_at.is_none())
        .collect();
    let series_ids: BTreeSet<_> = active.iter().filter_map(|task| task.series_id).collect();
    let mut stopped = BTreeSet::new();
    for id in series_ids {
        let mut series = TaskSeries::lock_owned(executor, id, owner_id)
            .await?
            .ok_or_else(TaskError::related_resource_not_found)?;
        let Some(latest_id) = series.latest_task_id else {
            continue;
        };

        if series.stopped_at.is_none() && active.iter().any(|task| task.id == latest_id) {
            toasty::update!(series {
                stopped_at: Some(now),
                updated_at: now
            })
            .exec(executor)
            .await?;
            stopped.insert(latest_id);
        }
    }
    Ok(stopped)
}
