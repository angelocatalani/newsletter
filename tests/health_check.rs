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

struct TestApp {
    address: String,
    pool: PgPool,
}

#[actix_rt::test]
async fn postgres_connection_works() {
    let database_options = &load_configuration()
        .unwrap()
        .database
        .pgserver_connection_options();
    PgConnection::connect_with(database_options)
        .await
        .expect("error connecting to postgres");
    let database_options = &load_configuration()
        .unwrap()
        .database
        .database_connection_options();
    PgConnection::connect_with(database_options)
        .await
        .expect("error connecting to postgres");
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
async fn subscribe_returns_a_400_with_missing_field() {
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

#[actix_rt::test]
async fn subscribe_returns_a_400_with_invalid_fields() {
    let subscribe_end_point = format!("{}/subscriptions", spawn_app().await.address);
    let invalid_data = vec![
        (
            String::from("name=&email=ursula_le_guin%40gmail.com"),
            String::from("empty name"),
        ),
        (
            String::from("name=ursula&email="),
            String::from("empty email"),
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

async fn send_post_request(endpoint: &str, body: String) -> Response {
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
