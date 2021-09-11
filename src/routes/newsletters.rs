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
use actix_web::http::HeaderMap;
use sha3::Digest;
use uuid::Uuid;

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
username=tracing::field::Empty,
uuid=tracing::field::Empty,
)
)]
pub async fn newsletters(
    article: web::Json<Article>,
    postgres_connection: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    request: web::HttpRequest,
) -> Result<HttpResponse, NewsletterError> {
    let credentials = get_credentials(request.headers()).map_err(NewsletterError::AuthError)?;
    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));
    let authenticated_uuid = validate_credentials(credentials, postgres_connection.as_ref())
        .await
        .map_err(NewsletterError::AuthError)?;

    tracing::Span::current().record("uuid", &tracing::field::display(authenticated_uuid));
    let confirmed_subscribers = get_confirmed_subscribers(postgres_connection.as_ref())
        .await
        .context("Failed to retrieve confirmed subscribers from db")?;

    for subscriber in confirmed_subscribers {
        match subscriber.email.try_into() {
            Ok(subscriber_email) => {
                send_article(&email_client, subscriber_email, &article)
                    .await
                    .map_err(|e| tracing::warn!("Error sending new article: {}", e))
                    .ok();
            }
            Err(e) => {
                tracing::warn!("Invalid email retrieved from db: {}", e)
            }
        }
    }

    Ok(HttpResponse::Ok().finish())
}

struct Credentials {
    username: String,
    password: String,
}

fn get_credentials(headers: &HeaderMap) -> anyhow::Result<Credentials> {
    let authorization_header: &str = headers
        .get("Authorization")
        .context("Missing `Authorization` header")?
        .to_str()
        .context("Invalid `Authorization` content")?;
    let encoded_credentials = authorization_header
        .strip_prefix("Basic ")
        .context("Authorization scheme is not Basic")?;
    let decoded_credentials_bytes =
        base64::decode(encoded_credentials).context("Credentials cannot be base64 decoded")?;
    let decoded_credentials = String::from_utf8(decoded_credentials_bytes)
        .context("Invalid credentials: not UTF8 chars")?;
    let mut credentials = decoded_credentials.split(":");
    let username = credentials
        .next()
        .context("Invalid credentials: missing username")?;
    let password = credentials
        .next()
        .context("Invalid credentials: missing password")?;
    Ok(Credentials {
        username: username.to_string(),
        password: password.to_string(),
    })
}

struct AuthenticatedUser {
    id: Uuid,
}

async fn validate_credentials(
    credentials: Credentials,
    postgres_connection: &PgPool,
) -> anyhow::Result<Uuid> {
    let password_hash = format!(
        "{:x}",
        sha3::Sha3_256::digest(credentials.password.as_ref())
    );
    let user = sqlx::query_as!(
        AuthenticatedUser,
        r#"
        SELECT id
        FROM users
        WHERE username=$1 AND password_hash=$2
        "#,
        credentials.username,
        password_hash
    )
    .fetch_optional(postgres_connection)
    .await
    .context("Error fetching user from dm")?
    .context("User not found")?;
    Ok(user.id)
}

struct ConfirmedSubscriber {
    email: String,
}

#[tracing::instrument(name = "Retrieving confirmed subscribers", skip(postgres_connection))]
async fn get_confirmed_subscribers(
    postgres_connection: &PgPool,
) -> Result<Vec<ConfirmedSubscriber>, sqlx::Error> {
    let rows = sqlx::query_as!(
        ConfirmedSubscriber,
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
    )
    .fetch_all(postgres_connection)
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
