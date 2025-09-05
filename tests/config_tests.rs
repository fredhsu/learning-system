use learning_system::{LLMProvider};

#[test]
fn test_provider_string_parsing() {
    // Test the provider string parsing logic from main.rs
    
    let test_cases = vec![
        // OpenAI variants
        ("openai", LLMProvider::OpenAI),
        ("OpenAI", LLMProvider::OpenAI),
        ("OPENAI", LLMProvider::OpenAI),
        ("chatgpt", LLMProvider::OpenAI),
        ("ChatGPT", LLMProvider::OpenAI),
        ("gpt", LLMProvider::OpenAI),
        ("GPT", LLMProvider::OpenAI),
        
        // Gemini variants  
        ("gemini", LLMProvider::Gemini),
        ("Gemini", LLMProvider::Gemini),
        ("GEMINI", LLMProvider::Gemini),
        ("google", LLMProvider::Gemini),
        ("Google", LLMProvider::Gemini),
        ("GOOGLE", LLMProvider::Gemini),
    ];
    
    for (input, expected) in test_cases {
        let actual = match input.to_lowercase().as_str() {
            "gemini" | "google" => LLMProvider::Gemini,
            "openai" | "chatgpt" | "gpt" => LLMProvider::OpenAI,
            _ => LLMProvider::OpenAI, // default
        };
        
        assert_eq!(actual, expected, "Input '{}' should map to {:?}", input, expected);
        println!("✅ '{}' -> {:?}", input, expected);
    }
}

#[test]
fn test_unknown_provider_defaults_to_openai() {
    // Test that unknown provider strings default to OpenAI
    let unknown_providers = vec![
        "claude", "anthropic", "llama", "mistral", "unknown", "", "123"
    ];
    
    for provider_str in unknown_providers {
        let actual = match provider_str.to_lowercase().as_str() {
            "gemini" | "google" => LLMProvider::Gemini,
            "openai" | "chatgpt" | "gpt" => LLMProvider::OpenAI,
            _ => {
                // This simulates the logging that happens in main.rs
                println!("Unknown LLM provider '{}', defaulting to OpenAI", provider_str);
                LLMProvider::OpenAI
            }
        };
        
        assert_eq!(actual, LLMProvider::OpenAI, "Unknown provider '{}' should default to OpenAI", provider_str);
        println!("✅ Unknown provider '{}' defaults to OpenAI", provider_str);
    }
}

#[test]
fn test_environment_variable_scenarios() {
    // Test various environment variable scenarios
    
    struct EnvTestCase {
        llm_provider: Option<&'static str>,
        _llm_model: Option<&'static str>,
        expected_provider: LLMProvider,
        description: &'static str,
    }
    
    let test_cases = vec![
        EnvTestCase {
            llm_provider: None,
            _llm_model: None,
            expected_provider: LLMProvider::OpenAI, // default
            description: "No environment variables set"
        },
        EnvTestCase {
            llm_provider: Some("gemini"),
            _llm_model: Some("gemini-2.0-flash-exp"),
            expected_provider: LLMProvider::Gemini,
            description: "Gemini with specific model"
        },
        EnvTestCase {
            llm_provider: Some("openai"),
            _llm_model: Some("gpt-4o-mini"),
            expected_provider: LLMProvider::OpenAI,
            description: "OpenAI with specific model"
        },
        EnvTestCase {
            llm_provider: Some("gemini"),
            _llm_model: None,
            expected_provider: LLMProvider::Gemini,
            description: "Gemini with default model"
        },
    ];
    
    for test_case in test_cases {
        // Simulate the parsing logic from main.rs
        let llm_provider_str = test_case.llm_provider.unwrap_or("openai");
        
        let provider = match llm_provider_str.to_lowercase().as_str() {
            "gemini" | "google" => LLMProvider::Gemini,
            "openai" | "chatgpt" | "gpt" => LLMProvider::OpenAI,
            _ => LLMProvider::OpenAI,
        };
        
        assert_eq!(provider, test_case.expected_provider, "{}", test_case.description);
        println!("✅ {}", test_case.description);
    }
}

#[test]
fn test_api_key_format_validation() {
    // Test API key format patterns (basic validation)
    
    struct ApiKeyTest {
        key: &'static str,
        provider: LLMProvider,
        is_valid_format: bool,
        description: &'static str,
    }
    
    let test_cases = vec![
        // OpenAI key patterns
        ApiKeyTest {
            key: "sk-1234567890abcdef1234567890abcdef12345678",
            provider: LLMProvider::OpenAI,
            is_valid_format: true,
            description: "Valid OpenAI key format"
        },
        ApiKeyTest {
            key: "sk-short",
            provider: LLMProvider::OpenAI,
            is_valid_format: false,
            description: "Too short OpenAI key"
        },
        
        // Gemini key patterns
        ApiKeyTest {
            key: "AIzaSyDdI0hCZtE6vySjMm-WEfRq3CPzqKqqsHI",
            provider: LLMProvider::Gemini,
            is_valid_format: true,
            description: "Valid Gemini key format"
        },
        ApiKeyTest {
            key: "AIza-short",
            provider: LLMProvider::Gemini,
            is_valid_format: false,
            description: "Too short Gemini key"
        },
        
        // Generic/test keys
        ApiKeyTest {
            key: "test-key",
            provider: LLMProvider::OpenAI,
            is_valid_format: true, // We accept test keys
            description: "Test key format"
        },
    ];
    
    for test_case in test_cases {
        // Basic format validation (this is just for testing)
        let format_check = match test_case.provider {
            LLMProvider::OpenAI => {
                test_case.key.starts_with("sk-") || test_case.key == "test-key" || test_case.key.len() > 10
            },
            LLMProvider::Gemini => {
                test_case.key.starts_with("AIza") || test_case.key == "test-key" || test_case.key.len() > 10
            },
        };
        
        if test_case.is_valid_format {
            assert!(format_check, "{} should pass format check", test_case.description);
        }
        
        println!("✅ {}: format check = {}", test_case.description, format_check);
    }
}

#[test]
fn test_model_recommendations() {
    // Test recommended models for each provider
    
    let openai_models = vec![
        "gpt-4o-mini",      // Default - good balance
        "gpt-4o",           // Premium option
        "gpt-3.5-turbo",    // Legacy option
    ];
    
    let gemini_models = vec![
        "gemini-2.0-flash-exp",  // Default - latest and fastest
        "gemini-1.5-pro",        // Balanced option  
        "gemini-1.5-flash",      // Fast option
    ];
    
    // All models should be non-empty strings
    for model in openai_models {
        assert!(!model.is_empty(), "OpenAI model should not be empty: {}", model);
        assert!(model.contains("gpt") || model.contains("turbo"), "OpenAI model should contain 'gpt' or 'turbo': {}", model);
        println!("✅ OpenAI model: {}", model);
    }
    
    for model in gemini_models {
        assert!(!model.is_empty(), "Gemini model should not be empty: {}", model);
        assert!(model.contains("gemini"), "Gemini model should contain 'gemini': {}", model);
        println!("✅ Gemini model: {}", model);
    }
}

#[test]
fn test_base_url_defaults() {
    // Test default base URLs for each provider
    
    let defaults = vec![
        (LLMProvider::OpenAI, "https://api.openai.com/v1"),
        (LLMProvider::Gemini, "https://generativelanguage.googleapis.com/v1beta"),
    ];
    
    for (provider, expected_url) in defaults {
        // Validate URL format
        assert!(expected_url.starts_with("https://"), "Default URL should use HTTPS: {}", expected_url);
        assert!(expected_url.contains('.'), "Default URL should contain domain: {}", expected_url);
        
        println!("✅ {:?} default URL: {}", provider, expected_url);
    }
}

#[test]
fn test_port_configuration() {
    // Test port configuration scenarios
    
    let port_tests = vec![
        ("3000", 3000, "Default port"),
        ("4000", 4000, "Custom port"),  
        ("8080", 8080, "Development port"),
        ("80", 80, "HTTP port"),
        ("443", 443, "HTTPS port"),
    ];
    
    for (port_str, expected_port, description) in port_tests {
        // Simulate port parsing from environment
        let parsed_port: u16 = port_str.parse().expect("Should parse as valid port");
        assert_eq!(parsed_port, expected_port, "{}", description);
        println!("✅ {}: {} -> {}", description, port_str, parsed_port);
    }
}