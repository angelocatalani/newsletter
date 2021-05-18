use actix_web::http::StatusCode;
use actix_web::{
    HttpResponse,
    ResponseError,
};
use custom_error::custom_error;

use crate::domain::MalformedInput;
use crate::email_client::EmailClientError;

custom_error! {
///! Error inside route handler
pub RouteError
    InvalidFormData{source:MalformedInput} = "Invalid body data: {source}",
    DatabaseError{source: sqlx::Error} = "{source}",
    EmailError{source: EmailClientError} = "{source}",
    MissingTokenError{subscription_token: String} = "The subscription_token:\
    {subscription_token} does not exists",
}

impl ResponseError for RouteError {
    fn status_code(&self) -> StatusCode {
        match self {
            RouteError::InvalidFormData { .. } => StatusCode::BAD_REQUEST,
            RouteError::DatabaseError { .. } | RouteError::EmailError { .. } => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            RouteError::MissingTokenError { .. } => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            RouteError::InvalidFormData { .. } => HttpResponse::BadRequest().finish(),
            RouteError::DatabaseError { .. } | RouteError::EmailError { .. } => {
                HttpResponse::InternalServerError().finish()
            }
            RouteError::MissingTokenError { .. } => HttpResponse::NotFound().finish(),
        }
    }
}
