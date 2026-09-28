use axum::{
    Json,
    body::Body,
    extract::{Query, State},
    http::{Request, StatusCode, Uri},
};
use jiff::Timestamp;
use tower::ServiceExt;

use super::{ListTagsQuery, TagSort, list};
use crate::{
    app_state::AppState,
    db::{Tag, User},
    web::{
        auth::CurrentUser,
        list::{PageResponse, SortOrder},
        router::get_app_router,
        tag::{TagError, response::TagResponse},
        test_support::state_with_user,
    },
};

async fn insert_tag(
    db: &mut toasty::Db,
    owner_id: i64,
    name: &str,
    color: Option<&str>,
    created_at: Timestamp,
    updated_at: Timestamp,
) -> toasty::Result<Tag> {
    toasty::create!(Tag {
        name: name.to_owned(),
        color: color.map(str::to_owned),
        owner_id,
        created_at,
        updated_at,
    })
    .exec(db)
    .await
}

async fn list_for(
    state: &AppState,
    owner_id: i64,
    request: ListTagsQuery,
) -> Result<PageResponse<TagResponse>, TagError> {
    let mut db = state.db().clone();
    let user = User::get_by_id(&mut db, &owner_id).await?;
    let Json(response) = list(State(state.clone()), CurrentUser(user), Query(request)).await?;
    Ok(response)
}

#[tokio::test]
async fn lists_only_owned_tags_by_default() -> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let old = Timestamp::from_second(1)?;
    let new = Timestamp::from_second(2)?;
    let first = insert_tag(&mut db, user.id, "First", None, old, old).await?;
    let second = insert_tag(&mut db, user.id, "Second", Some("#3B82F6"), new, new).await?;
    let other = toasty::create!(User {
        email: "other@example.com".to_owned(),
        password_hash: "unused".to_owned(),
        created_at: old,
        updated_at: old,
    })
    .exec(&mut db)
    .await?;
    insert_tag(&mut db, other.id, "Other", None, new, new).await?;

    let response = list_for(&state, user.id, ListTagsQuery::default()).await?;

    assert_eq!(response.total, 2);
    assert_eq!(response.limit, 50);
    assert_eq!(response.offset, 0);
    let ids: Vec<_> = response.items.iter().map(|tag| tag.id).collect();
    assert_eq!(ids, [second.id, first.id]);
    assert_eq!(
        response.items.first().and_then(|tag| tag.color.as_deref()),
        Some("#3B82F6")
    );
    assert!(response.items.iter().all(|tag| tag.owner_id == user.id));
    Ok(())
}

#[tokio::test]
async fn sorts_and_paginates_tags() -> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let created_at = Timestamp::UNIX_EPOCH;
    let first = insert_tag(
        &mut db,
        user.id,
        "Zulu",
        None,
        created_at,
        Timestamp::from_second(1)?,
    )
    .await?;
    let middle = insert_tag(
        &mut db,
        user.id,
        "Bravo",
        None,
        created_at,
        Timestamp::from_second(2)?,
    )
    .await?;
    let last = insert_tag(
        &mut db,
        user.id,
        "Alpha",
        None,
        created_at,
        Timestamp::from_second(3)?,
    )
    .await?;

    let page = list_for(
        &state,
        user.id,
        ListTagsQuery {
            sort_by: TagSort::Name,
            order: SortOrder::Asc,
            limit: Some(1),
            offset: Some(1),
        },
    )
    .await?;
    assert_eq!(page.total, 3);
    assert_eq!(page.limit, 1);
    assert_eq!(page.offset, 1);
    assert_eq!(page.items.first().map(|tag| tag.id), Some(middle.id));

    let default_order = list_for(&state, user.id, ListTagsQuery::default()).await?;
    let ids: Vec<_> = default_order.items.iter().map(|tag| tag.id).collect();
    assert_eq!(ids, [last.id, middle.id, first.id]);

    let updated_order = list_for(
        &state,
        user.id,
        ListTagsQuery {
            sort_by: TagSort::UpdatedAt,
            order: SortOrder::Asc,
            ..ListTagsQuery::default()
        },
    )
    .await?;
    let ids: Vec<_> = updated_order.items.iter().map(|tag| tag.id).collect();
    assert_eq!(ids, [first.id, middle.id, last.id]);
    Ok(())
}

#[tokio::test]
async fn rejects_invalid_page_bounds() -> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    for request in [
        ListTagsQuery {
            limit: Some(0),
            ..ListTagsQuery::default()
        },
        ListTagsQuery {
            limit: Some(101),
            ..ListTagsQuery::default()
        },
        ListTagsQuery {
            offset: Some(usize::MAX),
            ..ListTagsQuery::default()
        },
    ] {
        let result = list_for(&state, user.id, request).await;
        assert!(matches!(result, Err(TagError::BadRequest(_))));
    }
    Ok(())
}

#[test]
fn parses_list_query_parameters() -> Result<(), Box<dyn std::error::Error>> {
    let uri: Uri = "/tags?sort_by=name&order=asc&limit=10&offset=5".parse()?;
    let Query(query) = Query::<ListTagsQuery>::try_from_uri(&uri)?;
    assert!(matches!(query.sort_by, TagSort::Name));
    assert!(matches!(query.order, SortOrder::Asc));
    assert_eq!(query.limit, Some(10));
    assert_eq!(query.offset, Some(5));

    let invalid: Uri = "/tags?sort_by=invalid".parse()?;
    assert!(Query::<ListTagsQuery>::try_from_uri(&invalid).is_err());
    Ok(())
}

#[tokio::test]
async fn list_route_requires_authentication() -> Result<(), Box<dyn std::error::Error>> {
    let (state, _) = state_with_user().await?;
    let request = Request::builder().uri("/tags").body(Body::empty())?;
    let response = get_app_router(state).oneshot(request).await?;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    Ok(())
}
