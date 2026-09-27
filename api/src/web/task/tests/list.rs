use axum::{
    Json,
    body::Body,
    extract::{Query, State},
    http::{Request, StatusCode, Uri},
};
use jiff::Timestamp;
use tower::ServiceExt;

use super::{ListTasksQuery, TaskSort, TaskStatus, list};
use crate::{
    app_state::AppState,
    db::{Project, Tag, Task, TaskPriority, TaskTag, User},
    web::{
        auth::CurrentUser,
        list::{PageResponse, SortOrder},
        router::get_app_router,
        task::{TaskError, response::TaskResponse},
        test_support::state_with_user,
    },
};

async fn insert_task(
    db: &mut toasty::Db,
    owner_id: i64,
    name: &str,
    created_at: Timestamp,
    completed_at: Option<Timestamp>,
    project_id: Option<i64>,
    priority: TaskPriority,
) -> toasty::Result<Task> {
    toasty::create!(Task {
        name: name.to_owned(),
        owner_id,
        created_at,
        updated_at: created_at,
        completed_at,
        project_id,
        priority,
    })
    .exec(db)
    .await
}

async fn insert_due_task(
    db: &mut toasty::Db,
    owner_id: i64,
    name: &str,
    due_at: Option<Timestamp>,
    completed_at: Option<Timestamp>,
) -> toasty::Result<Task> {
    let now = Timestamp::now();
    toasty::create!(Task {
        name: name.to_owned(),
        owner_id,
        due_at,
        completed_at,
        created_at: now,
        updated_at: now,
    })
    .exec(db)
    .await
}

async fn list_for(
    state: &AppState,
    owner_id: i64,
    request: ListTasksQuery,
) -> Result<PageResponse<TaskResponse>, TaskError> {
    let mut db = state.db().clone();
    let user = User::get_by_id(&mut db, &owner_id).await?;
    let Json(response) = list(State(state.clone()), CurrentUser(user), Query(request)).await?;
    Ok(response)
}

#[tokio::test]
async fn lists_only_owned_active_tasks_by_default() -> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let old = Timestamp::from_second(1)?;
    let new = Timestamp::from_second(2)?;
    let first = insert_task(
        &mut db,
        user.id,
        "First",
        old,
        None,
        None,
        TaskPriority::None,
    )
    .await?;
    let second = insert_task(
        &mut db,
        user.id,
        "Second",
        new,
        None,
        None,
        TaskPriority::High,
    )
    .await?;
    insert_task(
        &mut db,
        user.id,
        "Completed",
        new,
        Some(new),
        None,
        TaskPriority::None,
    )
    .await?;
    let other = toasty::create!(User {
        email: "other@example.com".to_owned(),
        password_hash: "unused".to_owned(),
        created_at: old,
        updated_at: old,
    })
    .exec(&mut db)
    .await?;
    insert_task(
        &mut db,
        other.id,
        "Other",
        new,
        None,
        None,
        TaskPriority::None,
    )
    .await?;

    let response = list_for(&state, user.id, ListTasksQuery::default()).await?;

    assert_eq!(response.total, 2);
    assert_eq!(response.limit, 50);
    assert_eq!(response.offset, 0);
    let ids: Vec<_> = response.items.iter().map(|task| task.id).collect();
    assert_eq!(ids, [second.id, first.id]);
    assert!(response.items.iter().all(|task| task.owner_id == user.id));
    Ok(())
}

#[tokio::test]
async fn filters_overdue_and_upcoming_tasks() -> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let past = Timestamp::from_second(1)?;
    let future = Timestamp::from_second(4_000_000_000)?;
    let overdue = insert_due_task(&mut db, user.id, "Overdue", Some(past), None).await?;
    let upcoming = insert_due_task(&mut db, user.id, "Future", Some(future), None).await?;
    insert_due_task(&mut db, user.id, "No deadline", None, None).await?;
    insert_due_task(&mut db, user.id, "Completed", Some(past), Some(past)).await?;
    insert_due_task(
        &mut db,
        user.id,
        "Completed future",
        Some(future),
        Some(past),
    )
    .await?;

    let response = list_for(
        &state,
        user.id,
        ListTasksQuery {
            status: TaskStatus::Overdue,
            ..ListTasksQuery::default()
        },
    )
    .await?;
    assert_eq!(response.total, 1);
    assert_eq!(response.items.first().map(|task| task.id), Some(overdue.id));

    let response = list_for(
        &state,
        user.id,
        ListTasksQuery {
            status: TaskStatus::Upcoming,
            ..ListTasksQuery::default()
        },
    )
    .await?;
    assert_eq!(response.total, 1);
    assert_eq!(
        response.items.first().map(|task| task.id),
        Some(upcoming.id)
    );

    let completed = list_for(
        &state,
        user.id,
        ListTasksQuery {
            status: TaskStatus::Completed,
            ..ListTasksQuery::default()
        },
    )
    .await?;
    assert_eq!(completed.total, 2);

    let unfiltered = list_for(&state, user.id, ListTasksQuery::default()).await?;
    assert_eq!(unfiltered.total, 3);
    Ok(())
}

#[tokio::test]
async fn filters_tasks_and_counts_the_same_scope() -> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let now = Timestamp::now();
    let project = toasty::create!(Project {
        name: "Project".to_owned(),
        owner_id: user.id,
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    let tag = toasty::create!(Tag {
        name: "Tag".to_owned(),
        owner_id: user.id,
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    let matching = insert_task(
        &mut db,
        user.id,
        "Matching",
        now,
        Some(now),
        Some(project.id),
        TaskPriority::High,
    )
    .await?;
    insert_task(
        &mut db,
        user.id,
        "Active",
        now,
        None,
        Some(project.id),
        TaskPriority::High,
    )
    .await?;
    insert_task(
        &mut db,
        user.id,
        "Other priority",
        now,
        Some(now),
        Some(project.id),
        TaskPriority::Low,
    )
    .await?;
    insert_task(
        &mut db,
        user.id,
        "No project",
        now,
        Some(now),
        None,
        TaskPriority::High,
    )
    .await?;
    toasty::create!(TaskTag {
        task_id: matching.id,
        tag_id: tag.id,
    })
    .exec(&mut db)
    .await?;

    let response = list_for(
        &state,
        user.id,
        ListTasksQuery {
            status: TaskStatus::Completed,
            project_id: Some(project.id),
            priority: Some(TaskPriority::High),
            tag_id: Some(tag.id),
            ..ListTasksQuery::default()
        },
    )
    .await?;
    assert_eq!(response.total, 1);
    assert_eq!(
        response.items.first().map(|task| task.id),
        Some(matching.id)
    );
    assert_eq!(
        response.items.first().map(|task| task.tag_ids.as_slice()),
        Some([tag.id].as_slice())
    );

    let all = list_for(
        &state,
        user.id,
        ListTasksQuery {
            status: TaskStatus::All,
            ..ListTasksQuery::default()
        },
    )
    .await?;
    assert_eq!(all.total, 4);
    Ok(())
}

#[tokio::test]
async fn sorts_and_paginates_tasks_with_tag_ids() -> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let old = Timestamp::from_second(1)?;
    let new = Timestamp::from_second(2)?;
    let first = insert_task(
        &mut db,
        user.id,
        "Zulu",
        old,
        None,
        None,
        TaskPriority::None,
    )
    .await?;
    let second = insert_task(
        &mut db,
        user.id,
        "Alpha",
        new,
        None,
        None,
        TaskPriority::None,
    )
    .await?;
    let first_tag = toasty::create!(Tag {
        name: "First".to_owned(),
        owner_id: user.id,
        created_at: old,
        updated_at: old,
    })
    .exec(&mut db)
    .await?;
    let second_tag = toasty::create!(Tag {
        name: "Second".to_owned(),
        owner_id: user.id,
        created_at: old,
        updated_at: old,
    })
    .exec(&mut db)
    .await?;
    for tag_id in [second_tag.id, first_tag.id] {
        toasty::create!(TaskTag {
            task_id: second.id,
            tag_id,
        })
        .exec(&mut db)
        .await?;
    }

    let page = list_for(
        &state,
        user.id,
        ListTasksQuery {
            sort_by: TaskSort::Name,
            order: SortOrder::Asc,
            limit: Some(1),
            offset: Some(0),
            ..ListTasksQuery::default()
        },
    )
    .await?;
    assert_eq!(page.total, 2);
    assert_eq!(page.limit, 1);
    assert_eq!(page.items.first().map(|task| task.id), Some(second.id));
    assert_eq!(
        page.items.first().map(|task| task.tag_ids.as_slice()),
        Some([first_tag.id, second_tag.id].as_slice())
    );

    let next_page = list_for(
        &state,
        user.id,
        ListTasksQuery {
            limit: Some(1),
            offset: Some(1),
            ..ListTasksQuery::default()
        },
    )
    .await?;
    assert_eq!(next_page.items.first().map(|task| task.id), Some(first.id));
    assert_eq!(next_page.offset, 1);
    Ok(())
}

#[tokio::test]
async fn rejects_invalid_page_and_filter_ids() -> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    for request in [
        ListTasksQuery {
            limit: Some(0),
            ..ListTasksQuery::default()
        },
        ListTasksQuery {
            limit: Some(101),
            ..ListTasksQuery::default()
        },
        ListTasksQuery {
            offset: Some(usize::MAX),
            ..ListTasksQuery::default()
        },
        ListTasksQuery {
            project_id: Some(0),
            ..ListTasksQuery::default()
        },
        ListTasksQuery {
            tag_id: Some(-1),
            ..ListTasksQuery::default()
        },
    ] {
        let result = list_for(&state, user.id, request).await;
        assert!(matches!(result, Err(TaskError::BadRequest(_))));
    }
    Ok(())
}

#[test]
fn parses_list_query_parameters() -> Result<(), Box<dyn std::error::Error>> {
    let uri: Uri = "/tasks?status=completed&project_id=2&priority=high&tag_id=3&sort_by=due_at&order=asc&limit=10&offset=5".parse()?;
    let Query(query) = Query::<ListTasksQuery>::try_from_uri(&uri)?;
    assert!(matches!(query.status, TaskStatus::Completed));
    assert_eq!(query.project_id, Some(2));
    assert_eq!(query.priority, Some(TaskPriority::High));
    assert_eq!(query.tag_id, Some(3));
    assert!(matches!(query.sort_by, TaskSort::DueAt));
    assert!(matches!(query.order, SortOrder::Asc));
    assert_eq!(query.limit, Some(10));
    assert_eq!(query.offset, Some(5));

    let overdue: Uri = "/tasks?status=overdue".parse()?;
    let Query(query) = Query::<ListTasksQuery>::try_from_uri(&overdue)?;
    assert!(matches!(query.status, TaskStatus::Overdue));

    let upcoming: Uri = "/tasks?status=upcoming".parse()?;
    let Query(query) = Query::<ListTasksQuery>::try_from_uri(&upcoming)?;
    assert!(matches!(query.status, TaskStatus::Upcoming));

    let invalid: Uri = "/tasks?status=invalid".parse()?;
    assert!(Query::<ListTasksQuery>::try_from_uri(&invalid).is_err());
    Ok(())
}

#[tokio::test]
async fn list_route_requires_authentication() -> Result<(), Box<dyn std::error::Error>> {
    let (state, _) = state_with_user().await?;
    let request = Request::builder().uri("/tasks").body(Body::empty())?;
    let response = get_app_router(state).oneshot(request).await?;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    Ok(())
}
