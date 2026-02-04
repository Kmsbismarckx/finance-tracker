//! HTTP handlers для работы со счетами.
//!
//! Handler в Axum — это async функция, которая:
//! 1. Принимает extractors (State, Json, Path, Query...)
//! 2. Вызывает бизнес-логику
//! 3. Возвращает response (Json, StatusCode, или impl IntoResponse)

use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::application::dto::{
    AccountResponse, CreateAccountRequest, DepositRequest, MessageResponse, WithdrawRequest,
};
use crate::application::services::AccountService;
use crate::infrastructure::database::PostgresAccountRepository;
use crate::presentation::api::error::ApiError;

/// Type alias для удобства — конкретный тип нашего сервиса.
type AppAccountService = AccountService<PostgresAccountRepository>;

/// POST /api/accounts — создание нового счёта.
///
/// # Extractors
/// - `State(service)` — извлекает shared state (наш сервис)
/// - `Json(request)` — парсит JSON body в структуру
///
/// # Возвращает
/// - `Ok(Json<AccountResponse>)` — 200 с данными счёта
/// - `Err(ApiError)` — ошибка (400, 409, 500)
pub async fn create_account(
    State(service): State<AppAccountService>,
    Json(request): Json<CreateAccountRequest>,
) -> Result<Json<AccountResponse>, ApiError> {
    // Вызываем use case
    // ? — пробрасывает ошибку, которая автоматически конвертируется в ApiError
    let account = service.create_account(request).await?;
    Ok(Json(account))
}

/// GET /api/accounts — получение списка всех счетов.
pub async fn get_accounts(
    State(service): State<AppAccountService>,
) -> Result<Json<Vec<AccountResponse>>, ApiError> {
    let accounts = service.get_all_accounts().await?;
    Ok(Json(accounts))
}

/// GET /api/accounts/:id — получение счёта по ID.
///
/// # Path extractor
/// `Path(id)` извлекает `:id` из URL и парсит как Uuid.
/// Если ID невалидный — Axum автоматически вернёт 400.
pub async fn get_account(
    State(service): State<AppAccountService>,
    Path(id): Path<Uuid>,
) -> Result<Json<AccountResponse>, ApiError> {
    let account = service.get_account(id).await?;
    Ok(Json(account))
}

/// POST /api/accounts/:id/deposit — пополнение счёта.
pub async fn deposit(
    State(service): State<AppAccountService>,
    Path(id): Path<Uuid>,
    Json(request): Json<DepositRequest>,
) -> Result<Json<AccountResponse>, ApiError> {
    let account = service.deposit(id, request).await?;
    Ok(Json(account))
}

/// POST /api/accounts/:id/withdraw — снятие денег.
pub async fn withdraw(
    State(service): State<AppAccountService>,
    Path(id): Path<Uuid>,
    Json(request): Json<WithdrawRequest>,
) -> Result<Json<AccountResponse>, ApiError> {
    let account = service.withdraw(id, request).await?;
    Ok(Json(account))
}

/// DELETE /api/accounts/:id — удаление счёта.
pub async fn delete_account(
    State(service): State<AppAccountService>,
    Path(id): Path<Uuid>,
) -> Result<Json<MessageResponse>, ApiError> {
    service.delete_account(id).await?;
    Ok(Json(MessageResponse::new("Account deleted successfully")))
}
