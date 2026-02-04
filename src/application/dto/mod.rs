//! DTO (Data Transfer Objects) — объекты для передачи данных между слоями.
//!
//! # Зачем нужны DTO?
//! 1. Отделяют внутреннее представление (Domain) от внешнего (API)
//! 2. Позволяют менять API не трогая бизнес-логику
//! 3. Контролируют что именно видит клиент
//!
//! # Request vs Response
//! - Request DTO: что приходит от клиента (`Deserialize`)
//! - Response DTO: что отправляем клиенту (`Serialize`)

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::Account;

// ═══════════════════════════════════════════════════════════════════
// REQUEST DTOs — входящие данные от клиента
// ═══════════════════════════════════════════════════════════════════

/// Запрос на создание счёта.
///
/// # Пример JSON
/// ```json
/// {
///   "name": "Wallet",
///   "currency": "USD"
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    pub name: String,
    pub currency: String,
}

/// Запрос на пополнение счёта.
///
/// # Поле `amount`
/// Сумма в основных единицах валюты (доллары, рубли), НЕ в копейках.
/// Конвертация в копейки происходит в сервисе.
#[derive(Debug, Deserialize)]
pub struct DepositRequest {
    pub amount: f64,
}

/// Запрос на снятие денег.
#[derive(Debug, Deserialize)]
pub struct WithdrawRequest {
    pub amount: f64,
}

// ═══════════════════════════════════════════════════════════════════
// RESPONSE DTOs — исходящие данные для клиента
// ═══════════════════════════════════════════════════════════════════

/// Ответ с информацией о счёте.
///
/// # Отличия от доменной сущности
/// - `balance` как f64 (для удобства клиента)
/// - `created_at` как String (ISO 8601 формат)
#[derive(Debug, Serialize)]
pub struct AccountResponse {
    pub id: Uuid,
    pub name: String,
    pub balance: f64, // В рублях/долларах, не в копейках
    pub currency: String,
    pub created_at: String, // RFC 3339 формат
    pub updated_at: String,
}

/// Конвертация из доменной сущности в DTO.
///
/// # Trait `From<T>`
/// Позволяет использовать `.into()`:
/// ```text
/// let response: AccountResponse = account.into();
/// ```
impl From<Account> for AccountResponse {
    fn from(account: Account) -> Self {
        // ВАЖНО: сначала вызываем методы, потом перемещаем поля
        // Иначе получим ошибку "borrow of moved value"
        let balance = account.balance_as_f64();
        let created_at = account.created_at.to_rfc3339();
        let updated_at = account.updated_at.to_rfc3339();

        Self {
            id: account.id,
            name: account.name, // String перемещается (move)
            balance,
            currency: account.currency,
            created_at,
            updated_at,
        }
    }
}

/// Простой ответ с сообщением.
///
/// Используется для операций без возвращаемых данных (delete).
#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}

impl MessageResponse {
    /// Создаёт MessageResponse из любого типа, реализующего Into<String>.
    ///
    /// # Пример
    /// ```text
    /// MessageResponse::new("Success")       // &str
    /// MessageResponse::new(format!("OK"))   // String
    /// ```
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}
