use std::collections::HashMap;

use axum::{
    Json,
    extract::{Query, State},
};
use jiff::Timestamp;
use serde::Deserialize;
use toasty::stmt::{List, Query as DbQuery};

use crate::{
    app_state::AppState,
    db::{Task, TaskPriority, TaskTag},
    web::{
        auth::CurrentUser,
        list::{Page, PageResponse, SortOrder},
        task::{TaskError, response::TaskResponse},
    },
};

#[derive(Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub(super) enum TaskStatus {
    #[default]
    Active,
    Completed,
    Overdue,
    Upcoming,
    All,
}

#[derive(Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub(super) enum TaskSort {
    Name,
    DueAt,
    #[default]
    CreatedAt,
    UpdatedAt,
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub(in crate::web::task) struct ListTasksQuery {
    status: TaskStatus,
    project_id: Option<i64>,
    priority: Option<TaskPriority>,
    tag_id: Option<i64>,
    sort_by: TaskSort,
    order: SortOrder,
    limit: Option<usize>,
    offset: Option<usize>,
}

pub(in crate::web::task) async fn list(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Query(request): Query<ListTasksQuery>,
) -> Result<Json<PageResponse<TaskResponse>>, TaskError> {
    let page = Page::new(request.limit, request.offset).ok_or_else(TaskError::bad_request)?;
    if request.project_id.is_some_and(|id| id <= 0) || request.tag_id.is_some_and(|id| id <= 0) {
        return Err(TaskError::bad_request());
    }

    let mut db = state.db().clone();
    let now = Timestamp::now();
    let tagged_task_ids = fetch_task_ids_by_tag(&mut db, request.tag_id).await?;
    if tagged_task_ids.as_ref().is_some_and(Vec::is_empty) {
        return Ok(Json(PageResponse::new(Vec::new(), 0, &page)));
    }

    let total = filtered_tasks(user.id, &request, tagged_task_ids.as_deref(), now)
        .count()
        .exec(&mut db)
        .await?;
    let tasks = sorted_tasks(
        filtered_tasks(user.id, &request, tagged_task_ids.as_deref(), now),
        request.sort_by,
        request.order,
    )
    .limit(page.limit)
    .offset(page.offset)
    .exec(&mut db)
    .await?;

    let mut tag_ids_by_task = fetch_tag_ids_for_tasks(&mut db, &tasks).await?;
    let items = tasks
        .into_iter()
        .map(|task| {
            let tag_ids = tag_ids_by_task.remove(&task.id).unwrap_or_default();
            TaskResponse::new(task, tag_ids)
        })
        .collect();

    Ok(Json(PageResponse::new(items, total, &page)))
}

async fn fetch_task_ids_by_tag(
    db: &mut toasty::Db,
    tag_id: Option<i64>,
) -> Result<Option<Vec<i64>>, TaskError> {
    let Some(tag_id) = tag_id else {
        return Ok(None);
    };

    let links = TaskTag::filter_by_tag_id(tag_id).exec(db).await?;
    Ok(Some(links.into_iter().map(|link| link.task_id).collect()))
}

fn filtered_tasks(
    owner_id: i64,
    request: &ListTasksQuery,
    tagged_task_ids: Option<&[i64]>,
    now: Timestamp,
) -> DbQuery<List<Task>> {
    let mut query = DbQuery::<List<Task>>::all().filter(Task::fields().owner_id().eq(owner_id));
    query = match request.status {
        TaskStatus::Active => query.filter(Task::fields().completed_at().is_none()),
        TaskStatus::Completed => query.filter(Task::fields().completed_at().is_some()),
        TaskStatus::Overdue => query
            .filter(Task::fields().completed_at().is_none())
            .filter(Task::fields().due_at().lt(Some(now))),
        TaskStatus::Upcoming => query
            .filter(Task::fields().completed_at().is_none())
            .filter(Task::fields().due_at().gt(Some(now))),
        TaskStatus::All => query,
    };
    if let Some(project_id) = request.project_id {
        query = query.filter(Task::fields().project_id().eq(project_id));
    }
    if let Some(priority) = request.priority {
        query = query.filter(Task::fields().priority().eq(priority));
    }
    if let Some(tagged_task_ids) = tagged_task_ids {
        query = query.filter(Task::fields().id().in_list(tagged_task_ids));
    }
    query
}

fn sorted_tasks(
    query: DbQuery<List<Task>>,
    sort_by: TaskSort,
    order: SortOrder,
) -> DbQuery<List<Task>> {
    match (sort_by, order) {
        (TaskSort::Name, SortOrder::Asc) => {
            query.order_by((Task::fields().name().asc(), Task::fields().id().asc()))
        }
        (TaskSort::Name, SortOrder::Desc) => {
            query.order_by((Task::fields().name().desc(), Task::fields().id().desc()))
        }
        (TaskSort::DueAt, SortOrder::Asc) => {
            query.order_by((Task::fields().due_at().asc(), Task::fields().id().asc()))
        }
        (TaskSort::DueAt, SortOrder::Desc) => {
            query.order_by((Task::fields().due_at().desc(), Task::fields().id().desc()))
        }
        (TaskSort::CreatedAt, SortOrder::Asc) => {
            query.order_by((Task::fields().created_at().asc(), Task::fields().id().asc()))
        }
        (TaskSort::CreatedAt, SortOrder::Desc) => query.order_by((
            Task::fields().created_at().desc(),
            Task::fields().id().desc(),
        )),
        (TaskSort::UpdatedAt, SortOrder::Asc) => {
            query.order_by((Task::fields().updated_at().asc(), Task::fields().id().asc()))
        }
        (TaskSort::UpdatedAt, SortOrder::Desc) => query.order_by((
            Task::fields().updated_at().desc(),
            Task::fields().id().desc(),
        )),
    }
}

async fn fetch_tag_ids_for_tasks(
    db: &mut toasty::Db,
    tasks: &[Task],
) -> Result<HashMap<i64, Vec<i64>>, TaskError> {
    let mut tag_ids_by_task = HashMap::new();
    if tasks.is_empty() {
        return Ok(tag_ids_by_task);
    }

    let task_ids: Vec<_> = tasks.iter().map(|task| task.id).collect();
    let links = DbQuery::<List<TaskTag>>::all()
        .filter(TaskTag::fields().task_id().in_list(task_ids))
        .order_by((
            TaskTag::fields().task_id().asc(),
            TaskTag::fields().tag_id().asc(),
        ))
        .exec(db)
        .await?;

    for link in links {
        tag_ids_by_task
            .entry(link.task_id)
            .or_insert_with(Vec::new)
            .push(link.tag_id);
    }

    Ok(tag_ids_by_task)
}

#[cfg(test)]
#[path = "../tests/list.rs"]
mod tests;
