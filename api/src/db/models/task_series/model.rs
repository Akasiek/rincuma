#![allow(
    clippy::unused_self,
    reason = "Toasty generates enum variant field accessors that do not use self."
)]

use crate::{
    db::{Task, User},
    recurrence::{End, Frequency, Schedule, ScheduleError},
};
use jiff::{
    Timestamp,
    civil::{Date, DateTime},
};
use toasty::{Deferred, Embed, Model};

use super::SeriesPattern;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Embed, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SeriesFrequency {
    Daily,
    Weekly,
    Monthly,
}

/// Stores one end mode without allowing a simultaneous date and count limit.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Embed, serde::Serialize, serde::Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum SeriesEnd {
    Never,
    Through { date: Date },
    After { count: u32 },
}

#[derive(Debug, Model)]
pub(crate) struct TaskSeries {
    #[key]
    #[auto]
    pub(crate) id: i64,

    #[index]
    pub(crate) owner_id: i64,
    #[belongs_to(key = owner_id, references = id)]
    pub(crate) owner: Deferred<User>,

    pub(crate) frequency: SeriesFrequency,
    pub(crate) interval: u32,
    pub(crate) time_zone: String,
    /// Intended local date and time, unaffected by month clamping or DST gaps.
    pub(crate) anchor: DateTime,
    pub(crate) end: SeriesEnd,
    pub(crate) pattern: Option<SeriesPattern>,

    /// Counts created tasks, including the first, but excluding undone successors.
    #[default(1)]
    pub(crate) occurrence_count: u32,
    /// Cursor for completion/undo. Not an owning relation: history outlives tasks.
    pub(crate) latest_task_id: Option<i64>,
    /// Explicitly stopped series differ from series that reached their end mode.
    pub(crate) stopped_at: Option<Timestamp>,

    pub(crate) created_at: Timestamp,
    pub(crate) updated_at: Timestamp,

    #[has_many(pair = series)]
    pub(crate) tasks: Deferred<Vec<Task>>,
}

impl TaskSeries {
    /// Serializes changes to one owner's series within the caller's transaction.
    /// The no-op write locks the row before reading its count and latest task.
    pub(crate) async fn lock_owned(
        executor: &mut dyn toasty::Executor,
        id: i64,
        owner_id: i64,
    ) -> toasty::Result<Option<Self>> {
        toasty::sql::statement(
            "UPDATE task_series SET updated_at = updated_at WHERE id = $1 AND owner_id = $2",
        )
        .bind(id)
        .bind(owner_id)
        .exec(executor)
        .await?;

        Self::filter_by_id(id)
            .filter(Self::fields().owner_id().eq(owner_id))
            .first()
            .exec(executor)
            .await
    }
}

impl From<SeriesFrequency> for Frequency {
    fn from(value: SeriesFrequency) -> Self {
        match value {
            SeriesFrequency::Daily => Self::Daily,
            SeriesFrequency::Weekly => Self::Weekly,
            SeriesFrequency::Monthly => Self::Monthly,
        }
    }
}

impl From<SeriesEnd> for End {
    fn from(value: SeriesEnd) -> Self {
        match value {
            SeriesEnd::Never => Self::Never,
            SeriesEnd::Through { date } => Self::Through(date),
            SeriesEnd::After { count } => Self::After(count),
        }
    }
}

impl TryFrom<&TaskSeries> for Schedule {
    type Error = ScheduleError;

    fn try_from(series: &TaskSeries) -> Result<Self, Self::Error> {
        Self::new(
            series.frequency.into(),
            series.interval,
            &series.time_zone,
            series.anchor,
            series.end.into(),
        )?
        .with_pattern(series.pattern.map(Into::into))
    }
}
