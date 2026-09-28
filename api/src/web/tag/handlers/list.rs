use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;
use toasty::stmt::{List, Query as DbQuery};

use crate::{
    app_state::AppState,
    db::Tag,
    web::{
        auth::CurrentUser,
        list::{Page, PageResponse, SortOrder},
        tag::{TagError, response::TagResponse},
    },
};

#[derive(Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub(super) enum TagSort {
    Name,
    #[default]
    CreatedAt,
    UpdatedAt,
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub(in crate::web::tag) struct ListTagsQuery {
    sort_by: TagSort,
    order: SortOrder,
    limit: Option<usize>,
    offset: Option<usize>,
}

pub(in crate::web::tag) async fn list(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Query(request): Query<ListTagsQuery>,
) -> Result<Json<PageResponse<TagResponse>>, TagError> {
    let page = Page::new(request.limit, request.offset).ok_or_else(TagError::bad_request)?;
    let mut db = state.db().clone();

    let total = owned_tags(user.id).count().exec(&mut db).await?;
    let tags = sorted_tags(owned_tags(user.id), request.sort_by, request.order)
        .limit(page.limit)
        .offset(page.offset)
        .exec(&mut db)
        .await?;
    let items = tags.into_iter().map(Into::into).collect();

    Ok(Json(PageResponse::new(items, total, &page)))
}

fn owned_tags(owner_id: i64) -> DbQuery<List<Tag>> {
    DbQuery::<List<Tag>>::all().filter(Tag::fields().owner_id().eq(owner_id))
}

fn sorted_tags(
    query: DbQuery<List<Tag>>,
    sort_by: TagSort,
    order: SortOrder,
) -> DbQuery<List<Tag>> {
    match (sort_by, order) {
        (TagSort::Name, SortOrder::Asc) => {
            query.order_by((Tag::fields().name().asc(), Tag::fields().id().asc()))
        }
        (TagSort::Name, SortOrder::Desc) => {
            query.order_by((Tag::fields().name().desc(), Tag::fields().id().desc()))
        }
        (TagSort::CreatedAt, SortOrder::Asc) => {
            query.order_by((Tag::fields().created_at().asc(), Tag::fields().id().asc()))
        }
        (TagSort::CreatedAt, SortOrder::Desc) => {
            query.order_by((Tag::fields().created_at().desc(), Tag::fields().id().desc()))
        }
        (TagSort::UpdatedAt, SortOrder::Asc) => {
            query.order_by((Tag::fields().updated_at().asc(), Tag::fields().id().asc()))
        }
        (TagSort::UpdatedAt, SortOrder::Desc) => {
            query.order_by((Tag::fields().updated_at().desc(), Tag::fields().id().desc()))
        }
    }
}

#[cfg(test)]
#[path = "../tests/list.rs"]
mod tests;
