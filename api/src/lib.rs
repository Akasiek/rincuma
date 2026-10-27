use crate::app_state::AppState;
use crate::config::AppConfig;

mod app_state;
mod auth;
mod config;
mod db;
mod notifications;
pub mod recurrence;
mod tracer;
mod web;

pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenvy::dotenv().ok();
    tracer::init();

    tracing::info!(version = env!("CARGO_PKG_VERSION"), "Starting API");
    let config = AppConfig::from_env()?;
    tracing::info!("Connecting to database");
    let db = db::connect(&config).await?;
    tracing::info!("Database connection established");
    let state = AppState::new(config, db);

    let notification_worker = notifications::worker::spawn(state.clone())?;
    let web_server_result = web::run(state).await;

    notification_worker.abort();
    if let Err(error) = &web_server_result {
        tracing::error!(error = %error, "API server stopped unexpectedly");
    } else {
        tracing::info!("API server stopped");
    }
    tracing::info!("Reminder worker stop requested");
    web_server_result?;

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
