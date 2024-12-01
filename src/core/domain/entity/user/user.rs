use thiserror::Error;

use crate::core::domain::entity::user::User;

#[derive(Debug, Error)]
pub enum CreateUserValidationError {}

#[derive(Debug)]
pub struct UnvalidatedCreateUserInput {
    pub id: i32,
    pub name: String,
    pub email: String,
}

impl TryFrom<UnvalidatedCreateUserInput> for User {
    type Error = CreateUserValidationError;

    fn try_from(value: UnvalidatedCreateUserInput) -> Result<Self, Self::Error> {
        Ok(User {
            id: value.id,
            name: value.name,
            email: value.email,
        })
    }
}
