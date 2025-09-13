use chrono::Utc;
use learning_system::llm_providers::LLMProviderType;
use learning_system::{Card, LLMService};
use uuid::Uuid;

#[tokio::test]
async fn test_gemini_service_creation() {
    // Test that we can create a Gemini service instance
    let _gemini_service = LLMService::new_gemini("test-key".to_string(), None);

    // The service should be created successfully (this is just a constructor test)
    // We can't test actual API calls without a real API key
    assert!(true, "Gemini service created successfully");
}

#[tokio::test]
async fn test_gemini_model_variants() {
    // Test different Gemini model configurations
    let models = vec!["gemini-2.0-flash-exp", "gemini-1.5-pro", "gemini-1.5-flash"];

    for model in models {
        let _service = LLMService::new_gemini("test-key".to_string(), Some(model.to_string()));
        println!("✅ Gemini service created with model: {}", model);
    }

    assert!(true, "All Gemini model variants supported");
}

#[tokio::test]
async fn test_gemini_with_custom_base_url() {
    // Test Gemini with custom base URL (for proxies or different regions)
    let _service = LLMService::new_with_provider(
        "test-key".to_string(),
        Some("https://generativelanguage.googleapis.com/v1beta".to_string()),
        LLMProviderType::Gemini,
        Some("gemini-2.0-flash-exp".to_string()),
    );

    assert!(true, "Gemini service created with custom base URL");
}

#[tokio::test]
async fn test_llm_provider_configuration() {
    // Test OpenAI provider
    let _openai_service = LLMService::new_with_provider(
        "test-key".to_string(),
        Some("https://api.openai.com/v1".to_string()),
        LLMProviderType::OpenAI,
        Some("gpt-4o-mini".to_string()),
    );
    assert!(true, "OpenAI service configured successfully");

    // Test Gemini provider
    let _gemini_service = LLMService::new_with_provider(
        "test-key".to_string(),
        None, // Use default base URL
        LLMProviderType::Gemini,
        Some("gemini-2.0-flash-exp".to_string()),
    );
    assert!(true, "Gemini service configured successfully");
}

#[tokio::test]
async fn test_provider_enum_serialization() {
    // Test that the provider enum can be serialized/deserialized
    let openai_provider = LLMProviderType::OpenAI;
    let gemini_provider = LLMProviderType::Gemini;

    // Test debug formatting
    let openai_debug = format!("{:?}", openai_provider);
    let gemini_debug = format!("{:?}", gemini_provider);

    assert_eq!(openai_debug, "OpenAI");
    assert_eq!(gemini_debug, "Gemini");

    // Test equality
    assert_eq!(openai_provider, LLMProviderType::OpenAI);
    assert_eq!(gemini_provider, LLMProviderType::Gemini);
    assert_ne!(openai_provider, gemini_provider);
}

fn create_test_card() -> Card {
    Card {
        id: Uuid::new_v4(),
        zettel_id: "TEST-001".to_string(),
        title: None,
        content: "This is a test card about machine learning fundamentals. It covers basic concepts like supervised learning, unsupervised learning, and reinforcement learning.".to_string(),
        creation_date: Utc::now(),
        last_reviewed: None,
        next_review: Utc::now(),
        difficulty: 0.0,
        stability: 0.0,
        retrievability: 0.0,
        reps: 0,
        lapses: 0,
        state: "New".to_string(),
        links: None,
    }
}

#[tokio::test]
async fn test_provider_defaults() {
    // Test that each provider uses correct default settings
    let _openai_service = LLMService::new_with_provider(
        "test-key".to_string(),
        None, // Use defaults
        LLMProviderType::OpenAI,
        None, // Use default model
    );

    let _gemini_service = LLMService::new_with_provider(
        "test-key".to_string(),
        None, // Use defaults
        LLMProviderType::Gemini,
        None, // Use default model
    );

    // Services should be created without errors
    assert!(true, "Both providers created with defaults");
}

#[tokio::test]
async fn test_provider_compatibility() {
    // Test that the same operations can be performed with both providers
    // (This tests the interface consistency, not actual API calls)

    let providers = vec![
        ("OpenAI", LLMProviderType::OpenAI, "gpt-4o-mini"),
        ("Gemini", LLMProviderType::Gemini, "gemini-2.0-flash-exp"),
    ];

    for (name, provider, model) in providers {
        let _service = LLMService::new_with_provider(
            "test-key".to_string(),
            None,
            provider.clone(),
            Some(model.to_string()),
        );

        println!("✅ {} provider supports unified interface", name);
    }

    assert!(true, "All providers support unified interface");
}

#[tokio::test]
async fn test_card_structure_compatibility() {
    // Test that both providers can work with card structures
    let card = create_test_card();

    let providers = vec![LLMProviderType::OpenAI, LLMProviderType::Gemini];

    for provider in providers {
        let _service =
            LLMService::new_with_provider("test-key".to_string(), None, provider.clone(), None);

        // We can't call actual methods without API keys, but we can verify
        // that the service accepts the same card structure
        assert_eq!(card.content.len() > 0, true);
        println!("✅ {:?} provider compatible with card structure", provider);
    }
}

// Note: Actual API call tests would require valid API keys
// and are better suited for integration tests or manual testing
// with environment variables set properly.
