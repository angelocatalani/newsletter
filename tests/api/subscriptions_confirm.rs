use crate::api::helpers::{
    extract_confirmation_links,
    send_get_request,
    send_post_request,
    spawn_app,
};
use reqwest::Url;
use serde_json::Value;
use wiremock::matchers::{
    method,
    path,
};
use wiremock::{
    Mock,
    ResponseTemplate,
};

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

    let subscribe_end_point = format!("{}/subscriptions", test_app.address);
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com".to_string();
    send_post_request(&subscribe_end_point, body).await;

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

    let mut subscription_confirm_endpoint = Url::parse(subscription_confirm_endpoint).unwrap();
    subscription_confirm_endpoint
        .set_port(Some(test_app.port))
        .unwrap();
    println!("{}", subscription_confirm_endpoint.as_str());
    let response = send_get_request(subscription_confirm_endpoint.as_str()).await;
    assert_eq!(200, response.status().as_u16());

    todo!("check status is updated and row is deleted")
}
