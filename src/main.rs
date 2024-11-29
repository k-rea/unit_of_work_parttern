use std::sync::Arc;
use async_trait::async_trait;
use axum::{extract::{Json, State}, http::StatusCode, Router};
use anyhow::{anyhow, Error};
use axum::routing::post;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction};

// --- Domainモデル ---
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn insert_user(
        &self,
        transaction: &mut dyn TransactionWrapper,
        user: User,
    ) -> Result<(), Error>;
}

pub struct PgUserRepository;

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn insert_user(
        &self,
        transaction: &mut dyn TransactionWrapper,
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
pub struct SqlxTransaction<'t> {
    transaction: Transaction<'t, Postgres>,
}

impl<'a> SqlxTransaction<'a> {
    pub fn new(transaction: Transaction<'a, Postgres>) -> Self {
        Self { transaction }
    }
}

#[async_trait]
pub trait TransactionWrapper: Send {
    async fn execute(&mut self, query: &str, params: Vec<Box<dyn ToSql>>) -> Result<(), anyhow::Error>;
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

        sqlx_query.execute(&mut *self.transaction).await.map_err(|e| {
            anyhow!("Failed to execute query: {:?}, error: {:?}", query, e)
        })?;

        Ok(())
    }
}


pub struct MockTransactionWrapper;

#[async_trait]
impl TransactionWrapper for MockTransactionWrapper {
    async fn execute(&mut self, query: &str, params: Vec<Box<dyn ToSql>>) -> Result<(), Error> {
        println!("Mock execute: query = {}, params = {:?}", query, params);
        Ok(())
    }
}

// --- State ---
pub struct AppState {
    pub pool: PgPool,
    pub user_repository: Arc<dyn UserRepository>,
}

// --- handlers ---
async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(user): Json<User>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut transaction = SqlxTransaction::new(state.pool.begin().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to begin transaction: {:?}", e),
        )
    })?);
    state
        .user_repository
        .insert_user(&mut transaction, user)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to insert user: {:?}", e),
            )
        })?;

    transaction.transaction.commit().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to commit transaction: {:?}", e),
        )
    })?;
    Ok(StatusCode::CREATED)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let database_url = "postgres://postgres:postgres@localhost:5452/app";
    let pool = PgPool::connect(&database_url).await?;
    // State を作成
    let state = Arc::new(AppState {
        pool,
        user_repository: Arc::new(PgUserRepository),
    });

    let app = Router::new()
        .route("/users", post(create_user))
        .with_state(state);

    println!("Server running at http://localhost:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}