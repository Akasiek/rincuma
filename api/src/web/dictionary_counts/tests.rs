#![allow(clippy::panic_in_result_fn)]

use jiff::Timestamp;

use super::{projects, tags};
use crate::{
    db::{Project, Tag, Task, TaskTag, User},
    web::test_support::state_with_user,
};

#[tokio::test]
async fn counts_owned_live_tasks_including_completed_tasks()
-> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let now = Timestamp::now();
    let project = toasty::create!(Project {
        name: "Project".to_owned(),
        owner_id: user.id,
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    let tag = toasty::create!(Tag {
        name: "Tag".to_owned(),
        owner_id: user.id,
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    let other = toasty::create!(User {
        email: "counts@example.test".to_owned(),
        password_hash: "unused".to_owned(),
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;

    for (owner_id, completed_at, deleted_at) in [
        (user.id, None, None),
        (user.id, Some(now), None),
        (user.id, None, Some(now)),
        (other.id, None, None),
    ] {
        let task = toasty::create!(Task {
            name: "Task".to_owned(),
            owner_id,
            project_id: Some(project.id),
            completed_at,
            deleted_at,
            created_at: now,
            updated_at: now,
        })
        .exec(&mut db)
        .await?;
        toasty::create!(TaskTag {
            task_id: task.id,
            tag_id: tag.id
        })
        .exec(&mut db)
        .await?;
    }

    assert_eq!(
        projects(&mut db, user.id, &[project.id])
            .await?
            .get(&project.id),
        Some(&2)
    );
    assert_eq!(
        tags(&mut db, user.id, &[tag.id]).await?.get(&tag.id),
        Some(&2)
    );
    assert!(
        projects(&mut db, user.id, &[project.id + 1])
            .await?
            .is_empty()
    );
    assert!(tags(&mut db, user.id, &[tag.id + 1]).await?.is_empty());
    assert!(projects(&mut db, user.id, &[]).await?.is_empty());
    assert!(tags(&mut db, user.id, &[]).await?.is_empty());
    Ok(())
}
