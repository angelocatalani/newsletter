use crate::domain::errors::MalformedInput;

use actix_web::http::StatusCode;
use actix_web::{
    HttpResponse,
    ResponseError,
};
use custom_error::custom_error;

custom_error! {
///! Error inside route handler
pub RouteError
    InvalidFormData{source:MalformedInput} = "Invalid body data: {source}",
    DatabaseError{source: sqlx::Error} = "{source}",
}

impl ResponseError for RouteError {
    fn status_code(&self) -> StatusCode {
        match self {
            RouteError::InvalidFormData { .. } => StatusCode::BAD_REQUEST,
            RouteError::DatabaseError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            RouteError::InvalidFormData { .. } => HttpResponse::BadRequest().finish(),
            RouteError::DatabaseError { .. } => HttpResponse::InternalServerError().finish(),
        }
    }
}
