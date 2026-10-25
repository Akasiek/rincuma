use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use jiff::Timestamp;

use crate::{
    app_state::AppState,
    web::{
        auth::CurrentUser,
        task::{TaskError, deletion::delete_subtree},
    },
};

pub(in crate::web::task) async fn delete(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<i64>,
) -> Result<StatusCode, TaskError> {
    let mut db = state.db().clone();
    let mut transaction = db.transaction().await?;
    delete_subtree(&mut transaction, id, user.id, Timestamp::now()).await?;
    transaction.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
#[path = "../tests/delete.rs"]
mod tests;
