#[derive(Debug, thiserror::Error)]
pub(crate) enum ConfigError {
    #[error("invalid environment configuration: {0}")]
    Environment(#[from] envy::Error),

    #[error("PORT must be greater than zero")]
    ZeroPort,
}
