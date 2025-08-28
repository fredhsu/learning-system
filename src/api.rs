use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    card_service::CardService,
    llm_service::LLMService,
    models::*,
};

#[derive(Clone)]
pub struct AppState {
    pub card_service: CardService,
    pub llm_service: LLMService,
}

#[derive(Deserialize)]
pub struct ReviewRequest {
    pub rating: i32,
}

#[derive(Deserialize)]
pub struct QuizAnswerRequest {
    pub question_index: usize,
    pub answer: String,
}

#[derive(Deserialize)]
pub struct SearchParams {
    pub q: Option<String>,
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

// Card endpoints
pub async fn create_card(
    State(state): State<AppState>,
    Json(request): Json<CreateCardRequest>,
) -> Result<Json<ApiResponse<Card>>, StatusCode> {
    match state.card_service.create_card(request).await {
        Ok(card) => Ok(Json(ApiResponse::success(card))),
        Err(e) => {
            eprintln!("Error creating card: {}", e);
            let error_msg = e.to_string();
            if error_msg.contains("already exists") {
                Ok(Json(ApiResponse::error(error_msg)))
            } else if error_msg.contains("required") {
                Ok(Json(ApiResponse::error("Zettel ID is required".to_string())))
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

pub async fn get_card(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Card>>, StatusCode> {
    match state.card_service.get_card(id).await {
        Ok(Some(card)) => Ok(Json(ApiResponse::success(card))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error getting card: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_card_by_zettel_id(
    State(state): State<AppState>,
    Path(zettel_id): Path<String>,
) -> Result<Json<ApiResponse<Card>>, StatusCode> {
    match state.card_service.get_card_by_zettel_id(&zettel_id).await {
        Ok(Some(card)) => Ok(Json(ApiResponse::success(card))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error getting card by zettel ID: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_card(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateCardRequest>,
) -> Result<Json<ApiResponse<Card>>, StatusCode> {
    match state.card_service.update_card(id, request).await {
        Ok(Some(card)) => Ok(Json(ApiResponse::success(card))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error updating card: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_all_cards(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Card>>>, StatusCode> {
    match state.card_service.get_all_cards().await {
        Ok(cards) => Ok(Json(ApiResponse::success(cards))),
        Err(e) => {
            eprintln!("Error getting all cards: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_cards_due(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Card>>>, StatusCode> {
    match state.card_service.get_cards_due_for_review().await {
        Ok(cards) => Ok(Json(ApiResponse::success(cards))),
        Err(e) => {
            eprintln!("Error getting due cards: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_linked_cards(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<Card>>>, StatusCode> {
    match state.card_service.get_linked_cards(id).await {
        Ok(cards) => Ok(Json(ApiResponse::success(cards))),
        Err(e) => {
            eprintln!("Error getting linked cards: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn search_cards(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<ApiResponse<Vec<Card>>>, StatusCode> {
    let search_query = params.q.as_deref().unwrap_or("");
    
    match state.card_service.search_cards(search_query).await {
        Ok(cards) => Ok(Json(ApiResponse::success(cards))),
        Err(e) => {
            eprintln!("Error searching cards: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Topic endpoints
pub async fn create_topic(
    State(state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<Topic>>, StatusCode> {
    let name = request["name"].as_str().ok_or(StatusCode::BAD_REQUEST)?;
    let description = request["description"].as_str().map(|s| s.to_string());

    match state.card_service.create_topic(name.to_string(), description).await {
        Ok(topic) => Ok(Json(ApiResponse::success(topic))),
        Err(e) => {
            eprintln!("Error creating topic: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_topics(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Topic>>>, StatusCode> {
    match state.card_service.get_all_topics().await {
        Ok(topics) => Ok(Json(ApiResponse::success(topics))),
        Err(e) => {
            eprintln!("Error getting topics: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Quiz endpoints
pub async fn generate_quiz(
    State(state): State<AppState>,
    Path(card_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<QuizQuestion>>>, StatusCode> {
    let card = match state.card_service.get_card(card_id).await {
        Ok(Some(card)) => card,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error getting card for quiz: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match state.llm_service.generate_quiz_questions(&card).await {
        Ok(questions) => Ok(Json(ApiResponse::success(questions))),
        Err(e) => {
            eprintln!("Error generating quiz: {}", e);
            // Fallback to local generation if LLM fails
            match state.llm_service.generate_quiz_questions_local(&card, "").await {
                Ok(questions) => Ok(Json(ApiResponse::success(questions))),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

pub async fn submit_quiz_answer(
    State(state): State<AppState>,
    Path(card_id): Path<Uuid>,
    Json(request): Json<QuizAnswerRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let card = match state.card_service.get_card(card_id).await {
        Ok(Some(card)) => card,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error getting card for quiz answer: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // For now, we'll need to store the question somewhere or pass it in the request
    // This is a simplified version - in a real app, you'd store the active quiz session
    let dummy_question = QuizQuestion {
        question: "What is the main concept?".to_string(),
        question_type: "short_answer".to_string(),
        options: None,
        correct_answer: Some("Based on card content".to_string()),
    };

    match state.llm_service.grade_answer(&card, &dummy_question, &request.answer).await {
        Ok(grading_result) => {
            // Update card with FSRS based on the suggested rating
            if let Ok(Some(updated_card)) = state.card_service.review_card(card_id, grading_result.suggested_rating).await {
                let response = json!({
                    "grading": grading_result,
                    "updated_card": updated_card
                });
                Ok(Json(ApiResponse::success(response)))
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
        Err(e) => {
            eprintln!("Error grading answer: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Review endpoints
pub async fn review_card(
    State(state): State<AppState>,
    Path(card_id): Path<Uuid>,
    Json(request): Json<ReviewRequest>,
) -> Result<Json<ApiResponse<Card>>, StatusCode> {
    match state.card_service.review_card(card_id, request.rating).await {
        Ok(Some(card)) => Ok(Json(ApiResponse::success(card))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error reviewing card: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn delete_card(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<bool>>, StatusCode> {
    match state.card_service.delete_card(id).await {
        Ok(deleted) => {
            if deleted {
                Ok(Json(ApiResponse::success(true)))
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(e) => {
            eprintln!("Error deleting card: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Card routes
        .route("/api/cards", post(create_card))
        .route("/api/cards", get(get_all_cards))
        .route("/api/cards/search", get(search_cards))
        .route("/api/cards/zettel/:zettel_id", get(get_card_by_zettel_id))
        .route("/api/cards/:id", get(get_card))
        .route("/api/cards/:id", put(update_card))
        .route("/api/cards/:id", delete(delete_card))
        .route("/api/cards/due", get(get_cards_due))
        .route("/api/cards/:id/links", get(get_linked_cards))
        
        // Topic routes
        .route("/api/topics", post(create_topic))
        .route("/api/topics", get(get_topics))
        
        // Quiz routes
        .route("/api/cards/:id/quiz", get(generate_quiz))
        .route("/api/cards/:id/quiz/answer", post(submit_quiz_answer))
        
        // Review routes
        .route("/api/cards/:id/review", post(review_card))
        
        .with_state(state)
}