use axum::{
    Json,
    body::Body,
    extract::{Path, State},
    http::{Request, StatusCode},
    response::IntoResponse,
};
use jiff::Timestamp;
use tower::ServiceExt;

use super::{SaveTaskRequest, update};
use crate::{
    auth::session::create_session,
    db::{Project, Tag, Task, TaskPriority, TaskTag, User},
    web::{
        auth::CurrentUser, router::get_app_router, task::TaskError, test_support::state_with_user,
    },
};

fn request(name: &str) -> SaveTaskRequest {
    SaveTaskRequest {
        name: name.to_owned(),
        description: None,
        due_at: None,
        priority: None,
        project_id: None,
        parent_id: None,
        tag_ids: Vec::new(),
    }
}

async fn task(
    db: &mut toasty::Db,
    owner_id: i64,
    project_id: Option<i64>,
    parent_id: Option<i64>,
) -> Result<Task, Box<dyn std::error::Error>> {
    let now: Timestamp = "2020-01-01T00:00:00Z".parse()?;
    Ok(toasty::create!(Task {
        name: "Original task".to_owned(),
        description: Some("Original description".to_owned()),
        due_at: Some(now),
        completed_at: Some(now),
        priority: TaskPriority::High,
        owner_id,
        project_id,
        parent_id,
        created_at: now,
        updated_at: now,
    })
    .exec(db)
    .await?)
}

async fn project(db: &mut toasty::Db, owner_id: i64, archived: bool) -> toasty::Result<Project> {
    let now = Timestamp::now();
    toasty::create!(Project {
        name: "Project".to_owned(),
        archived_at: archived.then_some(now),
        owner_id,
        created_at: now,
        updated_at: now,
    })
    .exec(db)
    .await
}

async fn tag(db: &mut toasty::Db, owner_id: i64) -> toasty::Result<Tag> {
    let now = Timestamp::now();
    toasty::create!(Tag {
        name: "Tag".to_owned(),
        owner_id,
        created_at: now,
        updated_at: now,
    })
    .exec(db)
    .await
}

async fn link(db: &mut toasty::Db, task_id: i64, tag_id: i64) -> toasty::Result<()> {
    toasty::create!(TaskTag { task_id, tag_id })
        .exec(db)
        .await?;
    Ok(())
}

#[tokio::test]
async fn updates_fields_relations_and_replaces_tags() -> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let original = task(&mut db, user.id, None, None).await?;
    let unrelated = task(&mut db, user.id, None, None).await?;
    let project = project(&mut db, user.id, false).await?;
    let parent = task(&mut db, user.id, Some(project.id), None).await?;
    let old_tag = tag(&mut db, user.id).await?;
    let first_tag = tag(&mut db, user.id).await?;
    let second_tag = tag(&mut db, user.id).await?;
    link(&mut db, original.id, old_tag.id).await?;
    link(&mut db, original.id, first_tag.id).await?;
    link(&mut db, unrelated.id, old_tag.id).await?;
    let due_at = Timestamp::now();
    let request = SaveTaskRequest {
        name: "  Updated task  ".to_owned(),
        description: Some("Updated description".to_owned()),
        due_at: Some(due_at),
        priority: Some(TaskPriority::Urgent),
        project_id: Some(project.id),
        parent_id: Some(parent.id),
        tag_ids: vec![second_tag.id, first_tag.id],
    };

    let Json(response) = update(
        State(state),
        CurrentUser(user),
        Path(original.id),
        Json(request),
    )
    .await?;
    assert_eq!(response.id, original.id);
    assert_eq!(response.name, "Updated task");
    assert_eq!(response.description.as_deref(), Some("Updated description"));
    assert_eq!(response.due_at, Some(due_at));
    assert_eq!(response.priority, TaskPriority::Urgent);
    assert_eq!(response.project_id, Some(project.id));
    assert_eq!(response.parent_id, Some(parent.id));
    assert_eq!(response.tag_ids, [second_tag.id, first_tag.id]);
    assert_eq!(response.owner_id, original.owner_id);
    assert_eq!(response.created_at, original.created_at);
    assert_eq!(response.completed_at, original.completed_at);
    assert!(response.updated_at > original.updated_at);

    let saved = Task::get_by_id(&mut db, &original.id).await?;
    assert_eq!(saved.name, response.name);
    assert_eq!(saved.description, response.description);
    assert_eq!(saved.due_at, response.due_at);
    assert_eq!(saved.priority, response.priority);
    assert_eq!(saved.project_id, response.project_id);
    assert_eq!(saved.parent_id, response.parent_id);
    assert_eq!(saved.updated_at, response.updated_at);
    assert_eq!(saved.completed_at, original.completed_at);
    let links = TaskTag::filter_by_task_id(original.id)
        .exec(&mut db)
        .await?;
    let mut tag_ids: Vec<_> = links.into_iter().map(|link| link.tag_id).collect();
    tag_ids.sort_unstable();
    assert_eq!(tag_ids, [first_tag.id, second_tag.id]);
    let unrelated_links = TaskTag::filter_by_task_id(unrelated.id)
        .exec(&mut db)
        .await?;
    assert_eq!(unrelated_links.len(), 1);
    assert_eq!(
        unrelated_links.first().map(|link| link.tag_id),
        Some(old_tag.id)
    );
    Ok(())
}

#[tokio::test]
async fn clears_optional_fields_and_tags() -> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let project = project(&mut db, user.id, false).await?;
    let parent = task(&mut db, user.id, Some(project.id), None).await?;
    let original = task(&mut db, user.id, Some(project.id), Some(parent.id)).await?;
    let tag = tag(&mut db, user.id).await?;
    link(&mut db, original.id, tag.id).await?;

    let Json(response) = update(
        State(state),
        CurrentUser(user),
        Path(original.id),
        Json(request("Task")),
    )
    .await?;
    assert!(response.description.is_none());
    assert!(response.due_at.is_none());
    assert_eq!(response.priority, TaskPriority::None);
    assert!(response.project_id.is_none());
    assert!(response.parent_id.is_none());
    assert!(response.tag_ids.is_empty());
    assert_eq!(response.completed_at, original.completed_at);
    let saved = Task::get_by_id(&mut db, &original.id).await?;
    assert!(saved.description.is_none());
    assert!(saved.due_at.is_none());
    assert_eq!(saved.priority, TaskPriority::None);
    assert!(saved.project_id.is_none());
    assert!(saved.parent_id.is_none());
    assert!(
        TaskTag::filter_by_task_id(original.id)
            .exec(&mut db)
            .await?
            .is_empty()
    );
    Ok(())
}

#[tokio::test]
async fn rejects_invalid_input_without_changing_task() -> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let original = task(&mut db, user.id, None, None).await?;
    for request in [
        request("  "),
        request(&"x".repeat(256)),
        SaveTaskRequest {
            tag_ids: vec![1, 1],
            ..request("Task")
        },
        SaveTaskRequest {
            tag_ids: vec![0],
            ..request("Task")
        },
        SaveTaskRequest {
            tag_ids: vec![-1],
            ..request("Task")
        },
    ] {
        let user = User::get_by_id(&mut db, &user.id).await?;
        let result = update(
            State(state.clone()),
            CurrentUser(user),
            Path(original.id),
            Json(request),
        )
        .await;
        assert!(matches!(result, Err(TaskError::BadRequest(_))));
    }
    let saved = Task::get_by_id(&mut db, &original.id).await?;
    assert_eq!(saved.name, original.name);
    assert_eq!(saved.updated_at, original.updated_at);
    Ok(())
}

#[tokio::test]
async fn hides_missing_and_unowned_tasks() -> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let now = Timestamp::now();
    let other = toasty::create!(User {
        email: "other@example.com".to_owned(),
        password_hash: "unused".to_owned(),
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    let original = task(&mut db, other.id, None, None).await?;
    for id in [original.id, original.id + 1] {
        let user = User::get_by_id(&mut db, &user.id).await?;
        let result = update(
            State(state.clone()),
            CurrentUser(user),
            Path(id),
            Json(request("Task")),
        )
        .await;
        let error = result.err().ok_or("expected task lookup to fail")?;
        assert!(matches!(error, TaskError::RelatedResourceNotFound(_)));
        assert_eq!(error.into_response().status(), StatusCode::NOT_FOUND);
    }
    let saved = Task::get_by_id(&mut db, &original.id).await?;
    assert_eq!(saved.name, original.name);
    assert_eq!(saved.updated_at, original.updated_at);
    Ok(())
}

#[tokio::test]
async fn rejects_unavailable_relations_without_changing_task_or_tags()
-> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let now = Timestamp::now();
    let other = toasty::create!(User {
        email: "other@example.com".to_owned(),
        password_hash: "unused".to_owned(),
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    let original = task(&mut db, user.id, None, None).await?;
    let own_tag = tag(&mut db, user.id).await?;
    link(&mut db, original.id, own_tag.id).await?;
    let other_project = project(&mut db, other.id, false).await?;
    let archived = project(&mut db, user.id, true).await?;
    let other_parent = task(&mut db, other.id, None, None).await?;
    let other_tag = tag(&mut db, other.id).await?;
    for request in [
        SaveTaskRequest {
            project_id: Some(other_project.id),
            ..request("Task")
        },
        SaveTaskRequest {
            project_id: Some(archived.id),
            ..request("Task")
        },
        SaveTaskRequest {
            project_id: Some(archived.id + 1),
            ..request("Task")
        },
        SaveTaskRequest {
            parent_id: Some(other_parent.id),
            ..request("Task")
        },
        SaveTaskRequest {
            parent_id: Some(other_parent.id + 1),
            ..request("Task")
        },
        SaveTaskRequest {
            tag_ids: vec![own_tag.id, other_tag.id],
            ..request("Task")
        },
        SaveTaskRequest {
            tag_ids: vec![own_tag.id, other_tag.id + 1],
            ..request("Task")
        },
    ] {
        let user = User::get_by_id(&mut db, &user.id).await?;
        let result = update(
            State(state.clone()),
            CurrentUser(user),
            Path(original.id),
            Json(request),
        )
        .await;
        assert!(matches!(result, Err(TaskError::RelatedResourceNotFound(_))));
    }
    let saved = Task::get_by_id(&mut db, &original.id).await?;
    assert_eq!(saved.name, original.name);
    assert_eq!(saved.description, original.description);
    assert_eq!(saved.due_at, original.due_at);
    assert_eq!(saved.priority, original.priority);
    assert_eq!(saved.project_id, original.project_id);
    assert_eq!(saved.parent_id, original.parent_id);
    assert_eq!(saved.updated_at, original.updated_at);
    let links = TaskTag::filter_by_task_id(original.id)
        .exec(&mut db)
        .await?;
    assert_eq!(links.len(), 1);
    assert_eq!(links.first().map(|link| link.tag_id), Some(own_tag.id));
    Ok(())
}

#[tokio::test]
async fn rejects_self_parent_and_descendant_cycles() -> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let original = task(&mut db, user.id, None, None).await?;
    let child = task(&mut db, user.id, None, Some(original.id)).await?;
    let grandchild = task(&mut db, user.id, None, Some(child.id)).await?;
    for parent_id in [original.id, child.id, grandchild.id] {
        let user = User::get_by_id(&mut db, &user.id).await?;
        let request = SaveTaskRequest {
            parent_id: Some(parent_id),
            ..request("Task")
        };
        let result = update(
            State(state.clone()),
            CurrentUser(user),
            Path(original.id),
            Json(request),
        )
        .await;
        assert!(matches!(result, Err(TaskError::BadRequest(_))));
    }
    let saved = Task::get_by_id(&mut db, &original.id).await?;
    assert!(saved.parent_id.is_none());
    assert_eq!(saved.updated_at, original.updated_at);
    Ok(())
}

#[tokio::test]
async fn rejects_project_mismatch_with_parent_and_children()
-> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let project = project(&mut db, user.id, false).await?;
    let parent = task(&mut db, user.id, Some(project.id), None).await?;
    let original = task(&mut db, user.id, None, None).await?;
    let requests = [
        (
            original.id,
            SaveTaskRequest {
                parent_id: Some(parent.id),
                ..request("Task")
            },
        ),
        (parent.id, request("Task")),
    ];
    let child = task(&mut db, user.id, Some(project.id), Some(parent.id)).await?;
    for (id, request) in requests {
        let user = User::get_by_id(&mut db, &user.id).await?;
        let result = update(
            State(state.clone()),
            CurrentUser(user),
            Path(id),
            Json(request),
        )
        .await;
        assert!(matches!(result, Err(TaskError::BadRequest(_))));
    }
    assert_eq!(
        Task::get_by_id(&mut db, &parent.id).await?.project_id,
        Some(project.id)
    );
    assert_eq!(
        Task::get_by_id(&mut db, &child.id).await?.project_id,
        Some(project.id)
    );
    Ok(())
}

#[tokio::test]
async fn edits_task_in_unchanged_archived_project() -> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let project = project(&mut db, user.id, true).await?;
    let original = task(&mut db, user.id, Some(project.id), None).await?;
    let request = SaveTaskRequest {
        project_id: Some(project.id),
        ..request("Updated task")
    };
    let Json(response) = update(
        State(state),
        CurrentUser(user),
        Path(original.id),
        Json(request),
    )
    .await?;
    assert_eq!(response.name, "Updated task");
    assert_eq!(response.project_id, Some(project.id));
    Ok(())
}

#[tokio::test]
async fn update_route_authenticates_and_deserializes_request()
-> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let original = task(&mut db, user.id, None, None).await?;
    let token = create_session(&mut db, user.id, Timestamp::now()).await?;
    let app = get_app_router(state);
    for authenticated in [false, true] {
        let mut request = Request::builder()
            .method("PUT")
            .uri(format!("/tasks/{}", original.id))
            .header("Content-Type", "application/json");
        if authenticated {
            request = request.header("Cookie", format!("rincuma_session={}", token.expose()));
        }
        let request = request.body(Body::from(r#"{"name":"Updated via HTTP"}"#))?;
        let response = app.clone().oneshot(request).await?;
        assert_eq!(
            response.status(),
            if authenticated {
                StatusCode::OK
            } else {
                StatusCode::UNAUTHORIZED
            }
        );
    }
    let saved = Task::get_by_id(&mut db, &original.id).await?;
    assert_eq!(saved.name, "Updated via HTTP");
    assert_eq!(saved.completed_at, original.completed_at);
    Ok(())
}
