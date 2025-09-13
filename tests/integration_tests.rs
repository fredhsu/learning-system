use learning_system::{
    CardService, CreateCardRequest, CreateCardWithZettelLinksRequest, Database, UpdateCardRequest,
    UpdateCardWithZettelLinksRequest,
};
use uuid::Uuid;

#[tokio::test]
async fn test_card_creation_and_retrieval() {
    let db = Database::new("sqlite::memory:").await.unwrap();
    let card_service = CardService::new(db);

    let create_request = CreateCardRequest {
        zettel_id: "TEST-001".to_string(),
        title: Some("Test Card".to_string()),
        content: "Test card content with LaTeX: $x^2 + y^2 = z^2$".to_string(),
        topic_ids: vec![],
        links: None,
    };

    let created_card = card_service.create_card(create_request).await.unwrap();
    assert_eq!(
        created_card.content,
        "Test card content with LaTeX: $x^2 + y^2 = z^2$"
    );
    assert_eq!(created_card.reps, 0);
    assert_eq!(created_card.state, "New");

    let retrieved_card = card_service.get_card(created_card.id).await.unwrap();
    assert!(retrieved_card.is_some());
    assert_eq!(retrieved_card.unwrap().content, created_card.content);
}

#[tokio::test]
async fn test_fsrs_scheduling() {
    use chrono::Utc;
    use learning_system::FSRSScheduler;

    let scheduler = FSRSScheduler::new();
    let now = Utc::now();

    let db = Database::new("sqlite::memory:").await.unwrap();
    let card_service = CardService::new(db);

    let create_request = CreateCardRequest {
        zettel_id: "TEST-002".to_string(),
        title: None,
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

    let topic = card_service
        .create_topic(
            "Mathematics".to_string(),
            Some("Mathematical concepts and formulas".to_string()),
        )
        .await
        .unwrap();

    assert_eq!(topic.name, "Mathematics");
    assert_eq!(
        topic.description,
        Some("Mathematical concepts and formulas".to_string())
    );

    let topics = card_service.get_all_topics().await.unwrap();
    assert_eq!(topics.len(), 1);
    assert_eq!(topics[0].name, "Mathematics");
}

#[tokio::test]
async fn test_review_workflow() {
    let db = Database::new("sqlite::memory:").await.unwrap();
    let card_service = CardService::new(db);

    let create_request = CreateCardRequest {
        zettel_id: "TEST-003".to_string(),
        title: None,
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

#[tokio::test]
async fn test_card_update() {
    use learning_system::UpdateCardRequest;

    let db = Database::new("sqlite::memory:").await.unwrap();
    let card_service = CardService::new(db);

    let create_request = CreateCardRequest {
        zettel_id: "UPDATE-001".to_string(),
        title: None,
        content: "Original content".to_string(),
        topic_ids: vec![],
        links: None,
    };

    let card = card_service.create_card(create_request).await.unwrap();
    assert_eq!(card.content, "Original content");
    assert!(card.links.is_none());

    // Test updating content
    let update_request = UpdateCardRequest {
        zettel_id: Some("UPDATE-002".to_string()),
        title: None,
        content: Some("Updated content".to_string()),
        topic_ids: None,
        links: Some(vec![Uuid::new_v4()]),
    };

    let updated_card = card_service
        .update_card(card.id, update_request)
        .await
        .unwrap();
    assert!(updated_card.is_some());

    let updated_card = updated_card.unwrap();
    assert_eq!(updated_card.content, "Updated content");
    assert!(updated_card.links.is_some());

    // Verify the card was actually updated in the database
    let retrieved_card = card_service.get_card(card.id).await.unwrap();
    assert!(retrieved_card.is_some());
    assert_eq!(retrieved_card.unwrap().content, "Updated content");
}

#[tokio::test]
async fn test_card_update_nonexistent() {
    use learning_system::UpdateCardRequest;

    let db = Database::new("sqlite::memory:").await.unwrap();
    let card_service = CardService::new(db);

    let fake_id = Uuid::new_v4();
    let update_request = UpdateCardRequest {
        zettel_id: Some("UPDATE-003".to_string()),
        title: None,
        content: Some("This should fail".to_string()),
        topic_ids: None,
        links: None,
    };

    let result = card_service
        .update_card(fake_id, update_request)
        .await
        .unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_card_deletion() {
    let db = Database::new("sqlite::memory:").await.unwrap();
    let card_service = CardService::new(db);

    let create_request = CreateCardRequest {
        zettel_id: "DELETE-001".to_string(),
        title: None,
        content: "Card to be deleted".to_string(),
        topic_ids: vec![],
        links: None,
    };

    let card = card_service.create_card(create_request).await.unwrap();

    // Verify card exists
    let retrieved_card = card_service.get_card(card.id).await.unwrap();
    assert!(retrieved_card.is_some());

    // Delete the card
    let deleted = card_service.delete_card(card.id).await.unwrap();
    assert!(deleted);

    // Verify card no longer exists
    let retrieved_card = card_service.get_card(card.id).await.unwrap();
    assert!(retrieved_card.is_none());

    // Verify it's not in the list of all cards
    let all_cards = card_service.get_all_cards().await.unwrap();
    assert!(!all_cards.iter().any(|c| c.id == card.id));
}

#[tokio::test]
async fn test_card_deletion_nonexistent() {
    let db = Database::new("sqlite::memory:").await.unwrap();
    let card_service = CardService::new(db);

    let fake_id = Uuid::new_v4();
    let deleted = card_service.delete_card(fake_id).await.unwrap();
    assert!(!deleted);
}

#[tokio::test]
async fn test_database_card_operations() {
    let db = Database::new("sqlite::memory:").await.unwrap();

    let create_request = CreateCardRequest {
        zettel_id: "DB-001".to_string(),
        title: None,
        content: "Direct database test".to_string(),
        topic_ids: vec![],
        links: Some(vec![Uuid::new_v4()]),
    };

    // Test direct database creation
    let card = db.create_card(create_request).await.unwrap();
    assert_eq!(card.content, "Direct database test");
    assert!(card.links.is_some());

    // Test direct database retrieval
    let retrieved = db.get_card(card.id).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.as_ref().unwrap().content, "Direct database test");

    // Test direct database update
    let mut updated_card = card.clone();
    updated_card.content = "Updated direct database test".to_string();
    updated_card.links = None;

    db.update_card_content(&updated_card).await.unwrap();

    let retrieved_after_update = db.get_card(card.id).await.unwrap();
    assert!(retrieved_after_update.is_some());
    assert_eq!(
        retrieved_after_update.as_ref().unwrap().content,
        "Updated direct database test"
    );
    assert!(retrieved_after_update.as_ref().unwrap().links.is_none());

    // Test direct database deletion
    let deleted = db.delete_card(card.id).await.unwrap();
    assert!(deleted);

    let retrieved_after_delete = db.get_card(card.id).await.unwrap();
    assert!(retrieved_after_delete.is_none());
}

#[tokio::test]
async fn test_card_links_functionality() {
    let db = Database::new("sqlite::memory:").await.unwrap();
    let card_service = CardService::new(db);

    // Create first card
    let card1 = card_service
        .create_card(CreateCardRequest {
            zettel_id: "LINK-001".to_string(),
            title: None,
            content: "Card 1".to_string(),
            topic_ids: vec![],
            links: None,
        })
        .await
        .unwrap();

    // Create second card
    let card2 = card_service
        .create_card(CreateCardRequest {
            zettel_id: "LINK-002".to_string(),
            title: None,
            content: "Card 2".to_string(),
            topic_ids: vec![],
            links: None,
        })
        .await
        .unwrap();

    // Update first card to link to second card
    let update_request = UpdateCardRequest {
        zettel_id: Some("LINK-003".to_string()),
        title: None,
        content: None,
        topic_ids: None,
        links: Some(vec![card2.id]),
    };

    let updated_card1 = card_service
        .update_card(card1.id, update_request)
        .await
        .unwrap();
    assert!(updated_card1.is_some());

    // Test getting linked cards
    let linked_cards = card_service.get_linked_cards(card1.id).await.unwrap();
    assert_eq!(linked_cards.len(), 1);
    assert_eq!(linked_cards[0].id, card2.id);
    assert_eq!(linked_cards[0].content, "Card 2");
}

#[tokio::test]
async fn test_multiple_card_operations() {
    let db = Database::new("sqlite::memory:").await.unwrap();
    let card_service = CardService::new(db);

    // Create multiple cards
    let mut card_ids = Vec::new();
    for i in 1..=5 {
        let create_request = CreateCardRequest {
            zettel_id: format!("MULTI-{:03}", i),
            title: None,
            content: format!("Test card {}", i),
            topic_ids: vec![],
            links: None,
        };
        let card = card_service.create_card(create_request).await.unwrap();
        card_ids.push(card.id);
    }

    // Verify all cards exist
    let all_cards = card_service.get_all_cards().await.unwrap();
    assert_eq!(all_cards.len(), 5);

    // Update some cards
    for (i, &card_id) in card_ids.iter().enumerate() {
        if i % 2 == 0 {
            let update_request = UpdateCardRequest {
                zettel_id: Some(format!("MULTI-UPDATE-{:03}", i + 1)),
                title: None,
                content: Some(format!("Updated test card {}", i + 1)),
                topic_ids: None,
                links: None,
            };
            let updated = card_service
                .update_card(card_id, update_request)
                .await
                .unwrap();
            assert!(updated.is_some());
        }
    }

    // Delete some cards
    for (i, &card_id) in card_ids.iter().enumerate() {
        if i % 3 == 0 {
            let deleted = card_service.delete_card(card_id).await.unwrap();
            assert!(deleted);
        }
    }

    // Verify final state
    let remaining_cards = card_service.get_all_cards().await.unwrap();
    assert_eq!(remaining_cards.len(), 3); // 5 - 2 deleted = 3

    // Verify deleted cards don't exist
    for (i, &card_id) in card_ids.iter().enumerate() {
        if i % 3 == 0 {
            let card = card_service.get_card(card_id).await.unwrap();
            assert!(card.is_none());
        }
    }
}

#[tokio::test]
async fn test_search_functionality() {
    let db = Database::new("sqlite::memory:").await.unwrap();
    let card_service = CardService::new(db);

    // Create multiple cards with different content
    let cards_data = vec![
        (
            "Mathematics formulas and physics: $E = mc^2$",
            "math, physics",
        ),
        ("Programming concepts in Rust", "programming, rust"),
        ("History of ancient civilizations", "history"),
        (
            "Mathematics and programming intersection",
            "math, programming",
        ),
        ("Physics concepts and formulas", "physics"),
    ];

    let mut created_card_ids = Vec::new();
    for (content, _topics) in &cards_data {
        let create_request = CreateCardRequest {
            zettel_id: format!("SEARCH-{:03}", created_card_ids.len() + 1),
            title: None,
            content: content.to_string(),
            topic_ids: vec![],
            links: None,
        };
        let card = card_service.create_card(create_request).await.unwrap();
        created_card_ids.push(card.id);
    }

    // Test search functionality
    let search_results = card_service.search_cards("mathematics").await.unwrap();
    assert_eq!(search_results.len(), 2); // Should find 2 cards with "mathematics"

    let search_results = card_service.search_cards("programming").await.unwrap();
    assert_eq!(search_results.len(), 2); // Should find 2 cards with "programming"

    let search_results = card_service.search_cards("physics").await.unwrap();
    assert_eq!(search_results.len(), 2); // Should find 2 cards with "physics"

    let search_results = card_service.search_cards("nonexistent").await.unwrap();
    assert_eq!(search_results.len(), 0); // Should find no cards

    // Test case-insensitive search
    let search_results = card_service.search_cards("MATHEMATICS").await.unwrap();
    assert_eq!(search_results.len(), 2); // Should still find 2 cards

    // Test partial word search
    let search_results = card_service.search_cards("math").await.unwrap();
    assert_eq!(search_results.len(), 2); // Should find cards containing "math"
}

#[tokio::test]
async fn test_error_handling_edge_cases() {
    let db = Database::new("sqlite::memory:").await.unwrap();
    let card_service = CardService::new(db);

    // Test updating with empty content
    let card = card_service
        .create_card(CreateCardRequest {
            zettel_id: "EDGE-001".to_string(),
            title: None,
            content: "Original".to_string(),
            topic_ids: vec![],
            links: None,
        })
        .await
        .unwrap();

    let update_request = UpdateCardRequest {
        zettel_id: Some("EDGE-002".to_string()),
        title: None,
        content: Some("".to_string()),
        topic_ids: None,
        links: None,
    };

    let updated = card_service
        .update_card(card.id, update_request)
        .await
        .unwrap();
    assert!(updated.is_some());
    assert_eq!(updated.unwrap().content, "");

    // Test with very long content
    let long_content = "a".repeat(10000);
    let update_request = UpdateCardRequest {
        zettel_id: Some("EDGE-003".to_string()),
        title: None,
        content: Some(long_content.clone()),
        topic_ids: None,
        links: None,
    };

    let updated = card_service
        .update_card(card.id, update_request)
        .await
        .unwrap();
    assert!(updated.is_some());
    assert_eq!(updated.unwrap().content, long_content);
}

#[tokio::test]
async fn test_ui_preview_functionality() {
    let db = Database::new("sqlite::memory:").await.unwrap();
    let card_service = CardService::new(db);

    // Test cards with content that would trigger preview mode (>100 characters)
    let long_content = "This is a very long card content that exceeds 100 characters and should trigger the preview functionality in the UI. ".repeat(2);

    let create_request = CreateCardRequest {
        zettel_id: "UI-PREVIEW-001".to_string(),
        title: None,
        content: long_content.clone(),
        topic_ids: vec![],
        links: None,
    };

    let card = card_service.create_card(create_request).await.unwrap();
    assert!(card.content.len() > 100);
    assert_eq!(card.content, long_content);

    // Test that content is properly stored and retrieved
    let retrieved_card = card_service.get_card(card.id).await.unwrap();
    assert!(retrieved_card.is_some());
    assert_eq!(retrieved_card.unwrap().content.len(), long_content.len());
}

#[tokio::test]
async fn test_keyboard_shortcut_ratings() {
    use learning_system::FSRSScheduler;

    let _scheduler = FSRSScheduler::new();

    // Test that all keyboard shortcut ratings (1-4) work correctly
    let rating_mappings = vec![(1, "Again"), (2, "Hard"), (3, "Good"), (4, "Easy")];

    for (rating_int, _rating_name) in rating_mappings {
        let rating = FSRSScheduler::get_rating_from_int(rating_int);
        assert!(rating.is_some(), "Rating {} should be valid", rating_int);
    }

    // Test invalid ratings
    let invalid_ratings = vec![0, 5, -1, 10];
    for invalid_rating in invalid_ratings {
        let rating = FSRSScheduler::get_rating_from_int(invalid_rating);
        assert!(
            rating.is_none(),
            "Rating {} should be invalid",
            invalid_rating
        );
    }
}

// Backlinks functionality tests
#[tokio::test]
async fn test_backlinks_creation() {
    let db = Database::new("sqlite::memory:").await.unwrap();
    let card_service = CardService::new(db);

    // Create three cards
    let card_a = card_service
        .create_card(CreateCardRequest {
            zettel_id: "BACKLINK-A".to_string(),
            title: None,
            content: "Card A".to_string(),
            topic_ids: vec![],
            links: None,
        })
        .await
        .unwrap();

    let card_b = card_service
        .create_card(CreateCardRequest {
            zettel_id: "BACKLINK-B".to_string(),
            title: None,
            content: "Card B".to_string(),
            topic_ids: vec![],
            links: None,
        })
        .await
        .unwrap();

    let card_c = card_service
        .create_card(CreateCardRequest {
            zettel_id: "BACKLINK-C".to_string(),
            title: None,
            content: "Card C".to_string(),
            topic_ids: vec![],
            links: None,
        })
        .await
        .unwrap();

    // Link card A to cards B and C
    let update_request = UpdateCardRequest {
        zettel_id: None,
        title: None,
        content: None,
        topic_ids: None,
        links: Some(vec![card_b.id, card_c.id]),
    };

    let updated_card_a = card_service
        .update_card(card_a.id, update_request)
        .await
        .unwrap();
    assert!(updated_card_a.is_some());

    // Test that forward links work
    let linked_cards = card_service.get_linked_cards(card_a.id).await.unwrap();
    assert_eq!(linked_cards.len(), 2);
    let linked_ids: Vec<Uuid> = linked_cards.iter().map(|c| c.id).collect();
    assert!(linked_ids.contains(&card_b.id));
    assert!(linked_ids.contains(&card_c.id));

    // Test that backlinks are created (this will require new functionality)
    let backlinks_b = card_service.get_backlinks(card_b.id).await.unwrap();
    assert_eq!(backlinks_b.len(), 1);
    assert_eq!(backlinks_b[0].id, card_a.id);

    let backlinks_c = card_service.get_backlinks(card_c.id).await.unwrap();
    assert_eq!(backlinks_c.len(), 1);
    assert_eq!(backlinks_c[0].id, card_a.id);

    // Card A should have no backlinks initially
    let backlinks_a = card_service.get_backlinks(card_a.id).await.unwrap();
    assert_eq!(backlinks_a.len(), 0);
}

#[tokio::test]
async fn test_backlinks_update_and_removal() {
    let db = Database::new("sqlite::memory:").await.unwrap();
    let card_service = CardService::new(db);

    // Create three cards
    let card_a = card_service
        .create_card(CreateCardRequest {
            zettel_id: "BACKLINK-UPDATE-A".to_string(),
            title: None,
            content: "Card A".to_string(),
            topic_ids: vec![],
            links: None,
        })
        .await
        .unwrap();

    let card_b = card_service
        .create_card(CreateCardRequest {
            zettel_id: "BACKLINK-UPDATE-B".to_string(),
            title: None,
            content: "Card B".to_string(),
            topic_ids: vec![],
            links: None,
        })
        .await
        .unwrap();

    let card_c = card_service
        .create_card(CreateCardRequest {
            zettel_id: "BACKLINK-UPDATE-C".to_string(),
            title: None,
            content: "Card C".to_string(),
            topic_ids: vec![],
            links: None,
        })
        .await
        .unwrap();

    // Initially link A to B
    let update_request = UpdateCardRequest {
        zettel_id: None,
        title: None,
        content: None,
        topic_ids: None,
        links: Some(vec![card_b.id]),
    };
    card_service
        .update_card(card_a.id, update_request)
        .await
        .unwrap();

    // Verify initial backlink
    let backlinks_b = card_service.get_backlinks(card_b.id).await.unwrap();
    assert_eq!(backlinks_b.len(), 1);
    assert_eq!(backlinks_b[0].id, card_a.id);

    // Update A to link to C instead of B
    let update_request = UpdateCardRequest {
        zettel_id: None,
        title: None,
        content: None,
        topic_ids: None,
        links: Some(vec![card_c.id]),
    };
    card_service
        .update_card(card_a.id, update_request)
        .await
        .unwrap();

    // Verify backlinks are updated
    let backlinks_b = card_service.get_backlinks(card_b.id).await.unwrap();
    assert_eq!(backlinks_b.len(), 0); // B should no longer have backlinks from A

    let backlinks_c = card_service.get_backlinks(card_c.id).await.unwrap();
    assert_eq!(backlinks_c.len(), 1);
    assert_eq!(backlinks_c[0].id, card_a.id); // C should now have backlink from A

    // Remove all links from A
    let update_request = UpdateCardRequest {
        zettel_id: None,
        title: None,
        content: None,
        topic_ids: None,
        links: Some(vec![]),
    };
    card_service
        .update_card(card_a.id, update_request)
        .await
        .unwrap();

    // Verify all backlinks are removed
    let backlinks_c = card_service.get_backlinks(card_c.id).await.unwrap();
    assert_eq!(backlinks_c.len(), 0);
}

#[tokio::test]
async fn test_backlinks_bidirectional_linking() {
    let db = Database::new("sqlite::memory:").await.unwrap();
    let card_service = CardService::new(db);

    // Create two cards
    let card_a = card_service
        .create_card(CreateCardRequest {
            zettel_id: "BIDIR-A".to_string(),
            title: None,
            content: "Card A".to_string(),
            topic_ids: vec![],
            links: None,
        })
        .await
        .unwrap();

    let card_b = card_service
        .create_card(CreateCardRequest {
            zettel_id: "BIDIR-B".to_string(),
            title: None,
            content: "Card B".to_string(),
            topic_ids: vec![],
            links: None,
        })
        .await
        .unwrap();

    // Link A to B
    let update_request = UpdateCardRequest {
        zettel_id: None,
        title: None,
        content: None,
        topic_ids: None,
        links: Some(vec![card_b.id]),
    };
    card_service
        .update_card(card_a.id, update_request)
        .await
        .unwrap();

    // Link B to A
    let update_request = UpdateCardRequest {
        zettel_id: None,
        title: None,
        content: None,
        topic_ids: None,
        links: Some(vec![card_a.id]),
    };
    card_service
        .update_card(card_b.id, update_request)
        .await
        .unwrap();

    // Both cards should have forward links and backlinks
    let forward_links_a = card_service.get_linked_cards(card_a.id).await.unwrap();
    assert_eq!(forward_links_a.len(), 1);
    assert_eq!(forward_links_a[0].id, card_b.id);

    let backlinks_a = card_service.get_backlinks(card_a.id).await.unwrap();
    assert_eq!(backlinks_a.len(), 1);
    assert_eq!(backlinks_a[0].id, card_b.id);

    let forward_links_b = card_service.get_linked_cards(card_b.id).await.unwrap();
    assert_eq!(forward_links_b.len(), 1);
    assert_eq!(forward_links_b[0].id, card_a.id);

    let backlinks_b = card_service.get_backlinks(card_b.id).await.unwrap();
    assert_eq!(backlinks_b.len(), 1);
    assert_eq!(backlinks_b[0].id, card_a.id);
}

#[tokio::test]
async fn test_backlinks_card_deletion() {
    let db = Database::new("sqlite::memory:").await.unwrap();
    let card_service = CardService::new(db);

    // Create two cards
    let card_a = card_service
        .create_card(CreateCardRequest {
            zettel_id: "DELETE-BACKLINK-A".to_string(),
            title: None,
            content: "Card A".to_string(),
            topic_ids: vec![],
            links: None,
        })
        .await
        .unwrap();

    let card_b = card_service
        .create_card(CreateCardRequest {
            zettel_id: "DELETE-BACKLINK-B".to_string(),
            title: None,
            content: "Card B".to_string(),
            topic_ids: vec![],
            links: None,
        })
        .await
        .unwrap();

    // Link A to B
    let update_request = UpdateCardRequest {
        zettel_id: None,
        title: None,
        content: None,
        topic_ids: None,
        links: Some(vec![card_b.id]),
    };
    card_service
        .update_card(card_a.id, update_request)
        .await
        .unwrap();

    // Verify backlink exists
    let backlinks_b = card_service.get_backlinks(card_b.id).await.unwrap();
    assert_eq!(backlinks_b.len(), 1);
    assert_eq!(backlinks_b[0].id, card_a.id);

    // Delete card A
    let deleted = card_service.delete_card(card_a.id).await.unwrap();
    assert!(deleted);

    // Verify backlinks to B are cleaned up
    let backlinks_b = card_service.get_backlinks(card_b.id).await.unwrap();
    assert_eq!(backlinks_b.len(), 0);

    // Verify card A is gone
    let card_a_check = card_service.get_card(card_a.id).await.unwrap();
    assert!(card_a_check.is_none());
}

#[tokio::test]
async fn test_backlinks_multiple_sources() {
    let db = Database::new("sqlite::memory:").await.unwrap();
    let card_service = CardService::new(db);

    // Create four cards
    let card_a = card_service
        .create_card(CreateCardRequest {
            zettel_id: "MULTI-SOURCE-A".to_string(),
            title: None,
            content: "Card A".to_string(),
            topic_ids: vec![],
            links: None,
        })
        .await
        .unwrap();

    let card_b = card_service
        .create_card(CreateCardRequest {
            zettel_id: "MULTI-SOURCE-B".to_string(),
            title: None,
            content: "Card B".to_string(),
            topic_ids: vec![],
            links: None,
        })
        .await
        .unwrap();

    let card_c = card_service
        .create_card(CreateCardRequest {
            zettel_id: "MULTI-SOURCE-C".to_string(),
            title: None,
            content: "Card C".to_string(),
            topic_ids: vec![],
            links: None,
        })
        .await
        .unwrap();

    let card_d = card_service
        .create_card(CreateCardRequest {
            zettel_id: "MULTI-SOURCE-D".to_string(),
            title: None,
            content: "Card D".to_string(),
            topic_ids: vec![],
            links: None,
        })
        .await
        .unwrap();

    // Link A to D, B to D, and C to D
    for (source_card, source_name) in [(card_a.id, "A"), (card_b.id, "B"), (card_c.id, "C")] {
        let update_request = UpdateCardRequest {
            zettel_id: None,
            title: None,
            content: None,
            topic_ids: None,
            links: Some(vec![card_d.id]),
        };
        card_service
            .update_card(source_card, update_request)
            .await
            .unwrap();

        // Verify link was created
        let linked_cards = card_service.get_linked_cards(source_card).await.unwrap();
        assert_eq!(
            linked_cards.len(),
            1,
            "Card {} should link to D",
            source_name
        );
        assert_eq!(linked_cards[0].id, card_d.id);
    }

    // Card D should have backlinks from A, B, and C
    let backlinks_d = card_service.get_backlinks(card_d.id).await.unwrap();
    assert_eq!(backlinks_d.len(), 3);

    let backlink_ids: Vec<Uuid> = backlinks_d.iter().map(|c| c.id).collect();
    assert!(backlink_ids.contains(&card_a.id));
    assert!(backlink_ids.contains(&card_b.id));
    assert!(backlink_ids.contains(&card_c.id));

    // Remove link from B to D
    let update_request = UpdateCardRequest {
        zettel_id: None,
        title: None,
        content: None,
        topic_ids: None,
        links: Some(vec![]),
    };
    card_service
        .update_card(card_b.id, update_request)
        .await
        .unwrap();

    // Card D should now have backlinks from only A and C
    let backlinks_d = card_service.get_backlinks(card_d.id).await.unwrap();
    assert_eq!(backlinks_d.len(), 2);

    let backlink_ids: Vec<Uuid> = backlinks_d.iter().map(|c| c.id).collect();
    assert!(backlink_ids.contains(&card_a.id));
    assert!(!backlink_ids.contains(&card_b.id));
    assert!(backlink_ids.contains(&card_c.id));
}

#[tokio::test]
async fn test_backlinks_with_zettel_linking() {
    let db = Database::new("sqlite::memory:").await.unwrap();
    let card_service = CardService::new(db);

    // Create cards using zettel linking functionality
    let card_a = card_service
        .create_card_with_zettel_links(CreateCardWithZettelLinksRequest {
            zettel_id: "ZETTEL-A".to_string(),
            title: None,
            content: "Card A".to_string(),
            topic_ids: vec![],
            zettel_links: None,
        })
        .await
        .unwrap();

    let card_b = card_service
        .create_card_with_zettel_links(CreateCardWithZettelLinksRequest {
            zettel_id: "ZETTEL-B".to_string(),
            title: None,
            content: "Card B".to_string(),
            topic_ids: vec![],
            zettel_links: None,
        })
        .await
        .unwrap();

    // Link A to B using zettel IDs
    let update_request = UpdateCardWithZettelLinksRequest {
        zettel_id: None,
        title: None,
        content: None,
        topic_ids: None,
        zettel_links: Some(vec!["ZETTEL-B".to_string()]),
    };
    card_service
        .update_card_with_zettel_links(card_a.id, update_request)
        .await
        .unwrap();

    // Verify forward link
    let linked_cards = card_service.get_linked_cards(card_a.id).await.unwrap();
    assert_eq!(linked_cards.len(), 1);
    assert_eq!(linked_cards[0].zettel_id, "ZETTEL-B");

    // Verify backlink
    let backlinks_b = card_service.get_backlinks(card_b.id).await.unwrap();
    assert_eq!(backlinks_b.len(), 1);
    assert_eq!(backlinks_b[0].zettel_id, "ZETTEL-A");
}

#[tokio::test]
async fn test_zettel_id_change_with_references() {
    let db = Database::new("sqlite::memory:").await.unwrap();
    let card_service = CardService::new(db);

    // Create a card with original Zettel ID
    let card_a = card_service
        .create_card(CreateCardRequest {
            zettel_id: "ORIGINAL-ID".to_string(),
            title: None,
            content: "This is the original card".to_string(),
            topic_ids: vec![],
            links: None,
        })
        .await
        .unwrap();

    // Create another card that references the first card in its content
    let _card_b = card_service
        .create_card(CreateCardRequest {
            zettel_id: "REFERENCING-CARD".to_string(),
            title: None,
            content: "This card references ORIGINAL-ID in its text content".to_string(),
            topic_ids: vec![],
            links: None,
        })
        .await
        .unwrap();

    // Create a third card that links to card A by UUID
    let update_request = UpdateCardRequest {
        zettel_id: None,
        title: None,
        content: None,
        topic_ids: None,
        links: Some(vec![card_a.id]),
    };
    let card_c = card_service
        .create_card(CreateCardRequest {
            zettel_id: "LINKING-CARD".to_string(),
            title: None,
            content: "This card has a UUID link to the original card".to_string(),
            topic_ids: vec![],
            links: None,
        })
        .await
        .unwrap();
    card_service
        .update_card(card_c.id, update_request)
        .await
        .unwrap();

    // Verify initial state
    let backlinks = card_service.get_backlinks(card_a.id).await.unwrap();
    assert_eq!(backlinks.len(), 1);
    assert_eq!(backlinks[0].id, card_c.id);

    // Change the Zettel ID of card A
    let update_request = UpdateCardRequest {
        zettel_id: Some("NEW-ID".to_string()),
        title: None,
        content: None,
        topic_ids: None,
        links: None,
    };
    let updated_card_a = card_service
        .update_card(card_a.id, update_request)
        .await
        .unwrap();
    assert!(updated_card_a.is_some());
    assert_eq!(updated_card_a.unwrap().zettel_id, "NEW-ID");

    // Verify that UUID-based links and backlinks still work correctly
    let backlinks_after_change = card_service.get_backlinks(card_a.id).await.unwrap();
    assert_eq!(backlinks_after_change.len(), 1);
    assert_eq!(backlinks_after_change[0].id, card_c.id);

    // Verify that forward links still work
    let forward_links = card_service.get_linked_cards(card_c.id).await.unwrap();
    assert_eq!(forward_links.len(), 1);
    assert_eq!(forward_links[0].id, card_a.id);
    assert_eq!(forward_links[0].zettel_id, "NEW-ID"); // Should reflect the new Zettel ID

    // Verify that the card can still be found by its new Zettel ID
    let card_by_new_id = card_service.get_card_by_zettel_id("NEW-ID").await.unwrap();
    assert!(card_by_new_id.is_some());
    assert_eq!(card_by_new_id.unwrap().id, card_a.id);

    // Verify that the old Zettel ID no longer resolves to the card
    let card_by_old_id = card_service
        .get_card_by_zettel_id("ORIGINAL-ID")
        .await
        .unwrap();
    assert!(card_by_old_id.is_none());
}

#[tokio::test]
async fn test_zettel_id_duplicate_prevention() {
    let db = Database::new("sqlite::memory:").await.unwrap();
    let card_service = CardService::new(db);

    // Create two cards with different Zettel IDs
    let card_a = card_service
        .create_card(CreateCardRequest {
            zettel_id: "CARD-A".to_string(),
            title: None,
            content: "Card A".to_string(),
            topic_ids: vec![],
            links: None,
        })
        .await
        .unwrap();

    let _card_b = card_service
        .create_card(CreateCardRequest {
            zettel_id: "CARD-B".to_string(),
            title: None,
            content: "Card B".to_string(),
            topic_ids: vec![],
            links: None,
        })
        .await
        .unwrap();

    // Try to change card A's Zettel ID to card B's Zettel ID (should fail)
    let update_request = UpdateCardRequest {
        zettel_id: Some("CARD-B".to_string()),
        title: None,
        content: None,
        topic_ids: None,
        links: None,
    };
    let result = card_service.update_card(card_a.id, update_request).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));

    // Verify that card A's Zettel ID hasn't changed
    let card_a_check = card_service.get_card(card_a.id).await.unwrap();
    assert!(card_a_check.is_some());
    assert_eq!(card_a_check.unwrap().zettel_id, "CARD-A");
}

#[tokio::test]
async fn test_zettel_id_change_preserves_all_relationships() {
    let db = Database::new("sqlite::memory:").await.unwrap();
    let card_service = CardService::new(db);

    // Create a complex linking scenario
    let card_a = card_service
        .create_card(CreateCardRequest {
            zettel_id: "CENTRAL-CARD".to_string(),
            title: None,
            content: "Central card with many relationships".to_string(),
            topic_ids: vec![],
            links: None,
        })
        .await
        .unwrap();

    let card_b = card_service
        .create_card(CreateCardRequest {
            zettel_id: "LINKED-CARD-1".to_string(),
            title: None,
            content: "Card 1".to_string(),
            topic_ids: vec![],
            links: None,
        })
        .await
        .unwrap();

    let card_c = card_service
        .create_card(CreateCardRequest {
            zettel_id: "LINKED-CARD-2".to_string(),
            title: None,
            content: "Card 2".to_string(),
            topic_ids: vec![],
            links: None,
        })
        .await
        .unwrap();

    // Set up bidirectional links
    // A links to B and C
    let update_request = UpdateCardRequest {
        zettel_id: None,
        title: None,
        content: None,
        topic_ids: None,
        links: Some(vec![card_b.id, card_c.id]),
    };
    card_service
        .update_card(card_a.id, update_request)
        .await
        .unwrap();

    // B links to A
    let update_request = UpdateCardRequest {
        zettel_id: None,
        title: None,
        content: None,
        topic_ids: None,
        links: Some(vec![card_a.id]),
    };
    card_service
        .update_card(card_b.id, update_request)
        .await
        .unwrap();

    // Verify initial state
    let forward_links_a = card_service.get_linked_cards(card_a.id).await.unwrap();
    assert_eq!(forward_links_a.len(), 2);
    let backlinks_a = card_service.get_backlinks(card_a.id).await.unwrap();
    assert_eq!(backlinks_a.len(), 1);
    let backlinks_b = card_service.get_backlinks(card_b.id).await.unwrap();
    assert_eq!(backlinks_b.len(), 1);
    let backlinks_c = card_service.get_backlinks(card_c.id).await.unwrap();
    assert_eq!(backlinks_c.len(), 1);

    // Change the Zettel ID of the central card
    let update_request = UpdateCardRequest {
        zettel_id: Some("NEW-CENTRAL-CARD".to_string()),
        title: None,
        content: None,
        topic_ids: None,
        links: None,
    };
    card_service
        .update_card(card_a.id, update_request)
        .await
        .unwrap();

    // Verify all relationships are preserved
    let forward_links_a_after = card_service.get_linked_cards(card_a.id).await.unwrap();
    assert_eq!(forward_links_a_after.len(), 2);
    let forward_link_ids: Vec<Uuid> = forward_links_a_after.iter().map(|c| c.id).collect();
    assert!(forward_link_ids.contains(&card_b.id));
    assert!(forward_link_ids.contains(&card_c.id));

    let backlinks_a_after = card_service.get_backlinks(card_a.id).await.unwrap();
    assert_eq!(backlinks_a_after.len(), 1);
    assert_eq!(backlinks_a_after[0].id, card_b.id);

    let backlinks_b_after = card_service.get_backlinks(card_b.id).await.unwrap();
    assert_eq!(backlinks_b_after.len(), 1);
    assert_eq!(backlinks_b_after[0].id, card_a.id);
    assert_eq!(backlinks_b_after[0].zettel_id, "NEW-CENTRAL-CARD"); // Should show new ID

    let backlinks_c_after = card_service.get_backlinks(card_c.id).await.unwrap();
    assert_eq!(backlinks_c_after.len(), 1);
    assert_eq!(backlinks_c_after[0].id, card_a.id);
    assert_eq!(backlinks_c_after[0].zettel_id, "NEW-CENTRAL-CARD"); // Should show new ID
}
