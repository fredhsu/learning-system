mod api;
mod card_service;
mod database;
mod fsrs_scheduler;
mod llm_service;
mod models;

use anyhow::Result;
use axum::{
    http::StatusCode,
    response::Html,
    routing::get,
    Router,
};
use std::env;
use tokio::fs;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::{info, Level};

use crate::{
    api::{create_router, AppState},
    card_service::CardService,
    database::Database,
    llm_service::LLMService,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();
    
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    // Load environment variables
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:learning.db".to_string());
    let llm_api_key = env::var("LLM_API_KEY").unwrap_or_else(|_| "your-api-key".to_string());
    let llm_base_url = env::var("LLM_BASE_URL").ok();
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());

    info!("Starting Learning System server...");

    // Initialize database
    let db = Database::new(&database_url).await?;
    info!("Database initialized successfully");

    // Initialize services
    let card_service = CardService::new(db);
    let llm_service = LLMService::new(llm_api_key, llm_base_url);

    // Create application state
    let state = AppState {
        card_service,
        llm_service,
    };

    // Build the application router
    let app = Router::new()
        // Serve static files
        .route("/", get(serve_index))
        .route("/index.html", get(serve_index))
        .route("/styles.css", get(serve_css))
        .route("/app.js", get(serve_js))
        // API routes
        .merge(create_router(state))
        // CORS middleware
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive())
        );

    // Start the server
    let addr = format!("0.0.0.0:{}", port);
    info!("Server starting on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn serve_index() -> Result<Html<String>, StatusCode> {
    match fs::read_to_string("static/index.html").await {
        Ok(content) => Ok(Html(content)),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

async fn serve_css() -> Result<(StatusCode, [(&'static str, &'static str); 1], String), StatusCode> {
    match fs::read_to_string("static/styles.css").await {
        Ok(content) => Ok((
            StatusCode::OK,
            [("content-type", "text/css")],
            content,
        )),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

async fn serve_js() -> Result<(StatusCode, [(&'static str, &'static str); 1], String), StatusCode> {
    match fs::read_to_string("static/app.js").await {
        Ok(content) => Ok((
            StatusCode::OK,
            [("content-type", "application/javascript")],
            content,
        )),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}
