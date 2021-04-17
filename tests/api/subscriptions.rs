use crate::api::helpers::*;

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
