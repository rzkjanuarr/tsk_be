use actix_web::{web, HttpResponse, Scope};
use serde::Deserialize;
use sqlx::PgPool;

use crate::db;
use crate::error::AppError;
use crate::models::{ApiResponse, CreateTaskRequest, TaskResponse, UpdateTaskStatusRequest};

#[derive(Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

async fn get_tasks(
    pool: web::Data<PgPool>,
    query: web::Query<PaginationQuery>,
) -> Result<HttpResponse, AppError> {
    let page = query.page.unwrap_or(0);
    let page_size = query.page_size.unwrap_or(5);

    let tasks = db::get_tasks_paginated(&pool, page, page_size).await?;
    let responses: Vec<TaskResponse> = tasks.into_iter().map(|t| t.into()).collect();

    Ok(HttpResponse::Ok().json(ApiResponse::ok(responses)))
}

async fn get_task(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let task = db::get_task_by_id(&pool, &id).await?;
    let response: TaskResponse = task.into();

    Ok(HttpResponse::Ok().json(ApiResponse::ok(response)))
}

async fn create_task(
    pool: web::Data<PgPool>,
    req: web::Json<CreateTaskRequest>,
) -> Result<HttpResponse, AppError> {
    // Validation
    if req.title.trim().is_empty() {
        return Err(AppError::BadRequest("Title is required".to_string()));
    }
    if req.description.trim().is_empty() {
        return Err(AppError::BadRequest("Description is required".to_string()));
    }

    let task = db::create_task(&pool, &req).await?;
    let response: TaskResponse = task.into();

    Ok(HttpResponse::Created().json(ApiResponse::ok(response)))
}

async fn update_task_status(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    req: web::Json<UpdateTaskStatusRequest>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let task = db::update_task_status(&pool, &id, &req.status).await?;
    let response: TaskResponse = task.into();

    Ok(HttpResponse::Ok().json(ApiResponse::ok(response)))
}

async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/health", web::get().to(health))
            .route("/tasks", web::get().to(get_tasks))
            .route("/tasks", web::post().to(create_task))
            .route("/tasks/{id}", web::get().to(get_task))
            .route("/tasks/{id}/status", web::patch().to(update_task_status))
    );
}
