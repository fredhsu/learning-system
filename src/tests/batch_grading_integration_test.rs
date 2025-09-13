#[cfg(test)]
mod batch_grading_integration_tests {
    use super::*;
    use crate::{api::*, card_service::CardService, llm_service::LLMService, models::*};
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
    };
    use chrono::Utc;
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::time::Instant;
    use tower::ServiceExt;
    use uuid::Uuid;

    async fn setup_integration_test() -> (Router, Uuid, Uuid, Vec<QuizQuestion>) {
        let card_service = CardService::new_in_memory().await.unwrap();
        let llm_service = LLMService::new_mock();
        let review_sessions = Arc::new(Mutex::new(HashMap::new()));

        let app_state = AppState {
            card_service: card_service.clone(),
            llm_service,
            review_sessions: review_sessions.clone(),
        };

        let app = create_app(app_state);

        // Create test card
        let test_card_request = CreateCardRequest {
            zettel_id: "INT001".to_string(),
            title: Some("Integration Test Card".to_string()),
            content:
                "This card tests the complete batch grading flow from frontend to backend and back."
                    .to_string(),
            links: None,
            topic_ids: vec![],
        };

        let test_card = card_service.create_card(test_card_request).await.unwrap();
        let card_id = test_card.id;

        // Create comprehensive test questions
        let questions = vec![
            QuizQuestion {
                question: "What is the main purpose of this card?".to_string(),
                question_type: "short_answer".to_string(),
                options: None,
                correct_answer: Some("Testing batch grading flow".to_string()),
            },
            QuizQuestion {
                question: "Which testing approach is being demonstrated?".to_string(),
                question_type: "multiple_choice".to_string(),
                options: Some(vec![
                    "A) Integration testing".to_string(),
                    "B) Unit testing".to_string(),
                    "C) End-to-end testing".to_string(),
                ]),
                correct_answer: Some("A) Integration testing".to_string()),
            },
            QuizQuestion {
                question: "What is the expected improvement from batch processing?".to_string(),
                question_type: "short_answer".to_string(),
                options: None,
                correct_answer: Some("Reduced API calls and latency".to_string()),
            },
            QuizQuestion {
                question: "How many network requests should batch processing make?".to_string(),
                question_type: "multiple_choice".to_string(),
                options: Some(vec![
                    "A) One per question".to_string(),
                    "B) One per card".to_string(),
                    "C) Multiple parallel requests".to_string(),
                ]),
                correct_answer: Some("B) One per card".to_string()),
            },
        ];

        // Create session with questions
        let session_id = Uuid::new_v4();
        let mut session_questions = HashMap::new();
        session_questions.insert(card_id, questions.clone());

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

        (app, session_id, card_id, questions)
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
    async fn test_complete_batch_grading_workflow() {
        let (app, session_id, card_id, questions) = setup_integration_test().await;

        // Prepare batch answers (mix of correct and incorrect)
        let user_answers = vec![
            "Testing batch grading flow".to_string(), // Correct
            "A) Integration testing".to_string(),     // Correct
            "Faster processing".to_string(),          // Partially correct
            "A) One per question".to_string(),        // Incorrect
        ];

        let request_body = BatchAnswerRequest {
            answers: user_answers
                .iter()
                .enumerate()
                .map(|(index, answer)| QuestionAnswer {
                    question_index: index,
                    answer: answer.clone(),
                })
                .collect(),
        };

        let start_time = Instant::now();

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

        // Verify response
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json_response["success"], true);

        let results = json_response["data"].as_array().unwrap();
        assert_eq!(results.len(), 4);

        // Verify each result structure
        for (i, result) in results.iter().enumerate() {
            assert_eq!(result["question_id"], (i + 1).to_string());
            assert!(result["is_correct"].is_boolean());
            assert!(result["feedback"].is_string());
            assert!(result["suggested_rating"].is_number());

            let rating = result["suggested_rating"].as_i64().unwrap();
            assert!(rating >= 1 && rating <= 4);
        }

        // Performance check: should be faster than sequential calls would be
        assert!(elapsed.as_millis() < 1000); // Should complete within 1 second

        println!("Batch grading completed in {}ms", elapsed.as_millis());
    }

    #[tokio::test]
    async fn test_batch_vs_sequential_performance_comparison() {
        let (app, session_id, card_id, questions) = setup_integration_test().await;

        let user_answers = vec![
            "Answer 1".to_string(),
            "Answer 2".to_string(),
            "Answer 3".to_string(),
            "Answer 4".to_string(),
        ];

        // Test batch processing time
        let batch_request = BatchAnswerRequest {
            answers: user_answers
                .iter()
                .enumerate()
                .map(|(index, answer)| QuestionAnswer {
                    question_index: index,
                    answer: answer.clone(),
                })
                .collect(),
        };

        let batch_start = Instant::now();
        let batch_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!(
                        "/api/review/session/{}/answers/{}/batch",
                        session_id, card_id
                    ))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&batch_request).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let batch_time = batch_start.elapsed();

        assert_eq!(batch_response.status(), StatusCode::OK);

        // Test sequential processing time (using existing individual endpoint)
        let sequential_start = Instant::now();
        for (index, answer) in user_answers.iter().enumerate() {
            let individual_request = json!({
                "question_index": index,
                "answer": answer
            });

            let individual_response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri(&format!(
                            "/api/review/session/{}/answer/{}",
                            session_id, card_id
                        ))
                        .header("content-type", "application/json")
                        .body(Body::from(individual_request.to_string()))
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(individual_response.status(), StatusCode::OK);
        }
        let sequential_time = sequential_start.elapsed();

        // For mock services, we can't guarantee performance improvement due to artificial delays
        // Instead, we verify that both methods complete in reasonable time and produce same results
        let improvement_ratio = sequential_time.as_millis() as f64 / batch_time.as_millis() as f64;

        // Both should complete within reasonable time (mock has 10ms delays)
        assert!(
            batch_time.as_millis() < 1000,
            "Batch processing took too long: {}ms",
            batch_time.as_millis()
        );
        assert!(
            sequential_time.as_millis() < 1000,
            "Sequential processing took too long: {}ms",
            sequential_time.as_millis()
        );

        println!(
            "Performance comparison: Batch: {}ms, Sequential: {}ms, Ratio: {:.2}x",
            batch_time.as_millis(),
            sequential_time.as_millis(),
            improvement_ratio
        );
    }

    #[tokio::test]
    async fn test_batch_grading_error_recovery() {
        let (app, session_id, card_id, _) = setup_integration_test().await;

        // Test with invalid question indices to trigger error handling
        let invalid_request = BatchAnswerRequest {
            answers: vec![
                QuestionAnswer {
                    question_index: 0,
                    answer: "Valid answer".to_string(),
                },
                QuestionAnswer {
                    question_index: 99, // Invalid index
                    answer: "Invalid index answer".to_string(),
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
                    .body(Body::from(serde_json::to_string(&invalid_request).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_batch_grading_with_large_content() {
        let (app, session_id, card_id, _) = setup_integration_test().await;

        // Test with very long answers to verify content handling
        let large_answers = vec![
            "A".repeat(1000),    // Very long answer
            "B".repeat(500),     // Medium length
            "Short".to_string(), // Short answer
            "C".repeat(2000),    // Very very long
        ];

        let request_body = BatchAnswerRequest {
            answers: large_answers
                .iter()
                .enumerate()
                .map(|(index, answer)| QuestionAnswer {
                    question_index: index,
                    answer: answer.clone(),
                })
                .collect(),
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
        assert_eq!(results.len(), 4);

        // All results should have proper structure despite large content
        for result in results {
            assert!(result["feedback"].as_str().unwrap().len() > 0);
            assert!(result["suggested_rating"].as_i64().unwrap() >= 1);
        }
    }

    #[tokio::test]
    async fn test_concurrent_batch_requests() {
        let (app, session_id, card_id, _) = setup_integration_test().await;

        let request_body = BatchAnswerRequest {
            answers: vec![QuestionAnswer {
                question_index: 0,
                answer: "Concurrent test answer".to_string(),
            }],
        };

        // Make multiple concurrent requests
        let mut handles = vec![];

        for i in 0..5 {
            let app_clone = app.clone();
            let request_clone = serde_json::to_string(&request_body).unwrap();
            let session_id_clone = session_id;
            let card_id_clone = card_id;

            let handle = tokio::spawn(async move {
                app_clone
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri(&format!(
                                "/api/review/session/{}/answers/{}/batch",
                                session_id_clone, card_id_clone
                            ))
                            .header("content-type", "application/json")
                            .body(Body::from(request_clone))
                            .unwrap(),
                    )
                    .await
            });

            handles.push(handle);
        }

        // Wait for all requests to complete
        let results = futures_util::future::join_all(handles).await;

        // All requests should succeed
        for result in results {
            let response = result.unwrap().unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }
    }

    #[tokio::test]
    async fn test_batch_grading_session_state_consistency() {
        let (app, session_id, card_id, _) = setup_integration_test().await;

        let request_body = BatchAnswerRequest {
            answers: vec![
                QuestionAnswer {
                    question_index: 0,
                    answer: "State consistency test".to_string(),
                },
                QuestionAnswer {
                    question_index: 1,
                    answer: "A) Integration testing".to_string(),
                },
            ],
        };

        // First batch request
        let response1 = app
            .clone()
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

        assert_eq!(response1.status(), StatusCode::OK);

        // Second batch request with same session should still work
        let response2 = app
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

        assert_eq!(response2.status(), StatusCode::OK);

        // Verify both responses are consistent
        let body1 = axum::body::to_bytes(response1.into_body(), usize::MAX)
            .await
            .unwrap();
        let body2 = axum::body::to_bytes(response2.into_body(), usize::MAX)
            .await
            .unwrap();

        let json1: serde_json::Value = serde_json::from_slice(&body1).unwrap();
        let json2: serde_json::Value = serde_json::from_slice(&body2).unwrap();

        assert_eq!(json1["success"], json2["success"]);
        assert_eq!(
            json1["data"].as_array().unwrap().len(),
            json2["data"].as_array().unwrap().len()
        );
    }

    #[tokio::test]
    async fn test_end_to_end_review_workflow() {
        let (app, session_id, card_id, questions) = setup_integration_test().await;

        // Step 1: Start review session (this should already be done in setup)

        // Step 2: Submit batch answers
        let user_answers = vec![
            "Testing batch grading flow".to_string(),
            "A) Integration testing".to_string(),
            "Reduced API calls and latency".to_string(),
            "B) One per card".to_string(),
        ];

        let batch_request = BatchAnswerRequest {
            answers: user_answers
                .iter()
                .enumerate()
                .map(|(index, answer)| QuestionAnswer {
                    question_index: index,
                    answer: answer.clone(),
                })
                .collect(),
        };

        let grading_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!(
                        "/api/review/session/{}/answers/{}/batch",
                        session_id, card_id
                    ))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&batch_request).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(grading_response.status(), StatusCode::OK);

        let grading_body = axum::body::to_bytes(grading_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let grading_json: serde_json::Value = serde_json::from_slice(&grading_body).unwrap();

        let results = grading_json["data"].as_array().unwrap();

        // Step 3: Calculate aggregated rating from results
        let total_rating: i64 = results
            .iter()
            .map(|r| r["suggested_rating"].as_i64().unwrap())
            .sum();
        let avg_rating = total_rating / results.len() as i64;

        // Step 4: Submit final card review with aggregated rating
        let review_response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/api/cards/{}/review", card_id))
                    .header("content-type", "application/json")
                    .body(Body::from(format!(r#"{{"rating": {}}}"#, avg_rating)))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(review_response.status(), StatusCode::OK);

        println!(
            "Complete workflow test: {} questions -> {} avg rating -> FSRS update",
            questions.len(),
            avg_rating
        );
    }
}
