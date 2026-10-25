use super::{
    project::Project,
    tag::{Tag, TaskTag},
    task_series::TaskSeries,
    user::User,
};
use jiff::Timestamp;
use toasty::{Deferred, Embed, Model};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize, Embed)]
#[serde(rename_all = "snake_case")]
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
    #[index]
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

    #[index]
    pub(crate) series_id: Option<i64>,
    #[belongs_to(key = series_id, references = id)]
    pub(crate) series: Deferred<Option<TaskSeries>>,
    /// Original scheduled instant, independent of occurrence-only due date edits.
    pub(crate) scheduled_at: Option<Timestamp>,
    /// Presence records a meaningful edit, independently of audit timestamp equality.
    /// An edited successor blocks undo of its predecessor.
    pub(crate) recurrence_modified_at: Option<Timestamp>,

    #[has_many]
    pub(crate) task_tags: Deferred<Vec<TaskTag>>,
    #[has_many(via = task_tags.tag)]
    pub(crate) tags: Deferred<Vec<Tag>>,
}

impl Task {
    /// Locks an owned task until the enclosing transaction ends, then reads it.
    ///
    /// Toasty has no locking query builder. Assigning the audit timestamp to
    /// itself acquires a write lock without changing its stored value. Reading
    /// after that write observes any completion committed by a competing request.
    pub(crate) async fn lock_owned(
        executor: &mut dyn toasty::Executor,
        id: i64,
        owner_id: i64,
    ) -> toasty::Result<Option<Self>> {
        // Raw SQL avoids Toasty's unsupported timestamp-column assignment lowering.
        // Both PostgreSQL and SQLite accept these numbered dollar placeholders.
        toasty::sql::statement(
            "UPDATE tasks SET updated_at = updated_at WHERE id = $1 AND owner_id = $2",
        )
        .bind(id)
        .bind(owner_id)
        .exec(executor)
        .await?;

        Self::get_owned(executor, id, owner_id).await
    }

    /// Returns a batch of incomplete tasks for one owner, ordered by task ID.
    ///
    /// Only tasks with `due_at >= start` and `due_at < before` are included.
    /// Tasks without a due date are excluded. Pass `0` for the first batch,
    /// then the last returned task ID to exclude previously processed tasks.
    /// An empty batch ends the scan for this owner and due-date range.
    pub(crate) async fn find_due_batch_after_task_id(
        executor: &mut dyn toasty::Executor,
        owner_id: i64,
        start: Timestamp,
        before: Timestamp,
        last_processed_task_id: i64,
        limit: usize,
    ) -> toasty::Result<Vec<Self>> {
        Self::filter(Self::fields().owner_id().eq(owner_id))
            .filter(Self::fields().completed_at().is_none())
            .filter(Self::fields().due_at().ge(Some(start)))
            .filter(Self::fields().due_at().lt(Some(before)))
            .filter(Self::fields().id().gt(last_processed_task_id))
            .order_by(Self::fields().id().asc())
            .limit(limit)
            .exec(executor)
            .await
    }

    pub(crate) async fn get_owned(
        executor: &mut dyn toasty::Executor,
        id: i64,
        owner_id: i64,
    ) -> toasty::Result<Option<Self>> {
        Ok(Self::filter_by_id(id)
            .first()
            .exec(executor)
            .await?
            .filter(|task| task.owner_id == owner_id))
    }
}
