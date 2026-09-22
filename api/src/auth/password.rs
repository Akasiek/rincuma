use argon2::{
    Argon2,
    password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash},
};
use std::sync::{Arc, OnceLock};
use thiserror::Error;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

const MAX_PASSWORD_LENGTH: usize = 1_024;
const MAX_CONCURRENT_HASHES: usize = 2;
pub(crate) const DUMMY_PASSWORD_HASH: &str =
    "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
static HASHING_LIMIT: OnceLock<Arc<Semaphore>> = OnceLock::new();

#[derive(Debug, Error)]
pub(crate) enum PasswordError {
    #[error("password exceeds the maximum length")]
    TooLong,
    #[error("password hash is invalid")]
    InvalidHash,
    #[error("password hashing failed")]
    Hashing,
    #[error("password hashing task failed")]
    Task(#[from] tokio::task::JoinError),
}

pub(crate) async fn hash_password(password: String) -> Result<String, PasswordError> {
    validate_length(&password)?;

    let permit = acquire_hashing_permit().await?;
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        Argon2::default()
            .hash_password(password.as_bytes())
            .map(|hash| hash.to_string())
            .map_err(|_| PasswordError::Hashing)
    })
    .await?
}

pub(crate) async fn verify_password(
    password: String,
    encoded_hash: String,
) -> Result<bool, PasswordError> {
    validate_length(&password)?;

    let permit = acquire_hashing_permit().await?;
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        let parsed_hash =
            PasswordHash::new(&encoded_hash).map_err(|_| PasswordError::InvalidHash)?;

        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    })
    .await?
}

fn validate_length(password: &str) -> Result<(), PasswordError> {
    if password.len() > MAX_PASSWORD_LENGTH {
        return Err(PasswordError::TooLong);
    }

    Ok(())
}

async fn acquire_hashing_permit() -> Result<OwnedSemaphorePermit, PasswordError> {
    hashing_limit()
        .acquire_owned()
        .await
        .map_err(|_| PasswordError::Hashing)
}

fn hashing_limit() -> Arc<Semaphore> {
    Arc::clone(HASHING_LIMIT.get_or_init(|| Arc::new(Semaphore::new(MAX_CONCURRENT_HASHES))))
}

#[cfg(test)]
mod tests {
    use super::{DUMMY_PASSWORD_HASH, PasswordError, hash_password, verify_password};

    #[tokio::test]
    async fn hashes_and_verifies_password() {
        let hash_result = hash_password("correct horse battery staple".to_owned()).await;
        assert!(hash_result.is_ok());
        let Ok(hash) = hash_result else {
            return;
        };

        assert!(hash.starts_with("$argon2id$"));
        assert!(matches!(
            verify_password("correct horse battery staple".to_owned(), hash.clone()).await,
            Ok(true)
        ));
        assert!(matches!(
            verify_password("wrong password".to_owned(), hash).await,
            Ok(false)
        ));
    }

    #[tokio::test]
    async fn rejects_excessively_long_password_before_hashing() {
        let result = hash_password("x".repeat(1_025)).await;

        assert!(matches!(result, Err(PasswordError::TooLong)));
    }

    #[tokio::test]
    async fn dummy_hash_is_valid_and_never_matches() {
        assert!(matches!(
            verify_password("any password".to_owned(), DUMMY_PASSWORD_HASH.to_owned()).await,
            Ok(false)
        ));
    }
}
