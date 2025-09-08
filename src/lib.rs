pub mod api;
pub mod card_service;
pub mod database;
mod efficiency_tests;
pub mod errors;
pub mod fsrs_scheduler;
pub mod llm_providers;
pub mod llm_service;
pub mod logging;
pub mod models;

#[cfg(test)]
mod tests {
    mod session_answer_test;
}

pub use card_service::CardService;
pub use database::Database;
pub use errors::*;
pub use fsrs_scheduler::FSRSScheduler;
pub use llm_providers::{LLMProvider, LLMProviderFactory, LLMProviderType, JsonResponseParser};

// Backward compatibility alias for tests - this should be removed in a future refactor
pub use llm_providers::LLMProviderType as LegacyLLMProvider;
pub use llm_service::LLMService;
pub use models::*;