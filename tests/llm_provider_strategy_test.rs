use learning_system::{LLMProviderType, LLMService};

#[tokio::test]
async fn test_llm_provider_strategy_pattern() {
    // Test OpenAI provider creation
    let openai_service =
        LLMService::new_with_provider("test-key".to_string(), None, LLMProviderType::OpenAI, None);

    // Verify the service is created successfully
    assert_eq!(openai_service.provider_name(), "OpenAI");

    // Test Gemini provider creation
    let gemini_service =
        LLMService::new_with_provider("test-key".to_string(), None, LLMProviderType::Gemini, None);

    // Verify the service is created successfully
    assert_eq!(gemini_service.provider_name(), "Gemini");

    println!("✅ LLM Provider Strategy Pattern refactoring successful");
    println!("✅ OpenAI provider: {}", openai_service.provider_name());
    println!("✅ Gemini provider: {}", gemini_service.provider_name());
}
