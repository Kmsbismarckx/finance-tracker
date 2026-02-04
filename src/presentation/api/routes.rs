//! Конфигурация маршрутов API.
//!
//! Здесь связываем URL пути с handlers.

use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::application::services::AccountService;
use crate::infrastructure::database::PostgresAccountRepository;
use crate::presentation::api::handlers;

/// Создаёт Router с настроенными маршрутами.
///
/// # Routing в Axum
/// - `Router::new()` — создаёт пустой роутер
/// - `.route(path, handler)` — добавляет маршрут
/// - `.with_state(state)` — делает state доступным во всех handlers
///
/// # Синтаксис путей
/// - `/api/accounts` — статический путь
/// - `/api/accounts/:id` — `:id` извлекается через `Path<Uuid>`
///
/// # HTTP методы
/// - `get(handler)` — GET запросы
/// - `post(handler)` — POST запросы
/// - `delete(handler)` — DELETE запросы
pub fn create_router(service: AccountService<PostgresAccountRepository>) -> Router {
    Router::new()
        // GET /api/accounts — список счетов
        .route("/api/accounts", get(handlers::get_accounts))
        // POST /api/accounts — создать счёт
        .route("/api/accounts", post(handlers::create_account))
        // GET /api/accounts/:id — получить счёт
        .route("/api/accounts/:id", get(handlers::get_account))
        // DELETE /api/accounts/:id — удалить счёт
        .route("/api/accounts/:id", delete(handlers::delete_account))
        // POST /api/accounts/:id/deposit — пополнить
        .route("/api/accounts/:id/deposit", post(handlers::deposit))
        // POST /api/accounts/:id/withdraw — снять
        .route("/api/accounts/:id/withdraw", post(handlers::withdraw))
        // Передаём сервис как shared state
        // Все handlers получат к нему доступ через State(service)
        .with_state(service)
}
