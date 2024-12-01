use axum::http::StatusCode;

use crate::core::port::create_user::{
    CreateUserError, CreateUserOutputBoundary, CreateUserOutputError,
};

pub struct CreateUserPresenter {
    pub(crate) output: Option<i32>,
}

impl CreateUserPresenter {
    pub fn new() -> Self {
        Self { output: None }
    }
    pub(crate) fn success(&self, output: i32) -> Result<StatusCode, (StatusCode, String)> {
        Ok(StatusCode::CREATED)
    }
    pub(crate) fn failure(&self, error: CreateUserError) -> (StatusCode, String) {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create user: {:?}", error),
        )
    }
}

impl CreateUserOutputBoundary for CreateUserPresenter {
    fn execute(&mut self, output: i32) -> Result<(), CreateUserOutputError> {
        self.output = Some(output);
        Ok(())
    }
}
