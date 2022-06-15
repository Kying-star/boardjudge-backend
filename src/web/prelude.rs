use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use sea_orm::DbErr;
use serde_json::json;
use thiserror::Error;

pub use crate::uuid;
pub type AppResult<T> = Result<T, AppError>;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database Error, message = `{0}`")]
    DatabaseError(#[from] DbErr),
    #[error("String Error, message = `{0}`")]
    StorageError(#[from] std::io::Error),
    #[error("Forbidden")]
    Forbidden(Option<anyhow::Error>),
    #[error("NotFound")]
    NotFound(Option<anyhow::Error>),
    #[error("BadRequest")]
    BadRequest(Option<anyhow::Error>),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::DatabaseError(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({
                        "type": "database_error",
                        "message": format!("{}", e),
                    })
                    .to_string(),
                )
                    .into_response()
            }
            AppError::StorageError(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({
                        "type": "storage_error",
                        "message": format!("{}", e),
                    })
                    .to_string(),
                )
                    .into_response()
            }
            AppError::Forbidden(e) => {
                return (
                    StatusCode::FORBIDDEN,
                    json!({
                        "type": "forbidden",
                        "message": format!("{:?}", e),
                    })
                    .to_string(),
                )
                    .into_response()
            }
            AppError::NotFound(e) => {
                return (
                    StatusCode::NOT_FOUND,
                    json!({
                        "type": "not_found",
                        "message": format!("{:?}", e),
                    })
                    .to_string(),
                )
                    .into_response()
            }
            AppError::BadRequest(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    json!({
                        "type": "bad_request",
                        "message": format!("{:?}", e),
                    })
                    .to_string(),
                )
                    .into_response()
            }
        }
    }
}

pub trait ErrorForOption {
    type T;
    fn found(self) -> Result<Self::T, AppError>;
    fn allow(self) -> Result<Self::T, AppError>;
    fn good(self) -> Result<Self::T, AppError>;
}

impl<T> ErrorForOption for Option<T> {
    type T = T;
    fn found(self) -> Result<T, AppError> {
        match self {
            Some(x) => Ok(x),
            None => Err(AppError::NotFound(None)),
        }
    }
    fn allow(self) -> Result<T, AppError> {
        match self {
            Some(x) => Ok(x),
            None => Err(AppError::Forbidden(None)),
        }
    }
    fn good(self) -> Result<T, AppError> {
        match self {
            Some(x) => Ok(x),
            None => Err(AppError::BadRequest(None)),
        }
    }
}

pub trait ErrorForResult {
    type T;
    fn found(self) -> Result<Self::T, AppError>;
    fn allow(self) -> Result<Self::T, AppError>;
    fn good(self) -> Result<Self::T, AppError>;
}

impl<T, E> ErrorForResult for Result<T, E>
where
    anyhow::Error: From<E>,
{
    type T = T;
    fn found(self) -> Result<T, AppError> {
        match self {
            Ok(x) => Ok(x),
            Err(x) => Err(AppError::NotFound(Some(x.into()))),
        }
    }
    fn allow(self) -> Result<T, AppError> {
        match self {
            Ok(x) => Ok(x),
            Err(x) => Err(AppError::Forbidden(Some(x.into()))),
        }
    }
    fn good(self) -> Result<T, AppError> {
        match self {
            Ok(x) => Ok(x),
            Err(x) => Err(AppError::BadRequest(Some(x.into()))),
        }
    }
}
