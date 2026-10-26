use axum::{
    Json,
    extract::{Path, State},
};
use jiff::Timestamp;
use serde::Serialize;

use crate::{
    app_state::AppState,
    web::{
        auth::CurrentUser,
        task::{TaskError, restoration::delete_with_undo},
    },
};

#[derive(Serialize)]
pub(in crate::web::task) struct DeletionResponse {
    pub(in crate::web::task) task_id: i64,
    pub(in crate::web::task) deleted_at: Timestamp,
    pub(in crate::web::task) expires_at: Timestamp,
}

pub(in crate::web::task) async fn delete(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<i64>,
) -> Result<Json<DeletionResponse>, TaskError> {
    let mut db = state.db().clone();
    let mut transaction = db.transaction().await?;
    let deletion = delete_with_undo(&mut transaction, id, user.id, Timestamp::now()).await?;

    transaction.commit().await?;
    Ok(Json(DeletionResponse {
        task_id: deletion.id,
        deleted_at: deletion.deleted_at,
        expires_at: deletion.expires_at,
    }))
}

#[cfg(test)]
#[path = "../tests/delete.rs"]
mod tests;
