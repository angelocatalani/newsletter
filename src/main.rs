use std::net::TcpListener;

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use newsletter::configuration::load_configuration;
use newsletter::run;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let configuration = load_configuration().expect("error reading local configuration");
    let tcp_listener = TcpListener::bind(configuration.application.binding_address())
        .expect("error binding to tcp address");
    let postgres_pool = PgPoolOptions::new()
        .max_connections(configuration.database.max_db_connections)
        .connect(&configuration.database.connection_string())
        .await
        .expect("error connecting to postgres");
    run(
        tcp_listener,
        postgres_pool,
        configuration.application.max_pending_connections,
    )?
    .await
}
