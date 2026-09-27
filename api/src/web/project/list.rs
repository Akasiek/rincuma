use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;
use toasty::stmt::{List, Query as DbQuery};

use crate::{
    app_state::AppState,
    db::Project,
    web::{
        auth::CurrentUser,
        list::{Page, PageResponse, SortOrder},
        project::{ProjectError, response::ProjectResponse},
    },
};

#[derive(Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub(super) enum ArchivedFilter {
    #[default]
    Active,
    Archived,
    All,
}

#[derive(Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub(super) enum ProjectSort {
    Name,
    #[default]
    CreatedAt,
    UpdatedAt,
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub(super) struct ListProjectsQuery {
    archived: ArchivedFilter,
    sort_by: ProjectSort,
    order: SortOrder,
    limit: Option<usize>,
    offset: Option<usize>,
}

pub(super) async fn list(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Query(request): Query<ListProjectsQuery>,
) -> Result<Json<PageResponse<ProjectResponse>>, ProjectError> {
    let page = Page::new(request.limit, request.offset).ok_or_else(ProjectError::bad_request)?;
    let mut db = state.db().clone();

    let total = filtered_projects(user.id, request.archived)
        .count()
        .exec(&mut db)
        .await?;
    let query = sorted_projects(
        filtered_projects(user.id, request.archived),
        request.sort_by,
        request.order,
    );
    let projects = query
        .limit(page.limit)
        .offset(page.offset)
        .exec(&mut db)
        .await?;
    let items = projects.into_iter().map(Into::into).collect();

    Ok(Json(PageResponse::new(items, total, &page)))
}

fn filtered_projects(owner_id: i64, archived: ArchivedFilter) -> DbQuery<List<Project>> {
    let query = DbQuery::<List<Project>>::all().filter(Project::fields().owner_id().eq(owner_id));
    match archived {
        ArchivedFilter::Active => query.filter(Project::fields().archived_at().is_none()),
        ArchivedFilter::Archived => query.filter(Project::fields().archived_at().is_some()),
        ArchivedFilter::All => query,
    }
}

fn sorted_projects(
    query: DbQuery<List<Project>>,
    sort_by: ProjectSort,
    order: SortOrder,
) -> DbQuery<List<Project>> {
    match (sort_by, order) {
        (ProjectSort::Name, SortOrder::Asc) => {
            query.order_by((Project::fields().name().asc(), Project::fields().id().asc()))
        }
        (ProjectSort::Name, SortOrder::Desc) => query.order_by((
            Project::fields().name().desc(),
            Project::fields().id().desc(),
        )),
        (ProjectSort::CreatedAt, SortOrder::Asc) => query.order_by((
            Project::fields().created_at().asc(),
            Project::fields().id().asc(),
        )),
        (ProjectSort::CreatedAt, SortOrder::Desc) => query.order_by((
            Project::fields().created_at().desc(),
            Project::fields().id().desc(),
        )),
        (ProjectSort::UpdatedAt, SortOrder::Asc) => query.order_by((
            Project::fields().updated_at().asc(),
            Project::fields().id().asc(),
        )),
        (ProjectSort::UpdatedAt, SortOrder::Desc) => query.order_by((
            Project::fields().updated_at().desc(),
            Project::fields().id().desc(),
        )),
    }
}

#[cfg(test)]
#[path = "tests/list.rs"]
mod tests;
