use axum::routing::post;
use axum::Router;
use std::sync::Arc;

use crate::adapter::web::app_state::AppState;
use crate::adapter::web::route::users;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/users", post(users::post))
        .with_state(state)
}
