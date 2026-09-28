use jiff::Timestamp;
use serde::Serialize;

use crate::db::{Task, TaskPriority};

#[derive(Serialize)]
pub(super) struct TaskResponse {
    pub(super) id: i64,
    pub(super) name: String,
    pub(super) description: Option<String>,
    pub(super) due_at: Option<Timestamp>,
    pub(super) completed_at: Option<Timestamp>,
    pub(super) priority: TaskPriority,
    pub(super) owner_id: i64,
    pub(super) project_id: Option<i64>,
    pub(super) parent_id: Option<i64>,
    pub(super) tag_ids: Vec<i64>,
    pub(super) created_at: Timestamp,
    pub(super) updated_at: Timestamp,
}

impl TaskResponse {
    pub(super) fn new(task: Task, tag_ids: Vec<i64>) -> Self {
        Self {
            id: task.id,
            name: task.name,
            description: task.description,
            due_at: task.due_at,
            completed_at: task.completed_at,
            priority: task.priority,
            owner_id: task.owner_id,
            project_id: task.project_id,
            parent_id: task.parent_id,
            tag_ids,
            created_at: task.created_at,
            updated_at: task.updated_at,
        }
    }
}
