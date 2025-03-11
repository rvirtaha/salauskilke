use std::sync::Arc;

use super::AppState;
use crate::http::opaque::CS;
use axum::{
    extract::State,
    response::Html,
    routing::{get, post},
    Json, Router,
};
use base64;
use base64::Engine;
use generic_array::GenericArray;
use opaque_ke::{RegistrationRequestLen, RegistrationResponseLen};
use serde::Deserialize;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/register/init", post(register_init))
        .route("/register/finish", post(register_finish))
        .route("/login/init", post(login_init))
        .route("/login/finish", post(login_finish))
        .route("/session", get(session))
}

#[derive(Deserialize)]
struct RegisterInitRequest {
    username: String,
    registration_request: String, // base64
}

async fn register_init(
    State(state): State<AppState>,
    Json(body): Json<RegisterInitRequest>,
) -> Result<String, String> {
    let binary_data = base64::engine::general_purpose::URL_SAFE
        .decode(&body.registration_request)
        .map_err(|_| "Invalid Base64 data")?;

    let registration_request: GenericArray<u8, RegistrationRequestLen<CS>> =
        GenericArray::from_exact_iter(binary_data.into_iter()).ok_or("Bad request")?;

    let mut opaque_controller = state
        .opaque_controller
        .lock()
        .map_err(|e| "Mutex Guard failed")?;

    let registration_response = opaque_controller
        .register_init(body.username, registration_request)
        .map_err(|_| "Protocol error")?;

    let response = base64::engine::general_purpose::URL_SAFE.encode(registration_response);
    Ok(response)
}
async fn register_finish() -> Result<Json<String>, ()> {
    todo!()
}
async fn login_init() -> Result<Json<String>, ()> {
    todo!()
}
async fn login_finish() -> Result<Json<String>, ()> {
    todo!()
}
async fn session() -> Result<Json<String>, ()> {
    todo!()
}
