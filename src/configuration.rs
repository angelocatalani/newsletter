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
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }
}
/// Load the configuration from the directory: `configuration`.
/// If the `RUN_MODE` env variable is not set to `local` or `production`,
/// it fails.
///
/// # Examples
///
/// ```rust
/// use newsletter::configuration::load_configuration;
///
/// load_configuration().expect("error loading configuration");
/// ```
pub fn load_configuration() -> Result<Settings, config::ConfigError> {
    let mut config = Config::new();
    config.merge(File::with_name("configuration/base"))?;
    let env = env::var("RUN_MODE").expect("RUN_MODE env variable is not set");
    config.merge(File::with_name(&format!("configuration/{}", env)))?;
    config.try_into()
}
