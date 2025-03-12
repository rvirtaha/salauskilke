#![allow(unused)]
mod utils;

use backend::{controllers::opaque::CS, utils::base64::Base64String};
use base64::Engine;
use generic_array::GenericArray;
use opaque_ke::{
    rand::rngs::OsRng, ClientLogin, ClientLoginFinishParameters, ClientRegistration,
    ClientRegistrationFinishParameters, CredentialResponse, CredentialResponseLen,
    RegistrationResponse, RegistrationResponseLen,
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
    username: String,
    password: String,
    base_url: &String,
    client: &Client,
    mut rng: OsRng,
) {
    let client_registration_start =
        ClientRegistration::<CS>::start(&mut rng, password.as_bytes()).unwrap();

    let registration_request = client_registration_start.message.serialize();

    let response = client
        .post(format!("{}/auth/register/init", base_url))
        .json(&json!({
            "username": username,
            "registration_request": Base64String::encode(&registration_request),
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let response_body = response.text().await.unwrap();
    let registration_response: GenericArray<u8, RegistrationResponseLen<CS>> =
        Base64String::decode(&response_body.into()).unwrap();

    let registration_finish = client_registration_start
        .state
        .finish(
            &mut rng,
            password.as_bytes(),
            RegistrationResponse::deserialize(&registration_response).unwrap(),
            ClientRegistrationFinishParameters::default(),
        )
        .map(|r| r.message.serialize())
        .map(|s| Base64String::encode(&s))
        .unwrap();

    let response = client
        .post(format!("{}/auth/register/finish", base_url))
        .json(&json!({
            "username": username,
            "registration_finish": registration_finish,
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), reqwest::StatusCode::OK)
}

async fn login(
    username: String,
    password: String,
    base_url: &String,
    client: &Client,
    mut rng: OsRng,
) {
    let login_start = ClientLogin::<CS>::start(&mut rng, password.as_bytes()).unwrap();
    let credential_request = login_start.message.serialize();

    let response = client
        .post(format!("{}/auth/login/init", base_url))
        .json(&json!({
            "username": username,
            "credential_request": Base64String::encode(&credential_request),
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let response_body = response.text().await.unwrap();
    let credential_response: GenericArray<u8, CredentialResponseLen<CS>> =
        Base64String::decode(&response_body.into()).unwrap();

    let login_finish = login_start
        .state
        .finish(
            password.as_bytes(),
            CredentialResponse::deserialize(&credential_response).unwrap(),
            ClientLoginFinishParameters::default(),
        )
        .unwrap();

    let response = client
        .post(format!("{}/auth/login/finish", base_url))
        .json(&json!({
            "username": username,
            "credential_finish": Base64String::encode(&login_finish.message.serialize()),
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), reqwest::StatusCode::OK)
}

#[tokio::test]
async fn test_register_login_e2e() {
    let (base_url, server_handle) = utils::setup_server().await;
    let client = Client::new();
    let mut rng = OsRng;

    const USERNAME: &str = "john.doe@example.com";
    const PASSWORD: &str = "salasana123";

    register(
        USERNAME.to_string(),
        PASSWORD.to_string(),
        &base_url,
        &client,
        rng,
    )
    .await;

    login(
        USERNAME.to_string(),
        PASSWORD.to_string(),
        &base_url,
        &client,
        rng,
    )
    .await;
}
