use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
pub enum ApiErrorKind {
    #[error("Unauthorized access")]
    Unauthorized,

    #[error("Invalid request")]
    BadRequest,

    #[error("Not Found")]
    NotFound,

    #[error("Internal Server Error")]
    InternalServerError,

    #[error("Forbidden")]
    Forbidden,
}

impl ApiErrorKind {
    pub fn to_status_code(&self) -> StatusCode {
        match self {
            ApiErrorKind::Unauthorized => StatusCode::UNAUTHORIZED,
            ApiErrorKind::BadRequest => StatusCode::BAD_REQUEST,
            ApiErrorKind::NotFound => StatusCode::NOT_FOUND,
            ApiErrorKind::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorKind::Forbidden => StatusCode::FORBIDDEN,
        }
    }

    pub fn message(&self) -> &str {
        match self {
            ApiErrorKind::Unauthorized => "Unauthorized access",
            ApiErrorKind::BadRequest => "Invalid request",
            ApiErrorKind::NotFound => "Resource not found",
            ApiErrorKind::InternalServerError => "Internal server error",
            ApiErrorKind::Forbidden => "Forbidden access",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: u16,
    pub message: String,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ApiError {{ code: {}, message: {} }}", self.code, self.message)
    }
}

impl ApiError {
    pub fn new(kind: ApiErrorKind) -> Self {
        ApiError {
            code: kind.to_status_code().as_u16(),
            message: kind.message().to_string(),
        }
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(StatusCode::from_u16(self.code).unwrap())
            .json(self)
    }
}

