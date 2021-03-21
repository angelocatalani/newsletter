use std::net::TcpListener;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use newsletter::configuration::load_configuration;
use newsletter::startup::run;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
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
