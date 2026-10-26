#![allow(clippy::panic_in_result_fn)]

use crate::web::task::recurrence::RecurrenceChange;

use std::{path::PathBuf, time::Duration};

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use jiff::Timestamp;
use tokio::task::JoinHandle;

use crate::{
    app_state::AppState,
    config::AppConfig,
    db::{SeriesEnd, Task, TaskSeries, User},
    web::{
        auth::CurrentUser,
        task::{
            completion::complete_locked,
            deletion::soft_delete_subtree,
            handlers::{reopen, update},
            locking::lock_owned_task,
            recurring_support::{finish_at, occurrences, recurring},
            reopening::reopen_locked,
            request::SaveTaskRequest,
        },
    },
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

async fn pair(
    state: &AppState,
    owner_id: i64,
) -> Result<(Task, Task, TaskSeries), Box<dyn std::error::Error>> {
    let mut db = state.db().clone();
    let (original, series) = recurring(&mut db, owner_id, SeriesEnd::After { count: 3 }).await?;
    let original = finish_at(state, owner_id, original.id, "2028-01-31T08:05:00Z").await?;
    let series = TaskSeries::get_by_id(&mut db, series.id).await?;
    let successor =
        Task::get_by_id(&mut db, series.latest_task_id.ok_or("missing successor")?).await?;
    Ok((original, successor, series))
}

async fn assert_waiting<T>(job: &mut JoinHandle<T>) -> TestResult {
    assert!(
        tokio::time::timeout(Duration::from_millis(100), job)
            .await
            .is_err()
    );
    Ok(())
}

async fn edit_wins(state: &AppState, owner_id: i64) -> TestResult {
    let mut db = state.db().clone();
    let (original, successor, series) = pair(state, owner_id).await?;
    let user = User::get_by_id(&mut db, owner_id).await?;
    let mut transaction = db.transaction().await?;
    let mut locked = lock_owned_task(&mut transaction, successor.id, owner_id).await?;
    let now: Timestamp = "2028-02-01T10:00:00Z".parse()?;
    toasty::update!(locked {
        name: "Edited while undo waits".to_owned(),
        recurrence_modified_at: Some(now),
        updated_at: now
    })
    .exec(&mut transaction)
    .await?;
    let second_state = state.clone();
    let mut undo = tokio::spawn(async move {
        reopen(State(second_state), CurrentUser(user), Path(original.id)).await
    });
    assert_waiting(&mut undo).await?;
    transaction.commit().await?;
    let error = undo
        .await?
        .err()
        .ok_or("expected edited successor rejection")?;
    assert_eq!(error.into_response().status(), StatusCode::CONFLICT);
    assert_eq!(
        Task::get_by_id(&mut db, successor.id).await?.name,
        "Edited while undo waits"
    );
    assert_eq!(
        TaskSeries::get_by_id(&mut db, series.id)
            .await?
            .occurrence_count,
        2
    );
    Ok(())
}

async fn undo_wins_over_an_edit(state: &AppState, owner_id: i64) -> TestResult {
    let mut db = state.db().clone();
    let (original, successor, series) = pair(state, owner_id).await?;
    let user = User::get_by_id(&mut db, owner_id).await?;
    let mut transaction = db.transaction().await?;
    let mut locked = lock_owned_task(&mut transaction, original.id, owner_id).await?;
    reopen_locked(
        &mut transaction,
        &mut locked,
        "2028-02-01T10:00:00Z".parse()?,
    )
    .await?;
    let request = SaveTaskRequest {
        recurrence: RecurrenceChange::Keep,
        name: "Late edit".to_owned(),
        description: successor.description,
        due_at: successor.due_at,
        priority: Some(successor.priority),
        project_id: successor.project_id,
        parent_id: successor.parent_id,
        tag_ids: Vec::new(),
    };
    let second_state = state.clone();
    let mut edit = tokio::spawn(async move {
        update(
            State(second_state),
            CurrentUser(user),
            Path(successor.id),
            Json(request),
        )
        .await
    });
    assert_waiting(&mut edit).await?;
    transaction.commit().await?;
    let error = edit
        .await?
        .err()
        .ok_or("expected deleted successor rejection")?;
    assert_eq!(error.into_response().status(), StatusCode::NOT_FOUND);
    assert_eq!(occurrences(&mut db, series.id).await?.len(), 1);
    assert_eq!(
        TaskSeries::get_by_id(&mut db, series.id)
            .await?
            .occurrence_count,
        1
    );
    Ok(())
}

async fn repeated_undo_is_serialized(state: &AppState, owner_id: i64) -> TestResult {
    let mut db = state.db().clone();
    let (original, _, series) = pair(state, owner_id).await?;
    let user = User::get_by_id(&mut db, owner_id).await?;
    let mut transaction = db.transaction().await?;
    let mut locked = lock_owned_task(&mut transaction, original.id, owner_id).await?;
    reopen_locked(
        &mut transaction,
        &mut locked,
        "2028-02-01T10:00:00Z".parse()?,
    )
    .await?;
    let second_state = state.clone();
    let mut undo = tokio::spawn(async move {
        reopen(State(second_state), CurrentUser(user), Path(original.id)).await
    });
    assert_waiting(&mut undo).await?;
    transaction.commit().await?;
    let Json(task) = undo.await??;
    assert_eq!(task.completed_at, None);
    assert_eq!(occurrences(&mut db, series.id).await?.len(), 1);
    assert_eq!(
        TaskSeries::get_by_id(&mut db, series.id)
            .await?
            .occurrence_count,
        1
    );
    Ok(())
}

async fn successor_completion_wins(state: &AppState, owner_id: i64) -> TestResult {
    let mut db = state.db().clone();
    let (original, successor, series) = pair(state, owner_id).await?;
    let user = User::get_by_id(&mut db, owner_id).await?;
    let mut transaction = db.transaction().await?;
    let mut locked = lock_owned_task(&mut transaction, successor.id, owner_id).await?;
    complete_locked(
        &mut transaction,
        &mut locked,
        &[],
        "2028-02-29T08:05:00Z".parse()?,
    )
    .await?;
    let second_state = state.clone();
    let mut undo = tokio::spawn(async move {
        reopen(State(second_state), CurrentUser(user), Path(original.id)).await
    });
    assert_waiting(&mut undo).await?;
    transaction.commit().await?;
    let error = undo
        .await?
        .err()
        .ok_or("expected completed successor rejection")?;
    assert_eq!(error.into_response().status(), StatusCode::CONFLICT);
    assert_eq!(occurrences(&mut db, series.id).await?.len(), 3);
    assert_eq!(
        TaskSeries::get_by_id(&mut db, series.id)
            .await?
            .occurrence_count,
        3
    );
    Ok(())
}

async fn successor_deletion_wins(state: &AppState, owner_id: i64) -> TestResult {
    let mut db = state.db().clone();
    let (original, successor, series) = pair(state, owner_id).await?;
    let user = User::get_by_id(&mut db, owner_id).await?;
    let mut transaction = db.transaction().await?;
    soft_delete_subtree(
        &mut transaction,
        successor.id,
        owner_id,
        "2028-02-01T10:00:00Z".parse()?,
    )
    .await?;
    let second_state = state.clone();
    let mut undo = tokio::spawn(async move {
        reopen(State(second_state), CurrentUser(user), Path(original.id)).await
    });
    assert_waiting(&mut undo).await?;
    transaction.commit().await?;
    let error = undo
        .await?
        .err()
        .ok_or("expected stopped series rejection")?;
    assert_eq!(error.into_response().status(), StatusCode::CONFLICT);
    assert!(
        TaskSeries::get_by_id(&mut db, series.id)
            .await?
            .stopped_at
            .is_some()
    );
    assert!(
        Task::get_by_id(&mut db, original.id)
            .await?
            .completed_at
            .is_some()
    );
    Ok(())
}

#[tokio::test]
#[ignore = "requires an empty PostgreSQL database in RINCUMA_TEST_POSTGRES_URL"]
async fn postgres_undo_races_preserve_edits_completion_and_counts() -> TestResult {
    let url = std::env::var("RINCUMA_TEST_POSTGRES_URL")?;
    let mut db = toasty::Db::builder()
        .models(toasty::models!(crate::*))
        .connect(&url)
        .await?;
    db.push_schema().await?;
    let now: Timestamp = "2028-01-31T08:00:00Z".parse()?;
    let user = toasty::create!(User {
        email: "undo-races@example.com".to_owned(),
        password_hash: "unused".to_owned(),
        created_at: now,
        updated_at: now
    })
    .exec(&mut db)
    .await?;
    let state = AppState::new(AppConfig::for_test(PathBuf::new()), db);
    edit_wins(&state, user.id).await?;
    undo_wins_over_an_edit(&state, user.id).await?;
    repeated_undo_is_serialized(&state, user.id).await?;
    successor_completion_wins(&state, user.id).await?;
    successor_deletion_wins(&state, user.id).await?;
    Ok(())
}
