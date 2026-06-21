use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

use crate::models::{Task, CreateTaskRequest, TaskStatus};
use crate::utils::generate_slug;
use crate::error::AppError;

/// Create a new task
pub async fn create_task(
    pool: &PgPool,
    req: &CreateTaskRequest,
) -> Result<Task, AppError> {
    let id = Uuid::new_v4().to_string();
    let slug = generate_slug();
    let status = TaskStatus::Pending.as_str();
    let now = Utc::now().to_rfc3339();

    let task = sqlx::query_as::<_, Task>(
        r#"
        INSERT INTO tasks (id, slug, title, description, status, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, slug, title, description, status, created_at, updated_at
        "#,
    )
    .bind(&id)
    .bind(&slug)
    .bind(&req.title)
    .bind(&req.description)
    .bind(status)
    .bind(&now)
    .bind(&now)
    .fetch_one(pool)
    .await?;

    log::info!("Task created: {}", task.id);
    Ok(task)
}

/// Get all tasks with pagination
pub async fn get_tasks_paginated(
    pool: &PgPool,
    page: i64,
    page_size: i64,
) -> Result<Vec<Task>, AppError> {
    let offset = page * page_size;

    let tasks = sqlx::query_as::<_, Task>(
        r#"
        SELECT id, slug, title, description, status, created_at, updated_at
        FROM tasks
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(tasks)
}

/// Get a single task by ID
pub async fn get_task_by_id(pool: &PgPool, id: &str) -> Result<Task, AppError> {
    let task = sqlx::query_as::<_, Task>(
        r#"
        SELECT id, slug, title, description, status, created_at, updated_at
        FROM tasks
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(task)
}

/// Update task status
pub async fn update_task_status(
    pool: &PgPool,
    id: &str,
    status: &str,
) -> Result<Task, AppError> {
    // Validate status
    if TaskStatus::from_str(status).is_none() {
        return Err(AppError::BadRequest("Invalid status value".to_string()));
    }

    let now = Utc::now().to_rfc3339();
    let task = sqlx::query_as::<_, Task>(
        r#"
        UPDATE tasks
        SET status = $1, updated_at = $2
        WHERE id = $3
        RETURNING id, slug, title, description, status, created_at, updated_at
        "#,
    )
    .bind(status)
    .bind(&now)
    .bind(id)
    .fetch_one(pool)
    .await?;

    log::info!("Task {} status updated to {}", id, status);
    Ok(task)
}

/// Get task count
pub async fn get_task_count(pool: &PgPool) -> Result<i64, AppError> {
    let row = sqlx::query("SELECT COUNT(*) as count FROM tasks")
        .fetch_one(pool)
        .await?;

    let count: i64 = row.get("count");
    Ok(count)
}
