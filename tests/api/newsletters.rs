use wiremock::matchers::{
    any,
    method,
    path,
};
use wiremock::{
    Mock,
    ResponseTemplate,
};

use crate::api::helpers;
use crate::api::helpers::{
    send_post_request,
    spawn_app,
    TestApp,
};

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
    let response = send_post_request(&newsletters_endpoint, body.to_string()).await;

    assert_eq!(200, response.status());
}

pub async fn create_pending_user(test_app: &TestApp) {
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
}
