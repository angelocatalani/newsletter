use reqwest::Url;
use wiremock::matchers::{
    any,
    method,
    path,
};
use wiremock::{
    Mock,
    ResponseTemplate,
};

use crate::api::helpers::{
    get_subscription_confirm_url,
    send_authenticated_json_post_request,
    send_get_request,
    send_json_post_request,
    send_post_request,
    spawn_app,
    TestApp,
};
use argon2::password_hash::SaltString;
use argon2::{
    Argon2,
    PasswordHasher,
};
use sqlx::PgPool;
use uuid::Uuid;

#[actix_rt::test]
async fn emails_are_not_sent_to_pending_users() {
    let test_app = spawn_app().await;
    create_pending_user(&test_app).await;
    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&test_app.email_server)
        .await;
    let newsletters_endpoint = format!("{}/newsletters", test_app.address);
    let body = serde_json::json!({
        "title": "any_title",
        "content": {
            "text": "any_text",
            "html": "any_html",
        }
    });
    create_authenticated_user("any_user", "any_password", &test_app.pool).await;
    let response = send_authenticated_json_post_request(
        &newsletters_endpoint,
        &body,
        "any_user",
        "any_password",
    )
    .await;
    assert_eq!(200, response.status());
}

#[actix_rt::test]
async fn emails_are_sent_to_confirmed_users() {
    let test_app = spawn_app().await;
    create_confirmed_subscriber(&test_app).await;
    Mock::given(method("POST"))
        .and(path("/send"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    let newsletters_endpoint = format!("{}/newsletters", test_app.address);
    let body = serde_json::json!({
        "title": "any_title",
        "content": {
            "text": "any_text",
            "html": "any_html",
        }
    });
    create_authenticated_user("any_user", "any_password", &test_app.pool).await;
    let response = send_authenticated_json_post_request(
        &newsletters_endpoint,
        &body,
        "any_user",
        "any_password",
    )
    .await;
    assert_eq!(200, response.status());
}

#[actix_rt::test]
async fn requests_missing_authorization_are_rejected() {
    let newsletters_endpoint = format!("{}/newsletters", spawn_app().await.address);
    let body = serde_json::json!({
        "title": "any_title",
        "content": {
            "text": "any_text",
            "html": "any_html",
        }
    });
    let response = send_json_post_request(&newsletters_endpoint, &body).await;
    assert_eq!(401, response.status());
    assert_eq!(
        "Basic realm=\"publish\"",
        response.headers().get("WWW-Authenticate").unwrap()
    );
}

pub async fn create_pending_user(test_app: &TestApp) -> Url {
    let _mock_guard = Mock::given(method("POST"))
        .and(path("/send"))
        .respond_with(ResponseTemplate::new(200))
        .named("create_pending_user")
        .expect(1)
        .mount_as_scoped(&test_app.email_server)
        .await;
    let subscriptions_endpoint = format!("{}/subscriptions", test_app.address);
    let subscriptions_body = String::from("name=le%20guin&email=ursula_le_guin%40gmail.com");
    send_post_request(&subscriptions_endpoint, subscriptions_body)
        .await
        .error_for_status()
        .unwrap();
    get_subscription_confirm_url(&test_app).await
}

async fn create_confirmed_subscriber(test_app: &TestApp) {
    let subscription_confirm_url = create_pending_user(test_app).await;
    send_get_request(subscription_confirm_url.as_str())
        .await
        .error_for_status()
        .unwrap();
}

async fn create_authenticated_user(username: &str, password: &str, pool: &PgPool) {
    let salt = SaltString::generate(&mut rand::thread_rng());
    let password_hash = Argon2::default()
        .hash_password(password.as_ref(), &salt)
        .unwrap()
        .to_string();
    sqlx::query!(
        r#"
        INSERT INTO users (id, username, phc_password)
        VALUES ($1, $2, $3)
        "#,
        Uuid::new_v4(),
        username,
        password_hash,
    )
    .execute(pool)
    .await
    .unwrap();
}
