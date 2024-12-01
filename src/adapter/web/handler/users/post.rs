use axum::http::StatusCode;
use std::sync::Arc;

use crate::core::domain::entity::user::user::UnvalidatedCreateUserInput;
use crate::core::port::create_user::CreateUserInputBoundary;

use crate::adapter::web::dto::create_user_web_input::CreateUserWebInput;
use crate::adapter::web::presenter::create_user::CreateUserPresenter;

pub struct UserHandler {
    use_case: Arc<dyn CreateUserInputBoundary>,
}

impl UserHandler {
    pub fn new(use_case: Arc<dyn CreateUserInputBoundary>) -> Self {
        Self { use_case }
    }

    pub async fn create_user(
        &self,
        user: CreateUserWebInput,
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
