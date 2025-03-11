use std::{
    iter::Copied,
    sync::{Arc, Mutex},
};

use axum::Router;
use opaque_ke::rand::rngs::OsRng;

use crate::{controllers::opaque, utils::config::Config};
mod auth;
mod index;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub opaque_controller: Arc<Mutex<opaque::OpaqueController<OsRng>>>,
}

pub async fn serve(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let port = config.port;

    let opaque_controller = opaque::OpaqueController::default();

    let state = AppState {
        config: Arc::new(config),
        opaque_controller: Arc::new(Mutex::new(opaque_controller)),
    };

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    let app = router(state.clone()).with_state(state);

    axum::serve(listener, app).await?;
    Ok(())
}

fn router(state: AppState) -> Router<AppState> {
    index::router().nest("/auth", auth::router(state.clone()))
}
