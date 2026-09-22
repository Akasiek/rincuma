use crate::app_state::AppState;
use crate::config::AppConfig;

mod app_state;
mod auth;
mod config;
mod db;
mod tracer;
mod web;

pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenvy::dotenv().ok();
    tracer::init();

    let config = AppConfig::from_env()?;
    let db = db::connect(&config).await?;
    let state = AppState::new(config, db);

    web::run(state).await?;

    Ok(())
}

#[cfg(feature = "migration-cli")]
/// Runs the Toasty schema migration command-line interface.
///
/// # Errors
///
/// Returns an error when configuration, database connection, or migration execution fails.
pub async fn run_migration_cli() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenvy::dotenv().ok();

    let config = AppConfig::from_env()?;
    let db = db::connect(&config).await?;
    toasty_cli::ToastyCli::new(db).parse_and_run().await?;

    Ok(())
}
