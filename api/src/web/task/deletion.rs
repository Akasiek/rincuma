use std::collections::{BTreeSet, HashSet};

use jiff::Timestamp;

use crate::{
    db::{Task, TaskSeries},
    web::task::{TaskError, locking::lock_subtree},
};

/// Locks every affected series before its tasks, then rechecks the subtree.
/// This shares undo's lock order even when a parent contains recurring children.
pub(super) async fn delete_subtree(
    executor: &mut dyn toasty::Executor,
    id: i64,
    owner_id: i64,
    now: Timestamp,
) -> Result<(), TaskError> {
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

    stop_deleted_series(executor, &current, owner_id, now).await?;
    for task in current.into_iter().rev() {
        Task::filter_by_id(task.id).delete().exec(executor).await?;
    }
    Ok(())
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
        let children = Task::filter(Task::fields().parent_id().in_list(parent_ids))
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
) -> Result<(), TaskError> {
    let active: Vec<_> = tasks
        .iter()
        .filter(|task| task.completed_at.is_none())
        .collect();
    let series_ids: BTreeSet<_> = active.iter().filter_map(|task| task.series_id).collect();

    for id in series_ids {
        let mut series = TaskSeries::lock_owned(executor, id, owner_id)
            .await?
            .ok_or_else(TaskError::related_resource_not_found)?;
        if active
            .iter()
            .any(|task| Some(task.id) == series.latest_task_id)
        {
            toasty::update!(series {
                stopped_at: Some(now),
                updated_at: now
            })
            .exec(executor)
            .await?;
        }
    }

    Ok(())
}
