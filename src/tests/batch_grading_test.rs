#[cfg(test)]
mod batch_grading_tests {
    use crate::{api::*, card_service::CardService, llm_service::LLMService, models::*};
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
    };
    use chrono::Utc;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use tower::ServiceExt;
    use uuid::Uuid;

    async fn setup_test_app() -> (Router, Uuid, Uuid, Uuid) {
        setup_test_app_with_llm_service(LLMService::new_mock()).await
    }

    async fn setup_test_app_with_mixed_results() -> (Router, Uuid, Uuid, Uuid) {
        setup_test_app_with_llm_service(LLMService::new_mock_with_mixed_results()).await
    }

    async fn setup_test_app_with_llm_service(
        llm_service: LLMService,
    ) -> (Router, Uuid, Uuid, Uuid) {
        let card_service = CardService::new_in_memory().await.unwrap();
        let review_sessions = Arc::new(Mutex::new(HashMap::new()));

        let app_state = AppState {
            card_service: card_service.clone(),
            llm_service,
            review_sessions: review_sessions.clone(),
        };

        let app = create_app(app_state);

        // Create test card
        let test_card_request = CreateCardRequest {
            zettel_id: "TEST001".to_string(),
            title: Some("Test Card".to_string()),
            content: "This is test content for batch grading".to_string(),
            links: None,
            topic_ids: vec![],
        };

        let test_card = card_service.create_card(test_card_request).await.unwrap();
        let card_id = test_card.id;

        // Create test session with questions
        let session_id = Uuid::new_v4();
        let questions = vec![
            QuizQuestion {
                question: "What is this card about?".to_string(),
                question_type: "short_answer".to_string(),
                options: None,
                correct_answer: Some("Test content".to_string()),
            },
            QuizQuestion {
                question: "Choose the correct option".to_string(),
                question_type: "multiple_choice".to_string(),
                options: Some(vec!["A) Option 1".to_string(), "B) Option 2".to_string()]),
                correct_answer: Some("A) Option 1".to_string()),
            },
        ];

        let mut session_questions = HashMap::new();
        session_questions.insert(card_id, questions);

        let review_session = ReviewSession {
            session_id,
            cards: vec![test_card],
            current_card: 0,
            questions: session_questions,
            created_at: Utc::now(),
        };

        {
            let mut sessions = review_sessions.lock().unwrap();
            sessions.insert(session_id, review_session);
        }

        (app, session_id, card_id, Uuid::new_v4())
    }

    #[derive(serde::Deserialize, serde::Serialize)]
    struct BatchAnswerRequest {
        answers: Vec<QuestionAnswer>,
    }

    #[derive(serde::Deserialize, serde::Serialize)]
    struct QuestionAnswer {
        question_index: usize,
        answer: String,
    }

    #[tokio::test]
    async fn test_batch_answer_submission_success() {
        let (app, session_id, card_id, _) = setup_test_app().await;

        let request_body = BatchAnswerRequest {
            answers: vec![
                QuestionAnswer {
                    question_index: 0,
                    answer: "Test content".to_string(),
                },
                QuestionAnswer {
                    question_index: 1,
                    answer: "A) Option 1".to_string(),
                },
            ],
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!(
                        "/api/review/session/{}/answers/{}/batch",
                        session_id, card_id
                    ))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json_response["success"], true);

        let results = json_response["data"].as_array().unwrap();
        assert_eq!(results.len(), 2);

        // Check first question result
        assert_eq!(results[0]["question_id"], "1");
        assert!(results[0]["is_correct"].as_bool().unwrap());
        assert!(results[0]["feedback"].as_str().unwrap().len() > 0);
        assert!(results[0]["suggested_rating"].as_i64().unwrap() >= 1);
        assert!(results[0]["suggested_rating"].as_i64().unwrap() <= 4);

        // Check second question result
        assert_eq!(results[1]["question_id"], "2");
        assert!(results[1]["is_correct"].as_bool().unwrap());
        assert!(results[1]["feedback"].as_str().unwrap().len() > 0);
        assert!(results[1]["suggested_rating"].as_i64().unwrap() >= 1);
        assert!(results[1]["suggested_rating"].as_i64().unwrap() <= 4);
    }

    #[tokio::test]
    async fn test_batch_answer_submission_invalid_session() {
        let (app, _, card_id, _) = setup_test_app().await;
        let invalid_session_id = Uuid::new_v4();

        let request_body = BatchAnswerRequest {
            answers: vec![QuestionAnswer {
                question_index: 0,
                answer: "Test answer".to_string(),
            }],
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!(
                        "/api/review/session/{}/answers/{}/batch",
                        invalid_session_id, card_id
                    ))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_batch_answer_submission_invalid_card() {
        let (app, session_id, _, _) = setup_test_app().await;
        let invalid_card_id = Uuid::new_v4();

        let request_body = BatchAnswerRequest {
            answers: vec![QuestionAnswer {
                question_index: 0,
                answer: "Test answer".to_string(),
            }],
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!(
                        "/api/review/session/{}/answers/{}/batch",
                        session_id, invalid_card_id
                    ))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_batch_answer_submission_invalid_question_index() {
        let (app, session_id, card_id, _) = setup_test_app().await;

        let request_body = BatchAnswerRequest {
            answers: vec![QuestionAnswer {
                question_index: 99, // Invalid index
                answer: "Test answer".to_string(),
            }],
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!(
                        "/api/review/session/{}/answers/{}/batch",
                        session_id, card_id
                    ))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_batch_answer_submission_empty_answers() {
        let (app, session_id, card_id, _) = setup_test_app().await;

        let request_body = BatchAnswerRequest { answers: vec![] };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!(
                        "/api/review/session/{}/answers/{}/batch",
                        session_id, card_id
                    ))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_batch_answer_submission_mixed_correctness() {
        let (app, session_id, card_id, _) = setup_test_app_with_mixed_results().await;

        let request_body = BatchAnswerRequest {
            answers: vec![
                QuestionAnswer {
                    question_index: 0,
                    answer: "Correct answer".to_string(),
                },
                QuestionAnswer {
                    question_index: 1,
                    answer: "B) Wrong Option".to_string(), // Incorrect
                },
            ],
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!(
                        "/api/review/session/{}/answers/{}/batch",
                        session_id, card_id
                    ))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let results = json_response["data"].as_array().unwrap();
        assert_eq!(results.len(), 2);

        println!("Test received results: {:#?}", results);
        println!("Result 0 is_correct: {}", results[0]["is_correct"]);
        println!("Result 1 is_correct: {}", results[1]["is_correct"]);

        // First answer should be correct
        assert!(results[0]["is_correct"].as_bool().unwrap());

        // Second answer should be incorrect
        assert!(!results[1]["is_correct"].as_bool().unwrap());
    }

    #[tokio::test]
    async fn test_batch_grading_performance() {
        let (app, session_id, card_id, _) = setup_test_app().await;

        // Test with larger number of questions to verify batch efficiency
        let request_body = BatchAnswerRequest {
            answers: (0..10)
                .map(|i| QuestionAnswer {
                    question_index: i % 2, // Alternate between the two available questions
                    answer: format!("Answer {}", i),
                })
                .collect(),
        };

        let start_time = std::time::Instant::now();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!(
                        "/api/review/session/{}/answers/{}/batch",
                        session_id, card_id
                    ))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let elapsed = start_time.elapsed();

        assert_eq!(response.status(), StatusCode::OK);

        // Performance assertion: batch should complete successfully
        // Mock services have a 10ms delay, so with 10 answers and overhead, expect < 1 second
        assert!(elapsed.as_millis() < 1000); // 1 second should be plenty for mock

        // Verify we got all responses back
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let results = json_response["data"].as_array().unwrap();
        assert_eq!(results.len(), 10);
    }

    #[tokio::test]
    async fn test_batch_response_format_consistency() {
        let (app, session_id, card_id, _) = setup_test_app().await;

        let request_body = BatchAnswerRequest {
            answers: vec![QuestionAnswer {
                question_index: 0,
                answer: "Test answer".to_string(),
            }],
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!(
                        "/api/review/session/{}/answers/{}/batch",
                        session_id, card_id
                    ))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

        // Verify response structure matches ApiResponse<Vec<BatchGradingResult>>
        assert!(json_response.is_object());
        assert!(json_response.get("success").is_some());
        assert!(json_response.get("data").is_some());

        let data = json_response["data"].as_array().unwrap();
        for result in data {
            assert!(result.get("question_id").is_some());
            assert!(result.get("is_correct").is_some());
            assert!(result.get("feedback").is_some());
            assert!(result.get("suggested_rating").is_some());

            // Verify suggested_rating is in valid range
            let rating = result["suggested_rating"].as_i64().unwrap();
            assert!(rating >= 1 && rating <= 4);
        }
    }

    #[tokio::test]
    async fn test_batch_answer_ordering_preservation() {
        let (app, session_id, card_id, _) = setup_test_app().await;

        let request_body = BatchAnswerRequest {
            answers: vec![
                QuestionAnswer {
                    question_index: 1,
                    answer: "Second question answer".to_string(),
                },
                QuestionAnswer {
                    question_index: 0,
                    answer: "First question answer".to_string(),
                },
            ],
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!(
                        "/api/review/session/{}/answers/{}/batch",
                        session_id, card_id
                    ))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let results = json_response["data"].as_array().unwrap();
        assert_eq!(results.len(), 2);

        // Results should be ordered by question_id (which corresponds to the order in the batch)
        assert_eq!(results[0]["question_id"], "1");
        assert_eq!(results[1]["question_id"], "2");
    }
}
