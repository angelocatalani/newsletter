use std::env;

use config::{
    Config,
    File,
};

#[derive(serde::Deserialize)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    pub host: String,
    pub max_pending_connections: u32,
    pub port: u16,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub connect_timeout_seconds: u64,
    pub database_name: String,
    pub host: String,
    pub password: String,
    pub port: u16,
    pub username: String,
    pub max_db_connections: u32,
}

impl ApplicationSettings {
    pub fn binding_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

impl DatabaseSettings {
    pub fn database_connection_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }
    pub fn pgserver_connection_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/",
            self.username, self.password, self.host, self.port
        )
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
/// use newsletter::configuration::load_configuration;
///
/// load_configuration();
/// ```
pub fn load_configuration() -> Settings {
    let mut config = Config::new();
    config
        .merge(File::with_name("configuration/base").required(true))
        .expect("error loading configuration/base");
    let app_environment = env::var("APP_ENVIRONMENT").expect("APP_ENVIRONMENT env variable is not set");
    config
        .merge(File::with_name(&format!("configuration/{}", app_environment)).required(true))
        .unwrap_or_else(|_| panic!("error loading configuration/{}", app_environment));

    // Add in settings from environment variables (with a prefix of APP and '__' as separator)
    // E.g. `APP_APPLICATION__PORT=5001 would set `Settings.application.port` settings.merge(config::Environment::with_prefix("app").separator("__"))?;
    config.merge(config::Environment::with_prefix("app").separator("__")).expect("error loading configuration from environment variables");

    config.try_into().expect("error loading configuration")
}
