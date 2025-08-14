use learning_system::{Database, CardService, CreateCardRequest};
use uuid::Uuid;

#[tokio::test]
async fn test_card_creation_and_retrieval() {
    let db = Database::new("sqlite::memory:").await.unwrap();
    let card_service = CardService::new(db);

    let create_request = CreateCardRequest {
        content: "Test card content with LaTeX: $x^2 + y^2 = z^2$".to_string(),
        topic_ids: vec![],
        links: None,
    };

    let created_card = card_service.create_card(create_request).await.unwrap();
    assert_eq!(created_card.content, "Test card content with LaTeX: $x^2 + y^2 = z^2$");
    assert_eq!(created_card.reps, 0);
    assert_eq!(created_card.state, "New");

    let retrieved_card = card_service.get_card(created_card.id).await.unwrap();
    assert!(retrieved_card.is_some());
    assert_eq!(retrieved_card.unwrap().content, created_card.content);
}

#[tokio::test]
async fn test_fsrs_scheduling() {
    use learning_system::FSRSScheduler;
    use chrono::Utc;

    let scheduler = FSRSScheduler::new();
    let now = Utc::now();

    let db = Database::new("sqlite::memory:").await.unwrap();
    let card_service = CardService::new(db);

    let create_request = CreateCardRequest {
        content: "FSRS test card".to_string(),
        topic_ids: vec![],
        links: None,
    };

    let card = card_service.create_card(create_request).await.unwrap();

    // Test rating conversion
    assert!(FSRSScheduler::get_rating_from_int(1).is_some());
    assert!(FSRSScheduler::get_rating_from_int(2).is_some());
    assert!(FSRSScheduler::get_rating_from_int(3).is_some());
    assert!(FSRSScheduler::get_rating_from_int(4).is_some());
    assert_eq!(FSRSScheduler::get_rating_from_int(5), None);

    // Test scheduling
    let rating = FSRSScheduler::get_rating_from_int(3).unwrap();
    let (updated_card, _review_log) = scheduler.schedule_card(&card, rating, now).unwrap();
    assert!(updated_card.next_review > now);
    assert_eq!(updated_card.reps, 1);
}

#[tokio::test]
async fn test_topic_creation() {
    let db = Database::new("sqlite::memory:").await.unwrap();
    let card_service = CardService::new(db);

    let topic = card_service.create_topic(
        "Mathematics".to_string(),
        Some("Mathematical concepts and formulas".to_string())
    ).await.unwrap();

    assert_eq!(topic.name, "Mathematics");
    assert_eq!(topic.description, Some("Mathematical concepts and formulas".to_string()));

    let topics = card_service.get_all_topics().await.unwrap();
    assert_eq!(topics.len(), 1);
    assert_eq!(topics[0].name, "Mathematics");
}

#[tokio::test]
async fn test_review_workflow() {
    let db = Database::new("sqlite::memory:").await.unwrap();
    let card_service = CardService::new(db);

    let create_request = CreateCardRequest {
        content: "Review test card".to_string(),
        topic_ids: vec![],
        links: None,
    };

    let card = card_service.create_card(create_request).await.unwrap();

    // Initially card should be due for review
    let due_cards = card_service.get_cards_due_for_review().await.unwrap();
    assert_eq!(due_cards.len(), 1);
    assert_eq!(due_cards[0].id, card.id);

    // Review the card with a "Good" rating
    let reviewed_card = card_service.review_card(card.id, 3).await.unwrap();
    assert!(reviewed_card.is_some());
    
    let reviewed_card = reviewed_card.unwrap();
    assert!(reviewed_card.next_review > card.next_review);
    assert_eq!(reviewed_card.reps, 1);
}