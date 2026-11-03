use std::path::PathBuf;

use jiff::Timestamp;

use crate::{app_state::AppState, config::AppConfig, db::User};

pub(crate) async fn test_db() -> toasty::Result<toasty::Db> {
    let db = toasty::Db::builder()
        .models(toasty::models!(crate::*))
        .build(toasty_driver_sqlite::Sqlite::in_memory())
        .await?;
    db.push_schema().await?;
    Ok(db)
}

pub(crate) async fn empty_state(config: AppConfig) -> toasty::Result<AppState> {
    Ok(AppState::new(config, test_db().await?))
}

pub(crate) async fn state_with_user() -> toasty::Result<(AppState, User)> {
    let mut db = test_db().await?;

    let now = Timestamp::now();
    let user = toasty::create!(User {
        email: "test@example.com".to_owned(),
        password_hash: "unused".to_owned(),
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;

    Ok((AppState::new(AppConfig::for_test(PathBuf::new()), db), user))
}
