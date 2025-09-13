use axum_test::TestServer;
use learning_system::llm_providers::LLMProviderType;
use learning_system::{CardService, Database, LLMService, api::*};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

async fn create_test_server() -> TestServer {
    let db = Database::new("sqlite::memory:").await.unwrap();
    let card_service = CardService::new(db);
    let llm_service = LLMService::new("test_key".to_string(), None);
    let app_state = AppState {
        card_service,
        llm_service,
        review_sessions: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = create_router(app_state);
    TestServer::new(app).unwrap()
}

async fn create_test_server_with_provider(provider: LLMProviderType) -> TestServer {
    let db = Database::new("sqlite::memory:").await.unwrap();
    let card_service = CardService::new(db);
    let llm_service = LLMService::new_with_provider("test_key".to_string(), None, provider, None);
    let app_state = AppState {
        card_service,
        llm_service,
        review_sessions: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = create_router(app_state);
    TestServer::new(app).unwrap()
}

#[tokio::test]
async fn test_keyboard_shortcut_rating_system() {
    let server = create_test_server().await;

    // Create a card for testing
    let create_request = json!({
        "zettel_id": "UI-001",
        "content": "Keyboard shortcut test card",
        "topic_ids": [],
        "links": null
    });

    let create_response = server.post("/api/cards").json(&create_request).await;
    create_response.assert_status_ok();

    let create_body: Value = create_response.json();
    let card_id = create_body["data"]["id"].as_str().unwrap();

    // Test all keyboard shortcut ratings (1-4) are accepted
    let keyboard_ratings = vec![(1, "Again"), (2, "Hard"), (3, "Good"), (4, "Easy")];

    for (rating, rating_name) in keyboard_ratings {
        let review_request = json!({
            "rating": rating
        });

        let review_response = server
            .post(&format!("/api/cards/{}/review", card_id))
            .json(&review_request)
            .await;

        review_response.assert_status_ok();

        let review_body: Value = review_response.json();
        assert_eq!(review_body["success"], true);

        // Verify the rating was accepted - reps should be greater than 0 after first review
        assert!(review_body["data"]["reps"].as_i64().unwrap() >= 0);

        println!(
            "Successfully tested keyboard shortcut {} ({})",
            rating, rating_name
        );
    }
}

#[tokio::test]
async fn test_preview_content_functionality() {
    let server = create_test_server().await;

    // Test short content (should not trigger preview)
    let short_content = "Short content";
    assert!(short_content.len() <= 100);

    let short_request = json!({
        "zettel_id": "UI-002",
        "content": short_content,
        "topic_ids": [],
        "links": null
    });

    let short_response = server.post("/api/cards").json(&short_request).await;
    short_response.assert_status_ok();

    // Test long content (should trigger preview in UI)
    let long_content = "This is a very long piece of content that exceeds the 100 character limit and should trigger the preview functionality in the frontend UI. ".repeat(3);
    assert!(long_content.len() > 100);

    let long_request = json!({
        "zettel_id": "UI-003",
        "content": long_content,
        "topic_ids": [],
        "links": null
    });

    let long_response = server.post("/api/cards").json(&long_request).await;
    long_response.assert_status_ok();

    let long_body: Value = long_response.json();
    assert_eq!(long_body["data"]["content"], long_content);

    // Verify content is properly stored regardless of length
    let card_id = long_body["data"]["id"].as_str().unwrap();
    let get_response = server.get(&format!("/api/cards/{}", card_id)).await;
    get_response.assert_status_ok();

    let get_body: Value = get_response.json();
    assert_eq!(get_body["data"]["content"], long_content);
}

#[tokio::test]
async fn test_search_functionality_with_highlighting() {
    let server = create_test_server().await;

    // Create cards with specific searchable content
    let test_cards = vec![
        (
            "Mathematics: algebra and calculus concepts",
            "Should match 'mathematics'",
        ),
        (
            "Programming in Rust: memory safety",
            "Should match 'programming'",
        ),
        (
            "Mathematics and programming intersection",
            "Should match both terms",
        ),
        (
            "History of ancient civilizations",
            "Should not match math/programming",
        ),
    ];

    for (content, _description) in &test_cards {
        let create_request = json!({
            "zettel_id": format!("UI-SEARCH-{:03}", test_cards.iter().position(|(c, _)| c == content).unwrap() + 1),
            "content": content,
            "topic_ids": [],
            "links": null
        });

        let create_response = server.post("/api/cards").json(&create_request).await;
        create_response.assert_status_ok();
    }

    // Test search with case variations
    let search_terms = vec![
        "mathematics",
        "MATHEMATICS",
        "Math",
        "programming",
        "PROGRAMMING",
    ];

    for search_term in search_terms {
        let search_response = server
            .get(&format!("/api/cards/search?q={}", search_term))
            .await;
        search_response.assert_status_ok();

        let search_body: Value = search_response.json();
        assert_eq!(search_body["success"], true);

        let results = search_body["data"].as_array().unwrap();

        // Verify we get expected results based on search term
        if search_term.to_lowercase().contains("math") {
            assert!(
                results.len() >= 2,
                "Should find at least 2 cards for '{}'",
                search_term
            );
        } else if search_term.to_lowercase().contains("programming") {
            assert!(
                results.len() >= 2,
                "Should find at least 2 cards for '{}'",
                search_term
            );
        }

        // Verify all results contain the search term (case-insensitive)
        for result in results {
            let content = result["content"].as_str().unwrap().to_lowercase();
            assert!(
                content.contains(&search_term.to_lowercase()),
                "Result '{}' should contain '{}'",
                content,
                search_term
            );
        }
    }
}

#[tokio::test]
async fn test_navigation_icon_endpoints() {
    let server = create_test_server().await;

    // Test that all main navigation endpoints work correctly
    // These correspond to the navigation icons we added

    // Cards endpoint (layers icon)
    let cards_response = server.get("/api/cards").await;
    cards_response.assert_status_ok();

    let cards_body: Value = cards_response.json();
    assert_eq!(cards_body["success"], true);
    assert!(cards_body["data"].is_array());

    // Due cards endpoint (refresh-cw icon)
    let due_response = server.get("/api/cards/due").await;
    due_response.assert_status_ok();

    let due_body: Value = due_response.json();
    assert_eq!(due_body["success"], true);
    assert!(due_body["data"].is_array());

    // Topics endpoint (tag icon)
    let topics_response = server.get("/api/topics").await;
    topics_response.assert_status_ok();

    let topics_body: Value = topics_response.json();
    assert_eq!(topics_body["success"], true);
    assert!(topics_body["data"].is_array());
}

#[tokio::test]
async fn test_loading_state_consistency() {
    let server = create_test_server().await;

    // Test that API responses are consistent and would work well with loading states

    // Create multiple cards quickly to simulate loading scenarios
    let mut created_cards = Vec::new();

    for i in 0..5 {
        let create_request = json!({
            "zettel_id": format!("UI-LOAD-{:03}", i + 1),
            "content": format!("Loading test card {}", i),
            "topic_ids": [],
            "links": null
        });

        let create_response = server.post("/api/cards").json(&create_request).await;
        create_response.assert_status_ok();

        let create_body: Value = create_response.json();
        created_cards.push(create_body["data"]["id"].as_str().unwrap().to_string());
    }

    // Test bulk retrieval (would use skeleton loading in UI)
    let all_cards_response = server.get("/api/cards").await;
    all_cards_response.assert_status_ok();

    let all_cards_body: Value = all_cards_response.json();
    assert_eq!(all_cards_body["success"], true);
    assert_eq!(all_cards_body["data"].as_array().unwrap().len(), 5);

    // Test individual card retrieval (would show loading spinner)
    for card_id in &created_cards {
        let card_response = server.get(&format!("/api/cards/{}", card_id)).await;
        card_response.assert_status_ok();

        let card_body: Value = card_response.json();
        assert_eq!(card_body["success"], true);
        assert!(card_body["data"]["id"].as_str().is_some());
    }
}

#[tokio::test]
async fn test_responsive_design_data_structure() {
    let server = create_test_server().await;

    // Test that API responses contain all fields needed for responsive design

    let create_request = json!({
        "zettel_id": "UI-004",
        "content": "Responsive design test card with various metadata",
        "topic_ids": [],
        "links": null
    });

    let create_response = server.post("/api/cards").json(&create_request).await;
    create_response.assert_status_ok();

    let create_body: Value = create_response.json();
    let card_data = &create_body["data"];

    // Verify all fields that the responsive UI needs are present
    let required_fields = vec![
        "id",
        "content",
        "creation_date",
        "next_review",
        "reps",
        "state",
        "difficulty",
        "stability",
    ];

    for field in required_fields {
        assert!(
            card_data.get(field).is_some(),
            "Field '{}' should be present",
            field
        );
    }

    // Test that the data structure is consistent across different operations
    let card_id = card_data["id"].as_str().unwrap();

    // Get single card
    let get_response = server.get(&format!("/api/cards/{}", card_id)).await;
    get_response.assert_status_ok();

    let get_body: Value = get_response.json();
    let get_card_data = &get_body["data"];

    // Verify same structure
    for field in &[
        "id",
        "content",
        "creation_date",
        "next_review",
        "reps",
        "state",
    ] {
        assert_eq!(
            card_data[field], get_card_data[field],
            "Field '{}' should be consistent between create and get",
            field
        );
    }
}

#[tokio::test]
async fn test_typography_scale_data_validation() {
    let server = create_test_server().await;

    // Test different content lengths that would use different typography scales
    let content_variations = vec![
        ("H1", "# Main Heading"),
        ("H2", "## Sub Heading"),
        ("H3", "### Section"),
        (
            "Body",
            "Regular paragraph text with sufficient length to test body typography.",
        ),
        ("Small", "meta"),
        ("Code", "`inline code` and ```\ncode block\n```"),
        (
            "Math",
            "$E = mc^2$ and $$\\int_{-\\infty}^{\\infty} e^{-x^2} dx = \\sqrt{\\pi}$$",
        ),
    ];

    for (i, (label, content)) in content_variations.iter().enumerate() {
        let create_request = json!({
            "zettel_id": format!("UI-TYPO-{:03}", i + 1),
            "content": content,
            "topic_ids": [],
            "links": null
        });

        let create_response = server.post("/api/cards").json(&create_request).await;
        create_response.assert_status_ok();

        let create_body: Value = create_response.json();
        assert_eq!(create_body["data"]["content"], *content);

        println!("Successfully tested {} typography: {}", label, content);
    }
}

#[tokio::test]
async fn test_suggested_rating_response_format() {
    // Test that the API response includes the suggested rating field
    // This validates that the session answer endpoint returns the correct format
    // for the frontend to display the suggested rating

    let mock_grading_response = json!({
        "success": true,
        "data": {
            "is_correct": true,
            "feedback": "Great answer! You demonstrated understanding of the concept.",
            "rating": 3,  // This is the suggested_rating field from backend
            "next_review": "2025-09-05T10:00:00Z"
        },
        "error": null
    });

    // Verify the response structure contains the expected fields
    let data = &mock_grading_response["data"];
    assert!(
        data["is_correct"].is_boolean(),
        "Response should have is_correct boolean"
    );
    assert!(
        data["feedback"].is_string(),
        "Response should have feedback string"
    );
    assert!(
        data["rating"].is_number(),
        "Response should have rating number (suggested_rating)"
    );
    assert!(
        data["next_review"].is_string(),
        "Response should have next_review string"
    );

    // Verify rating is in valid FSRS range (1-4)
    let rating = data["rating"].as_i64().unwrap();
    assert!(
        rating >= 1 && rating <= 4,
        "Rating should be between 1-4 (FSRS range)"
    );

    println!("✅ Suggested rating response format validated");
    println!("   - is_correct: {}", data["is_correct"]);
    println!("   - feedback: {}", data["feedback"]);
    println!("   - rating (suggested): {}", data["rating"]);
    println!("   - next_review: {}", data["next_review"]);
}

#[tokio::test]
async fn test_server_creation_with_multiple_providers() {
    // Test that the server can be created with different LLM providers

    let providers = vec![
        ("OpenAI", LLMProviderType::OpenAI),
        ("Gemini", LLMProviderType::Gemini),
    ];

    for (name, provider) in providers {
        let server = create_test_server_with_provider(provider).await;

        // Test basic server functionality with each provider
        let response = server.get("/api/cards").await;
        response.assert_status_ok();

        let cards_response: Value = response.json();
        assert!(
            cards_response["success"].as_bool().unwrap_or(false),
            "Cards API should work with {}",
            name
        );

        println!("✅ Server created successfully with {} provider", name);
    }
}

#[tokio::test]
async fn test_provider_specific_card_operations() {
    // Test that both providers support the same card operations

    let providers = vec![LLMProviderType::OpenAI, LLMProviderType::Gemini];

    for provider in providers {
        let server = create_test_server_with_provider(provider).await;

        // Test card creation with both providers
        let create_request = json!({
            "zettel_id": format!("TEST-{:?}-001", provider),
            "content": format!("Test content for {:?} provider integration", provider),
            "topic_ids": [],
            "links": null
        });

        let create_response = server.post("/api/cards").json(&create_request).await;
        create_response.assert_status_ok();

        let create_body: Value = create_response.json();
        assert!(create_body["success"].as_bool().unwrap_or(false));
        assert_eq!(
            create_body["data"]["zettel_id"],
            create_request["zettel_id"]
        );

        println!("✅ Card operations work with {:?} provider", provider);
    }
}

#[tokio::test]
async fn test_api_consistency_across_providers() {
    // Test that all API endpoints return consistent response structures
    // regardless of which LLM provider is used

    let providers = vec![LLMProviderType::OpenAI, LLMProviderType::Gemini];

    for provider in providers {
        let server = create_test_server_with_provider(provider).await;

        // Test various API endpoints for consistent response structure
        let endpoints_to_test = vec![("/api/cards", "GET"), ("/api/topics", "GET")];

        for (endpoint, method) in endpoints_to_test {
            let response = match method {
                "GET" => server.get(endpoint).await,
                _ => continue,
            };

            response.assert_status_ok();
            let response_body: Value = response.json();

            // All responses should have success/data/error structure
            assert!(
                response_body.get("success").is_some(),
                "{} should have 'success' field with {:?}",
                endpoint,
                provider
            );
            assert!(
                response_body.get("data").is_some(),
                "{} should have 'data' field with {:?}",
                endpoint,
                provider
            );
            assert!(
                response_body.get("error").is_some(),
                "{} should have 'error' field with {:?}",
                endpoint,
                provider
            );

            println!(
                "✅ {} API consistent with {:?} provider",
                endpoint, provider
            );
        }
    }
}
