use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use std::sync::Arc;

use crate::adapter::web::app_state::AppState;
use crate::adapter::web::dto::create_user_web_input::CreateUserWebInput;
use crate::adapter::web::handler::users::post::UserHandler;

pub async fn post(
    State(state): State<Arc<AppState>>,
    Json(user): Json<CreateUserWebInput>,
) -> Result<StatusCode, (StatusCode, String)> {
    let handler = UserHandler::new(state.user_create_use_case.clone());
    handler.create_user(user).await
}
