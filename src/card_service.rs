use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;

use crate::database::Database;
use crate::fsrs_scheduler::{FSRSScheduler, Rating};
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
        self.db.create_card(request).await
    }

    pub async fn get_card(&self, id: Uuid) -> Result<Option<Card>> {
        self.db.get_card(id).await
    }

    pub async fn update_card(&self, id: Uuid, request: UpdateCardRequest) -> Result<Option<Card>> {
        let mut card = match self.db.get_card(id).await? {
            Some(card) => card,
            None => return Ok(None),
        };

        if let Some(content) = request.content {
            card.content = content;
        }

        if let Some(links) = request.links {
            card.links = Some(serde_json::to_string(&links)?);
        }

        // Update the card in database
        self.db.update_card_after_review(&card).await?;

        // Handle topic updates if provided
        if let Some(topic_ids) = request.topic_ids {
            // This would require additional methods in Database to handle topic updates
            // For now, we'll skip this part but it should be implemented
        }

        Ok(Some(card))
    }

    pub async fn delete_card(&self, id: Uuid) -> Result<bool> {
        // This would require a delete method in the Database
        // For now, we'll return a placeholder
        Ok(false)
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

    pub async fn get_cards_by_topic(&self, topic_id: Uuid) -> Result<Vec<Card>> {
        // This would require additional database methods
        // For now, return empty vector
        Ok(Vec::new())
    }

    pub async fn search_cards(&self, query: &str) -> Result<Vec<Card>> {
        // This would require full-text search in the database
        // For now, return empty vector
        Ok(Vec::new())
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
}