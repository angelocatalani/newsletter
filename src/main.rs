use std::net::TcpListener;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use newsletter::configuration::load_configuration;
use newsletter::startup::run;
use tracing_subscriber::{EnvFilter, Registry};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing::subscriber::set_global_default;
use tracing_subscriber::layer::SubscriberExt;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new("zero2prod".into(),std::io::stdout );
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    // `set_global_default` can be used by applications to specify
    // what subscriber should be used to process spans.
    set_global_default(subscriber).expect("Failed to set subscriber");

    let configuration = load_configuration();
    let tcp_listener = TcpListener::bind(configuration.application.binding_address())
        .expect("error binding to tcp address");
    let postgres_pool = postgres_pool(
        &configuration.database.database_connection_url(),
        configuration.database.max_db_connections,
    )
    .await;

    run(
        tcp_listener,
        postgres_pool,
        configuration.application.max_pending_connections,
    )?
    .await
}

async fn postgres_pool(database_url: &str, max_connections: u32) -> PgPool {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await
        .expect("error creating postgres connection pool")
}
