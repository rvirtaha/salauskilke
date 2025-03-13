#![allow(unused)] // TODO remove this and fix the warnings
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]

use dotenv::dotenv;
use envconfig::Envconfig;

use backend::controllers;
use backend::http;
use backend::models::Models;
use backend::utils::config;
use backend::utils::pg_pool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let config = config::Config::init_from_env()?;
    let pool = pg_pool::create_pg_pool(&config.database_url, 3).await?;

    sqlx::migrate!("db/migrations").run(&pool).await?;

    let models = Models::new(pool);

    http::serve(config).await?;

    Ok(())
}
