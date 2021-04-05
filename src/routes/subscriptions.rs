use actix_web::{
    web,
    HttpResponse,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

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
) -> Result<HttpResponse, HttpResponse> {
    insert_subscriber(form, postgres_connection)
        .await
        .map_or_else(
            |_| Err(HttpResponse::InternalServerError().finish()),
            |_| Ok(HttpResponse::Ok().finish()),
        )
}

#[tracing::instrument(
    name = "inserting new subscriber details in the database",
    skip(form, postgres_connection)
)]
async fn insert_subscriber(
    form: web::Form<FormData>,
    postgres_connection: web::Data<PgPool>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at) VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
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
