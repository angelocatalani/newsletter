use actix_web::{
    web,
    HttpResponse,
};
use anyhow::Context;
use serde::Deserialize;
use sqlx::{
    PgPool,
    Postgres,
    Transaction,
};
use uuid::Uuid;

use crate::routes::NewsletterError;

#[derive(Debug, Deserialize)]
pub struct Parameter {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirming new subscriber", skip(postgres_connection))]
pub async fn confirm(
    postgres_connection: web::Data<PgPool>,
    parameter: web::Query<Parameter>,
) -> Result<HttpResponse, NewsletterError> {
    let mut transaction = postgres_connection
        .begin()
        .await
        .context("Failed to acquire database pool to confirm subscription")?;
    let subscriber_id =
        get_subscriber_id_and_remove_token(&parameter.subscription_token, &mut transaction)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => {
                    NewsletterError::MissingTokenError(parameter.subscription_token.clone())
                }
                other => NewsletterError::UnexpectedError(other.into()),
            })?;

    confirm_subscription(&subscriber_id, &mut transaction)
        .await
        .context("Failed to confirm subscription")?;
    transaction
        .commit()
        .await
        .context("Failed to commit transaction to confirm subscriptions")?;

    Ok(HttpResponse::Ok().finish())
}

async fn get_subscriber_id_and_remove_token(
    subscription_token: &str,
    postgres_transaction: &mut Transaction<'_, Postgres>,
) -> Result<Uuid, sqlx::Error> {
    let deleted_record = sqlx::query!(
        r#"
        DELETE FROM subscription_tokens WHERE subscription_token=$1 RETURNING subscriber_id
        "#,
        subscription_token
    )
    .fetch_one(postgres_transaction)
    .await?;
    Ok(deleted_record.subscriber_id)
}

async fn confirm_subscription(
    subscriber_id: &Uuid,
    postgres_transaction: &mut Transaction<'_, Postgres>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE subscriptions SET status = 'confirmed' WHERE id=$1
        "#,
        subscriber_id
    )
    .execute(postgres_transaction)
    .await?;
    Ok(())
}
