use std::fmt::Display;

use axum::response::{IntoResponse, Response};
use base64::Engine;
use generic_array::GenericArray;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum DecodeError {
    Base64Error(base64::DecodeError),
    LengthMismatch { expected: usize, actual: usize },
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeError::Base64Error(err) => write!(f, "Base64 decoding error: {}", err),
            DecodeError::LengthMismatch { expected, actual } => write!(
                f,
                "Incorrect length: expected {} bytes but got {}",
                expected, actual
            ),
        }
    }
}

impl std::error::Error for DecodeError {}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Base64String(String);

impl Base64String {
    pub fn decode<N: generic_array::ArrayLength<u8>>(
        &self,
    ) -> Result<GenericArray<u8, N>, DecodeError> {
        let binary_data = base64::engine::general_purpose::URL_SAFE
            .decode(self.0.clone())
            .map_err(DecodeError::Base64Error)?;
        let input_len = binary_data.len();
        let result: GenericArray<u8, N> = GenericArray::from_exact_iter(binary_data.into_iter())
            .ok_or(DecodeError::LengthMismatch {
                expected: N::to_usize(),
                actual: input_len,
            })?;

        Ok(result)
    }

    pub fn encode<N: generic_array::ArrayLength<u8>>(data: &GenericArray<u8, N>) -> Self {
        Base64String(base64::engine::general_purpose::URL_SAFE.encode(data))
    }
}

impl<N: generic_array::ArrayLength<u8>> From<&GenericArray<u8, N>> for Base64String {
    fn from(data: &GenericArray<u8, N>) -> Self {
        Base64String::encode(data)
    }
}

impl PartialEq<&str> for Base64String {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<String> for Base64String {
    fn eq(&self, other: &String) -> bool {
        self.0 == *other
    }
}

impl Display for Base64String {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl IntoResponse for Base64String {
    fn into_response(self) -> Response {
        self.0.into_response()
    }
}

impl From<String> for Base64String {
    fn from(value: String) -> Self {
        Base64String(value)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::expect_used)]

    use crate::controllers::opaque::CS;

    use super::*;
    use generic_array::typenum::{U3, U5};
    use opaque_ke::RegistrationRequestLen;

    #[test]
    fn correctly_decodes_input() {
        let input = Base64String("aGVsbG8=".to_string());
        let decoded: GenericArray<u8, U5> = input.decode().unwrap();
        assert_eq!(&decoded[..], b"hello");

        let input = Base64String("9Cd6foK9Xqr3w9TQ9tJZrrEDcckn9pkxch1R3j13yT4=".to_string());
        let decoded: GenericArray<u8, RegistrationRequestLen<CS>> = input.decode().unwrap();
        assert_eq!(
            decoded,
            [
                244, 39, 122, 126, 130, 189, 94, 170, 247, 195, 212, 208, 246, 210, 89, 174, 177,
                3, 113, 201, 39, 246, 153, 49, 114, 29, 81, 222, 61, 119, 201, 62
            ]
            .into()
        )
    }

    #[test]
    fn correctly_encodes_input() {
        let data: GenericArray<u8, U5> = GenericArray::clone_from_slice(b"hello");
        let encoded: Base64String = (&data).into();
        assert_eq!(encoded, "aGVsbG8=");
    }

    #[test]
    fn partial_eq_with_string_types_works() {
        let b64_string = Base64String("Hello world".to_string());

        assert_eq!(b64_string, "Hello world");
        assert_eq!(b64_string, "Hello world".to_string());
    }

    #[test]
    fn fails_on_invalid_base64() {
        let input = Base64String("invalid==".to_string());
        assert!(matches!(
            input.decode::<U3>(),
            Err(DecodeError::Base64Error(_))
        ));
    }

    #[test]
    fn fails_on_length_mismatch() {
        let input = Base64String("aGVsbG8=".to_string()); // "hello"
        assert!(matches!(
            input.decode::<U3>(),
            Err(DecodeError::LengthMismatch { .. })
        ));
    }
}
