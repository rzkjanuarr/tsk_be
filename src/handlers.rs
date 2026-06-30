use actix_web::{web, HttpResponse, HttpRequest};
use serde::Deserialize;
use sqlx::{MySql, Pool};

use crate::db;
use crate::error::AppError;
use crate::middleware::get_user_id;
use crate::models::{ApiResponse, CreateTaskRequest, TaskResponse, UpdateTaskStatusRequest, RegisterRequest, LoginRequest, AuthResponse, UserResponse};
use crate::utils::{hash_password, verify_password, create_token};

#[derive(Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

async fn get_tasks(
    pool: web::Data<Pool<MySql>>,
    query: web::Query<PaginationQuery>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let user_id = get_user_id(&req)?;
    let page = query.page.unwrap_or(0);
    let page_size = query.page_size.unwrap_or(5);

    let tasks = db::get_tasks_paginated(&pool, page, page_size, &user_id).await?;
    let responses: Vec<TaskResponse> = tasks.into_iter().map(|t| t.into()).collect();

    Ok(HttpResponse::Ok().json(ApiResponse::ok(responses)))
}

async fn get_task(
    pool: web::Data<Pool<MySql>>,
    path: web::Path<String>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let user_id = get_user_id(&req)?;
    let id = path.into_inner();
    let task = db::get_task_by_id(&pool, &id, &user_id).await?;
    let response: TaskResponse = task.into();

    Ok(HttpResponse::Ok().json(ApiResponse::ok(response)))
}

async fn create_task(
    pool: web::Data<Pool<MySql>>,
    req: web::Json<CreateTaskRequest>,
    http_req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let user_id = get_user_id(&http_req)?;
    // Validation
    if req.title.trim().is_empty() {
        return Err(AppError::BadRequest("Title is required".to_string()));
    }
    if req.description.trim().is_empty() {
        return Err(AppError::BadRequest("Description is required".to_string()));
    }

    let task = db::create_task(&pool, &req, &user_id).await?;
    let response: TaskResponse = task.into();

    Ok(HttpResponse::Created().json(ApiResponse::ok(response)))
}

async fn update_task_status(
    pool: web::Data<Pool<MySql>>,
    path: web::Path<String>,
    req: web::Json<UpdateTaskStatusRequest>,
    http_req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let user_id = get_user_id(&http_req)?;
    let id = path.into_inner();
    let task = db::update_task_status(&pool, &id, &req.status, &user_id).await?;
    let response: TaskResponse = task.into();

    Ok(HttpResponse::Ok().json(ApiResponse::ok(response)))
}

async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
}

async fn register(
    pool: web::Data<Pool<MySql>>,
    req: web::Json<RegisterRequest>,
) -> Result<HttpResponse, AppError> {
    // Validation
    if req.username.trim().is_empty() {
        return Err(AppError::BadRequest("Username is required".to_string()));
    }
    if req.email.trim().is_empty() {
        return Err(AppError::BadRequest("Email is required".to_string()));
    }
    if req.password.len() < 6 {
        return Err(AppError::BadRequest("Password must be at least 6 characters".to_string()));
    }

    // Check if email exists
    if db::email_exists(&pool, &req.email).await? {
        return Err(AppError::Conflict("Email already registered".to_string()));
    }

    // Hash password
    let hashed_password = hash_password(&req.password)?;

    // Create user
    let user = db::create_user(&pool, &req, &hashed_password).await?;

    // Create token
    let token = create_token(&user.id)?;

    let user_response: UserResponse = user.into();
    let auth_response = AuthResponse {
        token,
        user: user_response,
    };

    Ok(HttpResponse::Created().json(ApiResponse::ok(auth_response)))
}

async fn login(
    pool: web::Data<Pool<MySql>>,
    req: web::Json<LoginRequest>,
) -> Result<HttpResponse, AppError> {
    // Validation
    if req.email.trim().is_empty() {
        return Err(AppError::BadRequest("Email is required".to_string()));
    }
    if req.password.is_empty() {
        return Err(AppError::BadRequest("Password is required".to_string()));
    }

    // Get user by email
    let user = match db::get_user_by_email(&pool, &req.email).await {
        Ok(user) => user,
        Err(AppError::NotFound(_)) => {
            return Err(AppError::BadRequest("Invalid email or password".to_string()));
        }
        Err(e) => return Err(e),
    };

    // Verify password
    let is_valid = verify_password(&req.password, &user.password)?;
    if !is_valid {
        return Err(AppError::BadRequest("Invalid email or password".to_string()));
    }

    // Create token
    let token = create_token(&user.id)?;

    let user_response: UserResponse = user.into();
    let auth_response = AuthResponse {
        token,
        user: user_response,
    };

    Ok(HttpResponse::Ok().json(ApiResponse::ok(auth_response)))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/health", web::get().to(health))
            .route("/auth/register", web::post().to(register))
            .route("/auth/login", web::post().to(login))
            .service(
                web::scope("/tasks")
                    .wrap(crate::middleware::AuthMiddleware)
                    .route("", web::get().to(get_tasks))
                    .route("", web::post().to(create_task))
                    .route("/{id}", web::get().to(get_task))
                    .route("/{id}/status", web::patch().to(update_task_status))
            )
    );
}
