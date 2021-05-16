use serde_json::Value;
use wiremock::matchers::{
    method,
    path,
};
use wiremock::{
    Mock,
    ResponseTemplate,
};

use crate::api::helpers::*;

#[actix_rt::test]
async fn subscribe_returns_a_200_for_valid_form() {
    let test_app = spawn_app().await;
    Mock::given(method("POST"))
        .and(path("/email"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    let subscribe_end_point = format!("{}/subscriptions", test_app.address);
    let body = String::from("name=le%20guin&email=ursula_le_guin%40gmail.com");
    let response = send_post_request(&subscribe_end_point, body).await;
    assert_eq!(200, response.status().as_u16());
}

#[actix_rt::test]
async fn subscribe_adds_new_pending_subscriber_to_postgres() {
    let test_app = spawn_app().await;
    Mock::given(method("POST"))
        .and(path("/email"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    let subscribe_end_point = format!("{}/subscriptions", test_app.address);

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com".to_string();
    send_post_request(&subscribe_end_point, body).await;

    let added_record = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&test_app.pool)
        .await
        .expect("Failed to fetch saved subscription");
    assert_eq!(added_record.name, "le guin");
    assert_eq!(added_record.email, "ursula_le_guin@gmail.com");
    assert_eq!(added_record.status, "pending");
}

#[actix_rt::test]
async fn subscribe_sends_confirmation_email_with_verification_link() {
    let test_app = spawn_app().await;
    let email_server = test_app.email_server;
    Mock::given(method("POST"))
        .and(path("/email"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&email_server)
        .await;
    let subscribe_end_point = format!("{}/subscriptions", test_app.address);
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com".to_string();
    send_post_request(&subscribe_end_point, body).await;
    let request = &email_server.received_requests().await.unwrap()[0];
    let email_body: Value = serde_json::from_slice(&request.body).unwrap();
    let html_body = email_body["HtmlBody"].as_str().unwrap();
    assert_eq!(extract_confirmation_links(html_body).len(), 1);
    let text_body = email_body["TextBody"].as_str().unwrap();
    assert_eq!(extract_confirmation_links(text_body).len(), 1);

    let html_link = extract_confirmation_links(html_body)
        .first()
        .unwrap()
        .as_str();

    let text_link = extract_confirmation_links(text_body)
        .first()
        .unwrap()
        .as_str();

    assert_eq!(html_link, text_link);
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
