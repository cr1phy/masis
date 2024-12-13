use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
pub enum ApiErrorKind {
    #[error("Internal Server Error")]
    InternalServerError,

    #[error("Email already in use")]
    EmailAlreadyInUse,

    #[error("Username already in use")]
    UsernameAlreadyInUse,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Session expired or invalid")]
    InvalidSession,
}

impl ApiErrorKind {
    pub fn to_status_code(&self) -> StatusCode {
        match self {
            ApiErrorKind::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorKind::EmailAlreadyInUse => StatusCode::CONFLICT,
            ApiErrorKind::UsernameAlreadyInUse => StatusCode::CONFLICT,
            ApiErrorKind::InvalidCredentials => StatusCode::UNAUTHORIZED,
            ApiErrorKind::InvalidSession => StatusCode::UNAUTHORIZED,
        }
    }

    pub fn message(&self) -> &str {
        match self {
            ApiErrorKind::InternalServerError => "Internal server error",
            ApiErrorKind::EmailAlreadyInUse => "Email is already in use",
            ApiErrorKind::UsernameAlreadyInUse => "Username is already in use",
            ApiErrorKind::InvalidCredentials => "Invalid email or password",
            ApiErrorKind::InvalidSession => "Session expired or invalid",
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
        write!(
            f,
            "ApiError {{ code: {}, message: {} }}",
            self.code, self.message
        )
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
        HttpResponse::build(StatusCode::from_u16(self.code).unwrap()).json(self)
    }
}
