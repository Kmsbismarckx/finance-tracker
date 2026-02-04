//! Порт (интерфейс) для работы с хранилищем счетов.
//!
//! # Что такое Port в луковой архитектуре?
//! Порт — это trait (интерфейс), который определяет КОНТРАКТ.
//! Application слой говорит: "Мне нужен кто-то, кто умеет сохранять Account".
//! КАК именно — не важно (PostgreSQL, MongoDB, файл, память).
//!
//! # Инверсия зависимостей (DIP)
//! Application слой зависит от абстракции (trait), а не от конкретики (PostgreSQL).
//! Это позволяет:
//! - Легко менять БД
//! - Писать тесты с mock-репозиторием

use uuid::Uuid;

use crate::domain::entities::Account;

/// Порт для персистентности счетов.
///
/// # trait_variant::make
/// Этот макрос генерирует два трейта:
/// - `LocalAccountRepository` — базовый (для single-thread)
/// - `AccountRepository` — с `Send` bound (для async/multi-thread)
///
/// Нужен потому что async fn в trait требуют Send для работы с tokio.
///
/// # Ассоциированный тип `Error`
/// Каждая реализация определяет свой тип ошибки:
/// - PostgreSQL: `sqlx::Error`
/// - Mock: `std::convert::Infallible`
#[trait_variant::make(AccountRepository: Send)]
#[allow(dead_code)]
pub trait LocalAccountRepository {
    /// Тип ошибки, который возвращает эта реализация
    type Error: std::error::Error + Send + Sync + 'static;

    /// Создаёт новый счёт в хранилище
    async fn create(&self, account: &Account) -> Result<(), Self::Error>;

    /// Находит счёт по ID. Возвращает None если не найден.
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Account>, Self::Error>;

    /// Находит счёт по имени (case-insensitive)
    async fn find_by_name(&self, name: &str) -> Result<Option<Account>, Self::Error>;

    /// Возвращает все счета
    async fn find_all(&self) -> Result<Vec<Account>, Self::Error>;

    /// Обновляет существующий счёт
    async fn update(&self, account: &Account) -> Result<(), Self::Error>;

    /// Удаляет счёт по ID
    async fn delete(&self, id: Uuid) -> Result<(), Self::Error>;
}
