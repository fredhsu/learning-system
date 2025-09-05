use anyhow::Result;
use chrono::Utc;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::models::*;

#[derive(Clone)]
pub struct Database {
    pub pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(database_url).await?;
        let db = Database { pool };
        db.migrate().await?;
        Ok(db)
    }

    async fn migrate(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS cards (
                id TEXT PRIMARY KEY,
                zettel_id TEXT NOT NULL UNIQUE,
                title TEXT,
                content TEXT NOT NULL,
                creation_date TEXT NOT NULL,
                last_reviewed TEXT,
                next_review TEXT NOT NULL,
                difficulty REAL NOT NULL DEFAULT 0.0,
                stability REAL NOT NULL DEFAULT 0.0,
                retrievability REAL NOT NULL DEFAULT 0.0,
                reps INTEGER NOT NULL DEFAULT 0,
                lapses INTEGER NOT NULL DEFAULT 0,
                state TEXT NOT NULL DEFAULT 'New',
                links TEXT
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Add title column to existing tables if it doesn't exist
        sqlx::query(
            "ALTER TABLE cards ADD COLUMN title TEXT"
        )
        .execute(&self.pool)
        .await
        .ok(); // Ignore error if column already exists

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS topics (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                description TEXT
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS card_topics (
                card_id TEXT NOT NULL,
                topic_id TEXT NOT NULL,
                PRIMARY KEY (card_id, topic_id),
                FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE,
                FOREIGN KEY (topic_id) REFERENCES topics(id) ON DELETE CASCADE
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS reviews (
                id TEXT PRIMARY KEY,
                card_id TEXT NOT NULL,
                review_date TEXT NOT NULL,
                rating INTEGER NOT NULL,
                interval REAL NOT NULL,
                ease_factor REAL NOT NULL,
                FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS backlinks (
                source_card_id TEXT NOT NULL,
                target_card_id TEXT NOT NULL,
                PRIMARY KEY (source_card_id, target_card_id),
                FOREIGN KEY (source_card_id) REFERENCES cards(id) ON DELETE CASCADE,
                FOREIGN KEY (target_card_id) REFERENCES cards(id) ON DELETE CASCADE
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }


    // Card operations
    pub async fn create_card(&self, request: CreateCardRequest) -> Result<Card> {
        let card_id = Uuid::new_v4();
        let now = Utc::now();
        let links_json = request.links.as_ref().map(|l| serde_json::to_string(l).unwrap());

        // Validate that zettel_id doesn't already exist
        let existing = sqlx::query("SELECT id FROM cards WHERE zettel_id = ?1")
            .bind(&request.zettel_id)
            .fetch_optional(&self.pool)
            .await?;
        
        if existing.is_some() {
            return Err(anyhow::anyhow!("Zettelkasten ID '{}' already exists", request.zettel_id));
        }

        let card = Card {
            id: card_id,
            zettel_id: request.zettel_id,
            title: request.title,
            content: request.content,
            creation_date: now,
            last_reviewed: None,
            next_review: now,
            difficulty: 0.0,
            stability: 0.0,
            retrievability: 0.0,
            reps: 0,
            lapses: 0,
            state: "New".to_string(),
            links: links_json,
        };

        sqlx::query(
            r#"
            INSERT INTO cards (id, zettel_id, title, content, creation_date, last_reviewed, next_review, 
                             difficulty, stability, retrievability, reps, lapses, state, links)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
            "#,
        )
        .bind(card.id.to_string())
        .bind(&card.zettel_id)
        .bind(&card.title)
        .bind(&card.content)
        .bind(card.creation_date.to_rfc3339())
        .bind(card.last_reviewed.map(|d| d.to_rfc3339()))
        .bind(card.next_review.to_rfc3339())
        .bind(card.difficulty)
        .bind(card.stability)
        .bind(card.retrievability)
        .bind(card.reps)
        .bind(card.lapses)
        .bind(&card.state)
        .bind(&card.links)
        .execute(&self.pool)
        .await?;

        // Associate with topics
        for topic_id in request.topic_ids {
            sqlx::query(
                "INSERT INTO card_topics (card_id, topic_id) VALUES (?1, ?2)"
            )
            .bind(card.id.to_string())
            .bind(topic_id.to_string())
            .execute(&self.pool)
            .await?;
        }

        Ok(card)
    }

    pub async fn get_card(&self, id: Uuid) -> Result<Option<Card>> {
        let row = sqlx::query(
            "SELECT * FROM cards WHERE id = ?1"
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(Card {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                zettel_id: row.get("zettel_id"),
                title: row.get("title"),
                content: row.get("content"),
                creation_date: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("creation_date"))?.with_timezone(&Utc),
                last_reviewed: row.get::<Option<String>, _>("last_reviewed")
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
                next_review: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("next_review"))?.with_timezone(&Utc),
                difficulty: row.get("difficulty"),
                stability: row.get("stability"),
                retrievability: row.get("retrievability"),
                reps: row.get("reps"),
                lapses: row.get("lapses"),
                state: row.get("state"),
                links: row.get("links"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_all_cards(&self) -> Result<Vec<Card>> {
        let rows = sqlx::query("SELECT * FROM cards ORDER BY creation_date DESC")
            .fetch_all(&self.pool)
            .await?;

        self.rows_to_cards(rows)
    }

    pub async fn get_cards_due_for_review(&self) -> Result<Vec<Card>> {
        let now = Utc::now().to_rfc3339();
        let rows = sqlx::query(
            "SELECT * FROM cards WHERE next_review <= ?1 ORDER BY next_review ASC"
        )
        .bind(now)
        .fetch_all(&self.pool)
        .await?;

        self.rows_to_cards(rows)
    }

    fn rows_to_cards(&self, rows: Vec<sqlx::sqlite::SqliteRow>) -> Result<Vec<Card>> {
        let mut cards = Vec::new();
        for row in rows {
            cards.push(Card {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                zettel_id: row.get("zettel_id"),
                title: row.get("title"),
                content: row.get("content"),
                creation_date: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("creation_date"))?.with_timezone(&Utc),
                last_reviewed: row.get::<Option<String>, _>("last_reviewed")
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
                next_review: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("next_review"))?.with_timezone(&Utc),
                difficulty: row.get("difficulty"),
                stability: row.get("stability"),
                retrievability: row.get("retrievability"),
                reps: row.get("reps"),
                lapses: row.get("lapses"),
                state: row.get("state"),
                links: row.get("links"),
            });
        }

        Ok(cards)
    }

    pub async fn update_card_after_review(&self, card: &Card) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE cards 
            SET last_reviewed = ?1, next_review = ?2, difficulty = ?3, 
                stability = ?4, retrievability = ?5, reps = ?6, lapses = ?7, state = ?8
            WHERE id = ?9
            "#,
        )
        .bind(card.last_reviewed.map(|d| d.to_rfc3339()))
        .bind(card.next_review.to_rfc3339())
        .bind(card.difficulty)
        .bind(card.stability)
        .bind(card.retrievability)
        .bind(card.reps)
        .bind(card.lapses)
        .bind(&card.state)
        .bind(card.id.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Topic operations
    pub async fn create_topic(&self, name: String, description: Option<String>) -> Result<Topic> {
        let topic = Topic {
            id: Uuid::new_v4(),
            name,
            description,
        };

        sqlx::query(
            "INSERT INTO topics (id, name, description) VALUES (?1, ?2, ?3)"
        )
        .bind(topic.id.to_string())
        .bind(&topic.name)
        .bind(&topic.description)
        .execute(&self.pool)
        .await?;

        Ok(topic)
    }

    pub async fn get_all_topics(&self) -> Result<Vec<Topic>> {
        let rows = sqlx::query("SELECT * FROM topics ORDER BY name")
            .fetch_all(&self.pool)
            .await?;

        let mut topics = Vec::new();
        for row in rows {
            topics.push(Topic {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                name: row.get("name"),
                description: row.get("description"),
            });
        }

        Ok(topics)
    }

    // Review operations
    pub async fn create_review(&self, card_id: Uuid, rating: i32, interval: f64, ease_factor: f64) -> Result<Review> {
        let review = Review {
            id: Uuid::new_v4(),
            card_id,
            review_date: Utc::now(),
            rating,
            interval,
            ease_factor,
        };

        sqlx::query(
            "INSERT INTO reviews (id, card_id, review_date, rating, interval, ease_factor) VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
        )
        .bind(review.id.to_string())
        .bind(review.card_id.to_string())
        .bind(review.review_date.to_rfc3339())
        .bind(review.rating)
        .bind(review.interval)
        .bind(review.ease_factor)
        .execute(&self.pool)
        .await?;

        Ok(review)
    }

    pub async fn delete_card(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM cards WHERE id = ?1")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn update_card_content(&self, card: &Card) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE cards 
            SET zettel_id = ?1, title = ?2, content = ?3, links = ?4
            WHERE id = ?5
            "#,
        )
        .bind(&card.zettel_id)
        .bind(&card.title)
        .bind(&card.content)
        .bind(&card.links)
        .bind(card.id.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_card_by_zettel_id(&self, zettel_id: &str) -> Result<Option<Card>> {
        let row = sqlx::query(
            "SELECT * FROM cards WHERE zettel_id = ?1"
        )
        .bind(zettel_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(Card {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                zettel_id: row.get("zettel_id"),
                title: row.get("title"),
                content: row.get("content"),
                creation_date: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("creation_date"))?.with_timezone(&Utc),
                last_reviewed: row.get::<Option<String>, _>("last_reviewed")
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
                next_review: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("next_review"))?.with_timezone(&Utc),
                difficulty: row.get("difficulty"),
                stability: row.get("stability"),
                retrievability: row.get("retrievability"),
                reps: row.get("reps"),
                lapses: row.get("lapses"),
                state: row.get("state"),
                links: row.get("links"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn search_cards(&self, search_query: &str) -> Result<Vec<Card>> {
        // If empty query, return all cards
        if search_query.trim().is_empty() {
            return self.get_all_cards().await;
        }

        // Use LIKE query for case-insensitive search
        let query_pattern = format!("%{}%", search_query.to_lowercase());
        let rows = sqlx::query(
            "SELECT * FROM cards WHERE LOWER(content) LIKE ?1 ORDER BY creation_date DESC"
        )
        .bind(&query_pattern)
        .fetch_all(&self.pool)
        .await?;

        self.rows_to_cards(rows)
    }

    // Backlinks operations
    pub async fn create_backlinks(&self, source_card_id: Uuid, target_card_ids: &[Uuid]) -> Result<()> {
        for target_card_id in target_card_ids {
            sqlx::query(
                "INSERT OR IGNORE INTO backlinks (source_card_id, target_card_id) VALUES (?1, ?2)"
            )
            .bind(source_card_id.to_string())
            .bind(target_card_id.to_string())
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    pub async fn remove_backlinks(&self, source_card_id: Uuid, target_card_ids: &[Uuid]) -> Result<()> {
        for target_card_id in target_card_ids {
            sqlx::query(
                "DELETE FROM backlinks WHERE source_card_id = ?1 AND target_card_id = ?2"
            )
            .bind(source_card_id.to_string())
            .bind(target_card_id.to_string())
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn remove_all_backlinks_from_source(&self, source_card_id: Uuid) -> Result<()> {
        sqlx::query(
            "DELETE FROM backlinks WHERE source_card_id = ?1"
        )
        .bind(source_card_id.to_string())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_backlinks(&self, target_card_id: Uuid) -> Result<Vec<Card>> {
        let rows = sqlx::query(
            r#"
            SELECT c.* FROM cards c
            INNER JOIN backlinks b ON c.id = b.source_card_id
            WHERE b.target_card_id = ?1
            ORDER BY c.creation_date DESC
            "#
        )
        .bind(target_card_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        self.rows_to_cards(rows)
    }

    pub async fn update_backlinks(&self, source_card_id: Uuid, old_target_ids: &[Uuid], new_target_ids: &[Uuid]) -> Result<()> {
        // Remove old backlinks
        if !old_target_ids.is_empty() {
            self.remove_backlinks(source_card_id, old_target_ids).await?;
        }

        // Add new backlinks
        if !new_target_ids.is_empty() {
            self.create_backlinks(source_card_id, new_target_ids).await?;
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn get_cards_linking_to(&self, target_card_id: Uuid) -> Result<Vec<Card>> {
        let rows = sqlx::query(
            "SELECT * FROM cards WHERE links IS NOT NULL AND links != '[]'"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut linking_cards = Vec::new();
        for row in rows {
            if let Some(links_json) = row.get::<Option<String>, _>("links") {
                if let Ok(link_ids) = serde_json::from_str::<Vec<Uuid>>(&links_json) {
                    if link_ids.contains(&target_card_id) {
                        linking_cards.push(Card {
                            id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                            zettel_id: row.get("zettel_id"),
                            title: row.get("title"),
                            content: row.get("content"),
                            creation_date: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("creation_date"))?.with_timezone(&Utc),
                            last_reviewed: row.get::<Option<String>, _>("last_reviewed")
                                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
                            next_review: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("next_review"))?.with_timezone(&Utc),
                            difficulty: row.get("difficulty"),
                            stability: row.get("stability"),
                            retrievability: row.get("retrievability"),
                            reps: row.get("reps"),
                            lapses: row.get("lapses"),
                            state: row.get("state"),
                            links: Some(links_json),
                        });
                    }
                }
            }
        }

        Ok(linking_cards)
    }

    pub async fn find_cards_referencing_zettel_id(&self, zettel_id: &str) -> Result<Vec<Card>> {
        let search_pattern = format!("%{}%", zettel_id);
        let rows = sqlx::query(
            "SELECT * FROM cards WHERE content LIKE ?1 ORDER BY creation_date DESC"
        )
        .bind(&search_pattern)
        .fetch_all(&self.pool)
        .await?;

        self.rows_to_cards(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CreateCardRequest;

    #[tokio::test]
    async fn test_database_initialization() {
        let db = Database::new("sqlite::memory:").await.unwrap();
        
        // Test that tables were created by trying to query them
        let cards = db.get_all_cards().await.unwrap();
        assert_eq!(cards.len(), 0);
        
        let topics = db.get_all_topics().await.unwrap();
        assert_eq!(topics.len(), 0);
    }

    #[tokio::test]
    async fn test_database_delete_card() {
        let db = Database::new("sqlite::memory:").await.unwrap();
        
        let create_request = CreateCardRequest {
            zettel_id: "DB-TEST-001".to_string(),
            title: Some("Test Card Title".to_string()),
            content: "Test card for deletion".to_string(),
            topic_ids: vec![],
            links: None,
        };
        
        let card = db.create_card(create_request).await.unwrap();
        
        // Verify card exists
        let retrieved = db.get_card(card.id).await.unwrap();
        assert!(retrieved.is_some());
        
        // Delete the card
        let deleted = db.delete_card(card.id).await.unwrap();
        assert!(deleted);
        
        // Verify card is gone
        let retrieved = db.get_card(card.id).await.unwrap();
        assert!(retrieved.is_none());
        
        // Test deleting non-existent card
        let fake_id = Uuid::new_v4();
        let not_deleted = db.delete_card(fake_id).await.unwrap();
        assert!(!not_deleted);
    }

    #[tokio::test]
    async fn test_database_update_card_content() {
        let db = Database::new("sqlite::memory:").await.unwrap();
        
        let create_request = CreateCardRequest {
            zettel_id: "DB-TEST-002".to_string(),
            title: Some("Test Update Title".to_string()),
            content: "Original content".to_string(),
            topic_ids: vec![],
            links: Some(vec![Uuid::new_v4()]),
        };
        
        let mut card = db.create_card(create_request).await.unwrap();
        assert_eq!(card.content, "Original content");
        assert!(card.links.is_some());
        
        // Update the card
        card.content = "Updated content".to_string();
        card.links = None;
        
        db.update_card_content(&card).await.unwrap();
        
        // Retrieve and verify update
        let updated = db.get_card(card.id).await.unwrap();
        assert!(updated.is_some());
        let updated = updated.unwrap();
        assert_eq!(updated.content, "Updated content");
        assert!(updated.links.is_none());
    }

    #[tokio::test]
    async fn test_database_topic_operations() {
        let db = Database::new("sqlite::memory:").await.unwrap();
        
        // Test topic creation
        let topic = db.create_topic("Test Topic".to_string(), Some("Description".to_string())).await.unwrap();
        assert_eq!(topic.name, "Test Topic");
        assert_eq!(topic.description, Some("Description".to_string()));
        
        // Test getting all topics
        let topics = db.get_all_topics().await.unwrap();
        assert_eq!(topics.len(), 1);
        assert_eq!(topics[0].name, "Test Topic");
    }

    #[tokio::test]
    async fn test_database_cards_due_for_review() {
        let db = Database::new("sqlite::memory:").await.unwrap();
        
        let create_request = CreateCardRequest {
            zettel_id: "DB-TEST-003".to_string(),
            title: Some("Due Card Title".to_string()),
            content: "Due card".to_string(),
            topic_ids: vec![],
            links: None,
        };
        
        let _card = db.create_card(create_request).await.unwrap();
        
        // Card should be due for review immediately (next_review = creation_date)
        let due_cards = db.get_cards_due_for_review().await.unwrap();
        assert_eq!(due_cards.len(), 1);
        assert_eq!(due_cards[0].content, "Due card");
    }

    #[tokio::test]
    async fn test_database_backlinks_operations() {
        let db = Database::new("sqlite::memory:").await.unwrap();
        
        // Create two cards
        let card_a = db.create_card(CreateCardRequest {
            zettel_id: "BACKLINK-TEST-A".to_string(),
            title: Some("Card A Title".to_string()),
            content: "Card A".to_string(),
            topic_ids: vec![],
            links: None,
        }).await.unwrap();
        
        let card_b = db.create_card(CreateCardRequest {
            zettel_id: "BACKLINK-TEST-B".to_string(),
            title: Some("Card B Title".to_string()),
            content: "Card B".to_string(),
            topic_ids: vec![],
            links: None,
        }).await.unwrap();
        
        // Initially no backlinks
        let backlinks = db.get_backlinks(card_b.id).await.unwrap();
        assert_eq!(backlinks.len(), 0);
        
        // Create backlink from A to B
        db.create_backlinks(card_a.id, &[card_b.id]).await.unwrap();
        
        // Verify backlink exists
        let backlinks = db.get_backlinks(card_b.id).await.unwrap();
        assert_eq!(backlinks.len(), 1);
        assert_eq!(backlinks[0].id, card_a.id);
        
        // Remove backlink
        db.remove_backlinks(card_a.id, &[card_b.id]).await.unwrap();
        
        // Verify backlink is gone
        let backlinks = db.get_backlinks(card_b.id).await.unwrap();
        assert_eq!(backlinks.len(), 0);
    }

    #[tokio::test]
    async fn test_database_multiple_backlinks() {
        let db = Database::new("sqlite::memory:").await.unwrap();
        
        // Create three cards
        let card_a = db.create_card(CreateCardRequest {
            zettel_id: "MULTI-BACKLINK-A".to_string(),
            title: Some("Multi Card A".to_string()),
            content: "Card A".to_string(),
            topic_ids: vec![],
            links: None,
        }).await.unwrap();
        
        let card_b = db.create_card(CreateCardRequest {
            zettel_id: "MULTI-BACKLINK-B".to_string(),
            title: Some("Multi Card B".to_string()),
            content: "Card B".to_string(),
            topic_ids: vec![],
            links: None,
        }).await.unwrap();
        
        let card_c = db.create_card(CreateCardRequest {
            zettel_id: "MULTI-BACKLINK-C".to_string(),
            title: Some("Multi Card C".to_string()),
            content: "Card C".to_string(),
            topic_ids: vec![],
            links: None,
        }).await.unwrap();
        
        // Create backlinks from A to both B and C
        db.create_backlinks(card_a.id, &[card_b.id, card_c.id]).await.unwrap();
        
        // Verify both backlinks exist
        let backlinks_b = db.get_backlinks(card_b.id).await.unwrap();
        assert_eq!(backlinks_b.len(), 1);
        assert_eq!(backlinks_b[0].id, card_a.id);
        
        let backlinks_c = db.get_backlinks(card_c.id).await.unwrap();
        assert_eq!(backlinks_c.len(), 1);
        assert_eq!(backlinks_c[0].id, card_a.id);
        
        // Update backlinks - remove B, keep C
        db.update_backlinks(card_a.id, &[card_b.id, card_c.id], &[card_c.id]).await.unwrap();
        
        // Verify B has no backlinks, C still has one
        let backlinks_b = db.get_backlinks(card_b.id).await.unwrap();
        assert_eq!(backlinks_b.len(), 0);
        
        let backlinks_c = db.get_backlinks(card_c.id).await.unwrap();
        assert_eq!(backlinks_c.len(), 1);
        assert_eq!(backlinks_c[0].id, card_a.id);
    }

    #[tokio::test]
    async fn test_database_backlinks_cascade_delete() {
        let db = Database::new("sqlite::memory:").await.unwrap();
        
        // Create two cards
        let card_a = db.create_card(CreateCardRequest {
            zettel_id: "CASCADE-TEST-A".to_string(),
            title: Some("Cascade Card A".to_string()),
            content: "Card A".to_string(),
            topic_ids: vec![],
            links: None,
        }).await.unwrap();
        
        let card_b = db.create_card(CreateCardRequest {
            zettel_id: "CASCADE-TEST-B".to_string(),
            title: Some("Cascade Card B".to_string()),
            content: "Card B".to_string(),
            topic_ids: vec![],
            links: None,
        }).await.unwrap();
        
        // Create backlink
        db.create_backlinks(card_a.id, &[card_b.id]).await.unwrap();
        
        // Verify backlink exists
        let backlinks = db.get_backlinks(card_b.id).await.unwrap();
        assert_eq!(backlinks.len(), 1);
        
        // Delete source card A
        let deleted = db.delete_card(card_a.id).await.unwrap();
        assert!(deleted);
        
        // Verify backlinks are automatically cleaned up by foreign key cascade
        let backlinks = db.get_backlinks(card_b.id).await.unwrap();
        assert_eq!(backlinks.len(), 0);
    }
}