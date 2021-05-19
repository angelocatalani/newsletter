use std::convert::TryInto;

use actix_web::web::Data;
use actix_web::{
    web,
    HttpResponse,
};
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

#[derive(Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

#[tracing::instrument(
name = "adding new subscriber",
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
) -> Result<HttpResponse, RouteError> {
    let new_subscriber = build_new_subscriber(form)?;
    let subscription_token = generate_subscription_token();

    let mut transaction = postgres_connection.begin().await?;
    let subscriber_id = insert_subscriber(&new_subscriber, &mut transaction).await?;
    store_token(&subscription_token, &subscriber_id, &mut transaction).await?;
    transaction.commit().await?;

    send_confirmation_email(
        email_client,
        new_subscriber,
        &format!(
            "{}/subscriptions/confirm?subscription_token={}",
            app_base_url.into_inner().0,
            subscription_token
        ),
    )
    .await
    .map_err(|e| {
        tracing::error!("error sending email {:?}", e);
        e
    })?;
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
    skip(new_subscriber, postgres_transaction)
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
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(subscriber_id)
}

#[tracing::instrument(
    name = "storing a new token in the database",
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
