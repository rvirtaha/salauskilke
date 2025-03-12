use super::AppState;
use crate::http::opaque::CS;
use crate::utils::base64::Base64String;
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use base64;
use base64::Engine;
use generic_array::GenericArray;
use opaque_ke::RegistrationRequestLen;
use serde::Deserialize;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/register/init", post(register_init))
        .route("/register/finish", post(register_finish))
        .route("/login/init", post(login_init))
        .route("/login/finish", post(login_finish))
        .route("/session", get(session))
        .with_state(state)
}

#[derive(Deserialize)]
struct RegisterInitRequest {
    username: String,
    registration_request: Base64String,
}

async fn register_init(
    State(state): State<AppState>,
    Json(body): Json<RegisterInitRequest>,
) -> Result<String, String> {
    let registration_request: GenericArray<u8, RegistrationRequestLen<CS>> = body
        .registration_request
        .decode()
        .map_err(|e| e.to_string())?;

    let mut opaque_controller = state
        .opaque_controller
        .lock()
        .map_err(|_| "Mutex Guard failed")?;

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
