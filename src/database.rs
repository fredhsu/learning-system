use anyhow::Result;
use chrono::Utc;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::models::*;

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
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

        Ok(())
    }

    // Card operations
    pub async fn create_card(&self, request: CreateCardRequest) -> Result<Card> {
        let card_id = Uuid::new_v4();
        let now = Utc::now();
        let links_json = request.links.as_ref().map(|l| serde_json::to_string(l).unwrap());

        let card = Card {
            id: card_id,
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
            INSERT INTO cards (id, content, creation_date, last_reviewed, next_review, 
                             difficulty, stability, retrievability, reps, lapses, state, links)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            "#,
        )
        .bind(card.id.to_string())
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
}