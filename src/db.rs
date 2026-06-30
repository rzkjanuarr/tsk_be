use sqlx::{MySql, Pool, Row};
use uuid::Uuid;
use chrono::Utc;

use crate::models::{Task, CreateTaskRequest, TaskStatus, User, RegisterRequest};
use crate::utils::generate_slug;
use crate::error::AppError;

/// Create a new task
pub async fn create_task(
    pool: &Pool<MySql>,
    req: &CreateTaskRequest,
    user_id: &str,
) -> Result<Task, AppError> {
    let id = Uuid::new_v4().to_string();
    let slug = generate_slug();
    let status = TaskStatus::Pending.as_str();
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO tasks (id, user_id, slug, title, description, status, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(user_id)
    .bind(&slug)
    .bind(&req.title)
    .bind(&req.description)
    .bind(status)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;

    // Fetch the created task
    let task = get_task_by_id(pool, &id, user_id).await?;
    log::info!("Task created: {}", task.id);
    Ok(task)
}

/// Get all tasks with pagination for specific user
pub async fn get_tasks_paginated(
    pool: &Pool<MySql>,
    page: i64,
    page_size: i64,
    user_id: &str,
) -> Result<Vec<Task>, AppError> {
    let offset = page * page_size;

    let tasks = sqlx::query_as::<_, Task>(
        r#"
        SELECT id, user_id, slug, title, description, status, created_at, updated_at
        FROM tasks
        WHERE user_id = ?
        ORDER BY created_at DESC
        LIMIT ? OFFSET ?
        "#,
    )
    .bind(user_id)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(tasks)
}

/// Get a single task by ID for specific user
pub async fn get_task_by_id(pool: &Pool<MySql>, id: &str, user_id: &str) -> Result<Task, AppError> {
    let task = sqlx::query_as::<_, Task>(
        r#"
        SELECT id, user_id, slug, title, description, status, created_at, updated_at
        FROM tasks
        WHERE id = ? AND user_id = ?
        "#,
    )
    .bind(id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(task)
}

/// Update task status for specific user
pub async fn update_task_status(
    pool: &Pool<MySql>,
    id: &str,
    status: &str,
    user_id: &str,
) -> Result<Task, AppError> {
    // Validate status
    if TaskStatus::from_str(status).is_none() {
        return Err(AppError::BadRequest("Invalid status value".to_string()));
    }

    let now = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        UPDATE tasks
        SET status = ?, updated_at = ?
        WHERE id = ? AND user_id = ?
        "#,
    )
    .bind(status)
    .bind(&now)
    .bind(id)
    .bind(user_id)
    .execute(pool)
    .await?;

    // Fetch the updated task
    let task = get_task_by_id(pool, id, user_id).await?;
    log::info!("Task {} status updated to {}", id, status);
    Ok(task)
}

/// Get task count
pub async fn get_task_count(pool: &Pool<MySql>) -> Result<i64, AppError> {
    let row = sqlx::query("SELECT COUNT(*) as count FROM tasks")
        .fetch_one(pool)
        .await?;

    let count: i64 = row.get("count");
    Ok(count)
}

/// Create a new user
pub async fn create_user(
    pool: &Pool<MySql>,
    req: &RegisterRequest,
    hashed_password: &str,
) -> Result<User, AppError> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO users (id, username, email, password, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&req.username)
    .bind(&req.email)
    .bind(hashed_password)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;

    let user = get_user_by_id(pool, &id).await?;
    log::info!("User created: {}", user.id);
    Ok(user)
}

/// Get user by ID
pub async fn get_user_by_id(pool: &Pool<MySql>, id: &str) -> Result<User, AppError> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT id, username, email, password, created_at, updated_at
        FROM users
        WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

/// Get user by email
pub async fn get_user_by_email(pool: &Pool<MySql>, email: &str) -> Result<User, AppError> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT id, username, email, password, created_at, updated_at
        FROM users
        WHERE email = ?
        "#,
    )
    .bind(email)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

/// Check if email exists
pub async fn email_exists(pool: &Pool<MySql>, email: &str) -> Result<bool, AppError> {
    let row = sqlx::query("SELECT COUNT(*) as count FROM users WHERE email = ?")
        .bind(email)
        .fetch_one(pool)
        .await?;

    let count: i64 = row.get("count");
    Ok(count > 0)
}
