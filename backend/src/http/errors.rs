use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::controllers::errors::ServiceError;
use crate::utils::base64::DecodeError;

pub enum ApiError {
    BadRequest(String),
    Unauthorized(String),
    Forbidden,
    NotFound,
    InternalServerError,
}

pub type ApiResult<T> = Result<T, ApiError>;

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::BadRequest(message) => (StatusCode::BAD_REQUEST, message).into_response(),
            ApiError::Unauthorized(message) => (StatusCode::UNAUTHORIZED, message).into_response(),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden").into_response(),
            ApiError::NotFound => (StatusCode::NOT_FOUND, "Not found").into_response(),
            ApiError::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
            }
        }
    }
}

impl From<DecodeError> for ApiError {
    fn from(err: DecodeError) -> Self {
        ApiError::BadRequest(err.to_string())
    }
}

impl From<ServiceError> for ApiError {
    fn from(err: ServiceError) -> Self {
        match err {
            ServiceError::InternalError(err) => {
                println!("{:}", err);
                Self::InternalServerError
            }
            ServiceError::InvalidCredentials => Self::Unauthorized(err.to_string()),
            ServiceError::LoginSessionMissingOrExpired => Self::Unauthorized(err.to_string()),
        }
    }
}
