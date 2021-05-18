use crate::api::helpers::{
    extract_confirmation_links,
    send_get_request,
    send_post_request,
    spawn_app,
    TestApp,
};
use reqwest::{
    Response,
    Url,
};
use serde_json::Value;
use uuid::Uuid;
use wiremock::matchers::{
    method,
    path,
};
use wiremock::{
    Mock,
    ResponseTemplate,
};

struct ConfirmRequestDetails {
    response: Response,
    subscription_token: String,
    pending_subscriber_id: Uuid,
}

#[actix_rt::test]
async fn subscriptions_confirm_returns_a_404_with_invalid_token() {
    let test_app = spawn_app().await;

    let subscribe_end_point = format!(
        "{}/subscriptions/confirm?subscription_token=invalid-token",
        test_app.address
    );
    let response = send_get_request(&subscribe_end_point).await;
    assert_eq!(404, response.status().as_u16());
}

#[actix_rt::test]
async fn subscriptions_confirm_works() {
    let test_app = spawn_app().await;

    Mock::given(method("POST"))
        .and(path("/email"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    let confirm_request_details = subscribe_and_confirm(&test_app).await;

    assert_eq!(200, confirm_request_details.response.status().as_u16());

    let confirmed_subscriber = sqlx::query!(
        r#"SELECT status
        FROM subscriptions
        WHERE id=$1
        "#,
        confirm_request_details.pending_subscriber_id
    )
    .fetch_one(&test_app.pool)
    .await
    .expect("Failed to fetch saved subscriptions");

    assert_eq!(confirmed_subscriber.status, "confirmed");

    let pending_subscriber = sqlx::query!(
        r#"SELECT count(*)
        FROM subscription_tokens
        WHERE subscriber_id=$1 OR subscription_token=$2
        "#,
        confirm_request_details.pending_subscriber_id,
        confirm_request_details.subscription_token
    )
    .fetch_one(&test_app.pool)
    .await
    .expect("Failed to fetch saved subscription_tokens");

    assert_eq!(pending_subscriber.count, Some(0));
}

async fn subscribe_and_confirm(test_app: &TestApp) -> ConfirmRequestDetails {
    let subscribe_end_point = format!("{}/subscriptions", test_app.address);
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com".to_string();
    send_post_request(&subscribe_end_point, body).await;
    let subscription_confirm_url = get_subscription_confirm_url(&test_app).await;
    let subscription_token = subscription_confirm_url
        .query_pairs()
        .next()
        .unwrap()
        .1
        .to_string();
    let pending_subscriber_id = get_pending_subscriber_id(&test_app, &subscription_token).await;
    let response = send_get_request(subscription_confirm_url.as_str()).await;
    ConfirmRequestDetails {
        response,
        subscription_token,
        pending_subscriber_id,
    }
}

async fn get_pending_subscriber_id(test_app: &TestApp, subscription_token: &str) -> Uuid {
    let pending_subscriber = sqlx::query!(
        r#"SELECT id
        FROM subscriptions JOIN subscription_tokens
        ON subscriptions.id = subscription_tokens.subscriber_id
        WHERE subscription_tokens.subscription_token=$1
        "#,
        subscription_token
    )
    .fetch_one(&test_app.pool)
    .await
    .expect("Failed to fetch saved subscription_tokens");
    pending_subscriber.id
}

async fn get_subscription_confirm_url(test_app: &TestApp) -> Url {
    let request_body = &test_app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .first()
        .unwrap()
        .body
        .to_owned();
    let html_body = serde_json::from_slice::<Value>(request_body).unwrap()["HtmlBody"]
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
