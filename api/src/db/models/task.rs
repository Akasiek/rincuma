use super::{
    project::Project,
    tag::{Tag, TaskTag},
    user::User,
};
use jiff::Timestamp;
use toasty::{Deferred, Embed, Model};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Embed)]
pub(crate) enum TaskPriority {
    None,
    Low,
    Medium,
    High,
    Urgent,
}

#[derive(Debug, Model)]
pub(crate) struct Task {
    #[key]
    #[auto]
    pub(crate) id: i64,

    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) due_at: Option<Timestamp>,
    pub(crate) completed_at: Option<Timestamp>,
    pub(crate) created_at: Timestamp,
    pub(crate) updated_at: Timestamp,

    #[default(TaskPriority::None)]
    pub(crate) priority: TaskPriority,

    #[index]
    pub(crate) owner_id: i64,
    #[belongs_to(key = owner_id, references = id)]
    pub(crate) owner: Deferred<User>,

    #[index]
    pub(crate) parent_id: Option<i64>,
    #[belongs_to(key = parent_id, references = id)]
    pub(crate) parent: Deferred<Option<Task>>,

    #[index]
    pub(crate) project_id: Option<i64>,
    #[belongs_to(key = project_id, references = id)]
    pub(crate) project: Deferred<Option<Project>>,

    #[has_many]
    pub(crate) task_tags: Deferred<Vec<TaskTag>>,
    #[has_many(via = task_tags.tag)]
    pub(crate) tags: Deferred<Vec<Tag>>,
}
