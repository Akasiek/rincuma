use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;

use serde::Deserialize;

use crate::config::error::ConfigError;
use crate::config::model::AppConfig;

#[derive(Deserialize)]
struct EnvConfig {
    #[serde(default = "default_data_dir")]
    data_dir: PathBuf,
    #[serde(default = "default_host")]
    host: IpAddr,
    #[serde(default = "default_port")]
    port: u16,
    db_user: String,
    db_pass: String,
    #[serde(default = "default_db_host")]
    db_host: String,
    #[serde(default = "default_db_port")]
    db_port: u16,
    #[serde(default = "default_db_name")]
    db_name: String,
    #[serde(default = "default_cookie_secure")]
    cookie_secure: bool,
}

impl AppConfig {
    pub(crate) fn from_env() -> Result<Self, ConfigError> {
        let config = envy::from_env::<EnvConfig>()?;

        if config.port == 0 {
            return Err(ConfigError::ZeroPort);
        }

        let database_url = format!(
            "postgresql://{}:{}@{}:{}/{}",
            config.db_user, config.db_pass, config.db_host, config.db_port, config.db_name
        );

        Ok(Self {
            data_dir: config.data_dir,
            database_url,
            listen_address: SocketAddr::new(config.host, config.port),
            cookie_secure: config.cookie_secure,
        })
    }
}

fn default_data_dir() -> PathBuf {
    "./data".into()
}

fn default_host() -> IpAddr {
    IpAddr::from([0, 0, 0, 0])
}

fn default_port() -> u16 {
    7878
}

fn default_db_host() -> String {
    "localhost".into()
}

fn default_db_port() -> u16 {
    5432
}

fn default_db_name() -> String {
    "rincuma".into()
}

const fn default_cookie_secure() -> bool {
    true
}
