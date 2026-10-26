use toasty::stmt::{Expr, Type, Value};

use crate::db::Task;

/// Uses creation order, just like completion undo. Due dates and audit timestamps
/// can change without changing an occurrence's position in its series.
pub(super) async fn latest_completed_ids(
    executor: &mut dyn toasty::Executor,
    owner_id: i64,
) -> toasty::Result<Vec<i64>> {
    let rows = toasty::sql::query(
        "SELECT MAX(id) FROM tasks WHERE owner_id = $1 AND series_id IS NOT NULL \
         AND completed_at IS NOT NULL AND deleted_at IS NULL GROUP BY series_id",
    )
    .bind(owner_id)
    .column_types([Type::I64])
    .exec(executor)
    .await?;

    rows.into_iter().map(completed_id).collect()
}

fn completed_id(row: Value) -> toasty::Result<i64> {
    if let Value::Record(record) = row
        && let [Value::I64(id)] = record.fields.as_slice()
    {
        return Ok(*id);
    }

    Err(toasty::Error::invalid_result(
        "Expected a completed task ID.",
    ))
}

pub(super) fn visible_occurrences(latest_completed: &[i64]) -> Expr<bool> {
    let visible = Task::fields()
        .completed_at()
        .is_none()
        .or(Task::fields().series_id().is_none());

    if latest_completed.is_empty() {
        return visible;
    }

    visible.or(Task::fields().id().in_list(latest_completed))
}
