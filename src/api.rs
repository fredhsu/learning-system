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
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use chrono::Utc;
use tracing::{debug, error, info, warn};

use crate::{
    card_service::CardService,
    llm_service::LLMService,
    models::*,
};

#[derive(Clone)]
pub struct AppState {
    pub card_service: CardService,
    pub llm_service: LLMService,
    pub review_sessions: Arc<Mutex<HashMap<Uuid, ReviewSession>>>,
}

#[derive(Deserialize)]
pub struct ReviewRequest {
    pub rating: i32,
}

#[derive(Deserialize)]
pub struct QuizAnswerRequest {
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
    Json(request): Json<CreateCardWithZettelLinksRequest>,
) -> Result<Json<ApiResponse<Card>>, StatusCode> {
    match state.card_service.create_card_with_zettel_links(request).await {
        Ok(card) => Ok(Json(ApiResponse::success(card))),
        Err(e) => {
            error!(error = %e, "Error creating card");
            let error_msg = e.to_string();
            if error_msg.contains("already exists") {
                Ok(Json(ApiResponse::error(error_msg)))
            } else if error_msg.contains("required") {
                Ok(Json(ApiResponse::error("Zettel ID is required".to_string())))
            } else if error_msg.contains("not found") {
                Ok(Json(ApiResponse::error(error_msg)))
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
            error!(card_id = %id, error = %e, "Error getting card");
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
            error!(zettel_id = %zettel_id, error = %e, "Error getting card by zettel ID");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_card(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateCardWithZettelLinksRequest>,
) -> Result<Json<ApiResponse<Card>>, StatusCode> {
    match state.card_service.update_card_with_zettel_links(id, request).await {
        Ok(Some(card)) => Ok(Json(ApiResponse::success(card))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!(card_id = %id, error = %e, "Error updating card");
            let error_msg = e.to_string();
            if error_msg.contains("not found") {
                Ok(Json(ApiResponse::error(error_msg)))
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

pub async fn get_all_cards(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Card>>>, StatusCode> {
    match state.card_service.get_all_cards().await {
        Ok(cards) => Ok(Json(ApiResponse::success(cards))),
        Err(e) => {
            error!(error = %e, "Error getting all cards");
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
            error!(error = %e, "Error getting due cards");
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
            error!(card_id = %id, error = %e, "Error getting linked cards");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_backlinks(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<Card>>>, StatusCode> {
    match state.card_service.get_backlinks(id).await {
        Ok(cards) => Ok(Json(ApiResponse::success(cards))),
        Err(e) => {
            error!(card_id = %id, error = %e, "Error getting backlinks");
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
            error!(query = ?params.q, error = %e, "Error searching cards");
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
            error!(error = %e, "Error creating topic");
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
            error!(error = %e, "Error getting topics");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Review session endpoints
pub async fn start_review_session(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<ReviewSession>>, StatusCode> {
    // Get cards due for review with smart ordering
    let due_cards = match state.card_service.get_cards_due_optimized().await {
        Ok(cards) => cards,
        Err(e) => {
            error!(error = %e, "Error getting due cards");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if due_cards.is_empty() {
        let empty_session = ReviewSession {
            session_id: Uuid::new_v4(),
            cards: vec![],
            questions: HashMap::new(),
            current_card: 0,
            created_at: Utc::now(),
        };
        return Ok(Json(ApiResponse::success(empty_session)));
    }

    // Generate questions for all cards using batch processing
    let all_questions = match state.llm_service.generate_batch_quiz_questions(&due_cards).await {
        Ok(questions) => {
            info!(
                card_count = due_cards.len(),
                generated_count = questions.len(),
                "Successfully generated questions using batch processing"
            );
            questions
        }
        Err(e) => {
            warn!(
                card_count = due_cards.len(),
                error = %e,
                "Batch question generation failed, falling back to individual generation"
            );
            // Fallback to individual generation (the method handles its own fallbacks)
            let mut individual_questions = HashMap::new();
            for card in &due_cards {
                match state.llm_service.generate_quiz_questions(card).await {
                    Ok(questions) => {
                        individual_questions.insert(card.id, questions);
                    }
                    Err(card_e) => {
                        error!(card_id = %card.id, error = %card_e, "Error generating quiz for individual card");
                        // Use local generation as final fallback
                        match state.llm_service.generate_quiz_questions_local(card, "").await {
                            Ok(questions) => {
                                individual_questions.insert(card.id, questions);
                            }
                            Err(local_e) => {
                                error!(
                                    card_id = %card.id,
                                    error = %local_e,
                                    "All question generation methods failed for card"
                                );
                                // Skip this card if we can't generate questions
                                continue;
                            }
                        }
                    }
                }
            }
            individual_questions
        }
    };

    let session = ReviewSession {
        session_id: Uuid::new_v4(),
        cards: due_cards,
        questions: all_questions,
        current_card: 0,
        created_at: Utc::now(),
    };

    // Store session in memory
    {
        let mut sessions = state.review_sessions.lock().unwrap();
        sessions.insert(session.session_id, session.clone());
    }

    Ok(Json(ApiResponse::success(session)))
}

pub async fn get_review_session(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
) -> Result<Json<ApiResponse<ReviewSession>>, StatusCode> {
    let sessions = state.review_sessions.lock().unwrap();
    match sessions.get(&session_id) {
        Some(session) => Ok(Json(ApiResponse::success(session.clone()))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn submit_session_answer(
    State(state): State<AppState>,
    Path((session_id, card_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<QuizAnswerWithContext>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    info!(
        session_id = %session_id,
        card_id = %card_id,
        question_index = request.question_index,
        user_answer = %request.answer,
        "Submitting answer for session-based quiz"
    );

    // Get the session and validate it exists
    let session = {
        let sessions = state.review_sessions.lock().unwrap();
        match sessions.get(&session_id) {
            Some(session) => session.clone(),
            None => {
                warn!(session_id = %session_id, "Session not found for answer submission");
                return Err(StatusCode::NOT_FOUND);
            }
        }
    };

    // Get the card
    let card = match state.card_service.get_card(card_id).await {
        Ok(Some(card)) => {
            debug!(card_id = %card_id, zettel_id = %card.zettel_id, "Retrieved card for session answer grading");
            card
        },
        Ok(None) => {
            warn!(card_id = %card_id, "Card not found for session quiz answer");
            return Err(StatusCode::NOT_FOUND);
        },
        Err(e) => {
            error!(card_id = %card_id, error = %e, "Error getting card for session quiz answer");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Get the questions for this card from the session
    let questions = match session.questions.get(&card_id) {
        Some(questions) => questions,
        None => {
            warn!(session_id = %session_id, card_id = %card_id, "No questions found for card in session");
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    // Validate question index
    if request.question_index >= questions.len() {
        warn!(
            session_id = %session_id,
            card_id = %card_id,
            question_index = request.question_index,
            questions_count = questions.len(),
            "Invalid question index for session"
        );
        return Err(StatusCode::BAD_REQUEST);
    }

    let question = &questions[request.question_index];
    debug!(
        session_id = %session_id,
        card_id = %card_id,
        question_index = request.question_index,
        question_type = %question.question_type,
        "Retrieved question from session for grading"
    );

    // Grade the answer using the actual question context
    match state.llm_service.grade_answer(&card, question, &request.answer).await {
        Ok(grading_result) => {
            info!(
                session_id = %session_id,
                card_id = %card_id,
                question_index = request.question_index,
                is_correct = grading_result.is_correct,
                suggested_rating = grading_result.suggested_rating,
                "Session answer graded successfully, updating card review"
            );
            
            // Update card with FSRS based on the suggested rating
            match state.card_service.review_card(card_id, grading_result.suggested_rating).await {
                Ok(Some(updated_card)) => {
                    info!(
                        session_id = %session_id,
                        card_id = %card_id,
                        new_next_review = %updated_card.next_review.to_string(),
                        "Card review updated successfully after session quiz"
                    );
                    
                    Ok(Json(ApiResponse::success(json!({
                        "is_correct": grading_result.is_correct,
                        "feedback": grading_result.feedback,
                        "rating": grading_result.suggested_rating,
                        "next_review": updated_card.next_review
                    }))))
                },
                Ok(None) => {
                    warn!(card_id = %card_id, "Card not found when updating review after session quiz");
                    Err(StatusCode::NOT_FOUND)
                },
                Err(e) => {
                    error!(
                        card_id = %card_id,
                        rating = grading_result.suggested_rating,
                        error = %e,
                        "Error updating card review after session quiz"
                    );
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        },
        Err(e) => {
            error!(
                session_id = %session_id,
                card_id = %card_id,
                question_index = request.question_index,
                error = %e,
                "Error grading session quiz answer"
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Quiz endpoints (legacy)


// DEPRECATED: Use session-based answer submission instead (/api/review/session/:session_id/answer/:card_id)
// This legacy endpoint loses question context and causes multiple choice grading issues
pub async fn submit_quiz_answer(
    State(state): State<AppState>,
    Path(card_id): Path<Uuid>,
    Json(request): Json<QuizAnswerRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    warn!(
        card_id = %card_id,
        user_answer = %request.answer,
        "DEPRECATED: Using legacy quiz answer endpoint - multiple choice answers may be graded incorrectly. Use session-based submission instead."
    );
    info!(
        card_id = %card_id,
        user_answer = %request.answer,
        "Submitting quiz answer for legacy endpoint"
    );
    
    let card = match state.card_service.get_card(card_id).await {
        Ok(Some(card)) => {
            debug!(card_id = %card_id, zettel_id = %card.zettel_id, "Retrieved card for answer grading");
            card
        },
        Ok(None) => {
            warn!(card_id = %card_id, "Card not found for quiz answer");
            return Err(StatusCode::NOT_FOUND);
        },
        Err(e) => {
            error!(card_id = %card_id, error = %e, "Error getting card for quiz answer");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // For backward compatibility, generate a simple question
    let dummy_question = QuizQuestion {
        question: "What is the main concept described in this card?".to_string(),
        question_type: "short_answer".to_string(),
        options: None,
        correct_answer: Some("Based on the card content".to_string()),
    };

    debug!(card_id = %card_id, "Using dummy question for legacy quiz answer endpoint");

    match state.llm_service.grade_answer(&card, &dummy_question, &request.answer).await {
        Ok(grading_result) => {
            info!(
                card_id = %card_id,
                is_correct = grading_result.is_correct,
                suggested_rating = grading_result.suggested_rating,
                "Answer graded successfully, updating card review"
            );
            
            // Update card with FSRS based on the suggested rating
            match state.card_service.review_card(card_id, grading_result.suggested_rating).await {
                Ok(Some(updated_card)) => {
                    info!(
                        card_id = %card_id,
                        new_next_review = %updated_card.next_review.to_string(),
                        "Card review updated successfully"
                    );
                    let response = json!({
                        "grading": grading_result,
                        "updated_card": updated_card
                    });
                    Ok(Json(ApiResponse::success(response)))
                }
                Ok(None) => {
                    error!(card_id = %card_id, "Card not found when updating review");
                    Err(StatusCode::NOT_FOUND)
                }
                Err(e) => {
                    error!(card_id = %card_id, error = %e, "Error updating card review");
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Err(e) => {
            error!(card_id = %card_id, error = %e, "Error grading answer");
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
            error!(card_id = %card_id, rating = request.rating, error = %e, "Error reviewing card");
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
            error!(card_id = %id, error = %e, "Error deleting card");
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
        .route("/api/cards/:id/backlinks", get(get_backlinks))
        
        // Topic routes
        .route("/api/topics", post(create_topic))
        .route("/api/topics", get(get_topics))
        
        // Quiz routes (legacy - deprecated)
        .route("/api/cards/:id/quiz/answer", post(submit_quiz_answer))
        
        // Review session routes
        .route("/api/review/session/start", post(start_review_session))
        .route("/api/review/session/:id", get(get_review_session))
        .route("/api/review/session/:session_id/answer/:card_id", post(submit_session_answer))
        
        // Review routes
        .route("/api/cards/:id/review", post(review_card))
        
        .with_state(state)
}