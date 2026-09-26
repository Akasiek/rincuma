use super::{task::Task, user::User};
use jiff::Timestamp;
use toasty::{Deferred, Model};

#[derive(Debug, Model)]
pub(crate) struct Tag {
    #[key]
    #[auto]
    pub(crate) id: i64,

    pub(crate) name: String,
    pub(crate) color: Option<String>,
    pub(crate) created_at: Timestamp,
    pub(crate) updated_at: Timestamp,

    #[index]
    pub(crate) owner_id: i64,
    #[belongs_to(key = owner_id, references = id)]
    pub(crate) owner: Deferred<User>,

    #[has_many]
    pub(crate) task_tags: Deferred<Vec<TaskTag>>,

    #[has_many(via = task_tags.task)]
    pub(crate) tasks: Deferred<Vec<Task>>,
}

#[derive(Debug, Model)]
#[key(task_id, tag_id)]
pub(crate) struct TaskTag {
    #[index]
    pub(crate) task_id: i64,

    #[belongs_to(key = task_id, references = id)]
    pub(crate) task: Deferred<Task>,

    #[index]
    pub(crate) tag_id: i64,

    #[belongs_to(key = tag_id, references = id)]
    pub(crate) tag: Deferred<Tag>,
}
