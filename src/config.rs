use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::env;
use tracing::{info, warn};

use crate::llm_providers::LLMProviderType;

// Import logging macros
use crate::{log_system_event, log_validation};

/// Complete application configuration loaded from environment variables
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub llm: LLMConfig,
    pub server: ServerConfig,
    pub logging: LoggingConfig,
}

/// Database connection configuration
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

/// Large Language Model service configuration
#[derive(Debug, Clone, Deserialize)]
pub struct LLMConfig {
    pub api_key: String,
    pub base_url: Option<String>,
    pub provider: LLMProviderType,
    pub model: Option<String>,
}

/// HTTP server configuration
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
}

/// Logging system configuration
#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_enabled: bool,
    pub console_enabled: bool,
    pub log_directory: String,
}

impl Config {
    /// Load configuration from environment variables with sensible defaults
    pub fn from_env() -> Result<Self> {
        log_system_event!(config, "Loading application configuration from environment variables");

        let database_config = DatabaseConfig::from_env()?;
        let llm_config = LLMConfig::from_env()?;
        let server_config = ServerConfig::from_env()?;
        let logging_config = LoggingConfig::from_env()?;

        let config = Config {
            database: database_config,
            llm: llm_config,
            server: server_config,
            logging: logging_config,
        };

        log_system_event!(config, "Configuration loaded successfully");
        config.log_configuration_summary();

        Ok(config)
    }

    /// Log a summary of loaded configuration (without sensitive data)
    fn log_configuration_summary(&self) {
        info!(
            database_url_masked = %mask_sensitive_data(&self.database.url),
            llm_provider = ?self.llm.provider,
            llm_model = ?self.llm.model,
            server_address = %format!("{}:{}", self.server.host, self.server.port),
            log_level = %self.logging.level,
            "Configuration summary"
        );
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<()> {
        // Validate database URL format
        if !self.database.url.contains("sqlite:") && !self.database.url.contains("postgres://") {
            return Err(anyhow!("DATABASE_URL must start with 'sqlite:' or 'postgres://'"));
        }

        // Validate server port range
        if self.server.port == 0 {
            return Err(anyhow!("Server port must be greater than 0"));
        }

        // Validate LLM API key presence
        if self.llm.api_key.is_empty() || self.llm.api_key == "your-api-key" {
            warn!("LLM API key appears to be placeholder or empty - LLM features may not work");
        }

        // Validate log level
        if !["trace", "debug", "info", "warn", "error"].contains(&self.logging.level.to_lowercase().as_str()) {
            warn!("Invalid log level '{}', using 'info' as fallback", self.logging.level);
        }

        log_validation!(success, "configuration", "Configuration validation completed successfully");
        Ok(())
    }
}

impl DatabaseConfig {
    fn from_env() -> Result<Self> {
        let url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:learning_system.db".to_string());

        Ok(DatabaseConfig { url })
    }
}

impl LLMConfig {
    fn from_env() -> Result<Self> {
        let api_key = env::var("LLM_API_KEY")
            .unwrap_or_else(|_| "your-api-key".to_string());
        
        let base_url = env::var("LLM_BASE_URL").ok();
        
        let provider_str = env::var("LLM_PROVIDER")
            .unwrap_or_else(|_| "openai".to_string());
        
        let provider = match provider_str.to_lowercase().as_str() {
            "gemini" | "google" => LLMProviderType::Gemini,
            "openai" | "chatgpt" | "gpt" => LLMProviderType::OpenAI,
            _ => {
                info!("Unknown LLM provider '{}', defaulting to OpenAI", provider_str);
                LLMProviderType::OpenAI
            }
        };
        
        let model = env::var("LLM_MODEL").ok();

        Ok(LLMConfig {
            api_key,
            base_url,
            provider,
            model,
        })
    }
}

impl ServerConfig {
    fn from_env() -> Result<Self> {
        let port_str = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string());
        
        let port = port_str.parse::<u16>()
            .map_err(|_| anyhow!("Invalid PORT value: '{}'. Must be a number between 1-65535", port_str))?;
        
        let host = env::var("HOST")
            .unwrap_or_else(|_| "0.0.0.0".to_string());

        Ok(ServerConfig { port, host })
    }
}

impl LoggingConfig {
    fn from_env() -> Result<Self> {
        let level = env::var("RUST_LOG")
            .unwrap_or_else(|_| "info,learning_system=debug".to_string());
        
        let file_enabled = env::var("LOG_FILE_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);
        
        let console_enabled = env::var("LOG_CONSOLE_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);
        
        let log_directory = env::var("LOG_DIRECTORY")
            .unwrap_or_else(|_| "logs".to_string());

        Ok(LoggingConfig {
            level,
            file_enabled,
            console_enabled,
            log_directory,
        })
    }
}

/// Mask sensitive data in configuration for safe logging
fn mask_sensitive_data(data: &str) -> String {
    if data.len() <= 8 {
        "*".repeat(data.len())
    } else {
        format!("{}***{}", &data[..4], &data[data.len()-4..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_mask_sensitive_data() {
        assert_eq!(mask_sensitive_data("short"), "*****");
        assert_eq!(mask_sensitive_data("sqlite:learning_system.db"), "sqli***m.db");
        assert_eq!(mask_sensitive_data("sk-1234567890abcdef"), "sk-1***cdef");
    }

    #[test] 
    fn test_database_config_defaults() {
        // Clear environment variable to test default
        unsafe { env::remove_var("DATABASE_URL"); }
        
        let config = DatabaseConfig::from_env().unwrap();
        assert_eq!(config.url, "sqlite:learning_system.db");
    }

    #[test]
    fn test_server_config_defaults() {
        // Clear environment variables to test defaults
        unsafe {
            env::remove_var("PORT");
            env::remove_var("HOST");
        }
        
        let config = ServerConfig::from_env().unwrap();
        assert_eq!(config.port, 3000);
        assert_eq!(config.host, "0.0.0.0");
    }

    #[test]
    fn test_llm_provider_parsing() {
        let test_cases = vec![
            ("openai", LLMProviderType::OpenAI),
            ("OpenAI", LLMProviderType::OpenAI),
            ("chatgpt", LLMProviderType::OpenAI),
            ("gpt", LLMProviderType::OpenAI),
            ("gemini", LLMProviderType::Gemini),
            ("Gemini", LLMProviderType::Gemini),
            ("google", LLMProviderType::Gemini),
            ("unknown", LLMProviderType::OpenAI), // defaults to OpenAI
        ];

        for (input, expected) in test_cases {
            unsafe { env::set_var("LLM_PROVIDER", input); }
            let config = LLMConfig::from_env().unwrap();
            assert_eq!(config.provider, expected, "Input '{}' should map to {:?}", input, expected);
        }
        
        unsafe { env::remove_var("LLM_PROVIDER"); }
    }

    #[test]
    fn test_config_validation() {
        // Test valid configuration
        let config = Config {
            database: DatabaseConfig {
                url: "sqlite:test.db".to_string(),
            },
            llm: LLMConfig {
                api_key: "sk-valid-key".to_string(),
                base_url: None,
                provider: LLMProviderType::OpenAI,
                model: None,
            },
            server: ServerConfig {
                port: 3000,
                host: "0.0.0.0".to_string(),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_enabled: true,
                console_enabled: true,
                log_directory: "logs".to_string(),
            },
        };

        assert!(config.validate().is_ok());

        // Test invalid port
        let mut invalid_config = config.clone();
        invalid_config.server.port = 0;
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_invalid_port_parsing() {
        unsafe { env::set_var("PORT", "not-a-number"); }
        let result = ServerConfig::from_env();
        assert!(result.is_err());
        
        unsafe { env::remove_var("PORT"); }
    }
}