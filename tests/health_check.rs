use std::net::TcpListener;

use reqwest::Response;

#[actix_rt::test]
async fn health_check_works() {
    let health_check_endpoint = format!("{}/health_check", spawn_app());
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
    let subscribe_end_point = format!("{}/subscriptions", spawn_app());
    let body = String::from("name=le%20guin&email=ursula_le_guin%40gmail.com");
    let response = send_post_request(&subscribe_end_point, body).await;
    assert_eq!(200, response.status().as_u16());
}

#[actix_rt::test]
async fn subscribe_returns_a_400_with_invalid_form() {
    let subscribe_end_point = format!("{}/subscriptions", spawn_app());
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
fn spawn_app() -> String {
    // the tcp listens on the ip:port. It does not matter the protocol
    let tcp_listener = TcpListener::bind("127.0.0.1:0").expect("tcp error binding to port");
    let port = tcp_listener.local_addr().unwrap().port();
    tokio::spawn(newsletter::run(tcp_listener).expect("server error binding to address"));

    // the request is done with the protocol:ip:port
    format!("http://127.0.0.1:{}", port)
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
