use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use jiff::Timestamp;
use serde::Deserialize;

use crate::{
    app_state::AppState,
    web::{
        auth::CurrentUser,
        task::{TaskError, restoration::restore_deletion},
    },
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::web::task) struct RestoreTaskRequest {
    deleted_at: Timestamp,
}

pub(in crate::web::task) async fn restore(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<i64>,
    Json(request): Json<RestoreTaskRequest>,
) -> Result<StatusCode, TaskError> {
    let mut db = state.db().clone();
    let mut transaction = db.transaction().await?;

    restore_deletion(
        &mut transaction,
        id,
        user.id,
        request.deleted_at,
        Timestamp::now(),
    )
    .await?;

    transaction.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
#[path = "../tests/deletion_undo.rs"]
mod tests;
