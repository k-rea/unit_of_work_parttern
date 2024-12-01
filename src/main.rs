mod adapter;
mod core;

use anyhow::{anyhow, Error};
use async_trait::async_trait;
use axum::routing::post;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction};
use std::sync::Arc;

// --- Domainモデル ---
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
}

#[derive(Debug)]
pub struct UnvalidatedCreateUserInput {
    pub id: i32,
    pub name: String,
    pub email: String,
}

impl TryFrom<UnvalidatedCreateUserInput> for User {
    type Error = Error;

    fn try_from(value: UnvalidatedCreateUserInput) -> Result<Self, Self::Error> {
        Ok(User {
            id: value.id,
            name: value.name,
            email: value.email,
        })
    }
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn insert_user(
        &self,
        transaction: &mut Box<dyn TransactionWrapper>,
        user: User,
    ) -> Result<(), Error>;
}

pub struct PgUserRepository;

// use_case
#[async_trait]
pub trait CreateUserInputBoundary: Send + Sync {
    async fn execute(
        &self,
        input: UnvalidatedCreateUserInput,
        output_boundary: &mut dyn CreateUserOutputBoundary,
    ) -> Result<(), Error>;
}

pub struct CreateUserUseCase {
    repository: Arc<dyn UserRepository>,
    transaction_manager: Arc<dyn TransactionManager>,
}

impl CreateUserUseCase {
    pub fn new(
        repository: Arc<dyn UserRepository>,
        transaction_manager: Arc<dyn TransactionManager>,
    ) -> Self {
        Self {
            repository,
            transaction_manager,
        }
    }
}

#[async_trait]
impl CreateUserInputBoundary for CreateUserUseCase {
    async fn execute(
        &self,
        input: UnvalidatedCreateUserInput,
        output_boundary: &mut dyn CreateUserOutputBoundary,
    ) -> Result<(), Error> {
        let user = User::try_from(input)?;
        let id = user.id;
        let operation = Box::new(InsertUserOperation::new(
            user,
            self.repository.clone(),
        ) );
        self.transaction_manager.execute(operation).await?;

        output_boundary.execute(id)?;

        Ok(())
    }
}

pub trait CreateUserOutputBoundary: Send + Sync {
    fn execute(&mut self, output: i32) -> Result<(), Error>;
}

// ------------
pub struct CreateUserPresenter {
    output: Option<i32>,
}

impl CreateUserPresenter {
    pub fn new() -> Self {
        Self { output: None }
    }
    fn success(&self, output: i32) -> Result<StatusCode, (StatusCode, String)> {
        Ok(StatusCode::CREATED)
    }
    fn failure(&self, error: Error) -> (StatusCode, String) {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create user: {:?}", error),
        )
    }
}

impl CreateUserOutputBoundary for CreateUserPresenter {
    fn execute(&mut self, output: i32) -> Result<(), Error> {
        self.output = Some(output);
        Ok(())
    }
}

// -----
#[async_trait]
impl UserRepository for PgUserRepository {
    async fn insert_user(
        &self,
        transaction: &mut Box<dyn TransactionWrapper>,
        user: User,
    ) -> Result<(), Error> {
        let query = "INSERT INTO users (id, name, email) VALUES ($1, $2, $3)";
        let params: Vec<Box<dyn ToSql>> = vec![
            Box::new(user.id) as Box<dyn ToSql>,
            Box::new(user.name) as Box<dyn ToSql>,
            Box::new(user.email) as Box<dyn ToSql>,
        ];
        transaction.execute(query, params).await
    }
}

pub trait ToSql: Send + Sync + std::fmt::Debug {
    fn as_i32(&self) -> Option<i32> {
        None
    }
    fn as_string(&self) -> Option<String> {
        None
    }
}

impl ToSql for i32 {
    fn as_i32(&self) -> Option<i32> {
        Some(*self)
    }
}

impl ToSql for String {
    fn as_string(&self) -> Option<String> {
        Some(self.clone())
    }
}
// トレイトを分割して、ジェネリックな部分を別のトレイトに移動
// トランザクション操作を表す型消去されたトレイト
#[async_trait]
pub trait BoxedTransactionOperation: Send + Sync {
    async fn execute(&self, transaction: &mut Box<dyn TransactionWrapper>) -> Result<(), Error>;
}

#[async_trait]
pub trait TransactionManager: Send + Sync {
    async fn execute(&self, operation: Box<dyn BoxedTransactionOperation>) -> Result<(), Error>;
}

// 具体的な操作を表す構造体
pub struct InsertUserOperation {
    user: User,
    user_repository: Arc<dyn UserRepository>,
}

impl InsertUserOperation {
    pub fn new(user: User, user_repository: Arc<dyn UserRepository>) -> Self {
        Self { user,user_repository }
    }
}

// BoxedTransactionOperationの実装
#[async_trait]
impl BoxedTransactionOperation for InsertUserOperation {
    async fn execute(&self, transaction: &mut Box<dyn TransactionWrapper>) -> Result<(), Error> {
        self.user_repository
            .insert_user(transaction, self.user.clone())
            .await
    }
}

pub struct PgTransactionManager {
    pool: PgPool,
}

impl PgTransactionManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TransactionManager for PgTransactionManager {
    async fn execute(&self, operation: Box<dyn BoxedTransactionOperation>) -> Result<(), Error> {
        let mut transaction: Box<dyn TransactionWrapper> =
            Box::new(SqlxTransaction::new(self.pool.begin().await?));

        match operation.execute(&mut transaction).await {
            Ok(result) => {
                transaction.commit().await?;
                Ok(result)
            }
            Err(e) => {
                transaction.rollback().await?;
                Err(e)
            }
        }
    }
}

#[async_trait]
pub trait TransactionWrapper: Send + Sync {
    async fn execute(
        &mut self,
        query: &str,
        params: Vec<Box<dyn ToSql>>,
    ) -> Result<(), anyhow::Error>;
    async fn rollback(self: Box<Self>) -> Result<(), Error>;
    async fn commit(self: Box<Self>) -> Result<(), Error>;
}

pub struct SqlxTransaction<'t> {
    transaction: Transaction<'t, Postgres>,
}

impl<'a> SqlxTransaction<'a> {
    pub fn new(transaction: Transaction<'a, Postgres>) -> Self {
        Self { transaction }
    }
}

#[async_trait]
impl<'t> TransactionWrapper for SqlxTransaction<'t> {
    async fn execute(&mut self, query: &str, params: Vec<Box<dyn ToSql>>) -> Result<(), Error> {
        let mut sqlx_query = sqlx::query(query);

        for param in params {
            if let Some(value) = param.as_i32() {
                sqlx_query = sqlx_query.bind(value);
            } else if let Some(value) = param.as_string() {
                sqlx_query = sqlx_query.bind(value);
            } else {
                return Err(anyhow!(
                    "Unsupported parameter type: {:?}, expected i32 or String",
                    param
                ));
            }
        }

        sqlx_query
            .execute(&mut *self.transaction)
            .await
            .map_err(|e| anyhow!("Failed to execute query: {:?}, error: {:?}", query, e))?;
        Ok(())
    }

    async fn rollback(self: Box<Self>) -> Result<(), Error> {
        self.transaction.rollback().await.map_err(|e| anyhow!(e))
    }

    async fn commit(self: Box<Self>) -> Result<(), Error> {
        self.transaction
            .commit()
            .await
            .map_err(|e| anyhow!("Failed to commit transaction: {:?}", e))
    }
}

// --- State ---
pub struct AppState {
    pub transaction_manager: Arc<dyn TransactionManager>,
    pub create_user_repository: Arc<dyn UserRepository>,
    pub user_create_use_case: Arc<CreateUserUseCase>,
}

// --- handlers ---

pub struct UserHandler {
    use_case: Arc<dyn CreateUserInputBoundary>,
}

impl UserHandler {
    pub fn new(
        use_case: Arc<dyn CreateUserInputBoundary>,
    ) -> Self {
        Self {
            use_case,
        }
    }

    pub async fn create_user(
        &self,
        user: UserWebInput,
    ) -> Result<StatusCode, (StatusCode, String)> {
        let mut presenter = CreateUserPresenter::new();
        let input = UnvalidatedCreateUserInput::from(user);

        match self.use_case.execute(input, &mut presenter).await {
            Ok(_) => {
                if let Some(id) = presenter.output {
                    presenter.success(id)
                } else {
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Output not set by presenter".to_string(),
                    ))
                }
            }
            Err(error) => Err(presenter.failure(error)),
        }
    }
}

impl From<UserWebInput> for UnvalidatedCreateUserInput {
    fn from(value: UserWebInput) -> Self {
        Self {
            id: value.id,
            name: value.name,
            email: value.email,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserWebInput {
    pub id: i32,
    pub name: String,
    pub email: String,
}
async fn handle_create_user(
    State(state): State<Arc<AppState>>,
    Json(user): Json<UserWebInput>,
) -> Result<StatusCode, (StatusCode, String)> {
    let handler = UserHandler::new(
        state.user_create_use_case.clone(),
    );
    handler.create_user(user).await
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let database_url = "postgres://postgres:postgres@localhost:5452/app";
    let pool = PgPool::connect(&database_url).await?;
    // State を作成
    let transaction_manager = Arc::new(PgTransactionManager::new(pool));
    let create_user_repository = Arc::new(PgUserRepository);
    let user_create_use_case = Arc::new(CreateUserUseCase::new(
            create_user_repository.clone(),
            transaction_manager.clone(),
        ));
    let state = Arc::new(AppState {
        transaction_manager,
        create_user_repository,
        user_create_use_case,
    });

    let app = Router::new()
        .route("/users", post(handle_create_user))
        .with_state(state);

    println!("Server running at http://localhost:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
