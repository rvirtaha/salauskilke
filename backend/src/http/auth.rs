use super::{errors::ApiResult, AppState};
use crate::utils::base64::Base64String;
use crate::{controllers::errors::ServiceError, http::opaque::CS};
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
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
) -> ApiResult<Base64String> {
    let registration_request: GenericArray<u8, RegistrationRequestLen<CS>> =
        body.registration_request.decode()?;

    let mut opaque_controller = state
        .opaque_controller
        .try_lock()
        .map_err(|e| ServiceError::InternalError(e.to_string()))?;

    let registration_response =
        opaque_controller.register_init(body.username, registration_request)?;

    let response = Base64String::encode(&registration_response);
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
) -> ApiResult<()> {
    let registration_finish: GenericArray<u8, RegistrationUploadLen<CS>> =
        body.registration_finish.decode()?;

    let mut opaque_controller = state
        .opaque_controller
        .try_lock()
        .map_err(|e| ServiceError::InternalError(e.to_string()))?;

    opaque_controller.register_finish(body.username, registration_finish)?;

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
) -> ApiResult<Base64String> {
    let credential_request: GenericArray<u8, CredentialRequestLen<CS>> =
        body.credential_request.decode()?;

    let mut opaque_controller = state
        .opaque_controller
        .try_lock()
        .map_err(|e| ServiceError::InternalError(e.to_string()))?;

    let credential_response = opaque_controller.login_start(body.username, credential_request)?;

    let response = Base64String::encode(&credential_response);
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
) -> ApiResult<()> {
    let credential_finish: GenericArray<u8, CredentialFinalizationLen<CS>> =
        body.credential_finish.decode()?;

    let mut opaque_controller = state
        .opaque_controller
        .try_lock()
        .map_err(|e| ServiceError::InternalError(e.to_string()))?;

    opaque_controller.login_finish(body.username, credential_finish)?;

    Ok(())
}
async fn session() -> Result<Json<String>, ()> {
    todo!()
}
