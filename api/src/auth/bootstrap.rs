use jiff::Timestamp;
use thiserror::Error;
use toasty::Db;
use toasty_core::driver::IsolationLevel;

use crate::{
    auth::password::{PasswordError, hash_password},
    config::AdminCredentials,
    db::User,
};

#[derive(Debug, Error)]
pub(crate) enum BootstrapError {
    #[error("admin bootstrap database operation failed")]
    Database(#[from] toasty::Error),
    #[error("admin password hashing failed")]
    Password(#[from] PasswordError),
}

/// Creates the admin account from environment credentials when no users exist yet.
pub(crate) async fn bootstrap_admin(
    db: &Db,
    admin: Option<AdminCredentials>,
    registration_enabled: bool,
) -> Result<(), BootstrapError> {
    let mut db = db.clone();
    let mut transaction = db
        .transaction_builder()
        .isolation(IsolationLevel::Serializable)
        .begin()
        .await?;

    if User::any_exist(&mut transaction).await? {
        log_skipped_bootstrap(admin.is_some());
        return Ok(());
    }

    let Some(admin) = admin else {
        warn_when_locked_out(registration_enabled);
        return Ok(());
    };

    create_admin(&mut transaction, admin).await?;
    transaction.commit().await?;
    Ok(())
}

async fn create_admin(
    executor: &mut dyn toasty::Executor,
    admin: AdminCredentials,
) -> Result<(), BootstrapError> {
    let password_hash = hash_password(admin.password).await?;
    let now = Timestamp::now();

    toasty::create!(User {
        email: admin.email.clone(),
        password_hash,
        is_admin: true,
        created_at: now,
        updated_at: now,
    })
    .exec(executor)
    .await?;

    tracing::info!(email = %admin.email, "Created admin account from ADMIN_EMAIL");
    Ok(())
}

fn log_skipped_bootstrap(has_admin_credentials: bool) {
    if has_admin_credentials {
        tracing::info!("Users already exist, skipping admin bootstrap");
    }
}

fn warn_when_locked_out(registration_enabled: bool) {
    if !registration_enabled {
        tracing::warn!(
            "No users exist and registration is disabled; set ADMIN_EMAIL and ADMIN_PASSWORD to create the admin account"
        );
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic_in_result_fn)]

    use super::bootstrap_admin;
    use crate::{
        auth::password::verify_password, config::AdminCredentials, db::User,
        web::test_support::test_db,
    };

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn credentials() -> AdminCredentials {
        AdminCredentials {
            email: "admin@example.com".to_owned(),
            password: "correct horse battery staple".to_owned(),
        }
    }

    #[tokio::test]
    async fn creates_admin_in_empty_database() -> TestResult {
        let mut db = test_db().await?;

        bootstrap_admin(&db, Some(credentials()), false).await?;

        let admin = User::get_by_email(&mut db, "admin@example.com".to_owned()).await?;
        assert!(admin.is_admin);
        let password_matches = verify_password(
            "correct horse battery staple".to_owned(),
            admin.password_hash,
        )
        .await?;
        assert!(password_matches);
        Ok(())
    }

    #[tokio::test]
    async fn skips_admin_when_users_exist() -> TestResult {
        let mut db = test_db().await?;
        let now = jiff::Timestamp::now();
        toasty::create!(User {
            email: "existing@example.com".to_owned(),
            password_hash: "unused".to_owned(),
            created_at: now,
            updated_at: now,
        })
        .exec(&mut db)
        .await?;

        bootstrap_admin(&db, Some(credentials()), false).await?;

        let admin = User::filter_by_email("admin@example.com".to_owned())
            .first()
            .exec(&mut db)
            .await?;
        assert!(admin.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn leaves_empty_database_without_credentials() -> TestResult {
        let mut db = test_db().await?;

        bootstrap_admin(&db, None, false).await?;

        assert!(!User::any_exist(&mut db).await?);
        Ok(())
    }
}
