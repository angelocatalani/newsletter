use reqwest::{
    Response,
    Url,
};
use serde_json::Value;
use sqlx::{
    Connection,
    PgConnection,
    PgPool,
};
use uuid::Uuid;
use wiremock::MockServer;

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
    pub email_server: MockServer,
    pub base_url: String,
    pub port: u16,
}

/// When a `tokio` runtime is shut down all tasks spawned on it are dropped.
///
/// `actix_rt::test` spins up a new runtime at the beginning of each test case
/// and they shut down at the end of each test case.
pub async fn spawn_app() -> TestApp {
    lazy_static::initialize(&TRACING);
    let email_server = MockServer::start().await;

    let configuration = {
        let mut c = load_configuration().unwrap();
        c.database.name = Uuid::new_v4().to_string();
        c.application.port = 0;
        c.email_client.base_url = email_server.uri();
        c
    };

    let base_url = configuration.application.base_url.clone();
    let postgres_pool = setup_test_database(configuration.database.clone()).await;

    let app = NewsletterApp::from(configuration)
        .await
        .expect("error building app");

    tokio::spawn(app.server.expect("error building server"));

    TestApp {
        // the request is done with the protocol:ip:port
        address: format!("http://127.0.0.1:{}", app.port),
        pool: postgres_pool,
        email_server,
        base_url,
        port: app.port,
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

pub async fn send_authenticated_json_post_request(
    endpoint: &str,
    body: &Value,
    username: &str,
    password: &str,
) -> Response {
    reqwest::Client::new()
        .post(endpoint)
        .json(&body)
        .basic_auth(username, Some(password))
        .send()
        .await
        .expect("Fail to execute post request")
}

pub async fn send_json_post_request(endpoint: &str, body: &Value) -> Response {
    reqwest::Client::new()
        .post(endpoint)
        .json(&body)
        .send()
        .await
        .expect("Fail to execute post request")
}

pub async fn send_get_request(endpoint: &str) -> Response {
    reqwest::Client::new()
        .get(endpoint)
        .send()
        .await
        .expect("Fail to execute get request")
}

async fn setup_test_database(database_settings: DatabaseSettings) -> PgPool {
    let mut connection =
        PgConnection::connect_with(&database_settings.pgserver_connection_options())
            .await
            .expect("error connecting to postgres");

    sqlx::query(&format!("CREATE DATABASE \"{}\"", database_settings.name))
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

pub fn extract_confirmation_links(body: &str) -> Vec<linkify::Link> {
    linkify::LinkFinder::new()
        .links(&body)
        .filter(|link| *link.kind() == linkify::LinkKind::Url)
        .collect::<Vec<_>>()
}

pub async fn get_subscription_confirm_url(test_app: &TestApp) -> Url {
    let request_body = &test_app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .first()
        .unwrap()
        .body
        .to_owned();
    let html_body = serde_json::from_slice::<Value>(request_body).unwrap()["Messages"][0]
        ["HTMLPart"]
        .as_str()
        .unwrap()
        .to_owned();
    let subscription_confirm_endpoint = extract_confirmation_links(&html_body)
        .first()
        .unwrap()
        .as_str();

    let mut subscription_confirm_url = Url::parse(subscription_confirm_endpoint).unwrap();
    subscription_confirm_url
        .set_port(Some(test_app.port))
        .unwrap();
    subscription_confirm_url
}
