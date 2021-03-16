use std::net::TcpListener;

#[actix_rt::test]
async fn health_check_works() {
    let health_check_endpoint = format!("{}/health_check", spawn_app());

    let client = reqwest::Client::new();
    let response = client
        .get(&health_check_endpoint)
        .send()
        .await
        .expect("Failed to execute request.");
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

/// When a `tokio` runtime is shut down all tasks spawned on it are dropped.
///
/// `actix_rt::test` spins up a new runtime at the beginning of each test case
/// and they shut down at the end of each test case.
fn spawn_app() -> String {
    let tcp_listener = TcpListener::bind("127.0.0.1:0").expect("tcp error binding to port");
    let port = tcp_listener.local_addr().unwrap().port();
    tokio::spawn(newsletter::run(tcp_listener).expect("server error binding to address"));
    format!("127.0.0.1:{}", port)
}
