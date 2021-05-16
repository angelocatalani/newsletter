use crate::routes::RouteError;
use actix_web::{
    web,
    HttpResponse,
};
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct Parameter {
    subscription_token: String,
}

#[tracing::instrument(
    name = "confirming new subscriber",
    skip(postgres_connection, parameter)
)]
pub async fn confirm(
    parameter: web::Query<Parameter>,
    postgres_connection: web::Data<PgPool>,
) -> Result<HttpResponse, RouteError> {
    todo!("transactional check on subscription_token and update status");
    Ok(HttpResponse::Ok().finish())
}
