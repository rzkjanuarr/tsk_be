use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub success: bool,
    pub message: String,
}

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    BadRequest(String),
    InternalServerError(String),
    Conflict(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "{}", msg),
            AppError::BadRequest(msg) => write!(f, "{}", msg),
            AppError::InternalServerError(msg) => write!(f, "{}", msg),
            AppError::Conflict(msg) => write!(f, "{}", msg),
        }
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Conflict(_) => StatusCode::CONFLICT,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        let error_msg = self.to_string();

        HttpResponse::build(status).json(ApiError {
            success: false,
            message: error_msg,
        })
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        log::error!("Database error: {:?}", err);
        match err {
            sqlx::Error::RowNotFound => AppError::NotFound("Task not found".to_string()),
            sqlx::Error::Database(db_err) => {
                if db_err.message().contains("unique constraint") {
                    AppError::Conflict("Slug already exists".to_string())
                } else {
                    AppError::InternalServerError("Database error occurred".to_string())
                }
            }
            _ => AppError::InternalServerError("Database error occurred".to_string()),
        }
    }
}
