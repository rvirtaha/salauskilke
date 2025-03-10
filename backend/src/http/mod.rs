use axum::Router;

use crate::utils::config::Config;
mod index;

pub async fn serve(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let app = router();

    let listener =
        tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port.clone())).await?;

    axum::serve(listener, app).await?;
    Ok(())
}

fn router() -> Router {
    index::router()
}
