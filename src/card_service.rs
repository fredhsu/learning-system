use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;

use crate::database::Database;
use crate::fsrs_scheduler::FSRSScheduler;
use crate::models::*;

#[derive(Clone)]
pub struct CardService {
    db: Database,
    scheduler: FSRSScheduler,
}

impl CardService {
    pub fn new(db: Database) -> Self {
        Self {
            db,
            scheduler: FSRSScheduler::new(),
        }
    }

    // Card CRUD operations
    pub async fn create_card(&self, request: CreateCardRequest) -> Result<Card> {
        let card = self.db.create_card(request).await?;
        
        // Create backlinks for any initial links, but only for existing cards
        if let Some(ref links_json) = card.links {
            let links: Vec<Uuid> = serde_json::from_str(links_json).unwrap_or_default();
            if !links.is_empty() {
                // Filter out non-existent cards before creating backlinks
                let mut valid_links = Vec::new();
                for link_id in links {
                    if self.db.get_card(link_id).await?.is_some() {
                        valid_links.push(link_id);
                    }
                }
                if !valid_links.is_empty() {
                    self.db.create_backlinks(card.id, &valid_links).await?;
                }
            }
        }
        
        Ok(card)
    }

    pub async fn create_card_with_zettel_links(&self, request: CreateCardWithZettelLinksRequest) -> Result<Card> {
        let links = if let Some(zettel_links) = request.zettel_links {
            Some(self.resolve_zettel_ids_to_uuids(&zettel_links).await?)
        } else {
            None
        };

        let create_request = CreateCardRequest {
            zettel_id: request.zettel_id,
            content: request.content,
            topic_ids: request.topic_ids,
            links,
        };

        self.create_card(create_request).await // Use our own method to handle backlinks
    }

    pub async fn get_card(&self, id: Uuid) -> Result<Option<Card>> {
        self.db.get_card(id).await
    }

    pub async fn get_card_by_zettel_id(&self, zettel_id: &str) -> Result<Option<Card>> {
        self.db.get_card_by_zettel_id(zettel_id).await
    }

    pub async fn update_card(&self, id: Uuid, request: UpdateCardRequest) -> Result<Option<Card>> {
        let mut card = match self.db.get_card(id).await? {
            Some(card) => card,
            None => return Ok(None),
        };

        // Get old links for backlink maintenance
        let old_links: Vec<Uuid> = if let Some(ref links_json) = card.links {
            serde_json::from_str(links_json).unwrap_or_default()
        } else {
            Vec::new()
        };

        if let Some(zettel_id) = request.zettel_id {
            // Validate that the new zettel_id doesn't already exist (unless it's the same card)
            if let Some(existing) = self.db.get_card_by_zettel_id(&zettel_id).await? {
                if existing.id != card.id {
                    return Err(anyhow::anyhow!("Zettelkasten ID '{}' already exists", zettel_id));
                }
            }
            card.zettel_id = zettel_id;
        }

        if let Some(content) = request.content {
            card.content = content;
        }

        let mut new_links = old_links.clone();
        let mut links_changed = false;
        if let Some(links) = request.links {
            new_links = links;
            card.links = Some(serde_json::to_string(&new_links)?);
            links_changed = true;
        }

        // Update the card in database
        self.db.update_card_content(&card).await?;

        // Update backlinks if links changed, filtering out non-existent targets
        if links_changed {
            let mut valid_new_links = Vec::new();
            for link_id in &new_links {
                if self.db.get_card(*link_id).await?.is_some() {
                    valid_new_links.push(*link_id);
                }
            }
            self.db.update_backlinks(card.id, &old_links, &valid_new_links).await?;
        }

        // Handle topic updates if provided
        if let Some(_topic_ids) = request.topic_ids {
            // This would require additional methods in Database to handle topic updates
            // For now, we'll skip this part but it should be implemented
        }

        Ok(Some(card))
    }

    pub async fn update_card_with_zettel_links(&self, id: Uuid, request: UpdateCardWithZettelLinksRequest) -> Result<Option<Card>> {
        let links = if let Some(zettel_links) = request.zettel_links {
            Some(self.resolve_zettel_ids_to_uuids(&zettel_links).await?)
        } else {
            None
        };

        let update_request = UpdateCardRequest {
            zettel_id: request.zettel_id,
            content: request.content,
            topic_ids: request.topic_ids,
            links,
        };

        self.update_card(id, update_request).await
    }

    pub async fn delete_card(&self, id: Uuid) -> Result<bool> {
        self.db.delete_card(id).await
    }

    // Topic operations
    pub async fn create_topic(&self, name: String, description: Option<String>) -> Result<Topic> {
        self.db.create_topic(name, description).await
    }

    pub async fn get_all_topics(&self) -> Result<Vec<Topic>> {
        self.db.get_all_topics().await
    }

    pub async fn get_all_cards(&self) -> Result<Vec<Card>> {
        self.db.get_all_cards().await
    }

    // Review operations
    pub async fn get_cards_due_for_review(&self) -> Result<Vec<Card>> {
        self.db.get_cards_due_for_review().await
    }

    pub async fn review_card(&self, card_id: Uuid, rating: i32) -> Result<Option<Card>> {
        let card = match self.db.get_card(card_id).await? {
            Some(card) => card,
            None => return Ok(None),
        };

        let fsrs_rating = match FSRSScheduler::get_rating_from_int(rating) {
            Some(rating) => rating,
            None => return Err(anyhow::anyhow!("Invalid rating: {}", rating)),
        };

        let now = Utc::now();
        let (updated_card, review_log) = self.scheduler.schedule_card(&card, fsrs_rating, now)?;

        // Update the card in the database
        self.db.update_card_after_review(&updated_card).await?;

        // Create a review record
        self.db.create_review(
            card_id,
            rating,
            review_log.scheduled_days as f64,
            1.0, // Placeholder ease factor, FSRS doesn't use traditional ease factor
        ).await?;

        Ok(Some(updated_card))
    }

    #[allow(dead_code)]
    pub async fn get_cards_by_topic(&self, _topic_id: Uuid) -> Result<Vec<Card>> {
        // This would require additional database methods
        // For now, return empty vector
        Ok(Vec::new())
    }

    pub async fn search_cards(&self, search_query: &str) -> Result<Vec<Card>> {
        self.db.search_cards(search_query).await
    }

    pub async fn get_linked_cards(&self, card_id: Uuid) -> Result<Vec<Card>> {
        let card = match self.db.get_card(card_id).await? {
            Some(card) => card,
            None => return Ok(Vec::new()),
        };

        if let Some(links_json) = card.links {
            let link_ids: Vec<Uuid> = serde_json::from_str(&links_json)?;
            let mut linked_cards = Vec::new();
            
            for link_id in link_ids {
                if let Some(linked_card) = self.db.get_card(link_id).await? {
                    linked_cards.push(linked_card);
                }
            }
            
            Ok(linked_cards)
        } else {
            Ok(Vec::new())
        }
    }

    pub async fn get_backlinks(&self, card_id: Uuid) -> Result<Vec<Card>> {
        self.db.get_backlinks(card_id).await
    }

    pub async fn resolve_zettel_ids_to_uuids(&self, zettel_ids: &[String]) -> Result<Vec<Uuid>> {
        let mut uuids = Vec::new();
        
        for zettel_id in zettel_ids {
            let zettel_id = zettel_id.trim();
            if zettel_id.is_empty() {
                continue;
            }
            
            match self.db.get_card_by_zettel_id(zettel_id).await? {
                Some(card) => uuids.push(card.id),
                None => return Err(anyhow::anyhow!("Card with Zettel ID '{}' not found", zettel_id)),
            }
        }
        
        Ok(uuids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Database;

    async fn create_test_service() -> CardService {
        let db = Database::new("sqlite::memory:").await.unwrap();
        CardService::new(db)
    }

    #[tokio::test]
    async fn test_card_service_creation() {
        let service = create_test_service().await;
        let cards = service.get_all_cards().await.unwrap();
        assert_eq!(cards.len(), 0);
    }

    #[tokio::test]
    async fn test_card_service_crud_operations() {
        let service = create_test_service().await;

        // Create
        let create_request = CreateCardRequest {
            zettel_id: "SERVICE-TEST-001".to_string(),
            content: "Service test card".to_string(),
            topic_ids: vec![],
            links: None,
        };

        let created_card = service.create_card(create_request).await.unwrap();
        assert_eq!(created_card.content, "Service test card");

        // Read
        let retrieved_card = service.get_card(created_card.id).await.unwrap();
        assert!(retrieved_card.is_some());
        assert_eq!(retrieved_card.as_ref().unwrap().content, "Service test card");

        // Update
        let update_request = UpdateCardRequest {
            zettel_id: Some("SERVICE-TEST-002".to_string()),
            content: Some("Updated service test card".to_string()),
            topic_ids: None,
            links: None,
        };

        let updated_card = service.update_card(created_card.id, update_request).await.unwrap();
        assert!(updated_card.is_some());
        assert_eq!(updated_card.unwrap().content, "Updated service test card");

        // Delete
        let deleted = service.delete_card(created_card.id).await.unwrap();
        assert!(deleted);

        let retrieved_after_delete = service.get_card(created_card.id).await.unwrap();
        assert!(retrieved_after_delete.is_none());
    }

    #[tokio::test]
    async fn test_card_service_update_partial() {
        let service = create_test_service().await;

        let create_request = CreateCardRequest {
            zettel_id: "SERVICE-TEST-003".to_string(),
            content: "Original".to_string(),
            topic_ids: vec![],
            links: None,
        };

        let card = service.create_card(create_request).await.unwrap();

        // Update only content, leave links unchanged
        let update_request = UpdateCardRequest {
            zettel_id: Some("SERVICE-TEST-004".to_string()),
            content: Some("Updated".to_string()),
            topic_ids: None,
            links: None, // This should not change the existing links value
        };

        let updated = service.update_card(card.id, update_request).await.unwrap();
        assert!(updated.is_some());
        assert_eq!(updated.unwrap().content, "Updated");
    }

    #[tokio::test]
    async fn test_card_service_linked_cards() {
        let service = create_test_service().await;

        // Create two cards
        let card1 = service.create_card(CreateCardRequest {
            zettel_id: "SERVICE-TEST-005".to_string(),
            content: "Card 1".to_string(),
            topic_ids: vec![],
            links: None,
        }).await.unwrap();

        let card2 = service.create_card(CreateCardRequest {
            zettel_id: "SERVICE-TEST-006".to_string(),
            content: "Card 2".to_string(),
            topic_ids: vec![],
            links: None,
        }).await.unwrap();

        // Link card1 to card2
        let update_request = UpdateCardRequest {
            zettel_id: Some("SERVICE-TEST-007".to_string()),
            content: None,
            topic_ids: None,
            links: Some(vec![card2.id]),
        };

        service.update_card(card1.id, update_request).await.unwrap();

        // Test getting linked cards
        let linked_cards = service.get_linked_cards(card1.id).await.unwrap();
        assert_eq!(linked_cards.len(), 1);
        assert_eq!(linked_cards[0].id, card2.id);

        // Test getting linked cards for card without links
        let no_links = service.get_linked_cards(card2.id).await.unwrap();
        assert_eq!(no_links.len(), 0);
    }

    #[tokio::test]
    async fn test_card_service_review_functionality() {
        let service = create_test_service().await;

        let card = service.create_card(CreateCardRequest {
            zettel_id: "SERVICE-TEST-008".to_string(),
            content: "Review test".to_string(),
            topic_ids: vec![],
            links: None,
        }).await.unwrap();

        // Initially card should be due for review
        let due_cards = service.get_cards_due_for_review().await.unwrap();
        assert_eq!(due_cards.len(), 1);

        // Review the card
        let reviewed = service.review_card(card.id, 3).await.unwrap();
        assert!(reviewed.is_some());
        let reviewed = reviewed.unwrap();
        assert_eq!(reviewed.reps, 1);
        assert!(reviewed.next_review > card.next_review);
    }

    #[tokio::test]
    async fn test_card_service_topics() {
        let service = create_test_service().await;

        // Create a topic
        let topic = service.create_topic("Test Topic".to_string(), None).await.unwrap();
        assert_eq!(topic.name, "Test Topic");

        // Get all topics
        let topics = service.get_all_topics().await.unwrap();
        assert_eq!(topics.len(), 1);
        assert_eq!(topics[0].name, "Test Topic");
    }

    #[tokio::test]
    async fn test_card_service_zettel_id_linking() {
        let service = create_test_service().await;

        // Create three cards
        let card1 = service.create_card(CreateCardRequest {
            zettel_id: "1.1".to_string(),
            content: "Card 1 content".to_string(),
            topic_ids: vec![],
            links: None,
        }).await.unwrap();

        let _card2 = service.create_card(CreateCardRequest {
            zettel_id: "1.2".to_string(),
            content: "Card 2 content".to_string(),
            topic_ids: vec![],
            links: None,
        }).await.unwrap();

        let _card3 = service.create_card(CreateCardRequest {
            zettel_id: "2.1".to_string(),
            content: "Card 3 content".to_string(),
            topic_ids: vec![],
            links: None,
        }).await.unwrap();

        // Test linking by Zettel IDs (this will be the new functionality)
        let zettel_links = vec!["1.2".to_string(), "2.1".to_string()];
        let link_ids = service.resolve_zettel_ids_to_uuids(&zettel_links).await.unwrap();
        
        // Update card1 to link to cards 2 and 3 by Zettel ID
        let update_request = UpdateCardRequest {
            zettel_id: None,
            content: None,
            topic_ids: None,
            links: Some(link_ids),
        };

        service.update_card(card1.id, update_request).await.unwrap();

        // Test getting linked cards
        let linked_cards = service.get_linked_cards(card1.id).await.unwrap();
        assert_eq!(linked_cards.len(), 2);
        
        let linked_zettel_ids: Vec<String> = linked_cards.iter().map(|c| c.zettel_id.clone()).collect();
        assert!(linked_zettel_ids.contains(&"1.2".to_string()));
        assert!(linked_zettel_ids.contains(&"2.1".to_string()));
    }

    #[tokio::test]
    async fn test_resolve_zettel_ids_invalid() {
        let service = create_test_service().await;

        // Test with nonexistent Zettel IDs
        let invalid_zettel_ids = vec!["999.999".to_string(), "invalid-id".to_string()];
        let result = service.resolve_zettel_ids_to_uuids(&invalid_zettel_ids).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_resolve_zettel_ids_mixed() {
        let service = create_test_service().await;

        // Create one valid card
        let _card = service.create_card(CreateCardRequest {
            zettel_id: "valid.1".to_string(),
            content: "Valid card".to_string(),
            topic_ids: vec![],
            links: None,
        }).await.unwrap();

        // Test with mix of valid and invalid Zettel IDs
        let mixed_zettel_ids = vec!["valid.1".to_string(), "invalid.999".to_string()];
        let result = service.resolve_zettel_ids_to_uuids(&mixed_zettel_ids).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_card_service_nonexistent_operations() {
        let service = create_test_service().await;
        let fake_id = Uuid::new_v4();

        // Test getting nonexistent card
        let card = service.get_card(fake_id).await.unwrap();
        assert!(card.is_none());

        // Test updating nonexistent card
        let update_request = UpdateCardRequest {
            zettel_id: Some("SERVICE-TEST-009".to_string()),
            content: Some("Should fail".to_string()),
            topic_ids: None,
            links: None,
        };
        let updated = service.update_card(fake_id, update_request).await.unwrap();
        assert!(updated.is_none());

        // Test deleting nonexistent card
        let deleted = service.delete_card(fake_id).await.unwrap();
        assert!(!deleted);

        // Test reviewing nonexistent card
        let reviewed = service.review_card(fake_id, 3).await.unwrap();
        assert!(reviewed.is_none());

        // Test getting linked cards for nonexistent card
        let linked = service.get_linked_cards(fake_id).await.unwrap();
        assert_eq!(linked.len(), 0);
    }
}