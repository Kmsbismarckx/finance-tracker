//! Доменная сущность Account (счёт).
//!
//! Это ядро бизнес-логики. Этот модуль НЕ знает ничего о:
//! - Базе данных
//! - HTTP/API
//! - Фреймворках
//!
//! Только чистая бизнес-логика.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::errors::DomainError;

/// Сущность "Счёт" — основной объект предметной области.
///
/// # Поля
/// - `id` — уникальный идентификатор (UUID v4)
/// - `balance` — баланс в копейках/центах (i64 вместо f64 для точности)
/// - `currency` — код валюты (USD, RUB, EUR)
///
/// # Почему баланс в i64?
/// Floating point числа имеют проблемы с точностью:
/// ```text
/// 0.1 + 0.2 = 0.30000000000000004 // не 0.3!
/// ```
/// В финансах используют целые числа: 1050 = 10.50 рублей
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub name: String,
    pub balance: i64,
    pub currency: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Account {
    /// Создаёт новый счёт с нулевым балансом.
    ///
    /// # Arguments
    /// * `name` — название счёта ("Кошелёк", "Сбережения")
    /// * `currency` — код валюты ("RUB", "USD")
    ///
    /// # Пример
    /// ```text
    /// let account = Account::new("Wallet".to_string(), "USD".to_string());
    /// assert_eq!(account.balance, 0);
    /// ```
    pub fn new(name: String, currency: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(), // Генерируем случайный UUID
            name,
            balance: 0,
            currency,
            created_at: now,
            updated_at: now,
        }
    }

    /// Пополняет счёт на указанную сумму (в копейках).
    ///
    /// # Arguments
    /// * `amount` — сумма в минимальных единицах (копейки/центы)
    ///
    /// # Errors
    /// Возвращает `DomainError::InvalidAmount` если сумма <= 0
    ///
    /// # Пример
    /// ```text
    /// account.deposit(10050)?;  // Пополнить на 100.50
    /// ```
    pub fn deposit(&mut self, amount: i64) -> Result<(), DomainError> {
        // Бизнес-правило: нельзя вносить отрицательную или нулевую сумму
        if amount <= 0 {
            return Err(DomainError::InvalidAmount("Amount must be positive".into()));
        }
        self.balance += amount;
        self.updated_at = Utc::now(); // Обновляем timestamp
        Ok(())
    }

    /// Снимает деньги со счёта.
    ///
    /// # Errors
    /// - `InvalidAmount` — если сумма <= 0
    /// - `InsufficientFunds` — если недостаточно средств
    pub fn withdraw(&mut self, amount: i64) -> Result<(), DomainError> {
        if amount <= 0 {
            return Err(DomainError::InvalidAmount("Amount must be positive".into()));
        }
        // Бизнес-правило: нельзя уйти в минус
        if self.balance < amount {
            return Err(DomainError::InsufficientFunds {
                available: self.balance,
                requested: amount,
            });
        }
        self.balance -= amount;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Конвертирует баланс из копеек в рубли/доллары для отображения.
    ///
    /// # Пример
    /// ```text
    /// account.balance = 10050;
    /// assert_eq!(account.balance_as_f64(), 100.50);
    /// ```
    pub fn balance_as_f64(&self) -> f64 {
        self.balance as f64 / 100.0
    }
}
