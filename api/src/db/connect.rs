use crate::config::AppConfig;
use toasty::Db;

pub async fn connect(config: &AppConfig) -> toasty::Result<Db> {
    Db::builder()
        .models(toasty::models!(crate::*))
        .connect(&config.database_url())
        .await
}
