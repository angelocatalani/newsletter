use std::error::Error;

use actix_web::http::{
    header,
    StatusCode,
};
use actix_web::{
    HttpResponse,
    ResponseError,
};

#[derive(thiserror::Error)]
///! Error handling a newsletter route
pub enum NewsletterError {
    // this is the Display trait implementation
    #[error("Invalid data: {0}")]
    // String cannot be the source of ValidationError because it does not implement the Error trait
    ValidationError(String),
    // Here we define a new custom error to disambiguate from the `ValidationError` that has a
    // soruce as String but maps to a different error code
    #[error("Confirmation failed for missing token: {0}")]
    MissingTokenError(String),
    #[error("Authentication Error: {0}")]
    AuthError(#[source] anyhow::Error),
    #[error("Unexpected internal error: {0}")]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for NewsletterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}\n", self))?;
        let mut error = self.source();
        while let Some(content) = error {
            f.write_fmt(format_args!("Caused by:\n\t{}", content))?;
            error = content.source();
        }
        Ok(())
    }
}

impl ResponseError for NewsletterError {
    fn status_code(&self) -> StatusCode {
        match self {
            NewsletterError::ValidationError(_) => StatusCode::BAD_REQUEST,
            NewsletterError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            NewsletterError::MissingTokenError(_) => StatusCode::NOT_FOUND,
            NewsletterError::AuthError(_) => StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            NewsletterError::ValidationError(e) => HttpResponse::BadRequest().json(e),
            NewsletterError::UnexpectedError(_) => HttpResponse::InternalServerError().finish(),
            NewsletterError::MissingTokenError(missing_token) => {
                HttpResponse::NotFound().json(&format!("Token: {} not found", missing_token))
            }
            NewsletterError::AuthError(_) => HttpResponse::Unauthorized()
                .append_header((header::WWW_AUTHENTICATE, "Basic realm=\"publish\""))
                .finish(),
        }
    }
}
