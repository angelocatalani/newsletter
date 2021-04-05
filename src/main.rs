use std::net::TcpListener;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use newsletter::configuration::load_configuration;
use newsletter::startup::run;
use newsletter::telemetry::setup_tracing;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    setup_tracing("zero2prod".into(), "info".into());

    let configuration = load_configuration();
    let tcp_listener = TcpListener::bind(configuration.application.binding_address())
        .expect("error binding to tcp address");
    let postgres_pool = postgres_pool(
        &configuration.database.database_connection_url(),
        configuration.database.max_db_connections,
        configuration.database.connect_timeout_seconds
    )
    .await;

    run(
        tcp_listener,
        postgres_pool,
        configuration.application.max_pending_connections,
    )?
    .await
}

async fn postgres_pool(database_url: &str, max_connections: u32,connect_timeout_seconds:u64) -> PgPool {
    PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(connect_timeout_seconds))
        .max_connections(max_connections)
        .connect(database_url)
        .await
        .expect("error creating postgres connection pool")
}
