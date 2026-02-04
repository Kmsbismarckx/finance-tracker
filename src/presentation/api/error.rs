//! Преобразование ошибок в HTTP responses.
//!
//! # Почему отдельный модуль?
//! Presentation слой отвечает за то, КАК ошибки представлены клиенту:
//! - Доменные ошибки → понятные HTTP коды
//! - Технические ошибки → 500 без деталей (безопасность)

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::application::services::AccountServiceError;
use crate::domain::errors::DomainError;

/// Структура для HTTP ошибок API.
pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    /// Создаёт новую ошибку с указанным статусом и сообщением.
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    /// 500 Internal Server Error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }

    /// 404 Not Found
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }

    /// 400 Bad Request
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    /// 409 Conflict
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, message)
    }
}

/// Trait IntoResponse — как конвертировать ApiError в HTTP response.
///
/// Axum автоматически вызывает этот метод когда handler возвращает Err(ApiError).
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        // Создаём JSON body: {"error": "message"}
        let body = json!({
            "error": self.message
        });

        // Возвращаем tuple (StatusCode, Json) — Axum понимает этот формат
        (self.status, Json(body)).into_response()
    }
}

/// Конвертация AccountServiceError в ApiError.
///
/// # Маппинг ошибок
/// - AccountNotFound → 404
/// - AccountAlreadyExists → 409 Conflict
/// - InsufficientFunds → 400 Bad Request
/// - InvalidAmount → 400 Bad Request
/// - Repository errors → 500 (логируем, но не показываем детали)
impl<E: std::error::Error> From<AccountServiceError<E>> for ApiError {
    fn from(err: AccountServiceError<E>) -> Self {
        match err {
            // Доменные ошибки — можно показать пользователю
            AccountServiceError::Domain(domain_err) => match domain_err {
                DomainError::AccountNotFound(msg) => ApiError::not_found(msg),

                DomainError::AccountAlreadyExists(msg) => {
                    ApiError::conflict(format!("Account '{}' already exists", msg))
                }

                DomainError::InsufficientFunds {
                    available,
                    requested,
                } => ApiError::bad_request(format!(
                    "Insufficient funds: available {:.2}, requested {:.2}",
                    available as f64 / 100.0,
                    requested as f64 / 100.0
                )),

                DomainError::InvalidAmount(msg) => ApiError::bad_request(msg),
            },

            // Ошибки репозитория — логируем, но клиенту не показываем детали
            AccountServiceError::Repository(e) => {
                // tracing::error! — логирует ошибку (видно в консоли сервера)
                tracing::error!("Repository error: {}", e);
                // Клиенту отдаём generic сообщение (безопасность!)
                ApiError::internal("Internal server error")
            }
        }
    }
}
