use std::fmt::{Display, Formatter};

use opaque_ke::errors::ProtocolError;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub enum ServiceError {
    InternalError(String),
    InvalidCredentials,
    LoginSessionMissingOrExpired,
}

impl From<opaque_ke::errors::ProtocolError> for ServiceError {
    fn from(err: opaque_ke::errors::ProtocolError) -> Self {
        match err {
            ProtocolError::LibraryError(err) => ServiceError::InternalError(err.to_string()),
            ProtocolError::InvalidLoginError => ServiceError::InvalidCredentials,
            ProtocolError::SerializationError => ServiceError::InvalidCredentials,
            ProtocolError::ReflectedValueError => ServiceError::InvalidCredentials,
            ProtocolError::IdentityGroupElementError => ServiceError::InvalidCredentials,
        }
    }
}

impl Display for ServiceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceError::InternalError(err) => write!(f, "Internal error: {}", err),
            ServiceError::InvalidCredentials => write!(f, "Invalid credentials"),
            ServiceError::LoginSessionMissingOrExpired => {
                write!(f, "Login session is missing or expired")
            }
        }
    }
}
