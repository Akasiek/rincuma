use std::{path::PathBuf, time::Duration};

use jiff::{SignedDuration, Timestamp};

use crate::{
    app_state::AppState,
    config::AppConfig,
    db::{SeriesEnd, Tag, Task, TaskSeries, TaskTag, User},
    web::task::{
        TaskError,
        recurring_support::recurring,
        restoration::{delete_with_undo, restore_deletion},
    },
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[tokio::test]
#[ignore = "requires an empty PostgreSQL database in RINCUMA_TEST_POSTGRES_URL"]
async fn postgres_deletion_undo_restores_ids_and_serializes_conflicts() -> TestResult {
    let url = std::env::var("RINCUMA_TEST_POSTGRES_URL")?;
    let mut db = toasty::Db::builder()
        .models(toasty::models!(crate::*))
        .connect(&url)
        .await?;
    db.push_schema().await?;
    let now = Timestamp::now();
    let user = toasty::create!(User {
        email: "delete-undo@example.com".to_owned(),
        password_hash: "unused".to_owned(),
        created_at: now,
        updated_at: now
    })
    .exec(&mut db)
    .await?;
    let state = AppState::new(AppConfig::for_test(PathBuf::new()), db);
    restores_once(&state, user.id).await?;
    series_edit_wins(&state, user.id).await?;
    Ok(())
}

async fn restores_once(state: &AppState, owner_id: i64) -> TestResult {
    let mut db = state.db().clone();
    let (task, series) = recurring(&mut db, owner_id, SeriesEnd::Never).await?;
    let now = Timestamp::now();
    let tag = toasty::create!(Tag {
        name: "Original tag".to_owned(),
        owner_id,
        created_at: now,
        updated_at: now
    })
    .exec(&mut db)
    .await?;
    toasty::create!(TaskTag {
        task_id: task.id,
        tag_id: tag.id
    })
    .exec(&mut db)
    .await?;
    let mut tx = db.transaction().await?;
    let deletion = delete_with_undo(&mut tx, task.id, owner_id, now).await?;
    tx.commit().await?;
    let hidden = Task::get_by_id(&mut db, task.id).await?;
    assert!(hidden.deleted_at.is_some());
    assert_eq!(hidden.deletion_root_id, Some(deletion.id));
    assert!(Task::get_owned(&mut db, task.id, owner_id).await?.is_none());
    assert_eq!(
        TaskTag::filter_by_task_id(task.id)
            .exec(&mut db)
            .await?
            .len(),
        1
    );
    let mut tx = db.transaction().await?;
    restore_deletion(&mut tx, deletion.id, owner_id, deletion.deleted_at, now).await?;
    let second_state = state.clone();
    let mut duplicate = tokio::spawn(async move {
        let mut db = second_state.db().clone();
        let mut tx = db.transaction().await?;
        restore_deletion(&mut tx, deletion.id, owner_id, deletion.deleted_at, now).await?;
        tx.commit().await?;
        Ok::<_, TaskError>(())
    });
    assert!(
        tokio::time::timeout(Duration::from_millis(100), &mut duplicate)
            .await
            .is_err()
    );
    tx.commit().await?;
    assert!(matches!(
        duplicate.await?,
        Err(TaskError::RelatedResourceNotFound(_))
    ));
    let restored = Task::get_by_id(&mut db, task.id).await?;
    assert_eq!(restored.name, task.name);
    assert_eq!(restored.priority, task.priority);
    assert_eq!(restored.scheduled_at, task.scheduled_at);
    assert_eq!(restored.series_id, Some(series.id));
    assert_eq!(
        TaskTag::filter_by_task_id(task.id)
            .exec(&mut db)
            .await?
            .first()
            .map(|link| link.tag_id),
        Some(tag.id)
    );
    assert!(
        TaskSeries::get_by_id(&mut db, series.id)
            .await?
            .stopped_at
            .is_none()
    );
    let (next, _) = recurring(&mut db, owner_id, SeriesEnd::Never).await?;
    assert!(next.id > task.id);
    Ok(())
}

async fn series_edit_wins(state: &AppState, owner_id: i64) -> TestResult {
    let mut db = state.db().clone();
    let (task, series) = recurring(&mut db, owner_id, SeriesEnd::Never).await?;
    let now = Timestamp::now();
    let mut tx = db.transaction().await?;
    let deletion = delete_with_undo(&mut tx, task.id, owner_id, now).await?;
    tx.commit().await?;
    let mut tx = db.transaction().await?;
    let mut current = TaskSeries::lock_owned(&mut tx, series.id, owner_id)
        .await?
        .ok_or("missing series")?;
    let changed_at = now.checked_add(SignedDuration::from_secs(1))?;
    toasty::update!(current {
        updated_at: changed_at
    })
    .exec(&mut tx)
    .await?;
    let second_state = state.clone();
    let mut undo = tokio::spawn(async move {
        let mut db = second_state.db().clone();
        let mut tx = db.transaction().await?;
        restore_deletion(
            &mut tx,
            deletion.id,
            owner_id,
            deletion.deleted_at,
            changed_at,
        )
        .await?;
        tx.commit().await?;
        Ok::<_, TaskError>(())
    });
    assert!(
        tokio::time::timeout(Duration::from_millis(100), &mut undo)
            .await
            .is_err()
    );
    tx.commit().await?;
    assert!(matches!(undo.await?, Err(TaskError::Conflict(..))));
    assert!(Task::get_owned(&mut db, task.id, owner_id).await?.is_none());
    Ok(())
}
