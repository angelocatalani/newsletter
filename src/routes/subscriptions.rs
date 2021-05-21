use std::convert::TryInto;

use actix_web::web::Data;
use actix_web::{
    web,
    HttpResponse,
};
use anyhow::Context;
use chrono::Utc;
use rand::distributions::Alphanumeric;
use rand::{
    thread_rng,
    Rng,
};
use serde::Deserialize;
use sqlx::{
    PgPool,
    Postgres,
    Transaction,
};
use uuid::Uuid;

use crate::domain::AppBaseUrl;
use crate::domain::NewSubscriber;
use crate::email_client::EmailClient;
use crate::routes::NewsletterError;

#[derive(Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

#[tracing::instrument(
name = "Adding new subscriber",
skip(form, postgres_connection, email_client),
fields(
email = % form.email,
name = % form.name,
app_base_url = % app_base_url.0
)
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    postgres_connection: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    app_base_url: web::Data<AppBaseUrl>,
) -> Result<HttpResponse, NewsletterError> {
    // this error must be explicitly converted to a ValidationError because
    // for String do not implement the Error trait
    let new_subscriber = build_new_subscriber(form).map_err(NewsletterError::ValidationError)?;
    let subscription_token = generate_subscription_token();

    let mut transaction = postgres_connection
        .begin()
        .await
        .context("Failed to start SQL transaction to store a new subscriber")?;
    let subscriber_id = insert_subscriber(&new_subscriber, &mut transaction)
        .await
        .context("Failed to insert new subscriber")?;
    store_token(&subscription_token, &subscriber_id, &mut transaction)
        .await
        .context("Failed to store token")?;
    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a new subscriber")?;

    send_confirmation_email(
        email_client,
        new_subscriber,
        &format!(
            "{}/subscriptions/confirm?subscription_token={}",
            app_base_url.into_inner().0,
            subscription_token
        ),
    )
    .await?;
    // we do not need to log here the error because it is propagated up to the
    // tracing_actix_web::TracingLogger,
    //.map_err(|e| { tracing::error!("error sending email {:?}", e);e })?;
    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(name = "Validating form data", skip(form))]
fn build_new_subscriber(form: web::Form<FormData>) -> Result<NewSubscriber, String> {
    Ok(NewSubscriber {
        name: form.0.name.try_into()?,
        email: form.0.email.try_into()?,
    })
}

#[tracing::instrument(
    name = "Inserting new subscriber details in the database",
    skip(postgres_transaction)
)]
async fn insert_subscriber(
    new_subscriber: &NewSubscriber,
    postgres_transaction: &mut Transaction<'_, Postgres>,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, status, subscribed_at)
        VALUES ($1, $2, $3, 'pending', $4)
        "#,
        subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(postgres_transaction)
    .await?;
    Ok(subscriber_id)
}

#[tracing::instrument(
    name = "Storing a new token in the database",
    skip(postgres_transaction)
)]
async fn store_token(
    token: &str,
    subscriber_id: &Uuid,
    postgres_transaction: &mut Transaction<'_, Postgres>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscription_tokens (subscription_token, subscriber_id)
        VALUES ($1, $2)
        "#,
        token,
        subscriber_id,
    )
    .execute(postgres_transaction)
    .await?;
    Ok(())
}

#[tracing::instrument(
    name = "Sending confirmation email",
    skip(email_client, new_subscriber)
)]
async fn send_confirmation_email(
    email_client: Data<EmailClient>,
    new_subscriber: NewSubscriber,
    sub_link: &str,
) -> Result<(), anyhow::Error> {
    email_client
        .send_email(
            new_subscriber.email,
            "Newsletter Subscription",
            &format!(
                "Welcome to our newsletter!<br />Visit {} to confirm your subscription <br />",
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

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}
