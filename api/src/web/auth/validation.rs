use crate::auth::password::PasswordError;

use super::error::AuthError;

pub(super) fn normalize_email(email: &str) -> Result<String, AuthError> {
    let email = email.trim().to_lowercase();
    let Some((local, domain)) = email.split_once('@') else {
        return Err(AuthError::BadRequest);
    };

    if email.len() > 254
        || local.is_empty()
        || domain.is_empty()
        || domain.contains('@')
        || email.chars().any(char::is_whitespace)
    {
        return Err(AuthError::BadRequest);
    }

    Ok(email)
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn map_password_error(error: PasswordError) -> AuthError {
    match error {
        PasswordError::TooShort | PasswordError::TooLong => AuthError::BadRequest,
        PasswordError::InvalidHash | PasswordError::Hashing | PasswordError::Task(_) => {
            AuthError::Password(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_email;

    #[test]
    fn normalizes_email() {
        assert_eq!(
            normalize_email(" User@Example.COM ").ok().as_deref(),
            Some("user@example.com")
        );
    }

    #[test]
    fn rejects_invalid_email() {
        assert!(normalize_email("invalid").is_err());
        assert!(normalize_email("@").is_err());
        assert!(normalize_email("user@").is_err());
        assert!(normalize_email("@example.com").is_err());
        assert!(normalize_email("user @example.com").is_err());
        assert!(normalize_email("a@b@example.com").is_err());
    }
}
