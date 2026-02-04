//! Реализация репозитория счетов на PostgreSQL.
//!
//! Этот модуль — часть Infrastructure слоя.
//! Он реализует порт `AccountRepository` из Application слоя.

use sqlx::PgPool;
use uuid::Uuid;

use crate::application::ports::AccountRepository;
use crate::domain::entities::Account;

/// PostgreSQL реализация репозитория счетов.
///
/// # Connection Pool
/// `PgPool` — это пул соединений. Он:
/// - Переиспользует соединения (не создаёт новое на каждый запрос)
/// - Thread-safe (можно шарить между потоками)
/// - Реализует `Clone` через `Arc` (дешёвое клонирование)
#[derive(Clone)]
pub struct PostgresAccountRepository {
    pool: PgPool,
}

impl PostgresAccountRepository {
    /// Создаёт новый репозиторий с указанным пулом соединений.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Реализация порта AccountRepository для PostgreSQL.
impl AccountRepository for PostgresAccountRepository {
    /// Тип ошибки — sqlx::Error
    type Error = sqlx::Error;

    /// Создаёт новую запись в таблице accounts.
    ///
    /// # SQL
    /// ```sql
    /// INSERT INTO accounts (id, name, balance, currency, created_at, updated_at)
    /// VALUES ($1, $2, $3, $4, $5, $6)
    /// ```
    ///
    /// # Плейсхолдеры
    /// `$1, $2...` — синтаксис PostgreSQL для параметризованных запросов.
    /// Защищает от SQL injection.
    async fn create(&self, account: &Account) -> Result<(), Self::Error> {
        sqlx::query(
            r#"
            INSERT INTO accounts (id, name, balance, currency, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        // .bind() — привязывает значение к плейсхолдеру
        .bind(account.id)
        .bind(&account.name) // &String — передаём ссылку
        .bind(account.balance)
        .bind(&account.currency)
        .bind(account.created_at)
        .bind(account.updated_at)
        .execute(&self.pool) // Выполняем запрос
        .await?; // Ждём результат, пробрасываем ошибку

        Ok(())
    }

    /// Находит счёт по ID.
    ///
    /// # Возвращает
    /// - `Ok(Some(account))` — если найден
    /// - `Ok(None)` — если не найден
    /// - `Err(e)` — если ошибка БД
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Account>, Self::Error> {
        // query_as — автоматически маппит результат в структуру
        // <_, AccountRow> — первый параметр выводится автоматически
        let account = sqlx::query_as::<_, AccountRow>(
            r#"
            SELECT id, name, balance, currency, created_at, updated_at
            FROM accounts
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool) // Возвращает Option<T>
        .await?;

        // Конвертируем AccountRow в Account через .map()
        Ok(account.map(Into::into))
    }

    /// Находит счёт по имени (регистронезависимо).
    ///
    /// # SQL функция LOWER()
    /// `LOWER(name) = LOWER($1)` — сравнение без учёта регистра.
    /// "Wallet" найдёт "wallet", "WALLET", "WaLLeT".
    async fn find_by_name(&self, name: &str) -> Result<Option<Account>, Self::Error> {
        let account = sqlx::query_as::<_, AccountRow>(
            r#"
            SELECT id, name, balance, currency, created_at, updated_at
            FROM accounts
            WHERE LOWER(name) = LOWER($1)
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(account.map(Into::into))
    }

    /// Возвращает все счета, отсортированные по дате создания.
    async fn find_all(&self) -> Result<Vec<Account>, Self::Error> {
        let accounts = sqlx::query_as::<_, AccountRow>(
            r#"
            SELECT id, name, balance, currency, created_at, updated_at
            FROM accounts
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool) // Возвращает Vec<T>
        .await?;

        // Конвертируем Vec<AccountRow> в Vec<Account>
        Ok(accounts.into_iter().map(Into::into).collect())
    }

    /// Обновляет существующий счёт.
    async fn update(&self, account: &Account) -> Result<(), Self::Error> {
        sqlx::query(
            r#"
            UPDATE accounts
            SET name = $2, balance = $3, currency = $4, updated_at = $5
            WHERE id = $1
            "#,
        )
        .bind(account.id)
        .bind(&account.name)
        .bind(account.balance)
        .bind(&account.currency)
        .bind(account.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Удаляет счёт по ID.
    async fn delete(&self, id: Uuid) -> Result<(), Self::Error> {
        sqlx::query("DELETE FROM accounts WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════
// Внутренний тип для маппинга из SQL
// ═══════════════════════════════════════════════════════════════════

/// Внутренняя структура для маппинга строки из БД.
///
/// # Почему отдельная структура?
/// `sqlx::FromRow` требует точное соответствие колонок.
/// Доменная сущность может иметь другие поля/методы.
///
/// # Атрибут `#[derive(sqlx::FromRow)]`
/// Автоматически генерирует код для преобразования строки БД в структуру.
#[derive(sqlx::FromRow)]
struct AccountRow {
    id: Uuid,
    name: String,
    balance: i64,
    currency: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

/// Конвертация из AccountRow в доменную сущность Account.
impl From<AccountRow> for Account {
    fn from(row: AccountRow) -> Self {
        Account {
            id: row.id,
            name: row.name,
            balance: row.balance,
            currency: row.currency,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
