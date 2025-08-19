use learning_system::{Database, CardService, CreateCardRequest, UpdateCardRequest};
use uuid::Uuid;

#[tokio::test]
async fn test_database_connection_failure() {
    // Test with invalid database URL
    let result = Database::new("invalid://url").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_database_operations_with_invalid_data() {
    let db = Database::new("sqlite::memory:").await.unwrap();
    
    // Test creating card with empty content
    let create_request = CreateCardRequest {
        content: "".to_string(),
        topic_ids: vec![],
        links: None,
    };
    
    // Should succeed - empty content is valid
    let card = db.create_card(create_request).await.unwrap();
    assert_eq!(card.content, "");
    
    // Test with very long content
    let long_content = "a".repeat(100000);
    let create_request = CreateCardRequest {
        content: long_content.clone(),
        topic_ids: vec![],
        links: None,
    };
    
    let card = db.create_card(create_request).await.unwrap();
    assert_eq!(card.content, long_content);
}

#[tokio::test]
async fn test_concurrent_card_operations() {
    let db = Database::new("sqlite::memory:").await.unwrap();
    let service = CardService::new(db);
    
    // Create a card
    let create_request = CreateCardRequest {
        content: "Concurrent test card".to_string(),
        topic_ids: vec![],
        links: None,
    };
    
    let card = service.create_card(create_request).await.unwrap();
    
    // Try to perform multiple operations concurrently
    let card_id = card.id;
    
    let update_task1 = tokio::spawn({
        let service = service.clone();
        async move {
            let update_request = UpdateCardRequest {
                content: Some("Updated by task 1".to_string()),
                topic_ids: None,
                links: None,
            };
            service.update_card(card_id, update_request).await
        }
    });
    
    let update_task2 = tokio::spawn({
        let service = service.clone();
        async move {
            let update_request = UpdateCardRequest {
                content: Some("Updated by task 2".to_string()),
                topic_ids: None,
                links: None,
            };
            service.update_card(card_id, update_request).await
        }
    });
    
    let delete_task = tokio::spawn({
        let service = service.clone();
        async move {
            // Wait a bit to let updates happen first
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            service.delete_card(card_id).await
        }
    });
    
    // Wait for all tasks to complete
    let (update1_result, update2_result, delete_result) = 
        tokio::join!(update_task1, update_task2, delete_task);
    
    // All operations should complete without panicking
    assert!(update1_result.is_ok());
    assert!(update2_result.is_ok());
    assert!(delete_result.is_ok());
}

#[tokio::test]
async fn test_invalid_uuid_handling() {
    let db = Database::new("sqlite::memory:").await.unwrap();
    let service = CardService::new(db);
    
    // Test operations with various invalid UUIDs
    let invalid_ids = vec![
        Uuid::new_v4(), // Valid UUID but non-existent card
    ];
    
    for invalid_id in invalid_ids {
        // Get card
        let result = service.get_card(invalid_id).await.unwrap();
        assert!(result.is_none());
        
        // Update card
        let update_request = UpdateCardRequest {
            content: Some("Should not work".to_string()),
            topic_ids: None,
            links: None,
        };
        let result = service.update_card(invalid_id, update_request).await.unwrap();
        assert!(result.is_none());
        
        // Delete card
        let result = service.delete_card(invalid_id).await.unwrap();
        assert!(!result);
        
        // Review card
        let result = service.review_card(invalid_id, 3).await.unwrap();
        assert!(result.is_none());
    }
}

#[tokio::test]
async fn test_invalid_rating_handling() {
    let db = Database::new("sqlite::memory:").await.unwrap();
    let service = CardService::new(db);
    
    let create_request = CreateCardRequest {
        content: "Rating test card".to_string(),
        topic_ids: vec![],
        links: None,
    };
    
    let card = service.create_card(create_request).await.unwrap();
    
    // Test invalid ratings
    let invalid_ratings = vec![0, -1, 5, 10, 100];
    
    for rating in invalid_ratings {
        let result = service.review_card(card.id, rating).await;
        assert!(result.is_err(), "Rating {} should have failed", rating);
    }
    
    // Test valid ratings work
    let valid_ratings = vec![1, 2, 3, 4];
    for rating in valid_ratings {
        let result = service.review_card(card.id, rating).await;
        assert!(result.is_ok(), "Rating {} should have worked", rating);
    }
}

#[tokio::test]
async fn test_malformed_links_handling() {
    let db = Database::new("sqlite::memory:").await.unwrap();
    let service = CardService::new(db);
    
    // Create a card
    let create_request = CreateCardRequest {
        content: "Card with bad links".to_string(),
        topic_ids: vec![],
        links: Some(vec![Uuid::new_v4(), Uuid::new_v4()]), // Non-existent UUIDs
    };
    
    let card = service.create_card(create_request).await.unwrap();
    
    // Try to get linked cards - should handle non-existent links gracefully
    let linked_cards = service.get_linked_cards(card.id).await.unwrap();
    assert_eq!(linked_cards.len(), 0); // No valid linked cards found
}

#[tokio::test]
async fn test_topic_name_constraints() {
    let db = Database::new("sqlite::memory:").await.unwrap();
    let service = CardService::new(db);
    
    // Create topic with normal name
    let topic1 = service.create_topic("Normal Topic".to_string(), None).await.unwrap();
    assert_eq!(topic1.name, "Normal Topic");
    
    // Create topic with empty name - should work
    let topic2 = service.create_topic("".to_string(), None).await.unwrap();
    assert_eq!(topic2.name, "");
    
    // Create topic with very long name
    let long_name = "a".repeat(1000);
    let topic3 = service.create_topic(long_name.clone(), None).await.unwrap();
    assert_eq!(topic3.name, long_name);
    
    // Try to create duplicate topic name - should fail due to UNIQUE constraint
    let result = service.create_topic("Normal Topic".to_string(), None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_card_cascade_deletion() {
    let db = Database::new("sqlite::memory:").await.unwrap();
    
    // Create a topic
    let topic = db.create_topic("Test Topic".to_string(), None).await.unwrap();
    
    // Create a card associated with the topic
    let create_request = CreateCardRequest {
        content: "Card with topic".to_string(),
        topic_ids: vec![topic.id],
        links: None,
    };
    
    let card = db.create_card(create_request).await.unwrap();
    
    // Create a review for the card
    let _review = db.create_review(card.id, 3, 1.0, 2.5).await.unwrap();
    
    // Delete the card
    let deleted = db.delete_card(card.id).await.unwrap();
    assert!(deleted);
    
    // Verify card is gone
    let retrieved_card = db.get_card(card.id).await.unwrap();
    assert!(retrieved_card.is_none());
    
    // Topic should still exist (no cascade from card to topic)
    let topics = db.get_all_topics().await.unwrap();
    assert_eq!(topics.len(), 1);
}

#[tokio::test]
async fn test_extreme_date_handling() {
    use chrono::{DateTime, Utc, NaiveDate};
    use learning_system::FSRSScheduler;
    
    let scheduler = FSRSScheduler::new();
    
    // Create a card with extreme dates
    let extreme_past = DateTime::from_naive_utc_and_offset(
        NaiveDate::from_ymd_opt(1900, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap(),
        Utc
    );
    
    let extreme_future = DateTime::from_naive_utc_and_offset(
        NaiveDate::from_ymd_opt(2100, 12, 31).unwrap().and_hms_opt(23, 59, 59).unwrap(),
        Utc
    );
    
    let mut card = learning_system::Card {
        id: Uuid::new_v4(),
        content: "Extreme date card".to_string(),
        creation_date: extreme_past,
        last_reviewed: Some(extreme_past),
        next_review: extreme_future,
        difficulty: 3.0,
        stability: 1.0,
        retrievability: 0.5,
        reps: 5,
        lapses: 1,
        state: "Review".to_string(),
        links: None,
    };
    
    // Try to schedule with extreme dates
    let rating = learning_system::fsrs_scheduler::Rating::Good;
    let result = scheduler.schedule_card(&card, rating, Utc::now());
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_memory_stress() {
    let db = Database::new("sqlite::memory:").await.unwrap();
    let service = CardService::new(db);
    
    // Create many cards quickly to test memory usage
    let mut card_ids = Vec::new();
    
    for i in 0..100 {
        let create_request = CreateCardRequest {
            content: format!("Stress test card {}", i),
            topic_ids: vec![],
            links: None,
        };
        
        let card = service.create_card(create_request).await.unwrap();
        card_ids.push(card.id);
    }
    
    // Verify all cards were created
    let all_cards = service.get_all_cards().await.unwrap();
    assert_eq!(all_cards.len(), 100);
    
    // Update all cards
    for (i, &card_id) in card_ids.iter().enumerate() {
        let update_request = UpdateCardRequest {
            content: Some(format!("Updated stress test card {}", i)),
            topic_ids: None,
            links: None,
        };
        
        let result = service.update_card(card_id, update_request).await.unwrap();
        assert!(result.is_some());
    }
    
    // Delete all cards
    for &card_id in &card_ids {
        let deleted = service.delete_card(card_id).await.unwrap();
        assert!(deleted);
    }
    
    // Verify all cards are gone
    let remaining_cards = service.get_all_cards().await.unwrap();
    assert_eq!(remaining_cards.len(), 0);
}