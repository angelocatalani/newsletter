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
/// - the `configuration/${RUN_MODE}` file is missing
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
        .merge(File::with_name("configuration/base"))
        .expect("error loading configuration/base");
    let env = env::var("RUN_MODE").expect("RUN_MODE env variable is not set");
    config
        .merge(File::with_name(&format!("configuration/{}", env)))
        .unwrap_or_else(|_| panic!("error loading configuration/{}", env));
    config.try_into().expect("error loading configuration")
}
