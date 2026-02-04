//! Точка входа приложения.
//!
//! Здесь происходит "склейка" всех слоёв:
//! 1. Загрузка конфигурации
//! 2. Подключение к БД
//! 3. Создание зависимостей (Dependency Injection)
//! 4. Запуск HTTP сервера

mod application;
mod domain;
mod infrastructure;
mod presentation;

use sqlx::postgres::PgPoolOptions;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::application::services::AccountService;
use crate::infrastructure::config::Config;
use crate::infrastructure::database::PostgresAccountRepository;
use crate::presentation::api::routes::create_router;

/// Точка входа — async main с tokio runtime.
///
/// # Атрибут `#[tokio::main]`
/// Преобразует async fn main в обычный fn main с tokio runtime:
/// ```text
/// fn main() {
///     tokio::runtime::Runtime::new().unwrap().block_on(async { ... })
/// }
/// ```
///
/// # Возвращаемый тип
/// `Result<(), Box<dyn std::error::Error>>` — позволяет использовать `?`
/// для любых ошибок. `Box<dyn Error>` — trait object для любой ошибки.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ═══════════════════════════════════════════════════════════════
    // 1. Загрузка переменных окружения из .env файла
    // ═══════════════════════════════════════════════════════════════
    // .ok() — игнорируем ошибку если файла нет
    dotenvy::dotenv().ok();

    // ═══════════════════════════════════════════════════════════════
    // 2. Инициализация логирования (tracing)
    // ═══════════════════════════════════════════════════════════════
    tracing_subscriber::registry()
        // EnvFilter — фильтрует логи по уровню
        // "info,sqlx=warn" — всё на уровне INFO, но sqlx только WARN
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,sqlx=warn".into()),
        ))
        // fmt::layer — форматирует логи для консоли
        .with(tracing_subscriber::fmt::layer())
        .init();

    // ═══════════════════════════════════════════════════════════════
    // 3. Загрузка конфигурации
    // ═══════════════════════════════════════════════════════════════
    let config = Config::from_env()?;

    // ═══════════════════════════════════════════════════════════════
    // 4. Создание пула соединений с PostgreSQL
    // ═══════════════════════════════════════════════════════════════
    let pool = PgPoolOptions::new()
        .max_connections(5) // Максимум 5 соединений в пуле
        .connect(&config.database_url)
        .await?;

    // ═══════════════════════════════════════════════════════════════
    // 5. Применение миграций БД
    // ═══════════════════════════════════════════════════════════════
    // sqlx::migrate! — макрос, который включает миграции в бинарник
    // на этапе компиляции. Путь относительно Cargo.toml.
    sqlx::migrate!("./migrations").run(&pool).await?;

    tracing::info!("Database connected and migrations applied");

    // ═══════════════════════════════════════════════════════════════
    // 6. Dependency Injection — создание графа зависимостей
    // ═══════════════════════════════════════════════════════════════
    // Порядок важен: Repository → Service → Router
    let repository = PostgresAccountRepository::new(pool);
    let service = AccountService::new(repository);

    // ═══════════════════════════════════════════════════════════════
    // 7. Создание роутера с middleware
    // ═══════════════════════════════════════════════════════════════
    let app = create_router(service)
        // TraceLayer — логирует все HTTP запросы
        .layer(TraceLayer::new_for_http())
        // CorsLayer — разрешает cross-origin запросы (для фронтенда)
        .layer(
            CorsLayer::new()
                .allow_origin(Any) // Разрешить любой origin
                .allow_methods(Any) // Разрешить любые методы
                .allow_headers(Any), // Разрешить любые headers
        );

    // ═══════════════════════════════════════════════════════════════
    // 8. Запуск HTTP сервера
    // ═══════════════════════════════════════════════════════════════
    let addr = config.server_addr();
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server running on http://{}", addr);

    // axum::serve — запускает сервер и блокирует до завершения
    axum::serve(listener, app).await?;

    Ok(())
}
