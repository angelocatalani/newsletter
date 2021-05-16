use std::convert::TryInto;

use actix_web::{
    web,
    HttpResponse,
};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::NewSubscriber;
use crate::domain::{
    AppBaseUrl,
    MalformedInput,
};
use crate::email_client::{
    EmailClient,
    EmailClientError,
};
use crate::routes::RouteError;
use actix_web::web::Data;

#[derive(Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

#[tracing::instrument(
    name = "adding new subscriber",
    skip(form,postgres_connection,email_client),
    fields(
        email = %form.email,
        name = %form.name,
        app_base_url = %app_base_url.0
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    postgres_connection: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    app_base_url: web::Data<AppBaseUrl>,
) -> Result<HttpResponse, RouteError> {
    let new_subscriber = build_new_subscriber(form)?;

    insert_subscriber(&new_subscriber, postgres_connection).await?;
    send_confirmation_email(
        email_client,
        new_subscriber,
        &format!("{}/subscriptions/confirm", app_base_url.into_inner().0),
    )
    .await?;

    Ok(HttpResponse::Ok().finish())
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
        INSERT INTO subscriptions (id, email, name, status, subscribed_at)
        VALUES ($1, $2, $3, 'pending', $4)
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

#[tracing::instrument(
    name = "sending confirmation email",
    skip(email_client, new_subscriber)
)]
async fn send_confirmation_email(
    email_client: Data<EmailClient>,
    new_subscriber: NewSubscriber,
    sub_link: &str,
) -> Result<(), EmailClientError> {
    email_client
        .send_email(
            new_subscriber.email,
            "Newsletter Subscription",
            &format!(
                "Welcome to our newsletter!<br />Click <a href=\"{}\">here</a> to confirm your \
                 subscription.",
                sub_link
            ),
            &format!(
                "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
                sub_link
            ),
        )
        .await?;
    Ok(())
}
