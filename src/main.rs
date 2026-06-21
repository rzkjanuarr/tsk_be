mod models;
mod handlers;
mod db;
mod utils;
mod error;

use actix_web::{web, App, HttpServer, middleware::Logger};
use sqlx::postgres::PgPoolOptions;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tasks (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            slug VARCHAR(255) NOT NULL UNIQUE,
            title VARCHAR(255) NOT NULL,
            description TEXT NOT NULL,
            status VARCHAR(50) NOT NULL,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create tasks table");

    let pool_data = web::Data::new(pool);

    log::info!("🚀 Starting Task Backend Server...");

    HttpServer::new(move || {
        App::new()
            .app_data(pool_data.clone())
            .wrap(Logger::default())
            .configure(handlers::config)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
