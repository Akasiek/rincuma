use std::fmt;

use crate::auth::{email::normalize_email, password::validate_length};
use crate::config::error::ConfigError;

/// Credentials for the account created on first startup from `ADMIN_EMAIL` and `ADMIN_PASSWORD`.
pub(crate) struct AdminCredentials {
    pub(crate) email: String,
    pub(crate) password: String,
}

impl AdminCredentials {
    pub(super) fn from_env(
        email: Option<String>,
        password: Option<String>,
    ) -> Result<Option<Self>, ConfigError> {
        match (non_empty(email), non_empty(password)) {
            (None, None) => Ok(None),
            (Some(email), Some(password)) => Self::validated(&email, password).map(Some),
            _ => Err(ConfigError::IncompleteAdminCredentials),
        }
    }

    fn validated(email: &str, password: String) -> Result<Self, ConfigError> {
        let email = normalize_email(email).ok_or(ConfigError::InvalidAdminEmail)?;
        validate_length(&password).map_err(|_| ConfigError::InvalidAdminPassword)?;

        Ok(Self { email, password })
    }
}

/// Compose passes unset variables as empty strings, so treat them as absent.
fn non_empty(value: Option<String>) -> Option<String> {
    value.filter(|value| !value.is_empty())
}

impl fmt::Debug for AdminCredentials {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AdminCredentials")
            .field("email", &self.email)
            .field("password", &"<redacted>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::AdminCredentials;
    use crate::config::error::ConfigError;

    fn load(
        email: Option<&str>,
        password: Option<&str>,
    ) -> Result<Option<AdminCredentials>, ConfigError> {
        AdminCredentials::from_env(email.map(str::to_owned), password.map(str::to_owned))
    }

    #[test]
    fn missing_or_empty_credentials_are_absent() {
        assert!(matches!(load(None, None), Ok(None)));
        assert!(matches!(load(Some(""), Some("")), Ok(None)));
    }

    #[test]
    fn requires_both_credentials() {
        assert!(matches!(
            load(Some("admin@example.com"), None),
            Err(ConfigError::IncompleteAdminCredentials)
        ));
        assert!(matches!(
            load(Some("admin@example.com"), Some("")),
            Err(ConfigError::IncompleteAdminCredentials)
        ));
        assert!(matches!(
            load(None, Some("secret")),
            Err(ConfigError::IncompleteAdminCredentials)
        ));
    }

    #[test]
    fn rejects_invalid_credentials() {
        assert!(matches!(
            load(Some("invalid"), Some("secret")),
            Err(ConfigError::InvalidAdminEmail)
        ));
        assert!(matches!(
            load(Some("admin@example.com"), Some(&"x".repeat(1_025))),
            Err(ConfigError::InvalidAdminPassword)
        ));
    }

    #[test]
    fn normalizes_email() {
        let credentials = load(Some(" Admin@Example.COM "), Some("secret"));

        assert!(matches!(
            credentials,
            Ok(Some(AdminCredentials { ref email, .. })) if email == "admin@example.com"
        ));
    }

    #[test]
    fn debug_output_redacts_password() {
        let credentials = AdminCredentials {
            email: "admin@example.com".to_owned(),
            password: "secret".to_owned(),
        };

        assert!(!format!("{credentials:?}").contains("secret"));
    }
}
