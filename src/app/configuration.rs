use std::env;

use anyhow::Context;
use config::{
    Config,
    File,
};
use derivative::Derivative;
use sqlx::postgres::{
    PgConnectOptions,
    PgSslMode,
};

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
    pub email_client: EmailClientSettings,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct ApplicationSettings {
    pub base_url: String,
    pub host: String,
    pub max_pending_connections: u32,
    pub port: u16,
}

#[derive(Derivative, Clone, Debug, serde::Deserialize)]
pub struct DatabaseSettings {
    pub connect_timeout_seconds: u64,
    pub name: String,
    pub host: String,
    pub max_db_connections: u32,
    #[derivative(Debug = "ignore")]
    pub password: String,
    pub port: u16,
    pub require_ssl: bool,
    pub username: String,
}

#[derive(Derivative, Clone, Debug, serde::Deserialize)]
pub struct EmailClientSettings {
    pub base_url: String,
    pub sender_email: String,
    pub timeout_secs: u64,
    #[derivative(Debug = "ignore")]
    pub token: String,
}

impl ApplicationSettings {
    pub fn binding_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

impl DatabaseSettings {
    pub fn pgserver_connection_options(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(&self.password)
            .port(self.port)
            .ssl_mode(ssl_mode)
    }
    pub fn database_connection_options(&self) -> PgConnectOptions {
        self.pgserver_connection_options().database(&self.name)
    }
}

/// Load the configuration from the directory: `configuration`.
///
/// It fails if:
/// - the `RUN_MODE` env variable is not set
/// - the `configuration/base` file is missing
/// - the `configuration/${APP_ENVIRONMENT}` file is missing
/// - the `configuration/*` files have missing or unexpected fields
///
/// # Examples
///
/// ```rust
/// use newsletter::app::load_configuration;
///
/// assert!(load_configuration().is_ok());
/// ```
pub fn load_configuration() -> Result<Settings, anyhow::Error> {
    let mut config = Config::new();
    config.merge(File::with_name("configuration/base").required(true))?;
    let app_environment =
        env::var("APP_ENVIRONMENT").context("`APP_ENVIRONMENT` is missing or invalid")?;
    config.merge(File::with_name(&format!("configuration/{}", app_environment)).required(true))?;
    config.merge(config::Environment::with_prefix("app").separator("__"))?;
    config.try_into().map(Ok)?
}
