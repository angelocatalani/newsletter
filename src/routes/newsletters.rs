use std::convert::TryInto;

use actix_web::web::Data;
use actix_web::{
    web,
    HttpResponse,
};
use anyhow::Context;
use serde::Deserialize;
use sqlx::PgPool;

use crate::domain::SubscriberEmail;
use crate::email_client::EmailClient;
use crate::routes::NewsletterError;

#[derive(Deserialize)]
pub struct Article {
    title: String,
    content: ArticleContent,
}

#[derive(Deserialize)]
struct ArticleContent {
    text: String,
    html: String,
}

#[tracing::instrument(
name = "Sending newsletter to confirmed users",
skip(article, postgres_connection, email_client),
fields(
title = % article.title,
)
)]
pub async fn newsletters(
    article: web::Json<Article>,
    postgres_connection: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> Result<HttpResponse, NewsletterError> {
    let confirmed_subscribers = get_confirmed_subscribers(postgres_connection)
        .await
        .context("Failed to retrieve confirmed subscribers from db")?;
    for subscriber in confirmed_subscribers {
        match subscriber.email.try_into() {
            Ok(subscriber_email) => send_article(&email_client, subscriber_email, &article)
                .await
                .context("Failed to send new article")?,
            Err(e) => {
                tracing::warn!("Invalid email retrieved from db: {}", e)
            }
        }
    }

    Ok(HttpResponse::Ok().finish())
}

struct ConfirmedSubscriber {
    email: String,
}

#[tracing::instrument(name = "Retrieving confirmed subscribers", skip(postgres_connection))]
async fn get_confirmed_subscribers(
    postgres_connection: web::Data<PgPool>,
) -> Result<Vec<ConfirmedSubscriber>, sqlx::Error> {
    let rows = sqlx::query_as!(
        ConfirmedSubscriber,
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
    )
    .fetch_all(postgres_connection.as_ref())
    .await?;
    Ok(rows)
}

#[tracing::instrument(
name = "Sending article to confirmed user",
skip(email_client, article),
fields(
title = % article.title,
)
)]
async fn send_article(
    email_client: &Data<EmailClient>,
    subscriber: SubscriberEmail,
    article: &Article,
) -> Result<(), anyhow::Error> {
    email_client
        .send_email(
            subscriber,
            &article.title,
            &article.content.html,
            &article.content.text,
        )
        .await?;
    Ok(())
}
