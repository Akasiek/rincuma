use std::collections::{HashMap, HashSet};

use crate::db::{Task, TaskTag};
use toasty::stmt::{List, Query};

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum TaskIdentity {
    Task(i64),
    Series(i64),
}

impl TaskIdentity {
    fn from_task(task: &Task) -> Self {
        task.series_id.map_or(Self::Task(task.id), Self::Series)
    }
}

pub(in crate::web) async fn projects(
    db: &mut dyn toasty::Executor,
    owner_id: i64,
    ids: &[i64],
) -> toasty::Result<HashMap<i64, usize>> {
    if ids.is_empty() {
        return Ok(HashMap::new());
    }

    let tasks = Task::not_deleted()
        .filter(Task::fields().owner_id().eq(owner_id))
        .filter(Task::fields().project_id().in_list(ids))
        .exec(db)
        .await?;

    Ok(count_unique_tasks(tasks.iter().filter_map(|task| {
        task.project_id
            .map(|id| (id, TaskIdentity::from_task(task)))
    })))
}

pub(in crate::web) async fn tags(
    db: &mut dyn toasty::Executor,
    owner_id: i64,
    ids: &[i64],
) -> toasty::Result<HashMap<i64, usize>> {
    if ids.is_empty() {
        return Ok(HashMap::new());
    }

    let links = Query::<List<TaskTag>>::all()
        .filter(TaskTag::fields().tag_id().in_list(ids))
        .exec(db)
        .await?;
    if links.is_empty() {
        return Ok(HashMap::new());
    }

    let task_ids: Vec<_> = links.iter().map(|link| link.task_id).collect();
    let tasks = Task::not_deleted()
        .filter(Task::fields().owner_id().eq(owner_id))
        .filter(Task::fields().id().in_list(task_ids))
        .exec(db)
        .await?;
    let identities: HashMap<_, _> = tasks
        .iter()
        .map(|task| (task.id, TaskIdentity::from_task(task)))
        .collect();

    Ok(count_unique_tasks(links.into_iter().filter_map(|link| {
        identities
            .get(&link.task_id)
            .map(|identity| (link.tag_id, *identity))
    })))
}

fn count_unique_tasks(tasks: impl IntoIterator<Item = (i64, TaskIdentity)>) -> HashMap<i64, usize> {
    let mut groups: HashMap<i64, HashSet<TaskIdentity>> = HashMap::new();
    for (id, identity) in tasks {
        groups.entry(id).or_default().insert(identity);
    }

    groups
        .into_iter()
        .map(|(id, tasks)| (id, tasks.len()))
        .collect()
}
