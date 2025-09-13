use crate::api::ApiResponse;
use axum::{http::StatusCode, response::Json};
use tracing::{error, info, warn};

/// Centralized error types for consistent API error handling
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] anyhow::Error),

    #[error("LLM service error: {0}")]
    #[allow(dead_code)]
    LLMError(String),

    #[error("Resource already exists: {0}")]
    DuplicateResource(String),

    #[error("Bad request: {0}")]
    #[allow(dead_code)]
    BadRequest(String),

    #[error("Internal server error: {0}")]
    #[allow(dead_code)]
    InternalError(String),
}

/// Error context for structured logging
#[derive(Debug)]
pub struct ErrorContext {
    pub operation: String,
    pub resource_id: Option<String>,
    pub resource_type: String,
    pub user_friendly_message: Option<String>,
}

impl ErrorContext {
    pub fn new(operation: &str, resource_type: &str) -> Self {
        Self {
            operation: operation.to_string(),
            resource_id: None,
            resource_type: resource_type.to_string(),
            user_friendly_message: None,
        }
    }

    pub fn with_id(mut self, id: &str) -> Self {
        self.resource_id = Some(id.to_string());
        self
    }

    #[allow(dead_code)]
    pub fn with_user_message(mut self, message: &str) -> Self {
        self.user_friendly_message = Some(message.to_string());
        self
    }
}

impl ApiError {
    /// Convert API error to HTTP response with consistent structure and logging
    pub fn to_response_with_context(
        self,
        context: ErrorContext,
    ) -> (StatusCode, Json<ApiResponse<()>>) {
        match &self {
            ApiError::NotFound(_) => {
                info!(
                    operation = %context.operation,
                    resource_type = %context.resource_type,
                    resource_id = ?context.resource_id,
                    error = %self,
                    "Resource not found"
                );
                (
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::error(
                        context
                            .user_friendly_message
                            .unwrap_or_else(|| format!("{} not found", context.resource_type)),
                    )),
                )
            }
            ApiError::ValidationError(_) => {
                warn!(
                    operation = %context.operation,
                    resource_type = %context.resource_type,
                    resource_id = ?context.resource_id,
                    error = %self,
                    "Validation error"
                );
                (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::error(self.to_string())),
                )
            }
            ApiError::DuplicateResource(_) => {
                warn!(
                    operation = %context.operation,
                    resource_type = %context.resource_type,
                    resource_id = ?context.resource_id,
                    error = %self,
                    "Duplicate resource"
                );
                (
                    StatusCode::CONFLICT,
                    Json(ApiResponse::error(self.to_string())),
                )
            }
            ApiError::BadRequest(_) => {
                warn!(
                    operation = %context.operation,
                    resource_type = %context.resource_type,
                    resource_id = ?context.resource_id,
                    error = %self,
                    "Bad request"
                );
                (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::error(self.to_string())),
                )
            }
            ApiError::LLMError(_) => {
                error!(
                    operation = %context.operation,
                    resource_type = %context.resource_type,
                    resource_id = ?context.resource_id,
                    error = %self,
                    "LLM service error"
                );
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(ApiResponse::error(
                        "AI service temporarily unavailable. Please try again.".to_string(),
                    )),
                )
            }
            ApiError::DatabaseError(_) => {
                error!(
                    operation = %context.operation,
                    resource_type = %context.resource_type,
                    resource_id = ?context.resource_id,
                    error = %self,
                    "Database error"
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(
                        "Database operation failed. Please try again.".to_string(),
                    )),
                )
            }
            ApiError::InternalError(_) => {
                error!(
                    operation = %context.operation,
                    resource_type = %context.resource_type,
                    resource_id = ?context.resource_id,
                    error = %self,
                    "Internal server error"
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(
                        "An internal error occurred. Please try again.".to_string(),
                    )),
                )
            }
        }
    }

    /// Simple conversion without context (for backward compatibility)
    #[allow(dead_code)]
    pub fn to_response(self) -> (StatusCode, Json<ApiResponse<()>>) {
        let context = ErrorContext::new("unknown", "resource");
        self.to_response_with_context(context)
    }
}

/// Helper macro for structured error logging
#[macro_export]
macro_rules! api_error {
    (not_found, $operation:expr, $resource_type:expr, $id:expr) => {
        crate::errors::ApiError::NotFound(format!("{} with id '{}' not found", $resource_type, $id))
            .to_response_with_context(
                crate::errors::ErrorContext::new($operation, $resource_type).with_id($id),
            )
    };

    (validation, $operation:expr, $resource_type:expr, $message:expr) => {
        crate::errors::ApiError::ValidationError($message.to_string())
            .to_response_with_context(crate::errors::ErrorContext::new($operation, $resource_type))
    };

    (duplicate, $operation:expr, $resource_type:expr, $id:expr) => {
        crate::errors::ApiError::DuplicateResource(format!(
            "{} with id '{}' already exists",
            $resource_type, $id
        ))
        .to_response_with_context(
            crate::errors::ErrorContext::new($operation, $resource_type).with_id($id),
        )
    };

    (database, $operation:expr, $resource_type:expr, $error:expr) => {
        crate::errors::ApiError::DatabaseError($error)
            .to_response_with_context(crate::errors::ErrorContext::new($operation, $resource_type))
    };

    (llm, $operation:expr, $resource_type:expr, $error:expr) => {
        crate::errors::ApiError::LLMError($error.to_string())
            .to_response_with_context(crate::errors::ErrorContext::new($operation, $resource_type))
    };
}

/// Convert common anyhow errors to structured ApiErrors
impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        ApiError::DatabaseError(anyhow::Error::from(err))
    }
}

/// Helper function to detect error types from anyhow error messages
pub fn classify_database_error(error: &anyhow::Error) -> ApiError {
    let error_str = error.to_string().to_lowercase();

    if error_str.contains("already exists") || error_str.contains("unique constraint") {
        // Extract the relevant part of the error message
        if let Some(start) = error_str.find("'") {
            if let Some(end) = error_str[start + 1..].find("'") {
                let identifier = &error_str[start + 1..start + 1 + end];
                return ApiError::DuplicateResource(format!(
                    "Resource '{}' already exists",
                    identifier
                ));
            }
        }
        ApiError::DuplicateResource("Resource already exists".to_string())
    } else if error_str.contains("not found") || error_str.contains("no rows") {
        ApiError::NotFound("Resource not found".to_string())
    } else if error_str.contains("required") || error_str.contains("cannot be null") {
        ApiError::ValidationError("Required field is missing or invalid".to_string())
    } else {
        ApiError::DatabaseError(anyhow::anyhow!("{}", error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_context_creation() {
        let context = ErrorContext::new("create_card", "card")
            .with_id("123")
            .with_user_message("Custom message");

        assert_eq!(context.operation, "create_card");
        assert_eq!(context.resource_type, "card");
        assert_eq!(context.resource_id, Some("123".to_string()));
        assert_eq!(
            context.user_friendly_message,
            Some("Custom message".to_string())
        );
    }

    #[test]
    fn test_error_classification() {
        let duplicate_error = anyhow::anyhow!("UNIQUE constraint failed: cards.zettel_id");
        let classified = classify_database_error(&duplicate_error);
        assert!(matches!(classified, ApiError::DuplicateResource(_)));

        let not_found_error = anyhow::anyhow!("No rows returned");
        let classified = classify_database_error(&not_found_error);
        assert!(matches!(classified, ApiError::NotFound(_)));

        let validation_error = anyhow::anyhow!("Field cannot be null");
        let classified = classify_database_error(&validation_error);
        assert!(matches!(classified, ApiError::ValidationError(_)));
    }

    #[test]
    fn test_api_error_responses() {
        let error = ApiError::NotFound("Card not found".to_string());
        let context = ErrorContext::new("get_card", "card").with_id("123");
        let (status, _response) = error.to_response_with_context(context);

        assert_eq!(status, StatusCode::NOT_FOUND);
        // Note: We can't easily test the JSON content without deserializing,
        // but we can verify the status code mapping is correct

        let error = ApiError::ValidationError("Invalid data".to_string());
        let (status, _) = error.to_response();
        assert_eq!(status, StatusCode::BAD_REQUEST);

        let error = ApiError::DuplicateResource("Already exists".to_string());
        let (status, _) = error.to_response();
        assert_eq!(status, StatusCode::CONFLICT);
    }
}
