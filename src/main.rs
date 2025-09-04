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
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};
use tokio::fs;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::{
    api::{create_router, AppState},
    card_service::CardService,
    database::Database,
    llm_service::{LLMService, LLMProvider},
};

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();
    
    // Initialize comprehensive logging with file output
    let _guard = setup_logging()?;

    // Load environment variables
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:learning.db".to_string());
    let llm_api_key = env::var("LLM_API_KEY").unwrap_or_else(|_| "your-api-key".to_string());
    let llm_base_url = env::var("LLM_BASE_URL").ok();
    let llm_provider = env::var("LLM_PROVIDER").unwrap_or_else(|_| "openai".to_string());
    let llm_model = env::var("LLM_MODEL").ok();
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());

    info!("Starting Learning System server...");

    // Initialize database
    let db = Database::new(&database_url).await?;
    info!("Database initialized successfully");

    // Initialize services
    let card_service = CardService::new(db);
    
    // Parse LLM provider configuration
    let provider = match llm_provider.to_lowercase().as_str() {
        "gemini" | "google" => LLMProvider::Gemini,
        "openai" | "chatgpt" | "gpt" => LLMProvider::OpenAI,
        _ => {
            info!("Unknown LLM provider '{}', defaulting to OpenAI", llm_provider);
            LLMProvider::OpenAI
        }
    };
    
    let llm_service = LLMService::new_with_provider(llm_api_key, llm_base_url, provider.clone(), llm_model);
    
    info!("Initialized LLM service with provider: {:?}", provider);

    // Create application state
    let state = AppState {
        card_service,
        llm_service,
        review_sessions: Arc::new(Mutex::new(HashMap::new())),
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

fn setup_logging() -> Result<WorkerGuard> {
    use std::fs;
    use tracing_subscriber::fmt;

    // Create logs directory if it doesn't exist
    fs::create_dir_all("logs").unwrap_or_else(|e| {
        eprintln!("Warning: Could not create logs directory: {}", e);
    });

    // Configure log level from environment variable
    let default_log_level = "info,learning_system=debug";
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(default_log_level));

    // Set up file appender with daily rotation
    let file_appender = tracing_appender::rolling::daily("logs", "learning-system.log");
    let (non_blocking_file, guard) = tracing_appender::non_blocking(file_appender);

    // Configure console output
    let console_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(true);

    // Configure file output (no ANSI colors for files)
    let file_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(false)
        .with_writer(non_blocking_file);

    // Initialize subscriber with both console and file outputs
    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    info!("Logging initialized - writing to logs/learning-system.log with daily rotation");
    
    Ok(guard)
}
