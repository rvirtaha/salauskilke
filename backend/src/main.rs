#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]

mod http;
mod models;
mod services;
mod utils;

use dotenv::dotenv;
use envconfig::Envconfig;
use utils::config;
use utils::pg_pool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let config = config::Config::init_from_env()?;

    let pool = pg_pool::create_pg_pool(&config.database_url, 3).await?;

    sqlx::migrate!("db/migrations").run(&pool).await?;

    http::serve(config).await?;

    Ok(())
}
