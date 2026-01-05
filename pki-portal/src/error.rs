use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("not found")]
    NotFound,
    #[error("unprocessable request: {0}")]
    Invalid(String),
    #[error("not implemented")]
    NotImplemented,
}

#[derive(Serialize)]
struct ErrorBody {
    error: &'static str,
    message: String,
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ApiError::Db(_) => HttpResponse::InternalServerError().json(ErrorBody {
                error: "db_error",
                message: self.to_string(),
            }),
            ApiError::NotFound => HttpResponse::NotFound().json(ErrorBody {
                error: "not_found",
                message: self.to_string(),
            }),
            ApiError::Invalid(_) => HttpResponse::BadRequest().json(ErrorBody {
                error: "invalid",
                message: self.to_string(),
            }),
            ApiError::NotImplemented => HttpResponse::NotImplemented().json(ErrorBody {
                error: "not_implemented",
                message: self.to_string(),
            }),
        }
    }
}
