use axum::{response::Html, routing::get, Router};

pub fn router() -> Router {
    Router::new().route("/", get(root))
}

async fn root() -> Result<Html<String>, ()> {
    Ok("Hello world!".to_string().into())
}
