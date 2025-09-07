use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::{json, Value};
use learning_system::{api::*, CardService, Database, LLMService};
use uuid::Uuid;
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

#[tokio::test]
async fn test_api_create_card() {
    let server = create_test_server().await;
    
    let request_body = json!({
        "zettel_id": "API-001",
        "content": "Test API card creation",
        "topic_ids": [],
        "links": null
    });

    let response = server
        .post("/api/cards")
        .json(&request_body)
        .await;

    response.assert_status_ok();
    let body: Value = response.json();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["content"], "Test API card creation");
    assert_eq!(body["data"]["reps"], 0);
    assert_eq!(body["data"]["state"], "New");
}

#[tokio::test]
async fn test_api_get_all_cards() {
    let server = create_test_server().await;
    
    // First create a card
    let create_request = json!({
        "zettel_id": "API-002",
        "content": "Card for GET test",
        "topic_ids": [],
        "links": null
    });

    let create_response = server
        .post("/api/cards")
        .json(&create_request)
        .await;
    create_response.assert_status_ok();

    // Then get all cards
    let get_response = server.get("/api/cards").await;
    get_response.assert_status_ok();
    
    let body: Value = get_response.json();
    assert_eq!(body["success"], true);
    assert!(body["data"].is_array());
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
    assert_eq!(body["data"][0]["content"], "Card for GET test");
}

#[tokio::test]
async fn test_api_get_single_card() {
    let server = create_test_server().await;
    
    // Create a card first
    let create_request = json!({
        "zettel_id": "API-003",
        "content": "Single card test",
        "topic_ids": [],
        "links": null
    });

    let create_response = server
        .post("/api/cards")
        .json(&create_request)
        .await;
    create_response.assert_status_ok();
    
    let create_body: Value = create_response.json();
    let card_id = create_body["data"]["id"].as_str().unwrap();

    // Get the specific card
    let get_response = server
        .get(&format!("/api/cards/{}", card_id))
        .await;
    get_response.assert_status_ok();
    
    let get_body: Value = get_response.json();
    assert_eq!(get_body["success"], true);
    assert_eq!(get_body["data"]["content"], "Single card test");
    assert_eq!(get_body["data"]["id"], card_id);
}

#[tokio::test]
async fn test_api_get_nonexistent_card() {
    let server = create_test_server().await;
    
    let fake_id = Uuid::new_v4();
    let response = server
        .get(&format!("/api/cards/{}", fake_id))
        .await;
    
    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_api_update_card() {
    let server = create_test_server().await;
    
    // Create a card first
    let create_request = json!({
        "zettel_id": "API-004",
        "content": "Original content",
        "topic_ids": [],
        "links": null
    });

    let create_response = server
        .post("/api/cards")
        .json(&create_request)
        .await;
    create_response.assert_status_ok();
    
    let create_body: Value = create_response.json();
    let card_id = create_body["data"]["id"].as_str().unwrap();

    // Update the card
    let update_request = json!({
        "zettel_id": "API-005",
        "content": "Updated content via API",
        "links": null
    });

    let update_response = server
        .put(&format!("/api/cards/{}", card_id))
        .json(&update_request)
        .await;
    update_response.assert_status_ok();
    
    let update_body: Value = update_response.json();
    assert_eq!(update_body["success"], true);
    assert_eq!(update_body["data"]["content"], "Updated content via API");
    assert_eq!(update_body["data"]["id"], card_id);
}

#[tokio::test]
async fn test_api_update_nonexistent_card() {
    let server = create_test_server().await;
    
    let fake_id = Uuid::new_v4();
    let update_request = json!({
        "zettel_id": "API-006",
        "content": "This should fail",
        "links": null
    });

    let response = server
        .put(&format!("/api/cards/{}", fake_id))
        .json(&update_request)
        .await;
    
    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_api_delete_card() {
    let server = create_test_server().await;
    
    // Create a card first
    let create_request = json!({
        "zettel_id": "API-007",
        "content": "Card to be deleted",
        "topic_ids": [],
        "links": null
    });

    let create_response = server
        .post("/api/cards")
        .json(&create_request)
        .await;
    create_response.assert_status_ok();
    
    let create_body: Value = create_response.json();
    let card_id = create_body["data"]["id"].as_str().unwrap();

    // Delete the card
    let delete_response = server
        .delete(&format!("/api/cards/{}", card_id))
        .await;
    delete_response.assert_status_ok();
    
    let delete_body: Value = delete_response.json();
    assert_eq!(delete_body["success"], true);
    assert_eq!(delete_body["data"], true);

    // Verify the card is gone
    let get_response = server
        .get(&format!("/api/cards/{}", card_id))
        .await;
    get_response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_api_delete_nonexistent_card() {
    let server = create_test_server().await;
    
    let fake_id = Uuid::new_v4();
    let response = server
        .delete(&format!("/api/cards/{}", fake_id))
        .await;
    
    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_api_cards_due_for_review() {
    let server = create_test_server().await;
    
    // Create a card
    let create_request = json!({
        "zettel_id": "API-008",
        "content": "Due for review",
        "topic_ids": [],
        "links": null
    });

    let create_response = server
        .post("/api/cards")
        .json(&create_request)
        .await;
    create_response.assert_status_ok();

    // Get cards due for review
    let due_response = server.get("/api/cards/due").await;
    due_response.assert_status_ok();
    
    let due_body: Value = due_response.json();
    assert_eq!(due_body["success"], true);
    assert!(due_body["data"].is_array());
    assert_eq!(due_body["data"].as_array().unwrap().len(), 1);
    assert_eq!(due_body["data"][0]["content"], "Due for review");
}

#[tokio::test]
async fn test_api_create_topic() {
    let server = create_test_server().await;
    
    let request_body = json!({
        "name": "API Test Topic",
        "description": "Topic created via API"
    });

    let response = server
        .post("/api/topics")
        .json(&request_body)
        .await;

    response.assert_status_ok();
    let body: Value = response.json();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["name"], "API Test Topic");
    assert_eq!(body["data"]["description"], "Topic created via API");
}

#[tokio::test]
async fn test_api_get_topics() {
    let server = create_test_server().await;
    
    // Create a topic first
    let create_request = json!({
        "name": "Topic for GET test",
        "description": null
    });

    let create_response = server
        .post("/api/topics")
        .json(&create_request)
        .await;
    create_response.assert_status_ok();

    // Get all topics
    let get_response = server.get("/api/topics").await;
    get_response.assert_status_ok();
    
    let body: Value = get_response.json();
    assert_eq!(body["success"], true);
    assert!(body["data"].is_array());
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
    assert_eq!(body["data"][0]["name"], "Topic for GET test");
}

#[tokio::test]
async fn test_api_review_card() {
    let server = create_test_server().await;
    
    // Create a card first
    let create_request = json!({
        "zettel_id": "API-009",
        "content": "Card for review",
        "topic_ids": [],
        "links": null
    });

    let create_response = server
        .post("/api/cards")
        .json(&create_request)
        .await;
    create_response.assert_status_ok();
    
    let create_body: Value = create_response.json();
    let card_id = create_body["data"]["id"].as_str().unwrap();

    // Review the card
    let review_request = json!({
        "rating": 3
    });

    let review_response = server
        .post(&format!("/api/cards/{}/review", card_id))
        .json(&review_request)
        .await;
    review_response.assert_status_ok();
    
    let review_body: Value = review_response.json();
    assert_eq!(review_body["success"], true);
    assert_eq!(review_body["data"]["reps"], 1);
}

#[tokio::test]
async fn test_api_invalid_json() {
    let server = create_test_server().await;
    
    // Try to create a card with invalid JSON
    let response = server
        .post("/api/cards")
        .add_header("content-type", "application/json")
        .text("invalid json")
        .await;
    
    response.assert_status(StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

#[tokio::test]
async fn test_api_missing_fields() {
    let server = create_test_server().await;
    
    // Try to create a card without required fields
    let request_body = json!({
        "topic_ids": []
        // Missing "content" field
    });

    let response = server
        .post("/api/cards")
        .json(&request_body)
        .await;
    
    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_api_card_with_links() {
    let server = create_test_server().await;
    
    // Create first card
    let create_request1 = json!({
        "zettel_id": "API-010",
        "content": "Card 1",
        "topic_ids": [],
        "zettel_links": null
    });

    let create_response1 = server
        .post("/api/cards")
        .json(&create_request1)
        .await;
    create_response1.assert_status_ok();
    
    let create_body1: Value = create_response1.json();
    let card1_id = create_body1["data"]["id"].as_str().unwrap();

    // Create second card that links to the first
    let create_request2 = json!({
        "zettel_id": "API-011",
        "content": "Card 2 with links",
        "topic_ids": [],
        "zettel_links": ["API-010"]
    });

    let create_response2 = server
        .post("/api/cards")
        .json(&create_request2)
        .await;
    create_response2.assert_status_ok();
    
    let create_body2: Value = create_response2.json();
    let card2_id = create_body2["data"]["id"].as_str().unwrap();
    
    // Get linked cards
    let links_response = server
        .get(&format!("/api/cards/{}/links", card2_id))
        .await;
    links_response.assert_status_ok();
    
    let links_body: Value = links_response.json();
    assert_eq!(links_body["success"], true);
    assert!(links_body["data"].is_array());
    assert_eq!(links_body["data"].as_array().unwrap().len(), 1);
    assert_eq!(links_body["data"][0]["id"], card1_id);
}

#[tokio::test]
async fn test_api_search_cards() {
    let server = create_test_server().await;
    
    // Create multiple cards with searchable content
    let search_test_cards = vec![
        "Mathematics: quadratic equations and formulas",
        "Programming: Rust memory management concepts",
        "Physics: quantum mechanics and mathematics",
        "History: ancient civilizations and culture",
    ];

    for content in &search_test_cards {
        let create_request = json!({
            "zettel_id": format!("API-SEARCH-{:03}", search_test_cards.iter().position(|&c| c == *content).unwrap() + 1),
            "content": content,
            "topic_ids": [],
            "links": null
        });

        let create_response = server
            .post("/api/cards")
            .json(&create_request)
            .await;
        create_response.assert_status_ok();
    }

    // Test search functionality
    let search_response = server
        .get("/api/cards/search?q=mathematics")
        .await;
    search_response.assert_status_ok();
    
    let search_body: Value = search_response.json();
    assert_eq!(search_body["success"], true);
    assert!(search_body["data"].is_array());
    assert_eq!(search_body["data"].as_array().unwrap().len(), 2); // Should find 2 cards

    // Test case-insensitive search
    let search_response = server
        .get("/api/cards/search?q=PROGRAMMING")
        .await;
    search_response.assert_status_ok();
    
    let search_body: Value = search_response.json();
    assert_eq!(search_body["success"], true);
    assert_eq!(search_body["data"].as_array().unwrap().len(), 1); // Should find 1 card

    // Test search with no results
    let search_response = server
        .get("/api/cards/search?q=nonexistent")
        .await;
    search_response.assert_status_ok();
    
    let search_body: Value = search_response.json();
    assert_eq!(search_body["success"], true);
    assert_eq!(search_body["data"].as_array().unwrap().len(), 0); // Should find no cards

    // Test search with empty query (should return all cards)
    let search_response = server
        .get("/api/cards/search?q=")
        .await;
    search_response.assert_status_ok();
    
    let search_body: Value = search_response.json();
    assert_eq!(search_body["success"], true);
    assert_eq!(search_body["data"].as_array().unwrap().len(), 4); // Should return all cards
}

#[tokio::test]
async fn test_api_search_url_encoding() {
    let server = create_test_server().await;
    
    // Create a card with special characters
    let create_request = json!({
        "zettel_id": "API-012",
        "content": "Special chars: !@#$%^&*() and spaces",
        "topic_ids": [],
        "links": null
    });

    let create_response = server
        .post("/api/cards")
        .json(&create_request)
        .await;
    create_response.assert_status_ok();

    // Test search with URL-encoded special characters
    let search_response = server
        .get("/api/cards/search?q=Special%20chars")
        .await;
    search_response.assert_status_ok();
    
    let search_body: Value = search_response.json();
    assert_eq!(search_body["success"], true);
    assert_eq!(search_body["data"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_api_response_structure() {
    let server = create_test_server().await;
    
    // Test that all API responses follow the expected structure
    let create_request = json!({
        "zettel_id": "API-013",
        "content": "Response structure test",
        "topic_ids": [],
        "links": null
    });

    let create_response = server
        .post("/api/cards")
        .json(&create_request)
        .await;
    create_response.assert_status_ok();
    
    let create_body: Value = create_response.json();
    
    // Verify response structure
    assert!(create_body.get("success").is_some());
    assert!(create_body.get("data").is_some());
    assert_eq!(create_body["success"], true);
    
    // Verify card data structure contains expected fields
    let card_data = &create_body["data"];
    assert!(card_data.get("id").is_some());
    assert!(card_data.get("content").is_some());
    assert!(card_data.get("creation_date").is_some());
    assert!(card_data.get("next_review").is_some());
    assert!(card_data.get("reps").is_some());
    assert!(card_data.get("state").is_some());
}

#[tokio::test]
async fn test_api_error_response_format() {
    let server = create_test_server().await;
    
    // Test that error responses have the expected JSON structure
    let fake_id = Uuid::new_v4();
    let response = server
        .get(&format!("/api/cards/{}", fake_id))
        .await;
    
    response.assert_status(StatusCode::NOT_FOUND);
    let body: Value = response.json();
    
    // Verify the error response has the expected structure
    assert_eq!(body["success"], false);
    assert!(body["error"].is_string());
    assert!(body["error"].as_str().unwrap().contains("not found"));
}