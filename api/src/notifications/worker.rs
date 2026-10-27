use super::{NotificationProvider, Providers, delivery::deliver_reminder};
use crate::{
    app_state::AppState,
    db::{NotificationProviderKind, NotificationSettings, Task, User},
};
use jiff::Timestamp;
use std::time::Duration;

const CHECK_INTERVAL: Duration = Duration::from_mins(1);
const BATCH_SIZE: usize = 100;

pub(crate) fn spawn(state: AppState) -> Result<tokio::task::JoinHandle<()>, reqwest::Error> {
    let providers = Providers::new()?;
    tracing::info!(
        interval_seconds = CHECK_INTERVAL.as_secs(),
        batch_size = BATCH_SIZE,
        "Reminder worker started"
    );
    Ok(tokio::spawn(async move {
        run_due_task_scanner(&state, &providers).await;
    }))
}

async fn run_due_task_scanner(state: &AppState, providers: &Providers) {
    loop {
        tokio::time::sleep(delay_until_next_minute()).await;

        tracing::debug!("Scanning due reminders");
        if let Err(error) = check_due_tasks(state, providers, Timestamp::now()).await {
            tracing::error!(error = %error, "Reminder scan failed");
        }
    }
}

fn delay_until_next_minute() -> Duration {
    let elapsed_seconds = Timestamp::now().as_second().rem_euclid(60).unsigned_abs();
    let elapsed = Duration::new(elapsed_seconds, 0);

    CHECK_INTERVAL - elapsed
}

async fn check_due_tasks(
    state: &AppState,
    providers: &Providers,
    now: Timestamp,
) -> toasty::Result<()> {
    let mut db = state.db().clone();
    let mut last_processed_owner_id = 0;
    loop {
        let settings = NotificationSettings::find_enabled_batch_after_owner_id(
            &mut db,
            last_processed_owner_id,
            BATCH_SIZE,
        )
        .await?;

        if settings.is_empty() {
            return Ok(());
        }

        for settings in settings {
            last_processed_owner_id = settings.owner_id;
            process_owner_reminders(&mut db, providers, &settings, now).await?;
        }
    }
}

async fn process_owner_reminders(
    db: &mut toasty::Db,
    providers: &Providers,
    settings: &NotificationSettings,
    now: Timestamp,
) -> toasty::Result<()> {
    let user = User::filter_by_id(settings.owner_id)
        .first()
        .exec(db)
        .await?;
    if !user.is_some_and(|user| user.is_active) {
        return Ok(());
    }

    let Some(provider_kind) = NotificationProviderKind::from_str(&settings.provider) else {
        tracing::warn!(
            owner_id = settings.owner_id,
            "Unsupported notification provider; skipping destination"
        );
        return Ok(());
    };

    check_owner_tasks(db, providers.get(provider_kind), settings, now).await
}

async fn check_owner_tasks(
    db: &mut toasty::Db,
    provider: &dyn NotificationProvider,
    settings: &NotificationSettings,
    now: Timestamp,
) -> toasty::Result<()> {
    let start = get_start_of_minute(now);
    let before = get_start_of_minute(now).saturating_add(60);

    let before = Timestamp::from_second(before)
        .map_err(|error| toasty::Error::from_args(format_args!("{error}")))?;
    let start = Timestamp::from_second(start)
        .map_err(|error| toasty::Error::from_args(format_args!("{error}")))?;

    let mut last_processed_task_id = 0;
    loop {
        let tasks = Task::find_due_batch_after_task_id(
            db,
            settings.owner_id,
            start,
            before,
            last_processed_task_id,
            BATCH_SIZE,
        )
        .await?;

        if tasks.is_empty() {
            return Ok(());
        }

        for task in tasks {
            last_processed_task_id = task.id;
            deliver_reminder(db, provider, settings, &task).await?;
        }
    }
}

fn get_start_of_minute(timestamp: Timestamp) -> i64 {
    timestamp.as_second().div_euclid(60) * 60
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
