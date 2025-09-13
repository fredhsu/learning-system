// Macros file - tracing macros are imported within the macro definitions

/// Standardized logging macros for consistent field names and message patterns across the application
///
/// These macros ensure:
/// - Consistent field naming conventions
/// - Appropriate logging levels for different scenarios
/// - Structured logging with context
/// - Consistent message formatting

// ============================================================================
// API Operation Logging Macros
// ============================================================================

/// Log the start of an API operation with consistent fields
#[macro_export]
macro_rules! log_api_start {
    ($operation:expr, card_id = $card_id:expr) => {
        tracing::debug!(
            operation = $operation,
            card_id = %$card_id,
            "API operation started"
        );
    };
    ($operation:expr, zettel_id = $zettel_id:expr) => {
        tracing::debug!(
            operation = $operation,
            zettel_id = %$zettel_id,
            "API operation started"
        );
    };
    ($operation:expr, session_id = $session_id:expr) => {
        tracing::debug!(
            operation = $operation,
            session_id = %$session_id,
            "API operation started"
        );
    };
    ($operation:expr) => {
        tracing::debug!(
            operation = $operation,
            "API operation started"
        );
    };
}

/// Log successful completion of an API operation
#[macro_export]
macro_rules! log_api_success {
    ($operation:expr, card_id = $card_id:expr, $msg:expr) => {
        tracing::info!(
            operation = $operation,
            card_id = %$card_id,
            "API operation completed: {}", $msg
        );
    };
    ($operation:expr, zettel_id = $zettel_id:expr, $msg:expr) => {
        tracing::info!(
            operation = $operation,
            zettel_id = %$zettel_id,
            "API operation completed: {}", $msg
        );
    };
    ($operation:expr, session_id = $session_id:expr, $msg:expr) => {
        tracing::info!(
            operation = $operation,
            session_id = %$session_id,
            "API operation completed: {}", $msg
        );
    };
    ($operation:expr, count = $count:expr, $msg:expr) => {
        tracing::info!(
            operation = $operation,
            count = $count,
            "API operation completed: {}", $msg
        );
    };
    ($operation:expr, $msg:expr) => {
        tracing::info!(
            operation = $operation,
            "API operation completed: {}", $msg
        );
    };
}

/// Log API operation errors with consistent structure
#[macro_export]
macro_rules! log_api_error {
    ($operation:expr, card_id = $card_id:expr, error = $error:expr, $msg:expr) => {
        tracing::error!(
            operation = $operation,
            card_id = %$card_id,
            error = %$error,
            "API operation failed: {}", $msg
        );
    };
    ($operation:expr, zettel_id = $zettel_id:expr, error = $error:expr, $msg:expr) => {
        tracing::error!(
            operation = $operation,
            zettel_id = %$zettel_id,
            error = %$error,
            "API operation failed: {}", $msg
        );
    };
    ($operation:expr, session_id = $session_id:expr, error = $error:expr, $msg:expr) => {
        tracing::error!(
            operation = $operation,
            session_id = %$session_id,
            error = %$error,
            "API operation failed: {}", $msg
        );
    };
    ($operation:expr, error = $error:expr, $msg:expr) => {
        tracing::error!(
            operation = $operation,
            error = %$error,
            "API operation failed: {}", $msg
        );
    };
}

/// Log API warnings with context
#[macro_export]
macro_rules! log_api_warn {
    ($operation:expr, card_id = $card_id:expr, $msg:expr) => {
        tracing::warn!(
            operation = $operation,
            card_id = %$card_id,
            "API operation warning: {}", $msg
        );
    };
    ($operation:expr, session_id = $session_id:expr, $msg:expr) => {
        tracing::warn!(
            operation = $operation,
            session_id = %$session_id,
            "API operation warning: {}", $msg
        );
    };
    ($operation:expr, $msg:expr) => {
        tracing::warn!(
            operation = $operation,
            "API operation warning: {}", $msg
        );
    };
}

// ============================================================================
// Service Layer Logging Macros
// ============================================================================

/// Log service operation start with context
#[macro_export]
macro_rules! log_service_start {
    ($service:expr, $operation:expr, card_count = $count:expr) => {
        tracing::info!(
            service = $service,
            operation = $operation,
            card_count = $count,
            "Service operation started"
        );
    };
    ($service:expr, $operation:expr, card_id = $card_id:expr) => {
        tracing::info!(
            service = $service,
            operation = $operation,
            card_id = %$card_id,
            "Service operation started"
        );
    };
    ($service:expr, $operation:expr) => {
        tracing::info!(
            service = $service,
            operation = $operation,
            "Service operation started"
        );
    };
}

/// Log service operation success
#[macro_export]
macro_rules! log_service_success {
    ($service:expr, $operation:expr, card_count = $count:expr, duration_ms = $duration:expr) => {
        tracing::info!(
            service = $service,
            operation = $operation,
            card_count = $count,
            duration_ms = $duration,
            "Service operation completed successfully"
        );
    };
    ($service:expr, $operation:expr, card_id = $card_id:expr, duration_ms = $duration:expr) => {
        tracing::info!(
            service = $service,
            operation = $operation,
            card_id = %$card_id,
            duration_ms = $duration,
            "Service operation completed successfully"
        );
    };
    ($service:expr, $operation:expr, $msg:expr) => {
        tracing::info!(
            service = $service,
            operation = $operation,
            "Service operation completed: {}", $msg
        );
    };
}

/// Log service operation errors
#[macro_export]
macro_rules! log_service_error {
    ($service:expr, $operation:expr, card_id = $card_id:expr, error = $error:expr) => {
        tracing::error!(
            service = $service,
            operation = $operation,
            card_id = %$card_id,
            error = %$error,
            "Service operation failed"
        );
    };
    ($service:expr, $operation:expr, error = $error:expr) => {
        tracing::error!(
            service = $service,
            operation = $operation,
            error = %$error,
            "Service operation failed"
        );
    };
}

/// Log service warnings
#[macro_export]
macro_rules! log_service_warn {
    ($service:expr, $operation:expr, $msg:expr) => {
        tracing::warn!(
            service = $service,
            operation = $operation,
            "Service warning: {}",
            $msg
        );
    };
}

// ============================================================================
// Database Operation Logging Macros
// ============================================================================

/// Log database operation performance and results
#[macro_export]
macro_rules! log_db_operation {
    (debug, $operation:expr, card_id = $card_id:expr, duration_ms = $duration:expr) => {
        tracing::debug!(
            component = "database",
            operation = $operation,
            card_id = %$card_id,
            duration_ms = $duration,
            "Database operation completed"
        );
    };
    (debug, $operation:expr, count = $count:expr, duration_ms = $duration:expr) => {
        tracing::debug!(
            component = "database",
            operation = $operation,
            result_count = $count,
            duration_ms = $duration,
            "Database operation completed"
        );
    };
    (info, $operation:expr, $msg:expr) => {
        tracing::info!(
            component = "database",
            operation = $operation,
            "Database operation: {}", $msg
        );
    };
    (error, $operation:expr, error = $error:expr) => {
        tracing::error!(
            component = "database",
            operation = $operation,
            error = %$error,
            "Database operation failed"
        );
    };
}

// ============================================================================
// LLM Service Logging Macros
// ============================================================================

/// Log LLM service operations with provider context
#[macro_export]
macro_rules! log_llm_operation {
    (start, $operation:expr, provider = $provider:expr, card_count = $count:expr) => {
        tracing::info!(
            component = "llm_service",
            operation = $operation,
            provider = %$provider,
            card_count = $count,
            "LLM operation started"
        );
    };
    (success, $operation:expr, provider = $provider:expr, duration_ms = $duration:expr, tokens = $tokens:expr) => {
        tracing::info!(
            component = "llm_service",
            operation = $operation,
            provider = %$provider,
            duration_ms = $duration,
            tokens_used = $tokens,
            "LLM operation completed successfully"
        );
    };
    (error, $operation:expr, provider = $provider:expr, error = $error:expr, retry_count = $retry:expr) => {
        tracing::error!(
            component = "llm_service",
            operation = $operation,
            provider = %$provider,
            error = %$error,
            retry_count = $retry,
            "LLM operation failed"
        );
    };
    (warn, $operation:expr, $msg:expr) => {
        tracing::warn!(
            component = "llm_service",
            operation = $operation,
            "LLM operation warning: {}", $msg
        );
    };
}

// ============================================================================
// System Event Logging Macros
// ============================================================================

/// Log system startup and shutdown events
#[macro_export]
macro_rules! log_system_event {
    (startup, component = $component:expr, $msg:expr) => {
        tracing::info!(
            event_type = "startup",
            component = $component,
            "System event: {}",
            $msg
        );
    };
    (shutdown, component = $component:expr, $msg:expr) => {
        tracing::info!(
            event_type = "shutdown",
            component = $component,
            "System event: {}",
            $msg
        );
    };
    (config, $msg:expr) => {
        tracing::info!(event_type = "configuration", "System event: {}", $msg);
    };
}

// ============================================================================
// Performance Logging Macros
// ============================================================================

/// Log performance metrics with consistent structure
#[macro_export]
macro_rules! log_performance {
    ($operation:expr, duration_ms = $duration:expr, throughput = $throughput:expr) => {
        tracing::debug!(
            event_type = "performance",
            operation = $operation,
            duration_ms = $duration,
            throughput_ops_per_sec = $throughput,
            "Performance metrics"
        );
    };
    ($operation:expr, duration_ms = $duration:expr) => {
        tracing::debug!(
            event_type = "performance",
            operation = $operation,
            duration_ms = $duration,
            "Performance metrics"
        );
    };
}

// ============================================================================
// Validation Logging Macros
// ============================================================================

/// Log validation results consistently
#[macro_export]
macro_rules! log_validation {
    (success, $component:expr, $msg:expr) => {
        tracing::debug!(
            event_type = "validation",
            component = $component,
            result = "success",
            "Validation completed: {}", $msg
        );
    };
    (failure, $component:expr, error = $error:expr) => {
        tracing::warn!(
            event_type = "validation",
            component = $component,
            result = "failure",
            error = %$error,
            "Validation failed"
        );
    };
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    #[test]
    fn test_logging_macros_compile() {
        let card_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let _error = anyhow::anyhow!("test error");

        // Test that all macro variants compile successfully
        log_api_start!("test_operation", card_id = card_id);
        log_api_start!("test_operation", zettel_id = "TEST-001");
        log_api_start!("test_operation", session_id = session_id);
        log_api_start!("test_operation");

        log_api_success!("test_operation", card_id = card_id, "operation completed");
        log_api_success!("test_operation", count = 5, "cards processed");

        log_api_warn!("test_operation", card_id = card_id, "operation warning");

        log_service_start!("card_service", "create_card", card_id = card_id);
        log_service_success!("card_service", "create_card", "card created successfully");

        log_db_operation!(debug, "select_card", card_id = card_id, duration_ms = 10);
        log_db_operation!(info, "migration", "database initialized");

        log_llm_operation!(
            start,
            "generate_questions",
            provider = "openai",
            card_count = 5
        );
        log_llm_operation!(
            success,
            "generate_questions",
            provider = "openai",
            duration_ms = 1500,
            tokens = 1000
        );

        log_system_event!(startup, component = "server", "server starting");
        log_system_event!(config, "configuration loaded successfully");

        log_performance!("batch_processing", duration_ms = 2500, throughput = 100);
        log_performance!("single_operation", duration_ms = 50);

        log_validation!(success, "api_request", "request validated");
    }
}
