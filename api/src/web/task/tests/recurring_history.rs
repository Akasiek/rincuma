#![allow(clippy::panic_in_result_fn)]

use axum::{
    Json,
    extract::{Query, State},
};
use jiff::Timestamp;

use super::{ListTasksQuery, TaskStatus, list};
use crate::{
    app_state::AppState,
    db::{SeriesEnd, Task, TaskSeries, User},
    web::{
        auth::CurrentUser,
        task::{
            recurring_support::{finish_at, recurring, reopen_at},
            response::TaskResponse,
        },
        test_support::state_with_user,
    },
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

async fn page(
    state: &AppState,
    owner_id: i64,
    request: ListTasksQuery,
) -> Result<crate::web::list::PageResponse<TaskResponse>, Box<dyn std::error::Error>> {
    let mut db = state.db().clone();
    let user = User::get_by_id(&mut db, owner_id).await?;
    let Json(page) = list(State(state.clone()), CurrentUser(user), Query(request)).await?;
    Ok(page)
}

async fn verify_pagination(state: &AppState, owner_id: i64, ids: &[i64]) -> TestResult {
    for offset in 0..4 {
        let paginated = page(
            state,
            owner_id,
            ListTasksQuery {
                status: TaskStatus::All,
                limit: Some(1),
                offset: Some(offset),
                ..ListTasksQuery::default()
            },
        )
        .await?;
        assert_eq!(paginated.total, 3);
        assert_eq!(paginated.items.len(), usize::from(offset < 3));
        assert!(paginated.items.iter().all(|task| ids.contains(&task.id)));
    }
    Ok(())
}

async fn verify_older_date_filter(state: &AppState, owner_id: i64) -> TestResult {
    let old_period = page(
        state,
        owner_id,
        ListTasksQuery {
            status: TaskStatus::Completed,
            due_from: Some("2028-01-01T00:00:00Z".parse()?),
            due_before: Some("2028-02-01T00:00:00Z".parse()?),
            ..ListTasksQuery::default()
        },
    )
    .await?;
    assert_eq!(old_period.total, 0);
    assert!(old_period.items.is_empty());
    Ok(())
}

async fn verify_history_visibility(state: &AppState, owner_id: i64) -> TestResult {
    let mut db = state.db().clone();
    let (first, series) = recurring(&mut db, owner_id, SeriesEnd::Never).await?;
    finish_at(state, owner_id, first.id, "2028-01-31T08:05:00Z").await?;
    let series = TaskSeries::get_by_id(&mut db, series.id).await?;
    let second_id = series.latest_task_id.ok_or("missing second occurrence")?;
    finish_at(state, owner_id, second_id, "2028-02-29T08:05:00Z").await?;
    let series = TaskSeries::get_by_id(&mut db, series.id).await?;
    let latest_completed_id = series.latest_task_id.ok_or("missing third occurrence")?;
    finish_at(state, owner_id, latest_completed_id, "2028-03-31T08:05:00Z").await?;
    let series = TaskSeries::get_by_id(&mut db, series.id).await?;
    let active_id = series.latest_task_id.ok_or("missing active occurrence")?;
    let now = Timestamp::now();
    let ordinary = toasty::create!(Task {
        owner_id,
        name: "Ordinary completed task".to_owned(),
        completed_at: Some(now),
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    let all = page(
        state,
        owner_id,
        ListTasksQuery {
            status: TaskStatus::All,
            ..ListTasksQuery::default()
        },
    )
    .await?;
    let ids: Vec<_> = all.items.iter().map(|task| task.id).collect();
    assert_eq!(all.total, 3);
    assert!(ids.contains(&latest_completed_id));
    assert!(ids.contains(&active_id));
    assert!(ids.contains(&ordinary.id));
    assert!(!ids.contains(&first.id));
    assert!(!ids.contains(&second_id));
    verify_pagination(state, owner_id, &ids).await?;
    let completed = page(
        state,
        owner_id,
        ListTasksQuery {
            status: TaskStatus::Completed,
            ..ListTasksQuery::default()
        },
    )
    .await?;
    assert_eq!(completed.total, 2);
    verify_older_date_filter(state, owner_id).await?;
    reopen_at(state, owner_id, latest_completed_id, "2028-03-31T08:10:00Z").await?;
    let completed = page(
        state,
        owner_id,
        ListTasksQuery {
            status: TaskStatus::Completed,
            ..ListTasksQuery::default()
        },
    )
    .await?;
    assert_eq!(completed.total, 2);
    assert!(completed.items.iter().any(|task| task.id == second_id));
    assert!(
        !completed
            .items
            .iter()
            .any(|task| task.id == latest_completed_id)
    );
    assert!(
        Task::get_by_id(&mut db, first.id)
            .await?
            .completed_at
            .is_some()
    );
    Ok(())
}

#[tokio::test]
async fn recurring_history_retains_latest_completed_and_preserves_pagination_and_undo() -> TestResult
{
    let (state, user) = state_with_user().await?;
    verify_history_visibility(&state, user.id).await
}

#[tokio::test]
#[ignore = "requires an empty PostgreSQL database in RINCUMA_TEST_POSTGRES_URL"]
async fn postgres_recurring_history_visibility() -> TestResult {
    use crate::config::AppConfig;
    use std::path::PathBuf;
    let url = std::env::var("RINCUMA_TEST_POSTGRES_URL")?;
    let mut db = toasty::Db::builder()
        .models(toasty::models!(crate::*))
        .connect(&url)
        .await?;
    db.push_schema().await?;
    let now = Timestamp::now();
    let user = toasty::create!(User {
        email: "history@example.com".to_owned(),
        password_hash: "unused".to_owned(),
        created_at: now,
        updated_at: now
    })
    .exec(&mut db)
    .await?;
    let state = AppState::new(AppConfig::for_test(PathBuf::new()), db);
    verify_history_visibility(&state, user.id).await
}

#[tokio::test]
async fn completed_history_is_retained_per_series_even_after_stopping() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let mut retained = Vec::new();
    for end in [SeriesEnd::After { count: 1 }, SeriesEnd::Never] {
        let (task, mut series) = recurring(&mut db, user.id, end).await?;
        finish_at(&state, user.id, task.id, "2028-01-31T08:05:00Z").await?;
        retained.push(task.id);
        toasty::update!(series {
            stopped_at: Some(Timestamp::now())
        })
        .exec(&mut db)
        .await?;
    }
    let completed = page(
        &state,
        user.id,
        ListTasksQuery {
            status: TaskStatus::Completed,
            ..ListTasksQuery::default()
        },
    )
    .await?;
    assert_eq!(completed.total, 2);
    for id in retained {
        assert!(completed.items.iter().any(|task| task.id == id));
    }
    Ok(())
}
