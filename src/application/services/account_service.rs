//! Сервис для работы со счетами — реализация use cases.
//!
//! # Что такое Service в луковой архитектуре?
//! Сервис координирует выполнение бизнес-сценариев (use cases):
//! 1. Получает запрос (DTO)
//! 2. Валидирует бизнес-правила
//! 3. Вызывает методы доменных сущностей
//! 4. Сохраняет через репозиторий
//! 5. Возвращает результат (DTO)

use uuid::Uuid;

use crate::application::dto::{
    AccountResponse, CreateAccountRequest, DepositRequest, WithdrawRequest,
};
use crate::application::ports::AccountRepository;
use crate::domain::entities::Account;
use crate::domain::errors::DomainError;

/// Сервис для операций со счетами.
///
/// # Generic параметр `R`
/// Сервис параметризован типом репозитория `R: AccountRepository`.
/// Это Dependency Injection на уровне типов:
/// ```text
/// // В продакшене:
/// let service = AccountService::new(PostgresAccountRepository::new(pool));
///
/// // В тестах:
/// let service = AccountService::new(MockAccountRepository::new());
/// ```
///
/// # Почему `#[derive(Clone)]`?
/// Axum требует Clone для state, чтобы шарить между потоками.
/// Это безопасно, потому что `PgPool` внутри использует `Arc`.
#[derive(Clone)]
pub struct AccountService<R: AccountRepository> {
    repository: R,
}

impl<R: AccountRepository> AccountService<R> {
    /// Создаёт новый экземпляр сервиса.
    ///
    /// # Arguments
    /// * `repository` — реализация `AccountRepository` (PostgreSQL, Mock, etc.)
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// Use case: Создание нового счёта.
    ///
    /// # Бизнес-правила
    /// - Имя счёта должно быть уникальным
    ///
    /// # Поток выполнения
    /// 1. Проверить, нет ли счёта с таким именем
    /// 2. Создать доменную сущность `Account`
    /// 3. Сохранить в репозиторий
    /// 4. Вернуть DTO для API
    pub async fn create_account(
        &self,
        request: CreateAccountRequest,
    ) -> Result<AccountResponse, AccountServiceError<R::Error>> {
        // Проверяем уникальность имени
        // .await? — ждём результат и пробрасываем ошибку если есть
        if self
            .repository
            .find_by_name(&request.name)
            .await
            .map_err(AccountServiceError::Repository)? // Конвертируем ошибку репозитория
            .is_some() // Если нашли — значит уже существует
        {
            return Err(AccountServiceError::Domain(
                DomainError::AccountAlreadyExists(request.name),
            ));
        }

        // Создаём доменную сущность
        let account = Account::new(request.name, request.currency);

        // Сохраняем
        self.repository
            .create(&account)
            .await
            .map_err(AccountServiceError::Repository)?;

        // Конвертируем в DTO и возвращаем
        // .into() вызывает From<Account> for AccountResponse
        Ok(account.into())
    }

    /// Use case: Получение счёта по ID.
    pub async fn get_account(
        &self,
        id: Uuid,
    ) -> Result<AccountResponse, AccountServiceError<R::Error>> {
        let account = self
            .repository
            .find_by_id(id)
            .await
            .map_err(AccountServiceError::Repository)?
            // .ok_or_else() — конвертирует None в Err
            .ok_or_else(|| {
                AccountServiceError::Domain(DomainError::AccountNotFound(id.to_string()))
            })?;

        Ok(account.into())
    }

    /// Use case: Получение всех счетов.
    pub async fn get_all_accounts(
        &self,
    ) -> Result<Vec<AccountResponse>, AccountServiceError<R::Error>> {
        let accounts = self
            .repository
            .find_all()
            .await
            .map_err(AccountServiceError::Repository)?;

        // Конвертируем Vec<Account> в Vec<AccountResponse>
        // .into_iter() — создаёт итератор, забирающий ownership
        // .map(Into::into) — применяет .into() к каждому элементу
        // .collect() — собирает обратно в Vec
        Ok(accounts.into_iter().map(Into::into).collect())
    }

    /// Use case: Пополнение счёта.
    ///
    /// # Поток
    /// 1. Найти счёт
    /// 2. Вызвать доменный метод deposit()
    /// 3. Сохранить изменения
    pub async fn deposit(
        &self,
        id: Uuid,
        request: DepositRequest,
    ) -> Result<AccountResponse, AccountServiceError<R::Error>> {
        // Получаем счёт (mut потому что будем изменять)
        let mut account = self
            .repository
            .find_by_id(id)
            .await
            .map_err(AccountServiceError::Repository)?
            .ok_or_else(|| {
                AccountServiceError::Domain(DomainError::AccountNotFound(id.to_string()))
            })?;

        // Конвертируем доллары в центы
        // round() — округляем, чтобы избежать проблем с float
        let amount_cents = (request.amount * 100.0).round() as i64;

        // Вызываем доменный метод (там бизнес-правила)
        account
            .deposit(amount_cents)
            .map_err(AccountServiceError::Domain)?;

        // Сохраняем изменения
        self.repository
            .update(&account)
            .await
            .map_err(AccountServiceError::Repository)?;

        Ok(account.into())
    }

    /// Use case: Снятие денег со счёта.
    pub async fn withdraw(
        &self,
        id: Uuid,
        request: WithdrawRequest,
    ) -> Result<AccountResponse, AccountServiceError<R::Error>> {
        let mut account = self
            .repository
            .find_by_id(id)
            .await
            .map_err(AccountServiceError::Repository)?
            .ok_or_else(|| {
                AccountServiceError::Domain(DomainError::AccountNotFound(id.to_string()))
            })?;

        let amount_cents = (request.amount * 100.0).round() as i64;

        // withdraw() может вернуть InsufficientFunds
        account
            .withdraw(amount_cents)
            .map_err(AccountServiceError::Domain)?;

        self.repository
            .update(&account)
            .await
            .map_err(AccountServiceError::Repository)?;

        Ok(account.into())
    }

    /// Use case: Удаление счёта.
    pub async fn delete_account(&self, id: Uuid) -> Result<(), AccountServiceError<R::Error>> {
        // Сначала проверяем, существует ли счёт
        self.repository
            .find_by_id(id)
            .await
            .map_err(AccountServiceError::Repository)?
            .ok_or_else(|| {
                AccountServiceError::Domain(DomainError::AccountNotFound(id.to_string()))
            })?;

        self.repository
            .delete(id)
            .await
            .map_err(AccountServiceError::Repository)?;

        Ok(())
    }
}

/// Ошибки сервиса — объединяют доменные ошибки и ошибки репозитория.
///
/// # Generic параметр `E`
/// Тип ошибки репозитория (sqlx::Error для PostgreSQL).
///
/// # Почему два варианта?
/// - `Domain` — ошибки бизнес-логики (можно показать пользователю)
/// - `Repository` — технические ошибки (логируем, но не показываем детали)
#[derive(Debug, thiserror::Error)]
pub enum AccountServiceError<E: std::error::Error> {
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError), // #[from] — автоматическая конвертация через .into()

    #[error("Repository error: {0}")]
    Repository(E),
}
