use super::{NotificationProvider, builder::build_task_reminder};
use crate::db::{NotificationSettings, Task};

pub(super) async fn deliver_reminder(
    db: &mut toasty::Db,
    provider: &dyn NotificationProvider,
    settings: &NotificationSettings,
    selected_task: &Task,
) -> toasty::Result<()> {
    let Some(current_settings) = find_current_settings(db, settings).await? else {
        return Ok(());
    };
    let Some(task) = find_current_task(db, selected_task, settings.owner_id).await? else {
        return Ok(());
    };
    let Some(notification) = build_task_reminder(&task) else {
        return Ok(());
    };

    if let Err(error) = provider.send(&current_settings, &notification).await {
        tracing::warn!(task_id = task.id, owner_id = task.owner_id, error = %error, "Reminder delivery failed");
    } else {
        tracing::info!(
            task_id = task.id,
            owner_id = task.owner_id,
            "Reminder delivered"
        );
    }
    Ok(())
}

async fn find_current_settings(
    db: &mut toasty::Db,
    selected_settings: &NotificationSettings,
) -> toasty::Result<Option<NotificationSettings>> {
    let settings = NotificationSettings::filter_by_owner_id(selected_settings.owner_id)
        .first()
        .exec(db)
        .await?;

    Ok(settings
        .filter(|settings| settings.enabled && settings.provider == selected_settings.provider))
}

async fn find_current_task(
    db: &mut toasty::Db,
    selected_task: &Task,
    owner_id: i64,
) -> toasty::Result<Option<Task>> {
    let task = Task::get_owned(db, selected_task.id, owner_id).await?;

    Ok(task.filter(|task| task.completed_at.is_none() && task.due_at == selected_task.due_at))
}
