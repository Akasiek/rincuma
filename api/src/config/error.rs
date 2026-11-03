#[derive(Debug, thiserror::Error)]
pub(crate) enum ConfigError {
    #[error("invalid environment configuration: {0}")]
    Environment(#[from] envy::Error),

    #[error("PORT must be greater than zero")]
    ZeroPort,

    #[error("ADMIN_EMAIL and ADMIN_PASSWORD must be set together")]
    IncompleteAdminCredentials,

    #[error("ADMIN_EMAIL is not a valid email address")]
    InvalidAdminEmail,

    #[error("ADMIN_PASSWORD must be between 1 and 1024 bytes")]
    InvalidAdminPassword,
}
