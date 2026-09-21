use crate::app_state::AppState;
use crate::config::AppConfig;

mod app_state;
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
