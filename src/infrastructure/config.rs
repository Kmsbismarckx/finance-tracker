//! Конфигурация приложения.
//!
//! Загружает настройки из переменных окружения.

use std::env;

/// Конфигурация приложения.
///
/// # Переменные окружения
/// - `DATABASE_URL` — строка подключения к PostgreSQL (обязательно)
/// - `SERVER_HOST` — хост сервера (по умолчанию 127.0.0.1)
/// - `SERVER_PORT` — порт сервера (по умолчанию 3000)
#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
}

impl Config {
    /// Загружает конфигурацию из переменных окружения.
    ///
    /// # Errors
    /// Возвращает ошибку если `DATABASE_URL` не установлен.
    pub fn from_env() -> Result<Self, env::VarError> {
        Ok(Self {
            // env::var() — читает переменную окружения
            // ? — пробрасывает ошибку если не найдена
            database_url: env::var("DATABASE_URL")?,

            // unwrap_or_else — возвращает значение по умолчанию если ошибка
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".into()),

            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3000".into())
                .parse() // Парсим строку в число
                .unwrap_or(3000), // Если не удалось — 3000
        })
    }

    /// Возвращает адрес сервера в формате "host:port".
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}
