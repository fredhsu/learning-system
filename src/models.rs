use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Card {
    pub id: Uuid,
    pub zettel_id: String, // Human-readable unique identifier for Zettelkasten linking
    pub content: String,
    pub creation_date: DateTime<Utc>,
    pub last_reviewed: Option<DateTime<Utc>>,
    pub next_review: DateTime<Utc>,
    pub difficulty: f64,
    pub stability: f64,
    pub retrievability: f64,
    pub reps: i32,
    pub lapses: i32,
    pub state: String, // New, Learning, Review, Relearning
    pub links: Option<String>, // JSON array of linked card IDs
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Topic {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CardTopic {
    pub card_id: Uuid,
    pub topic_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Review {
    pub id: Uuid,
    pub card_id: Uuid,
    pub review_date: DateTime<Utc>,
    pub rating: i32, // 1=Again, 2=Hard, 3=Good, 4=Easy
    pub interval: f64,
    pub ease_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCardRequest {
    pub zettel_id: String, // Required user-defined unique identifier
    pub content: String,
    pub topic_ids: Vec<Uuid>,
    pub links: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCardWithZettelLinksRequest {
    pub zettel_id: String,
    pub content: String,
    pub topic_ids: Vec<Uuid>,
    pub zettel_links: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCardRequest {
    pub zettel_id: Option<String>,
    pub content: Option<String>,
    pub topic_ids: Option<Vec<Uuid>>,
    pub links: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCardWithZettelLinksRequest {
    pub zettel_id: Option<String>,
    pub content: Option<String>,
    pub topic_ids: Option<Vec<Uuid>>,
    pub zettel_links: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizQuestion {
    pub question: String,
    pub question_type: String, // "multiple_choice", "short_answer", "problem_solving"
    pub options: Option<Vec<String>>, // For multiple choice
    pub correct_answer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizResponse {
    pub card_id: Uuid,
    pub question: QuizQuestion,
    pub user_answer: String,
    pub is_correct: bool,
    pub feedback: String,
    pub rating: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSession {
    pub session_id: Uuid,
    pub cards: Vec<Card>,
    pub questions: HashMap<Uuid, Vec<QuizQuestion>>,
    pub current_card: usize,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizAnswerWithContext {
    pub question_index: usize,
    pub answer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchGradingRequest {
    pub question: QuizQuestion,
    pub user_answer: String,
    pub card_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchGradingResponse {
    pub results: Vec<BatchGradingResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchGradingResult {
    pub question_id: String,
    pub is_correct: bool,
    pub feedback: String,
    pub suggested_rating: i32,
}