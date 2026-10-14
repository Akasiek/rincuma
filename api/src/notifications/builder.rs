use super::Notification;
use crate::db::Task;

pub(super) fn build_task_reminder(task: &Task) -> Option<Notification> {
    Some(Notification {
        title: "Task is due".to_owned(),
        message: task.name.to_owned(),
    })
}
