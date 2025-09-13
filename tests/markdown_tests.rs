use axum::http::StatusCode;
use axum_test::TestServer;
use learning_system::{CardService, Database, LLMService, api::*};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

async fn create_test_server() -> TestServer {
    let database = Database::new("sqlite::memory:").await.unwrap();
    let card_service = CardService::new(database);
    let llm_service = LLMService::new("test_key".to_string(), None);
    let app_state = AppState {
        card_service,
        llm_service,
        review_sessions: Arc::new(Mutex::new(HashMap::new())),
    };

    let application = create_router(app_state);
    TestServer::new(application).unwrap()
}

#[tokio::test]
async fn test_card_with_markdown_content() {
    let test_server = create_test_server().await;

    let markdown_content = "# Markdown Test\n\n**Bold text** and *italic text*\n\n- List item 1\n- List item 2\n\n```rust\nlet x = 42;\n```\n\n$$E = mc^2$$";

    let request_body = json!({
        "zettel_id": "MARKDOWN-001",
        "content": markdown_content,
        "topic_ids": []
    });

    let response = test_server.post("/api/cards").json(&request_body).await;

    response.assert_status(StatusCode::OK);
    let response_body: Value = response.json();

    assert_eq!(response_body["success"], true);
    assert_eq!(response_body["data"]["content"], markdown_content);
}

#[tokio::test]
async fn test_card_content_persistence() {
    let test_server = create_test_server().await;

    let original_content =
        "## Test Content\n\nThis is a **test** with `code` and [link](http://example.com)";

    // Create card with markdown content
    let create_request = json!({
        "zettel_id": "MARKDOWN-002",
        "content": original_content,
        "topic_ids": []
    });

    let create_response = test_server.post("/api/cards").json(&create_request).await;

    create_response.assert_status(StatusCode::OK);
    let create_body: Value = create_response.json();
    let card_id = create_body["data"]["id"].as_str().unwrap();

    // Retrieve the card and verify content
    let get_response = test_server.get(&format!("/api/cards/{}", card_id)).await;

    get_response.assert_status(StatusCode::OK);
    let get_body: Value = get_response.json();

    assert_eq!(get_body["data"]["content"], original_content);
}
