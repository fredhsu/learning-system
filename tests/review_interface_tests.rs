use axum_test::TestServer;
use learning_system::llm_providers::LLMProviderType;
use learning_system::{CardService, Database, LLMService, api::*};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

async fn create_test_server() -> TestServer {
    let db = Database::new("sqlite::memory:").await.unwrap();
    let card_service = CardService::new(db);
    let llm_service =
        LLMService::new_with_provider("test_key".to_string(), None, LLMProviderType::OpenAI, None);
    let app_state = AppState {
        card_service,
        llm_service,
        review_sessions: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = create_router(app_state);
    TestServer::new(app).unwrap()
}

#[tokio::test]
async fn test_card_with_title_creation() {
    let server = create_test_server().await;

    // Create a card with a title
    let create_request = json!({
        "zettel_id": "REVIEW-TEST-001",
        "title": "Test Card Title",
        "content": "This is test content for the review interface",
        "topic_ids": [],
        "zettel_links": null
    });

    let response = server.post("/api/cards").json(&create_request).await;

    response.assert_status_ok();

    let body: Value = response.json();
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["data"]["title"].as_str().unwrap(), "Test Card Title");
    assert_eq!(
        body["data"]["zettel_id"].as_str().unwrap(),
        "REVIEW-TEST-001"
    );
}

#[tokio::test]
async fn test_card_without_title_creation() {
    let server = create_test_server().await;

    // Create a card without a title
    let create_request = json!({
        "zettel_id": "REVIEW-TEST-002",
        "content": "This is test content without a title",
        "topic_ids": [],
        "zettel_links": null
    });

    let response = server.post("/api/cards").json(&create_request).await;

    response.assert_status_ok();

    let body: Value = response.json();
    assert!(body["success"].as_bool().unwrap());
    assert!(body["data"]["title"].is_null());
    assert_eq!(
        body["data"]["zettel_id"].as_str().unwrap(),
        "REVIEW-TEST-002"
    );
}

#[tokio::test]
async fn test_card_update_with_title() {
    let server = create_test_server().await;

    // Create a card first
    let create_request = json!({
        "zettel_id": "REVIEW-TEST-003",
        "content": "Original content",
        "topic_ids": [],
        "zettel_links": null
    });

    let create_response = server.post("/api/cards").json(&create_request).await;

    create_response.assert_status_ok();
    let create_body: Value = create_response.json();
    let card_id = create_body["data"]["id"].as_str().unwrap();

    // Update the card with a title
    let update_request = json!({
        "zettel_id": "REVIEW-TEST-003-UPDATED",
        "title": "Updated Title",
        "content": "Updated content",
        "zettel_links": null
    });

    let response = server
        .put(&format!("/api/cards/{}", card_id))
        .json(&update_request)
        .await;

    response.assert_status_ok();

    let body: Value = response.json();
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["data"]["title"].as_str().unwrap(), "Updated Title");
    assert_eq!(body["data"]["content"].as_str().unwrap(), "Updated content");
}

#[tokio::test]
async fn test_card_list_includes_titles() {
    let server = create_test_server().await;

    // Create cards with and without titles
    let card1 = json!({
        "zettel_id": "REVIEW-LIST-001",
        "title": "First Card",
        "content": "First card content",
        "topic_ids": [],
        "zettel_links": null
    });

    let card2 = json!({
        "zettel_id": "REVIEW-LIST-002",
        "content": "Second card content (no title)",
        "topic_ids": [],
        "zettel_links": null
    });

    // Create the cards
    server
        .post("/api/cards")
        .json(&card1)
        .await
        .assert_status_ok();
    server
        .post("/api/cards")
        .json(&card2)
        .await
        .assert_status_ok();

    // Get all cards
    let response = server.get("/api/cards").await;
    response.assert_status_ok();

    let body: Value = response.json();
    assert!(body["success"].as_bool().unwrap());
    let cards = body["data"].as_array().unwrap();
    assert_eq!(cards.len(), 2);

    // Check that titles are included in the response
    let first_card = &cards[0];
    let second_card = &cards[1];

    // One should have a title, one should have null title
    let has_title_card = if first_card["title"].is_string() {
        first_card
    } else {
        second_card
    };
    let no_title_card = if first_card["title"].is_null() {
        first_card
    } else {
        second_card
    };

    assert_eq!(has_title_card["title"].as_str().unwrap(), "First Card");
    assert!(no_title_card["title"].is_null());
}

#[tokio::test]
async fn test_review_card_data_structure() {
    let server = create_test_server().await;

    // Create a card with title for review
    let create_request = json!({
        "zettel_id": "REVIEW-STRUCT-001",
        "title": "Review Card Title",
        "content": "# Review Content\n\nThis is content that should be collapsible in the review interface.",
        "topic_ids": [],
        "zettel_links": null
    });

    let create_response = server.post("/api/cards").json(&create_request).await;

    create_response.assert_status_ok();
    let create_body: Value = create_response.json();

    // Verify the card has the expected structure for the review interface
    let card_data = &create_body["data"];
    assert_eq!(card_data["title"].as_str().unwrap(), "Review Card Title");
    assert_eq!(
        card_data["zettel_id"].as_str().unwrap(),
        "REVIEW-STRUCT-001"
    );
    assert!(
        card_data["content"]
            .as_str()
            .unwrap()
            .contains("Review Content")
    );

    // The card should have all fields needed for the review interface
    assert!(card_data["id"].is_string());
    assert!(card_data["creation_date"].is_string());
    assert!(card_data["next_review"].is_string());
    assert!(card_data["state"].is_string());
}

#[tokio::test]
async fn test_collapsible_card_content_structure() {
    let server = create_test_server().await;

    // Create cards with different title/content combinations for collapsible testing
    let test_cases = vec![
        // Card with title - should show title by default
        (
            "COLLAPSE-001",
            Some("Short Title"),
            "This is the full content that should be collapsible",
        ),
        // Card without title - should show content preview
        (
            "COLLAPSE-002",
            None,
            "This is content without a title that should still be collapsible",
        ),
        // Card with long title
        (
            "COLLAPSE-003",
            Some("This is a very long title that might affect the collapsible display behavior"),
            "Short content",
        ),
    ];

    for (zettel_id, title, content) in test_cases {
        let mut create_request = json!({
            "zettel_id": zettel_id,
            "content": content,
            "topic_ids": [],
            "zettel_links": null
        });

        if let Some(title_text) = title {
            create_request["title"] = json!(title_text);
        }

        let create_response = server.post("/api/cards").json(&create_request).await;
        create_response.assert_status_ok();

        let create_body: Value = create_response.json();
        let card_data = &create_body["data"];

        // Verify card structure supports collapsible interface
        assert_eq!(card_data["content"], content);
        if let Some(expected_title) = title {
            assert_eq!(card_data["title"].as_str().unwrap(), expected_title);
        } else {
            assert!(card_data["title"].is_null());
        }

        // All cards should have the required fields for review interface
        assert!(card_data["id"].is_string());
        assert!(card_data["zettel_id"].is_string());
    }
}

#[tokio::test]
async fn test_review_interface_title_priority() {
    let server = create_test_server().await;

    // Test that review interface prioritizes title over content for display
    let create_request = json!({
        "zettel_id": "TITLE-PRIORITY-001",
        "title": "Display This Title",
        "content": "# This Markdown Header Should Not Override Title\n\nThe title field should take precedence in the review interface.",
        "topic_ids": [],
        "zettel_links": null
    });

    let create_response = server.post("/api/cards").json(&create_request).await;
    create_response.assert_status_ok();

    let create_body: Value = create_response.json();
    let card_data = &create_body["data"];

    // Verify that both title and content are preserved
    assert_eq!(card_data["title"].as_str().unwrap(), "Display This Title");
    assert!(
        card_data["content"]
            .as_str()
            .unwrap()
            .contains("This Markdown Header")
    );

    // The review interface should use the title field, not extract from content
    assert_ne!(
        card_data["title"].as_str().unwrap(),
        "This Markdown Header Should Not Override Title"
    );
}

#[tokio::test]
async fn test_expand_collapse_content_lengths() {
    let server = create_test_server().await;

    // Test different content lengths for expand/collapse behavior
    let content_cases = vec![
        ("Very short", "SHORT"),
        (
            "Medium length content that spans multiple lines and contains enough text to warrant collapsing in the review interface",
            "MEDIUM",
        ),
        (
            "Very long content that definitely needs to be collapsed by default. This content is intentionally long to test the collapsible interface behavior when dealing with large amounts of text that exceed normal display limits.",
            "LONG",
        ),
    ];

    for (i, (content, case_type)) in content_cases.iter().enumerate() {
        let create_request = json!({
            "zettel_id": format!("EXPAND-{}-{:03}", case_type, i + 1),
            "title": format!("Title for {} Content", case_type),
            "content": content,
            "topic_ids": [],
            "zettel_links": null
        });

        let create_response = server.post("/api/cards").json(&create_request).await;
        create_response.assert_status_ok();

        let create_body: Value = create_response.json();
        let card_data = &create_body["data"];

        // All content should be fully preserved regardless of length
        assert_eq!(card_data["content"].as_str().unwrap(), *content);
        assert_eq!(
            card_data["title"].as_str().unwrap(),
            format!("Title for {} Content", case_type)
        );

        // The frontend will handle the expand/collapse logic
        assert!(card_data["content"].as_str().unwrap().len() > 0);
    }
}
