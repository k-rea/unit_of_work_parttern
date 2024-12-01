use async_trait::async_trait;
use crate::core::domain::command::CommandError;
use crate::core::domain::entity::user::{User, UserCommand};
use crate::core::domain::transaction::{ToSql, TransactionWrapper};

pub struct PgUserRepository;
#[async_trait]
impl UserCommand for PgUserRepository {
    async fn insert(
        &self,
        transaction: &mut Box<dyn TransactionWrapper>,
        user: User,
    ) -> Result<(), CommandError> {
        let query = "INSERT INTO users (id, name, email) VALUES ($1, $2, $3)";
        let params: Vec<Box<dyn ToSql>> = vec![
            Box::new(user.id) as Box<dyn ToSql>,
            Box::new(user.name) as Box<dyn ToSql>,
            Box::new(user.email) as Box<dyn ToSql>,
        ];
        match transaction.execute(query, params).await {
            Ok(_) => Ok(()),
            Err(e) => {
                // SQLxのエラーを適切なドメインエラーに変換
                if e.to_string().contains("unique constraint") {
                    Err(CommandError::user_already_exists(user.id))
                } else if e.to_string().contains("deadlock") {
                    Err(CommandError::ConcurrencyError {entity_type: "User".to_string()})
                } else {
                    Err(CommandError::DatabaseError(e.to_string()))
                }
            }
        }
    }
}
