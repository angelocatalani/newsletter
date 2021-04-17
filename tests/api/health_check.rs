use sqlx::{
    Connection,
    PgConnection,
};

use newsletter::app::load_configuration;

use crate::api::helpers::*;

#[actix_rt::test]
async fn postgres_connection_works() {
    let database_options = &load_configuration()
        .unwrap()
        .database
        .pgserver_connection_options();
    PgConnection::connect_with(database_options)
        .await
        .expect("error connecting to postgres");
    let database_options = &load_configuration()
        .unwrap()
        .database
        .database_connection_options();
    PgConnection::connect_with(database_options)
        .await
        .expect("error connecting to postgres");
}

#[actix_rt::test]
async fn health_check_works() {
    let health_check_endpoint = format!("{}/health_check", spawn_app().await.address);
    let client = reqwest::Client::new();
    let response = client
        .get(&health_check_endpoint)
        .send()
        .await
        .expect("Fail to execute request.");
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
