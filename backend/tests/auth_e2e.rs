#![allow(unused)]
mod utils;

use backend::controllers::opaque::CS;
use base64::Engine;
use opaque_ke::{rand::rngs::OsRng, ClientRegistration};
use reqwest::Client;

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
