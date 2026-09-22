use toasty::Db;
use crate::config::AppConfig;

pub async fn connect(config: &AppConfig) -> toasty::Result<Db> {
    Db::builder()
        .models(toasty::models!(crate::*))
        .connect(&config.database_url())
        .await
}
