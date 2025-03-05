#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]

mod controllers;
mod http;
mod models;
mod utils;

use controllers::Controllers;
use dotenv::dotenv;
use envconfig::Envconfig;
use models::Models;
use utils::config;
use utils::pg_pool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let config = config::Config::init_from_env()?;
    let pool = pg_pool::create_pg_pool(&config.database_url, 3).await?;

    sqlx::migrate!("db/migrations").run(&pool).await?;

    let models = Models::new(pool);
    let services = Controllers::new();

    http::serve(config).await?;

    Ok(())
}
