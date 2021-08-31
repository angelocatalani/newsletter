use reqwest::Response;
use uuid::Uuid;
use wiremock::matchers::{
    method,
    path,
};
use wiremock::{
    Mock,
    ResponseTemplate,
};

use crate::api::helpers::{
    get_subscription_confirm_url,
    send_get_request,
    send_post_request,
    spawn_app,
    TestApp,
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
        .and(path("/send"))
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
