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
use opaque_ke::{
    CredentialFinalizationLen, CredentialRequestLen, RegistrationRequestLen, RegistrationUploadLen,
};
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
        .try_lock()
        .map_err(|e| e.to_string())?;

    let registration_response = opaque_controller
        .register_init(body.username, registration_request)
        .map_err(|_| "Protocol error")?;

    let response = base64::engine::general_purpose::URL_SAFE.encode(registration_response);
    Ok(response)
}

#[derive(Deserialize)]
struct RegisterFinishRequest {
    username: String,
    registration_finish: Base64String,
}
async fn register_finish(
    State(state): State<AppState>,
    Json(body): Json<RegisterFinishRequest>,
) -> Result<(), String> {
    let registration_finish: GenericArray<u8, RegistrationUploadLen<CS>> = body
        .registration_finish
        .decode()
        .map_err(|e| e.to_string())?;

    let mut opaque_controller = state
        .opaque_controller
        .try_lock()
        .map_err(|e| e.to_string())?;

    opaque_controller
        .register_finish(body.username, registration_finish)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[derive(Deserialize)]
struct LoginInitRequest {
    username: String,
    credential_request: Base64String,
}
async fn login_init(
    State(state): State<AppState>,
    Json(body): Json<LoginInitRequest>,
) -> Result<String, String> {
    let credential_request: GenericArray<u8, CredentialRequestLen<CS>> = body
        .credential_request
        .decode()
        .map_err(|e| e.to_string())?;

    let mut opaque_controller = state
        .opaque_controller
        .try_lock()
        .map_err(|e| e.to_string())?;

    let credential_response = opaque_controller
        .login_start(body.username, credential_request)
        .map_err(|e| e.to_string())?;

    let response = base64::engine::general_purpose::URL_SAFE.encode(credential_response);
    Ok(response)
}

#[derive(Deserialize)]
struct LoginFinishRequest {
    username: String,
    credential_finish: Base64String,
}
async fn login_finish(
    State(state): State<AppState>,
    Json(body): Json<LoginFinishRequest>,
) -> Result<(), String> {
    let credential_finish: GenericArray<u8, CredentialFinalizationLen<CS>> =
        body.credential_finish.decode().map_err(|e| e.to_string())?;

    let mut opaque_controller = state
        .opaque_controller
        .try_lock()
        .map_err(|e| e.to_string())?;

    opaque_controller
        .login_finish(body.username, credential_finish)
        .map_err(|e| e.to_string())?;

    Ok(())
}
async fn session() -> Result<Json<String>, ()> {
    todo!()
}
