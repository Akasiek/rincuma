use std::collections::{HashMap, HashSet};

use jiff::civil::DateTime;
use serde::Serialize;

use super::RecurrenceConfig;
use crate::{db::TaskSeries, web::task::response::TaskResponse};

#[derive(Serialize)]
pub(in crate::web::task) struct RecurrenceResponse {
    #[serde(flatten)]
    pub config: RecurrenceConfig,
    pub anchor: DateTime,
    pub occurrence_count: u32,
}

/// Loads rules for active occurrences in one query. Historical task responses
/// expose their series ID and original slot, without inheriting a later rule edit.
pub(in crate::web::task) async fn attach_recurrence(
    executor: &mut dyn toasty::Executor,
    tasks: &mut [TaskResponse],
) -> toasty::Result<()> {
    let ids: HashSet<_> = tasks.iter().filter_map(|task| task.series_id).collect();
    if ids.is_empty() {
        return Ok(());
    }

    let series = TaskSeries::filter(
        TaskSeries::fields()
            .id()
            .in_list(ids.into_iter().collect::<Vec<_>>()),
    )
    .exec(executor)
    .await?;

    let by_id: HashMap<_, _> = series
        .into_iter()
        .map(|series| (series.id, series))
        .collect();
    for task in tasks {
        let Some(series) = task.series_id.and_then(|id| by_id.get(&id)) else {
            continue;
        };
        let active = task.completed_at.is_none()
            && series.latest_task_id == Some(task.id)
            && series.stopped_at.is_none()
            && series.owner_id == task.owner_id;

        if active {
            task.recurrence = Some(RecurrenceResponse {
                config: RecurrenceConfig {
                    frequency: series.frequency,
                    interval: series.interval,
                    time_zone: series.time_zone.clone(),
                    end: series.end,
                },
                anchor: series.anchor,
                occurrence_count: series.occurrence_count,
            });
        }
    }

    Ok(())
}
