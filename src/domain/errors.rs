//! Доменные ошибки — ошибки бизнес-логики.
//!
//! Эти ошибки не знают об HTTP кодах или базе данных.
//! Они описывают что пошло не так с точки зрения бизнеса.

use thiserror::Error;

/// Перечисление всех возможных доменных ошибок.
///
/// # Атрибуты
/// - `#[derive(Error)]` — макрос из `thiserror`, реализует `std::error::Error`
/// - `#[error("...")]` — шаблон для `Display` trait (как будет выглядеть при печати)
#[derive(Debug, Error)]
pub enum DomainError {
    /// Некорректная сумма (отрицательная или ноль)
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),

    /// Недостаточно средств на счёте.
    /// Используем named fields для удобства форматирования.
    #[error("Insufficient funds: available {available}, requested {requested}")]
    InsufficientFunds { available: i64, requested: i64 },

    /// Счёт не найден
    #[error("Account not found: {0}")]
    AccountNotFound(String),

    /// Счёт с таким именем уже существует
    #[error("Account already exists: {0}")]
    AccountAlreadyExists(String),
}
