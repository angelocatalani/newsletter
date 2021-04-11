use crate::domain::errors::MalformedInput;
use crate::domain::new_subscriber::NewSubscriber;
use crate::routes::errors::RouteError;
use actix_web::http::StatusCode;
use actix_web::{
    web,
    HttpResponse,
    ResponseError,
};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use std::convert::TryInto;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

#[tracing::instrument(
    name = "adding new subscriber",
    skip(form,postgres_connection),
    fields(
        email = %form.email,
        name = %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    postgres_connection: web::Data<PgPool>,
) -> Result<HttpResponse, RouteError> {
    let new_subscriber = build_new_subscriber(form)?;

    insert_subscriber(&new_subscriber, postgres_connection)
        .await
        .map(|_| Ok(HttpResponse::Ok().finish()))?
}
#[tracing::instrument(name = "validating form data", skip(form))]
fn build_new_subscriber(form: web::Form<FormData>) -> Result<NewSubscriber, MalformedInput> {
    Ok(NewSubscriber {
        name: form.0.name.try_into().map_err(|e| {
            tracing::error!("{:?}", e);
            e
        })?,
        email: form.0.email.try_into().map_err(|e| {
            tracing::error!("{:?}", e);
            e
        })?,
    })
}

#[tracing::instrument(
    name = "inserting new subscriber details in the database",
    skip(new_subscriber, postgres_connection)
)]
async fn insert_subscriber(
    new_subscriber: &NewSubscriber,
    postgres_connection: web::Data<PgPool>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at) VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(postgres_connection.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
