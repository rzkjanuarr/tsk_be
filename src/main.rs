mod models;
mod handlers;
mod db;
mod utils;
mod error;
mod middleware;

use actix_web::{web, App, HttpServer, middleware::Logger};
use sqlx::mysql::MySqlPoolOptions;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let database_url = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            log::error!("DATABASE_URL environment variable not set!");
            std::process::exit(1);
        }
    };
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_address = format!("0.0.0.0:{}", port);

    log::info!("Connecting to MySQL database...");
    log::info!("Server will bind to: {}", bind_address);

    let pool = match MySqlPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect(&database_url)
        .await
    {
        Ok(pool) => pool,
        Err(e) => {
            log::error!("Failed to connect to database: {}", e);
            log::error!("Please check DATABASE_URL and make sure database is running!");
            std::process::exit(1);
        }
    };

    // Create users table if not exists
    if let Err(e) = sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id VARCHAR(36) PRIMARY KEY,
            username VARCHAR(255) NOT NULL,
            email VARCHAR(255) NOT NULL UNIQUE,
            password VARCHAR(255) NOT NULL,
            created_at VARCHAR(50) NOT NULL,
            updated_at VARCHAR(50) NOT NULL
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
        "#,
    )
    .execute(&pool)
    .await
    {
        log::error!("Failed to create users table: {}", e);
    }

    // Create tasks table if not exists
    if let Err(e) = sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tasks (
            id VARCHAR(36) PRIMARY KEY,
            user_id VARCHAR(36) NOT NULL,
            slug VARCHAR(255) NOT NULL UNIQUE,
            title VARCHAR(255) NOT NULL,
            description LONGTEXT NOT NULL,
            status VARCHAR(50) NOT NULL,
            created_at VARCHAR(50) NOT NULL,
            updated_at VARCHAR(50) NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
        "#,
    )
    .execute(&pool)
    .await
    {
        log::error!("Failed to create tasks table: {}", e);
    }

    log::info!("Users and Tasks tables ready");

    let pool_data = web::Data::new(pool);

    log::info!("🚀 Starting Task Backend Server on {}", bind_address);

    HttpServer::new(move || {
        App::new()
            .app_data(pool_data.clone())
            .wrap(Logger::default())
            .configure(handlers::config)
    })
    .bind(&bind_address)?
    .run()
    .await
}
