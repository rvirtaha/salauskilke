use axum::{response::Html, routing::get, Router};

use super::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(root))
}

async fn root() -> Result<Html<String>, ()> {
    Ok("Hello world!".to_string().into())
}
