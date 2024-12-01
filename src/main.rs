mod adapter;
pub mod core;
mod error;

use crate::adapter::config::AppConfig;
use crate::adapter::init::AppInitializer;
use crate::adapter::web::app_state::AppState;
use crate::adapter::web::create_router::create_router;
use crate::error::ApplicationError;
use axum::routing::post;
use axum::Router;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        println!("{}", e);
        std::process::exit(1)
    }
}

async fn run() -> Result<(), ApplicationError> {
    let config = AppConfig::load();

    let state = AppInitializer::initialize(config)
        .await
        .map_err(|e| ApplicationError::ConfigurationError(e.to_string()))?;

    let app = create_router(state);

    println!("Server running at http://localhost:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
