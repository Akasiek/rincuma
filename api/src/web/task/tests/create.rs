use axum::{Json, extract::State, http::StatusCode};
use jiff::Timestamp;

use super::{SaveTaskRequest, create};
use crate::{
    db::{Project, Tag, Task, TaskPriority, TaskTag, User},
    web::{auth::CurrentUser, task::TaskError, test_support::state_with_user},
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

#[tokio::test]
async fn creates_task_with_defaults() -> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let owner_id = user.id;

    let (status, Json(response)) = create(
        State(state.clone()),
        CurrentUser(user),
        Json(request("  Task  ")),
    )
    .await?;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(response.name, "Task");
    assert_eq!(response.owner_id, owner_id);
    assert_eq!(response.priority, TaskPriority::None);
    assert!(response.project_id.is_none());
    assert!(response.parent_id.is_none());
    assert!(response.tag_ids.is_empty());

    let mut db = state.db().clone();
    let saved = Task::get_by_id(&mut db, &response.id).await?;
    assert_eq!(saved.priority, TaskPriority::None);
    Ok(())
}

#[tokio::test]
async fn creates_task_with_owned_relations() -> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let owner_id = user.id;
    let now = Timestamp::now();
    let mut db = state.db().clone();
    let project = toasty::create!(Project {
        name: "Project".to_owned(),
        owner_id,
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    let parent = toasty::create!(Task {
        name: "Parent".to_owned(),
        owner_id,
        project_id: Some(project.id),
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    let first_tag = toasty::create!(Tag {
        name: "First".to_owned(),
        owner_id,
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    let second_tag = toasty::create!(Tag {
        name: "Second".to_owned(),
        owner_id,
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    let due_at = now;
    let request = SaveTaskRequest {
        name: "  Child task  ".to_owned(),
        description: Some("Description".to_owned()),
        due_at: Some(due_at),
        priority: Some(TaskPriority::High),
        project_id: Some(project.id),
        parent_id: Some(parent.id),
        tag_ids: vec![second_tag.id, first_tag.id],
    };

    let (status, Json(response)) =
        create(State(state.clone()), CurrentUser(user), Json(request)).await?;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(response.name, "Child task");
    assert_eq!(response.description.as_deref(), Some("Description"));
    assert_eq!(response.due_at, Some(due_at));
    assert!(response.completed_at.is_none());
    assert_eq!(response.priority, TaskPriority::High);
    assert_eq!(response.owner_id, owner_id);
    assert_eq!(response.project_id, Some(project.id));
    assert_eq!(response.parent_id, Some(parent.id));
    assert_eq!(response.tag_ids, [second_tag.id, first_tag.id]);
    assert_eq!(response.created_at, response.updated_at);

    let saved = Task::get_by_id(&mut db, &response.id).await?;
    assert_eq!(saved.name, response.name);
    assert_eq!(saved.owner_id, owner_id);
    assert_eq!(saved.project_id, Some(project.id));
    assert_eq!(saved.parent_id, Some(parent.id));
    for tag_id in [first_tag.id, second_tag.id] {
        let link = TaskTag::filter_by_tag_id(tag_id)
            .first()
            .exec(&mut db)
            .await?;
        assert_eq!(link.map(|link| link.task_id), Some(response.id));
    }
    Ok(())
}

#[tokio::test]
async fn rejects_invalid_task_input() -> Result<(), Box<dyn std::error::Error>> {
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
    ] {
        let (state, user) = state_with_user().await?;
        let result = create(State(state), CurrentUser(user), Json(request)).await;
        assert!(matches!(result, Err(TaskError::BadRequest(_))));
    }
    Ok(())
}

#[tokio::test]
async fn rejects_unowned_and_archived_relations() -> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let owner_id = user.id;
    let now = Timestamp::now();
    let mut db = state.db().clone();
    let other = toasty::create!(User {
        email: "other@example.com".to_owned(),
        password_hash: "unused".to_owned(),
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    let other_project = toasty::create!(Project {
        name: "Other project".to_owned(),
        owner_id: other.id,
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    let archived_project = toasty::create!(Project {
        name: "Archived project".to_owned(),
        owner_id,
        archived_at: Some(now),
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    let other_parent = toasty::create!(Task {
        name: "Other parent".to_owned(),
        owner_id: other.id,
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    let other_tag = toasty::create!(Tag {
        name: "Other tag".to_owned(),
        owner_id: other.id,
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;

    for request in [
        SaveTaskRequest {
            project_id: Some(other_project.id),
            ..request("Task")
        },
        SaveTaskRequest {
            project_id: Some(archived_project.id),
            ..request("Task")
        },
        SaveTaskRequest {
            parent_id: Some(other_parent.id),
            ..request("Task")
        },
        SaveTaskRequest {
            tag_ids: vec![other_tag.id],
            ..request("Task")
        },
    ] {
        let current_user = User::get_by_id(&mut db, &owner_id).await?;
        let result = create(
            State(state.clone()),
            CurrentUser(current_user),
            Json(request),
        )
        .await;
        assert!(matches!(result, Err(TaskError::RelatedResourceNotFound(_))));
    }
    Ok(())
}

#[tokio::test]
async fn rejects_parent_from_different_project() -> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let now = Timestamp::now();
    let mut db = state.db().clone();
    let project = toasty::create!(Project {
        name: "Project".to_owned(),
        owner_id: user.id,
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    let parent = toasty::create!(Task {
        name: "Parent".to_owned(),
        owner_id: user.id,
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    let request = SaveTaskRequest {
        project_id: Some(project.id),
        parent_id: Some(parent.id),
        ..request("Task")
    };

    let result = create(State(state), CurrentUser(user), Json(request)).await;
    assert!(matches!(result, Err(TaskError::BadRequest(_))));
    Ok(())
}
