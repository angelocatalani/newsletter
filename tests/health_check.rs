use reqwest::Response;
use sqlx::postgres::PgPoolOptions;
use sqlx::{
    Connection,
    PgConnection,
    PgPool,
};
use uuid::Uuid;

use newsletter::configuration::load_configuration;
use newsletter::telemetry::setup_tracing;
use std::net::TcpListener;

// ensure the `tracing` is instantiated only once
lazy_static::lazy_static! {
 static ref TRACING: () = setup_tracing("test".into(),"debug".into());
}

struct TestApp {
    address: String,
    pool: PgPool,
}

#[actix_rt::test]
async fn postgres_connection_works() {
    postgres_connection(&load_configuration().database.pgserver_connection_url()).await;
    postgres_connection(&load_configuration().database.database_connection_url()).await;
}

#[actix_rt::test]
async fn health_check_works() {
    let health_check_endpoint = format!("{}/health_check", spawn_app().await.address);
    let client = reqwest::Client::new();
    let response = client
        .get(&health_check_endpoint)
        .send()
        .await
        .expect("Fail to execute request.");
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[actix_rt::test]
async fn subscribe_returns_a_200_for_valid_form() {
    let subscribe_end_point = format!("{}/subscriptions", spawn_app().await.address);
    let body = String::from("name=le%20guin&email=ursula_le_guin%40gmail.com");
    let response = send_post_request(&subscribe_end_point, body).await;
    assert_eq!(200, response.status().as_u16());
}

#[actix_rt::test]
async fn subscribe_adds_new_record_to_postgres() {
    let test_app = spawn_app().await;

    let subscribe_end_point = format!("{}/subscriptions", test_app.address);

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com".to_string();
    send_post_request(&subscribe_end_point, body).await;

    let added_record = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&test_app.pool)
        .await
        .expect("Failed to fetch saved subscription");
    assert_eq!(added_record.name, "le guin");
    assert_eq!(added_record.email, "ursula_le_guin@gmail.com");
}

#[actix_rt::test]
async fn subscribe_returns_a_400_with_invalid_form() {
    let subscribe_end_point = format!("{}/subscriptions", spawn_app().await.address);
    let invalid_data = vec![
        (String::from(""), String::from("empty message")),
        (
            String::from("email=ursula_le_guin%40gmail.com"),
            String::from("missing name"),
        ),
        (
            String::from("name=le%20guin"),
            String::from("missing email"),
        ),
    ];
    for (body, error_message) in invalid_data {
        let response = send_post_request(&subscribe_end_point, body).await;
        assert_eq!(
            400,
            response.status().as_u16(),
            "Subscription with invalid body with {} did not fail",
            error_message
        );
    }
}

/// When a `tokio` runtime is shut down all tasks spawned on it are dropped.
///
/// `actix_rt::test` spins up a new runtime at the beginning of each test case
/// and they shut down at the end of each test case.
async fn spawn_app() -> TestApp {
    lazy_static::initialize(&TRACING);
    let configuration = load_configuration();

    // the tcp listens on the ip:port. It does not matter the protocol
    let tcp_listener = TcpListener::bind(&format!("{}:0", configuration.application.host))
        .expect("tcp error binding to port");
    let port = tcp_listener.local_addr().unwrap().port();

    let postgres_pool = setup_test_database(
        &configuration.database.pgserver_connection_url(),
        configuration.database.max_db_connections,
    )
    .await;
    tokio::spawn(
        newsletter::startup::run(
            tcp_listener,
            // cloning a postgres_pool does not create a new pool but it is always the same
            postgres_pool.clone(),
            configuration.application.max_pending_connections,
        )
        .expect("server error binding to address"),
    );

    TestApp {
        // the request is done with the protocol:ip:port
        address: format!("http://127.0.0.1:{}", port),
        pool: postgres_pool,
    }
}

async fn send_post_request(endpoint: &str, body: String) -> Response {
    reqwest::Client::new()
        .post(endpoint)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Fail to execute post request")
}

async fn setup_test_database(pgserver_connection_url: &str, max_db_connections: u32) -> PgPool {
    let mut connection = postgres_connection(pgserver_connection_url).await;

    let test_database_name = Uuid::new_v4().to_string();
    sqlx::query(&format!("CREATE DATABASE \"{}\"", test_database_name))
        .execute(&mut connection)
        .await
        .expect("error creating test database");

    let connection_pool = postgres_pool(
        &format!("{}{}", pgserver_connection_url, test_database_name),
        max_db_connections,
    )
    .await;

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}

async fn postgres_connection(database_url: &str) -> PgConnection {
    PgConnection::connect(database_url)
        .await
        .expect("error connecting to postgres")
}

async fn postgres_pool(database_url: &str, max_connections: u32) -> PgPool {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await
        .expect("error creating postgres connection pool")
}
