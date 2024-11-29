use async_trait::async_trait;
use anyhow::{anyhow, Error};
use sqlx::{PgPool, Postgres, Transaction};

// --- Domainモデル ---#[derive(Debug)]
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

#[tokio::main]
async fn main() -> Result<(), Error> {
    let database_url = "postgres://postgres:postgres@localhost:5452/app";
    let pool = PgPool::connect(&database_url).await?;
    let sqlx_transaction= pool.begin().await?;
    let mut transaction = SqlxTransaction::new(sqlx_transaction);

    let user_repository = PgUserRepository;

    let user = User {
        id: 2,
        name: "Bob".to_string(),
        email: "bob@test.com".to_string(),
    };

    user_repository.insert_user(&mut transaction, user).await?;
    transaction.transaction.commit().await?;

    println!("User inserted successfully!");
    Ok(())
}