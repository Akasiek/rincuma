#![allow(clippy::panic_in_result_fn)]

use axum::{
    Json,
    body::Body,
    extract::{Query, State},
    http::{Request, StatusCode, Uri},
};
use jiff::Timestamp;
use tower::ServiceExt;

use super::{ArchivedFilter, ListProjectsQuery, ProjectSort, list};
use crate::{
    app_state::AppState,
    db::{Project, User},
    web::{
        auth::CurrentUser,
        list::{PageResponse, SortOrder},
        project::{ProjectError, response::ProjectResponse},
        router::get_app_router,
        test_support::state_with_user,
    },
};

async fn insert_project(
    db: &mut toasty::Db,
    owner_id: i64,
    name: &str,
    created_at: Timestamp,
    updated_at: Timestamp,
    archived_at: Option<Timestamp>,
) -> toasty::Result<Project> {
    toasty::create!(Project {
        name: name.to_owned(),
        owner_id,
        created_at,
        updated_at,
        archived_at,
    })
    .exec(db)
    .await
}

async fn list_for(
    state: &AppState,
    owner_id: i64,
    request: ListProjectsQuery,
) -> Result<PageResponse<ProjectResponse>, ProjectError> {
    let mut db = state.db().clone();
    let user = User::get_by_id(&mut db, &owner_id).await?;
    let Json(response) = list(State(state.clone()), CurrentUser(user), Query(request)).await?;
    Ok(response)
}

#[tokio::test]
async fn lists_only_owned_active_projects_by_default() -> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let old = Timestamp::from_second(1)?;
    let new = Timestamp::from_second(2)?;
    let archived = Timestamp::from_second(3)?;
    let first = insert_project(&mut db, user.id, "First", old, old, None).await?;
    let second = insert_project(&mut db, user.id, "Second", new, new, None).await?;
    insert_project(
        &mut db,
        user.id,
        "Archived",
        archived,
        archived,
        Some(archived),
    )
    .await?;
    let other = toasty::create!(User {
        email: "other@example.com".to_owned(),
        password_hash: "unused".to_owned(),
        created_at: old,
        updated_at: old,
    })
    .exec(&mut db)
    .await?;
    insert_project(&mut db, other.id, "Other user's project", new, new, None).await?;

    let response = list_for(&state, user.id, ListProjectsQuery::default()).await?;

    assert_eq!(response.total, 2);
    assert_eq!(response.limit, 50);
    assert_eq!(response.offset, 0);
    assert_eq!(response.items.len(), 2);
    let ids: Vec<_> = response.items.iter().map(|project| project.id).collect();
    assert_eq!(ids, [second.id, first.id]);
    assert!(
        response
            .items
            .iter()
            .all(|project| project.owner_id == user.id)
    );
    Ok(())
}

#[tokio::test]
async fn filters_archived_projects_and_counts_the_same_scope()
-> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let now = Timestamp::now();
    insert_project(&mut db, user.id, "Active", now, now, None).await?;
    let archived = insert_project(&mut db, user.id, "Archived", now, now, Some(now)).await?;

    let archived_only = list_for(
        &state,
        user.id,
        ListProjectsQuery {
            archived: ArchivedFilter::Archived,
            ..ListProjectsQuery::default()
        },
    )
    .await?;
    assert_eq!(archived_only.total, 1);
    assert_eq!(archived_only.items.len(), 1);
    assert_eq!(
        archived_only.items.first().map(|project| project.id),
        Some(archived.id)
    );

    let all = list_for(
        &state,
        user.id,
        ListProjectsQuery {
            archived: ArchivedFilter::All,
            ..ListProjectsQuery::default()
        },
    )
    .await?;
    assert_eq!(all.total, 2);
    assert_eq!(all.items.len(), 2);
    Ok(())
}

#[tokio::test]
async fn sorts_and_paginates_projects() -> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let created_at = Timestamp::UNIX_EPOCH;
    let first = insert_project(
        &mut db,
        user.id,
        "Zulu",
        created_at,
        Timestamp::from_second(1)?,
        None,
    )
    .await?;
    let middle = insert_project(
        &mut db,
        user.id,
        "Bravo",
        created_at,
        Timestamp::from_second(2)?,
        None,
    )
    .await?;
    let last = insert_project(
        &mut db,
        user.id,
        "Alpha",
        created_at,
        Timestamp::from_second(3)?,
        None,
    )
    .await?;

    let response = list_for(
        &state,
        user.id,
        ListProjectsQuery {
            sort_by: ProjectSort::Name,
            order: SortOrder::Asc,
            limit: Some(1),
            offset: Some(1),
            ..ListProjectsQuery::default()
        },
    )
    .await?;

    assert_eq!(response.total, 3);
    assert_eq!(response.limit, 1);
    assert_eq!(response.offset, 1);
    assert_eq!(response.items.len(), 1);
    assert_eq!(
        response.items.first().map(|project| project.id),
        Some(middle.id)
    );

    let default_order = list_for(&state, user.id, ListProjectsQuery::default()).await?;
    let ids: Vec<_> = default_order
        .items
        .iter()
        .map(|project| project.id)
        .collect();
    assert_eq!(ids, [last.id, middle.id, first.id]);

    let updated_order = list_for(
        &state,
        user.id,
        ListProjectsQuery {
            sort_by: ProjectSort::UpdatedAt,
            order: SortOrder::Asc,
            ..ListProjectsQuery::default()
        },
    )
    .await?;
    let ids: Vec<_> = updated_order
        .items
        .iter()
        .map(|project| project.id)
        .collect();
    assert_eq!(ids, [first.id, middle.id, last.id]);
    Ok(())
}

#[tokio::test]
async fn rejects_invalid_page_bounds() -> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let owner_id = user.id;
    for request in [
        ListProjectsQuery {
            limit: Some(0),
            ..ListProjectsQuery::default()
        },
        ListProjectsQuery {
            limit: Some(101),
            ..ListProjectsQuery::default()
        },
        ListProjectsQuery {
            offset: Some(usize::MAX),
            ..ListProjectsQuery::default()
        },
    ] {
        let result = list_for(&state, owner_id, request).await;
        assert!(matches!(result, Err(ProjectError::BadRequest(_))));
    }
    Ok(())
}

#[test]
fn parses_list_query_parameters() -> Result<(), Box<dyn std::error::Error>> {
    let uri: Uri =
        "/projects?archived=archived&sort_by=name&order=asc&limit=10&offset=5".parse()?;
    let Query(query) = Query::<ListProjectsQuery>::try_from_uri(&uri)?;
    assert!(matches!(query.archived, ArchivedFilter::Archived));
    assert!(matches!(query.sort_by, ProjectSort::Name));
    assert!(matches!(query.order, SortOrder::Asc));
    assert_eq!(query.limit, Some(10));
    assert_eq!(query.offset, Some(5));

    let invalid: Uri = "/projects?sort_by=invalid".parse()?;
    assert!(Query::<ListProjectsQuery>::try_from_uri(&invalid).is_err());
    Ok(())
}

#[tokio::test]
async fn list_route_requires_authentication() -> Result<(), Box<dyn std::error::Error>> {
    let (state, _) = state_with_user().await?;
    let request = Request::builder().uri("/projects").body(Body::empty())?;
    let response = get_app_router(state).oneshot(request).await?;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    Ok(())
}
