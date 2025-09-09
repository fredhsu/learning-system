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
    errors::{ApiError, ErrorContext, classify_database_error},
    llm_service::LLMService,
    models::*,
};

// Import logging macros
use crate::{log_api_start, log_api_success, log_api_error, log_api_warn};

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
pub struct BatchAnswerRequest {
    pub answers: Vec<QuestionAnswer>,
}

#[derive(Deserialize)]
pub struct ParallelAnswerRequest {
    pub answers: Vec<QuestionAnswer>,
    pub processing_mode: Option<String>, // "parallel", "batch", "sequential"
    pub max_concurrent_tasks: Option<usize>, // Optional concurrency limit
}

#[derive(Deserialize)]
pub struct QuestionAnswer {
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

#[derive(Serialize)]
pub struct ParallelProcessingMetrics {
    pub total_processing_time_ms: u64,
    pub parallel_tasks_spawned: usize,
    pub concurrent_execution_count: usize,
    pub average_task_duration_ms: u64,
    pub processing_mode_used: String, // "parallel", "batch_fallback", "sequential_fallback"
    pub fallback_reason: Option<String>,
}

#[derive(Serialize)]
pub struct ParallelApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub metrics: Option<ParallelProcessingMetrics>,
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
) -> Result<Json<ApiResponse<Card>>, (StatusCode, Json<ApiResponse<()>>)> {
    info!(
        zettel_id = %request.zettel_id,
        title = ?request.title,
        "Creating new card"
    );
    
    match state.card_service.create_card_with_zettel_links(request.clone()).await {
        Ok(card) => {
            info!(
                card_id = %card.id,
                zettel_id = %card.zettel_id,
                "Card created successfully"
            );
            Ok(Json(ApiResponse::success(card)))
        }
        Err(e) => {
            let classified_error = classify_database_error(&e);
            let context = ErrorContext::new("create_card", "card")
                .with_id(&request.zettel_id);
            
            Err(classified_error.to_response_with_context(context))
        }
    }
}

pub async fn get_card(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Card>>, (StatusCode, Json<ApiResponse<()>>)> {
    log_api_start!("get_card", card_id = id);
    
    match state.card_service.get_card(id).await {
        Ok(Some(card)) => {
            log_api_success!("get_card", card_id = id, "card retrieved successfully");
            Ok(Json(ApiResponse::success(card)))
        }
        Ok(None) => {
            log_api_warn!("get_card", card_id = id, "card not found");
            let error = ApiError::NotFound(format!("Card with ID '{}' not found", id));
            let context = ErrorContext::new("get_card", "card")
                .with_id(&id.to_string());
            Err(error.to_response_with_context(context))
        }
        Err(e) => {
            log_api_error!("get_card", card_id = id, error = e, "database error retrieving card");
            let error = ApiError::DatabaseError(e);
            let context = ErrorContext::new("get_card", "card")
                .with_id(&id.to_string());
            Err(error.to_response_with_context(context))
        }
    }
}

pub async fn get_card_by_zettel_id(
    State(state): State<AppState>,
    Path(zettel_id): Path<String>,
) -> Result<Json<ApiResponse<Card>>, (StatusCode, Json<ApiResponse<()>>)> {
    debug!(zettel_id = %zettel_id, "Getting card by Zettel ID");
    
    match state.card_service.get_card_by_zettel_id(&zettel_id).await {
        Ok(Some(card)) => {
            debug!(
                card_id = %card.id,
                zettel_id = %zettel_id,
                "Card retrieved by Zettel ID successfully"
            );
            Ok(Json(ApiResponse::success(card)))
        }
        Ok(None) => {
            let error = ApiError::NotFound(format!("Card with Zettel ID '{}' not found", zettel_id));
            let context = ErrorContext::new("get_card_by_zettel_id", "card")
                .with_id(&zettel_id);
            Err(error.to_response_with_context(context))
        }
        Err(e) => {
            let error = ApiError::DatabaseError(e);
            let context = ErrorContext::new("get_card_by_zettel_id", "card")
                .with_id(&zettel_id);
            Err(error.to_response_with_context(context))
        }
    }
}

pub async fn update_card(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateCardWithZettelLinksRequest>,
) -> Result<Json<ApiResponse<Card>>, (StatusCode, Json<ApiResponse<()>>)> {
    info!(
        card_id = %id,
        zettel_id = ?request.zettel_id,
        "Updating card"
    );
    
    match state.card_service.update_card_with_zettel_links(id, request).await {
        Ok(Some(card)) => {
            info!(
                card_id = %id,
                zettel_id = %card.zettel_id,
                "Card updated successfully"
            );
            Ok(Json(ApiResponse::success(card)))
        }
        Ok(None) => {
            let error = ApiError::NotFound(format!("Card with ID '{}' not found", id));
            let context = ErrorContext::new("update_card", "card")
                .with_id(&id.to_string());
            Err(error.to_response_with_context(context))
        }
        Err(e) => {
            let classified_error = classify_database_error(&e);
            let context = ErrorContext::new("update_card", "card")
                .with_id(&id.to_string());
            Err(classified_error.to_response_with_context(context))
        }
    }
}

pub async fn get_all_cards(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Card>>>, (StatusCode, Json<ApiResponse<()>>)> {
    debug!("Getting all cards");
    
    match state.card_service.get_all_cards().await {
        Ok(cards) => {
            debug!(card_count = cards.len(), "All cards retrieved successfully");
            Ok(Json(ApiResponse::success(cards)))
        }
        Err(e) => {
            let error = ApiError::DatabaseError(e);
            let context = ErrorContext::new("get_all_cards", "card");
            Err(error.to_response_with_context(context))
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
/// Retrieve cards due for review with error handling
async fn get_due_cards_for_session(card_service: &CardService) -> Result<Vec<Card>, (StatusCode, Json<ApiResponse<()>>)> {
    match card_service.get_cards_due_optimized().await {
        Ok(cards) => {
            debug!(card_count = cards.len(), "Retrieved cards due for review");
            Ok(cards)
        }
        Err(e) => {
            let error = ApiError::DatabaseError(e);
            let context = ErrorContext::new("get_due_cards", "cards");
            Err(error.to_response_with_context(context))
        }
    }
}

/// Generate questions for all cards using batch processing with comprehensive fallbacks
async fn generate_session_questions(
    llm_service: &LLMService, 
    cards: &[Card]
) -> HashMap<Uuid, Vec<QuizQuestion>> {
    info!(card_count = cards.len(), "Starting question generation for review session");
    
    // Try batch processing first
    match llm_service.generate_batch_quiz_questions(cards).await {
        Ok(questions) => {
            info!(
                card_count = cards.len(),
                generated_count = questions.len(),
                "Successfully generated questions using batch processing"
            );
            questions
        }
        Err(e) => {
            warn!(
                card_count = cards.len(),
                error = %e,
                "Batch question generation failed, falling back to individual generation"
            );
            
            // Fallback to individual generation
            let mut individual_questions = HashMap::new();
            for card in cards {
                match llm_service.generate_quiz_questions(card).await {
                    Ok(questions) => {
                        individual_questions.insert(card.id, questions);
                    }
                    Err(card_e) => {
                        error!(
                            card_id = %card.id, 
                            error = %card_e, 
                            "Error generating quiz for individual card"
                        );
                        
                        // Use local generation as final fallback
                        match llm_service.generate_quiz_questions_local(card, "").await {
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
            
            info!(
                card_count = cards.len(),
                generated_count = individual_questions.len(),
                "Completed individual question generation with fallbacks"
            );
            individual_questions
        }
    }
}

/// Create and store a new review session
fn create_and_store_session(
    review_sessions: &Arc<Mutex<HashMap<Uuid, ReviewSession>>>,
    cards: Vec<Card>,
    questions: HashMap<Uuid, Vec<QuizQuestion>>
) -> ReviewSession {
    let session = ReviewSession {
        session_id: Uuid::new_v4(),
        cards,
        questions,
        current_card: 0,
        created_at: Utc::now(),
    };

    // Store session in memory
    {
        let mut sessions = review_sessions.lock().unwrap();
        sessions.insert(session.session_id, session.clone());
    }
    
    info!(
        session_id = %session.session_id,
        card_count = session.cards.len(),
        "Review session created and stored"
    );

    session
}

pub async fn start_review_session(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<ReviewSession>>, (StatusCode, Json<ApiResponse<()>>)> {
    info!("Starting new review session");
    
    // Step 1: Get cards due for review
    let due_cards = get_due_cards_for_session(&state.card_service).await?;

    // Handle empty case early
    if due_cards.is_empty() {
        info!("No cards due for review, creating empty session");
        let empty_session = ReviewSession {
            session_id: Uuid::new_v4(),
            cards: vec![],
            questions: HashMap::new(),
            current_card: 0,
            created_at: Utc::now(),
        };
        return Ok(Json(ApiResponse::success(empty_session)));
    }

    // Step 2: Generate questions for all cards
    let all_questions = generate_session_questions(&state.llm_service, &due_cards).await;

    // Step 3: Create and store the session
    let session = create_and_store_session(
        &state.review_sessions,
        due_cards,
        all_questions
    );

    info!(
        session_id = %session.session_id,
        card_count = session.cards.len(),
        question_count = session.questions.values().map(|q| q.len()).sum::<usize>(),
        "Review session started successfully"
    );

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
                "Session answer graded successfully (FSRS update deferred until card completion)"
            );
            
            // Return grading result without updating FSRS - let user rating handle the final update
            Ok(Json(ApiResponse::success(json!({
                "is_correct": grading_result.is_correct,
                "feedback": grading_result.feedback,
                "rating": grading_result.suggested_rating
            }))))
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

pub async fn submit_batch_session_answers(
    State(state): State<AppState>,
    Path((session_id, card_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<BatchAnswerRequest>,
) -> Result<Json<ApiResponse<Vec<BatchGradingResult>>>, StatusCode> {
    info!(
        session_id = %session_id,
        card_id = %card_id,
        answer_count = request.answers.len(),
        "Submitting batch answers for session-based quiz"
    );

    // Validate empty request
    if request.answers.is_empty() {
        warn!(session_id = %session_id, card_id = %card_id, "Empty batch answer request");
        return Err(StatusCode::BAD_REQUEST);
    }

    // Get the session and validate it exists
    let session = {
        let sessions = state.review_sessions.lock().unwrap();
        match sessions.get(&session_id) {
            Some(session) => session.clone(),
            None => {
                warn!(session_id = %session_id, "Session not found for batch answer submission");
                return Err(StatusCode::NOT_FOUND);
            }
        }
    };

    // Get the card
    let card = match state.card_service.get_card(card_id).await {
        Ok(Some(card)) => {
            debug!(card_id = %card_id, zettel_id = %card.zettel_id, "Retrieved card for batch answer grading");
            card
        },
        Ok(None) => {
            warn!(card_id = %card_id, "Card not found for batch session quiz answers");
            return Err(StatusCode::NOT_FOUND);
        },
        Err(e) => {
            error!(card_id = %card_id, error = %e, "Error getting card for batch session quiz answers");
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

    // Validate question indices
    for answer_request in &request.answers {
        if answer_request.question_index >= questions.len() {
            warn!(
                session_id = %session_id,
                card_id = %card_id,
                question_index = answer_request.question_index,
                questions_count = questions.len(),
                "Invalid question index in batch request"
            );
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    // Build batch grading requests
    let batch_requests: Vec<BatchGradingRequest> = request.answers
        .iter()
        .map(|answer_request| {
            let question = &questions[answer_request.question_index];
            BatchGradingRequest {
                card_content: card.content.clone(),
                question: question.clone(),
                user_answer: answer_request.answer.clone(),
            }
        })
        .collect();

    // Grade all answers in batch
    match state.llm_service.grade_batch_answers(&batch_requests).await {
        Ok(grading_results) => {
            info!(
                session_id = %session_id,
                card_id = %card_id,
                result_count = grading_results.len(),
                "Batch answers graded successfully (FSRS updates deferred until card completion)"
            );
            
            Ok(Json(ApiResponse::success(grading_results)))
        },
        Err(e) => {
            error!(
                session_id = %session_id,
                card_id = %card_id,
                error = %e,
                "Error grading batch session quiz answers"
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Submit answers for parallel processing with concurrent individual grading
pub async fn submit_parallel_session_answers(
    State(state): State<AppState>,
    Path((session_id, card_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<ParallelAnswerRequest>,
) -> Result<Json<ParallelApiResponse<Vec<BatchGradingResult>>>, StatusCode> {
    let start_time = std::time::Instant::now();
    
    info!(
        session_id = %session_id,
        card_id = %card_id,
        answer_count = request.answers.len(),
        processing_mode = ?request.processing_mode,
        max_concurrent = ?request.max_concurrent_tasks,
        "Submitting answers for parallel processing"
    );
    
    // Validate empty request
    if request.answers.is_empty() {
        warn!(session_id = %session_id, card_id = %card_id, "Empty parallel answer request");
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Get the session and validate it exists
    let session = {
        let sessions = state.review_sessions.lock().unwrap();
        match sessions.get(&session_id) {
            Some(session) => session.clone(),
            None => {
                warn!(session_id = %session_id, "Session not found for parallel answer submission");
                return Err(StatusCode::NOT_FOUND);
            }
        }
    };
    
    // Get the card
    let card = match state.card_service.get_card(card_id).await {
        Ok(Some(card)) => {
            debug!(card_id = %card_id, zettel_id = %card.zettel_id, "Retrieved card for parallel answer grading");
            card
        },
        Ok(None) => {
            warn!(card_id = %card_id, "Card not found for parallel session quiz answers");
            return Err(StatusCode::NOT_FOUND);
        },
        Err(e) => {
            error!(card_id = %card_id, error = %e, "Error getting card for parallel session quiz answers");
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
    
    // Validate question indices
    for answer_request in &request.answers {
        if answer_request.question_index >= questions.len() {
            warn!(
                session_id = %session_id,
                card_id = %card_id,
                question_index = answer_request.question_index,
                questions_count = questions.len(),
                "Invalid question index in parallel request"
            );
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    
    // Build questions and answers for parallel processing
    let questions_and_answers: Vec<(QuizQuestion, String)> = request.answers
        .iter()
        .map(|answer_request| {
            let question = &questions[answer_request.question_index];
            (question.clone(), answer_request.answer.clone())
        })
        .collect();
    
    // Process with fallback chain
    let processing_mode = request.processing_mode.as_deref();
    match state.llm_service.grade_answers_with_fallback(
        &card, 
        questions_and_answers, 
        processing_mode,
        request.max_concurrent_tasks
    ).await {
        Ok((grading_results, mode_used, fallback_reason)) => {
            let total_duration = start_time.elapsed();
            
            // Calculate metrics
            let metrics = ParallelProcessingMetrics {
                total_processing_time_ms: total_duration.as_millis() as u64,
                parallel_tasks_spawned: if mode_used == "parallel" { 
                    grading_results.len() 
                } else { 
                    0 
                },
                concurrent_execution_count: request.max_concurrent_tasks.unwrap_or(5).min(grading_results.len()),
                average_task_duration_ms: total_duration.as_millis() as u64 / grading_results.len().max(1) as u64,
                processing_mode_used: mode_used,
                fallback_reason,
            };
            
            info!(
                session_id = %session_id,
                card_id = %card_id,
                result_count = grading_results.len(),
                processing_mode = %metrics.processing_mode_used,
                total_duration_ms = metrics.total_processing_time_ms,
                "Parallel answers processed successfully (FSRS updates deferred until card completion)"
            );
            
            Ok(Json(ParallelApiResponse {
                success: true,
                data: Some(grading_results),
                error: None,
                metrics: Some(metrics),
            }))
        },
        Err(e) => {
            error!(
                session_id = %session_id,
                card_id = %card_id,
                error = %e,
                "Error processing parallel session quiz answers"
            );
            
            Ok(Json(ParallelApiResponse {
                success: false,
                data: None,
                error: Some("Failed to process answers".to_string()),
                metrics: None,
            }))
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
) -> Result<Json<ApiResponse<bool>>, (StatusCode, Json<ApiResponse<()>>)> {
    info!(card_id = %id, "Deleting card");
    
    match state.card_service.delete_card(id).await {
        Ok(deleted) => {
            if deleted {
                info!(card_id = %id, "Card deleted successfully");
                Ok(Json(ApiResponse::success(true)))
            } else {
                let error = ApiError::NotFound(format!("Card with ID '{}' not found", id));
                let context = ErrorContext::new("delete_card", "card")
                    .with_id(&id.to_string());
                Err(error.to_response_with_context(context))
            }
        }
        Err(e) => {
            let error = ApiError::DatabaseError(e);
            let context = ErrorContext::new("delete_card", "card")
                .with_id(&id.to_string());
            Err(error.to_response_with_context(context))
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
        .route("/api/review/session/:session_id/answers/:card_id/batch", post(submit_batch_session_answers))
        .route("/api/review/session/:session_id/answers/:card_id/parallel", post(submit_parallel_session_answers))
        
        // Review routes
        .route("/api/cards/:id/review", post(review_card))
        
        .with_state(state)
}

#[cfg(test)]
pub fn create_app(state: AppState) -> Router {
    create_router(state)
}