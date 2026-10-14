#![allow(clippy::panic_in_result_fn)]

use super::{check_owner_tasks, deliver_reminder};
use crate::{
    db::{NotificationProviderKind, NotificationSettings, Task},
    notifications::{Notification, NotificationError, NotificationProvider},
    web::test_support::state_with_user,
};
use jiff::Timestamp;
use std::{
    future::Future,
    pin::Pin,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[derive(Default)]
struct FakeProvider {
    messages: Mutex<Vec<String>>,
    fail: AtomicBool,
}

impl NotificationProvider for FakeProvider {
    fn send<'a>(
        &'a self,
        _: &'a NotificationSettings,
        notification: &'a Notification,
    ) -> Pin<Box<dyn Future<Output = Result<(), NotificationError>> + Send + 'a>> {
        Box::pin(async move {
            if self.fail.load(Ordering::Relaxed) {
                return Err(NotificationError::Transport);
            }
            self.messages
                .lock()
                .map_err(|_| NotificationError::Transport)?
                .push(notification.message.clone());
            Ok(())
        })
    }
}

async fn settings(
    db: &mut toasty::Db,
    owner_id: i64,
    created_at: Timestamp,
) -> toasty::Result<NotificationSettings> {
    toasty::create!(NotificationSettings {
        owner_id,
        provider: NotificationProviderKind::Ntfy.as_str().to_owned(),
        enabled: true,
        url: "http://localhost/tasks".to_owned(),
        created_at,
        updated_at: created_at,
    })
    .exec(db)
    .await
}

async fn task(
    db: &mut toasty::Db,
    owner_id: i64,
    due_at: Option<Timestamp>,
) -> toasty::Result<Task> {
    toasty::create!(Task {
        owner_id,
        name: "Prepare report 🦀".to_owned(),
        due_at,
        created_at: Timestamp::now(),
        updated_at: Timestamp::now(),
    })
    .exec(db)
    .await
}

#[tokio::test]
async fn sends_each_task_in_current_minute_and_excludes_other_tasks() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let now: Timestamp = "2026-10-12T12:00:10Z".parse()?;
    let settings = settings(&mut db, user.id, now).await?;
    task(&mut db, user.id, Some("2026-10-12T12:00:00Z".parse()?)).await?;
    task(&mut db, user.id, Some("2026-10-12T12:00:59.999Z".parse()?)).await?;
    task(&mut db, user.id, Some("2026-10-12T12:01:00Z".parse()?)).await?;
    task(&mut db, user.id, Some("2026-10-12T11:59:59Z".parse()?)).await?;
    task(&mut db, user.id, None).await?;
    let mut completed = task(&mut db, user.id, Some(now)).await?;
    toasty::update!(completed {
        completed_at: Some(now)
    })
    .exec(&mut db)
    .await?;
    let provider = FakeProvider::default();
    check_owner_tasks(&mut db, &provider, &settings, now).await?;
    assert_eq!(provider.messages.lock().map_err(|_| "poisoned")?.len(), 2);
    let next_minute: Timestamp = "2026-10-12T12:01:10Z".parse()?;
    check_owner_tasks(&mut db, &provider, &settings, next_minute).await?;
    assert_eq!(provider.messages.lock().map_err(|_| "poisoned")?.len(), 3);
    Ok(())
}

#[tokio::test]
async fn disabling_settings_prevents_dispatch_of_selected_tasks() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let now = Timestamp::now();
    let mut settings = settings(&mut db, user.id, now).await?;
    let selected = task(&mut db, user.id, Some(now)).await?;
    toasty::update!(settings { enabled: false })
        .exec(&mut db)
        .await?;
    let provider = FakeProvider::default();
    deliver_reminder(&mut db, &provider, &settings, &selected).await?;
    assert!(provider.messages.lock().map_err(|_| "poisoned")?.is_empty());
    Ok(())
}

#[tokio::test]
async fn processes_more_than_one_batch_without_skipping_tasks() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let now = Timestamp::now();
    let settings = settings(&mut db, user.id, now).await?;
    for _ in 0..=super::BATCH_SIZE {
        task(&mut db, user.id, Some(now)).await?;
    }
    let provider = FakeProvider::default();
    check_owner_tasks(&mut db, &provider, &settings, now).await?;
    assert_eq!(
        provider.messages.lock().map_err(|_| "poisoned")?.len(),
        super::BATCH_SIZE + 1
    );
    Ok(())
}

#[tokio::test]
async fn skips_selected_tasks_when_completed_or_rescheduled() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let now = Timestamp::now();
    let settings = settings(&mut db, user.id, now).await?;
    let completed_selection = task(&mut db, user.id, Some(now)).await?;
    let rescheduled_selection = task(&mut db, user.id, Some(now)).await?;
    let mut completed = Task::get_by_id(&mut db, &completed_selection.id).await?;
    toasty::update!(completed {
        completed_at: Some(now)
    })
    .exec(&mut db)
    .await?;
    let mut rescheduled = Task::get_by_id(&mut db, &rescheduled_selection.id).await?;
    toasty::update!(rescheduled {
        due_at: None::<Timestamp>
    })
    .exec(&mut db)
    .await?;
    let provider = FakeProvider::default();
    deliver_reminder(&mut db, &provider, &settings, &completed_selection).await?;
    deliver_reminder(&mut db, &provider, &settings, &rescheduled_selection).await?;
    assert!(provider.messages.lock().map_err(|_| "poisoned")?.is_empty());
    Ok(())
}

#[tokio::test]
async fn unknown_stored_provider_does_not_abort_the_worker() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let now = Timestamp::now();
    let mut destination = settings(&mut db, user.id, now).await?;
    toasty::update!(destination {
        provider: "future_provider".to_owned()
    })
    .exec(&mut db)
    .await?;
    task(&mut db, user.id, Some(now)).await?;
    let providers = crate::notifications::Providers::new()?;
    super::check_due_tasks(&state, &providers, now).await?;
    Ok(())
}
