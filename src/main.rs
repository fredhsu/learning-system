mod api;
mod card_service;
mod config;
mod database;
mod errors;
mod fsrs_scheduler;
mod llm_providers;
mod llm_service;
mod logging;
mod models;

use anyhow::Result;
use axum::{Router, http::StatusCode, response::Html, routing::get};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::fs;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    api::{AppState, create_router},
    card_service::CardService,
    config::Config,
    database::Database,
    llm_service::LLMService,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Load centralized configuration
    let config = Config::from_env()?;
    config.validate()?;

    // Initialize comprehensive logging with configuration
    let _guard = setup_logging(&config.logging)?;

    log_system_event!(
        startup,
        component = "server",
        "Learning System server starting"
    );

    // Initialize database
    let db = Database::new(&config.database.url).await?;
    log_system_event!(
        startup,
        component = "database",
        "Database initialized successfully"
    );

    // Initialize services
    let card_service = CardService::new(db);

    let llm_service = LLMService::new_with_provider(
        config.llm.api_key.clone(),
        config.llm.base_url.clone(),
        config.llm.provider,
        config.llm.model.clone(),
    );

    log_system_event!(
        startup,
        component = "llm_service",
        format!(
            "LLM service initialized with provider: {:?}",
            config.llm.provider
        )
        .as_str()
    );

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
        .layer(ServiceBuilder::new().layer(CorsLayer::permissive()));

    // Start the server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    log_system_event!(
        startup,
        component = "server",
        format!("Server starting on {}", addr).as_str()
    );

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn serve_static_file(
    file_path: &str,
    content_type: &'static str,
) -> Result<(StatusCode, [(&'static str, &'static str); 1], String), StatusCode> {
    match fs::read_to_string(file_path).await {
        Ok(content) => Ok((StatusCode::OK, [("content-type", content_type)], content)),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

async fn serve_index() -> Result<Html<String>, StatusCode> {
    match serve_static_file("static/index.html", "text/html").await {
        Ok((_, _, content)) => Ok(Html(content)),
        Err(status_code) => Err(status_code),
    }
}

async fn serve_css() -> Result<(StatusCode, [(&'static str, &'static str); 1], String), StatusCode>
{
    serve_static_file("static/styles.css", "text/css").await
}

async fn serve_js() -> Result<(StatusCode, [(&'static str, &'static str); 1], String), StatusCode> {
    serve_static_file("static/app.js", "application/javascript").await
}

fn setup_logging(logging_config: &crate::config::LoggingConfig) -> Result<Option<WorkerGuard>> {
    use std::fs;
    use tracing_subscriber::fmt;

    // Create logs directory if file logging is enabled
    if logging_config.file_enabled {
        fs::create_dir_all(&logging_config.log_directory).unwrap_or_else(|e| {
            eprintln!(
                "Warning: Could not create logs directory '{}': {}",
                logging_config.log_directory, e
            );
        });
    }

    // Configure log level from configuration
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&logging_config.level));

    // Handle different combinations of console and file logging
    let guard = match (logging_config.console_enabled, logging_config.file_enabled) {
        (true, true) => {
            // Both console and file logging enabled
            let file_appender = tracing_appender::rolling::daily(
                &logging_config.log_directory,
                "learning-system.log",
            );
            let (non_blocking_file, guard) = tracing_appender::non_blocking(file_appender);

            let console_layer = fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .with_ansi(true);

            let file_layer = fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .with_ansi(false)
                .with_writer(non_blocking_file);

            tracing_subscriber::registry()
                .with(env_filter)
                .with(console_layer)
                .with(file_layer)
                .init();

            Some(guard)
        }
        (true, false) => {
            // Console logging only
            let console_layer = fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .with_ansi(true);

            tracing_subscriber::registry()
                .with(env_filter)
                .with(console_layer)
                .init();

            None
        }
        (false, true) => {
            // File logging only
            let file_appender = tracing_appender::rolling::daily(
                &logging_config.log_directory,
                "learning-system.log",
            );
            let (non_blocking_file, guard) = tracing_appender::non_blocking(file_appender);

            let file_layer = fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .with_ansi(false)
                .with_writer(non_blocking_file);

            tracing_subscriber::registry()
                .with(env_filter)
                .with(file_layer)
                .init();

            Some(guard)
        }
        (false, false) => {
            // No logging configured - use minimal console
            eprintln!(
                "Warning: Both console and file logging are disabled. Enabling minimal console logging."
            );
            let console_layer = fmt::layer().compact();

            tracing_subscriber::registry()
                .with(env_filter)
                .with(console_layer)
                .init();

            None
        }
    };

    if logging_config.file_enabled {
        log_system_event!(
            config,
            format!(
                "Logging initialized - writing to {}/learning-system.log with daily rotation",
                logging_config.log_directory
            )
            .as_str()
        );
    } else {
        log_system_event!(config, "Logging initialized - console output only");
    }

    Ok(guard)
}
