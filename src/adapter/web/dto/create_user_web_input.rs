use crate::core::domain::entity::user::user::UnvalidatedCreateUserInput;
use serde::{Deserialize, Serialize};

impl From<CreateUserWebInput> for UnvalidatedCreateUserInput {
    fn from(value: CreateUserWebInput) -> Self {
        Self {
            id: value.id,
            name: value.name,
            email: value.email,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateUserWebInput {
    pub id: i32,
    pub name: String,
    pub email: String,
}
