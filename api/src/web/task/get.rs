use axum::{
    Json,
    extract::{Path, State},
};
use toasty::stmt::{List, Query};

use crate::{
    app_state::AppState,
    db::TaskTag,
    web::{
        auth::CurrentUser,
        task::{TaskError, relations::get_owned_task, response::TaskResponse},
    },
};

pub(super) async fn get_task(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<i64>,
) -> Result<Json<TaskResponse>, TaskError> {
    let mut db = state.db().clone();
    let task = get_owned_task(&mut db, id, user.id).await?;
    let task_tags = Query::<List<TaskTag>>::all()
        .filter(TaskTag::fields().task_id().eq(task.id))
        .order_by(TaskTag::fields().tag_id().asc())
        .exec(&mut db)
        .await?;
    let tag_ids = task_tags.into_iter().map(|link| link.tag_id).collect();

    Ok(Json(TaskResponse::new(task, tag_ids)))
}

#[cfg(test)]
#[path = "tests/get.rs"]
mod tests;
