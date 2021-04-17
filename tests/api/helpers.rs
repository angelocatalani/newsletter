use reqwest::Response;
use sqlx::{
    Connection,
    PgConnection,
    PgPool,
};
use uuid::Uuid;

use newsletter::app::{
    load_configuration,
    setup_tracing,
    DatabaseSettings,
    NewsletterApp,
};

// ensure the `tracing` is instantiated only once
lazy_static::lazy_static! {
 static ref TRACING: () = setup_tracing("test".into(),"debug".into());
}

pub struct TestApp {
    pub address: String,
    pub pool: PgPool,
}

/// When a `tokio` runtime is shut down all tasks spawned on it are dropped.
///
/// `actix_rt::test` spins up a new runtime at the beginning of each test case
/// and they shut down at the end of each test case.
pub async fn spawn_app() -> TestApp {
    lazy_static::initialize(&TRACING);

    let mut configuration = load_configuration().unwrap();
    configuration.database.database_name = Uuid::new_v4().to_string();
    configuration.application.port = 0;

    let postgres_pool = setup_test_database(configuration.database.clone()).await;

    let app = NewsletterApp::from(configuration)
        .await
        .expect("error building app");

    tokio::spawn(app.server.expect("error building server"));

    TestApp {
        // the request is done with the protocol:ip:port
        address: format!("http://127.0.0.1:{}", app.port),
        pool: postgres_pool,
    }
}

pub async fn send_post_request(endpoint: &str, body: String) -> Response {
    reqwest::Client::new()
        .post(endpoint)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Fail to execute post request")
}

async fn setup_test_database(database_settings: DatabaseSettings) -> PgPool {
    let mut connection =
        PgConnection::connect_with(&database_settings.pgserver_connection_options())
            .await
            .expect("error connecting to postgres");

    sqlx::query(&format!(
        "CREATE DATABASE \"{}\"",
        database_settings.database_name
    ))
    .execute(&mut connection)
    .await
    .expect("error creating test database");

    let connection_pool = NewsletterApp::postgres_pool(database_settings).await;

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
