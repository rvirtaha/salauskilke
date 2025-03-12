#![allow(unused)]
mod utils;

use backend::{controllers::opaque::CS, utils::base64::Base64String};
use base64::Engine;
use generic_array::GenericArray;
use opaque_ke::{
    rand::rngs::OsRng, ClientLogin, ClientLoginFinishParameters, ClientLoginFinishResult,
    ClientRegistration, ClientRegistrationFinishParameters, CredentialResponse,
    CredentialResponseLen, RegistrationResponse, RegistrationResponseLen,
};
use reqwest::Client;
use serde_json::json;

#[tokio::test]
async fn test_server_setup() {
    let (base_url, server_handle) = utils::setup_server().await;
    let client = Client::new();

    let response = client
        .get(base_url.to_string())
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let response_text = response.text().await.expect("Failed to read response");
    assert_eq!(response_text, "Hello world!");

    // Stop the server
    server_handle.abort();
}

async fn register(
    username: &str,
    password: &str,
    base_url: &str,
    client: &Client,
    rng: &mut OsRng,
) {
    let registration_start = ClientRegistration::<CS>::start(rng, password.as_bytes()).unwrap();
    let registration_request = Base64String::encode(&registration_start.message.serialize());

    let response_body = client
        .post(format!("{}/auth/register/init", base_url))
        .json(&json!({ "username": username, "registration_request": registration_request }))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .text()
        .await
        .unwrap();

    let registration_response = Base64String::decode(&response_body.into())
        .map(|r: GenericArray<u8, RegistrationResponseLen<CS>>| {
            RegistrationResponse::deserialize(&r)
        })
        .unwrap()
        .unwrap();

    let registration_finish = registration_start
        .state
        .finish(
            rng,
            password.as_bytes(),
            registration_response,
            ClientRegistrationFinishParameters::default(),
        )
        .unwrap()
        .message
        .serialize();

    client
        .post(format!("{}/auth/register/finish", base_url))
        .json(&json!({ "username": username, "registration_finish": Base64String::encode(&registration_finish) }))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

async fn login(
    username: &str,
    password: &str,
    base_url: &str,
    client: &Client,
    rng: &mut OsRng,
) -> ClientLoginFinishResult<CS> {
    let login_start = ClientLogin::<CS>::start(rng, password.as_bytes()).unwrap();
    let credential_request = Base64String::encode(&login_start.message.serialize());

    let response_body = client
        .post(format!("{}/auth/login/init", base_url))
        .json(&json!({ "username": username, "credential_request": credential_request }))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .text()
        .await
        .unwrap();

    let credential_response = Base64String::decode(&response_body.into())
        .map(|r: GenericArray<u8, CredentialResponseLen<CS>>| CredentialResponse::deserialize(&r))
        .unwrap()
        .unwrap();

    let login_finish = login_start
        .state
        .finish(
            password.as_bytes(),
            credential_response,
            ClientLoginFinishParameters::default(),
        )
        .unwrap();

    let login_finish_message = login_finish.message.serialize();

    let response = client
        .post(format!("{}/auth/login/finish", base_url))
        .json(&json!({ "username": username, "credential_finish": Base64String::encode(&login_finish_message) }))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    login_finish
}

#[tokio::test]
async fn test_register_login_e2e() {
    let (base_url, _server_handle) = utils::setup_server().await;
    let client = Client::new();
    let mut rng = OsRng;

    register(
        "john.doe@example.com",
        "salasana123",
        &base_url,
        &client,
        &mut rng,
    )
    .await;

    let login_finish = login(
        "john.doe@example.com",
        "salasana123",
        &base_url,
        &client,
        &mut rng,
    )
    .await;
}
