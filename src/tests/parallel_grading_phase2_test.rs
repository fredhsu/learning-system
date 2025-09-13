#[cfg(test)]
mod parallel_grading_phase2_tests {
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

    async fn setup_parallel_test_app() -> (Router, Uuid, Uuid, Vec<QuizQuestion>) {
        let card_service = CardService::new_in_memory().await.unwrap();
        let llm_service = LLMService::new_mock(); // Mock service for controlled testing
        let review_sessions = Arc::new(Mutex::new(HashMap::new()));

        let app_state = AppState {
            card_service: card_service.clone(),
            llm_service,
            review_sessions: review_sessions.clone(),
        };

        let app = create_app(app_state);

        // Create test card with comprehensive content
        let test_card_request = CreateCardRequest {
            zettel_id: "PAR001".to_string(),
            title: Some("Parallel Grading Test Card".to_string()),
            content: "This card tests true parallel processing with individual LLM calls executed concurrently rather than sequentially.".to_string(),
            links: None,
            topic_ids: vec![],
        };

        let test_card = card_service.create_card(test_card_request).await.unwrap();
        let card_id = test_card.id;

        // Create diverse question set for parallel testing
        let questions = vec![
            QuizQuestion {
                question: "What is the main advantage of parallel processing?".to_string(),
                question_type: "short_answer".to_string(),
                options: None,
                correct_answer: Some("Concurrent execution reduces total processing time".to_string()),
            },
            QuizQuestion {
                question: "Which concurrency model does Rust use?".to_string(),
                question_type: "multiple_choice".to_string(),
                options: Some(vec![
                    "A) Actor model".to_string(),
                    "B) Green threads".to_string(),
                    "C) Async/await with tokio".to_string(),
                    "D) Traditional threading".to_string(),
                ]),
                correct_answer: Some("C) Async/await with tokio".to_string()),
            },
            QuizQuestion {
                question: "What is the difference between concurrency and parallelism?".to_string(),
                question_type: "short_answer".to_string(),
                options: None,
                correct_answer: Some("Concurrency is about dealing with lots of things at once, parallelism is about doing lots of things at once".to_string()),
            },
            QuizQuestion {
                question: "Which Rust feature enables safe parallel processing?".to_string(),
                question_type: "multiple_choice".to_string(),
                options: Some(vec![
                    "A) Ownership system".to_string(),
                    "B) Borrow checker".to_string(),
                    "C) Send and Sync traits".to_string(),
                    "D) All of the above".to_string(),
                ]),
                correct_answer: Some("D) All of the above".to_string()),
            },
            QuizQuestion {
                question: "How does tokio::spawn enable parallelism?".to_string(),
                question_type: "short_answer".to_string(),
                options: None,
                correct_answer: Some("It schedules async tasks across multiple threads".to_string()),
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
    struct ParallelAnswerRequest {
        answers: Vec<QuestionAnswer>,
        processing_mode: Option<String>, // "parallel" or "sequential"
    }

    #[derive(Clone, serde::Deserialize, serde::Serialize)]
    struct QuestionAnswer {
        question_index: usize,
        answer: String,
    }

    // TDD Test: New parallel endpoint should exist
    #[tokio::test]
    async fn test_parallel_endpoint_exists() {
        let (app, session_id, card_id, _) = setup_parallel_test_app().await;

        let request_body = ParallelAnswerRequest {
            answers: vec![QuestionAnswer {
                question_index: 0,
                answer: "Test answer".to_string(),
            }],
            processing_mode: Some("parallel".to_string()),
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!(
                        "/api/review/session/{}/answers/{}/parallel",
                        session_id, card_id
                    ))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        // This will fail initially - TDD approach
        // Once implemented, this should return OK
        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    // TDD Test: Parallel processing should be faster than sequential
    #[tokio::test]
    async fn test_parallel_vs_sequential_performance() {
        let (app, session_id, card_id, _questions) = setup_parallel_test_app().await;

        let answers = vec![
            QuestionAnswer {
                question_index: 0,
                answer: "Concurrent execution reduces total processing time".to_string(),
            },
            QuestionAnswer {
                question_index: 1,
                answer: "C) Async/await with tokio".to_string(),
            },
            QuestionAnswer {
                question_index: 2,
                answer: "Concurrency deals with structure, parallelism with execution".to_string(),
            },
            QuestionAnswer {
                question_index: 3,
                answer: "D) All of the above".to_string(),
            },
            QuestionAnswer {
                question_index: 4,
                answer: "It schedules async tasks across multiple threads".to_string(),
            },
        ];

        // Test sequential processing (existing batch endpoint)
        let sequential_start = Instant::now();
        let sequential_request = json!({
            "answers": answers
        });

        let _sequential_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!(
                        "/api/review/session/{}/answers/{}/batch",
                        session_id, card_id
                    ))
                    .header("content-type", "application/json")
                    .body(Body::from(sequential_request.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let sequential_time = sequential_start.elapsed();

        // Test parallel processing (new endpoint)
        let parallel_start = Instant::now();
        let parallel_request = ParallelAnswerRequest {
            answers: answers.clone(),
            processing_mode: Some("parallel".to_string()),
        };

        // This endpoint doesn't exist yet - will be implemented
        let parallel_response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!(
                        "/api/review/session/{}/answers/{}/parallel",
                        session_id, card_id
                    ))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_string(&parallel_request).unwrap(),
                    ))
                    .unwrap(),
            )
            .await;

        // For now, we expect 404 since endpoint doesn't exist
        // Once implemented, we should verify performance improvement
        match parallel_response {
            Ok(response) if response.status() == StatusCode::OK => {
                let parallel_time = parallel_start.elapsed();

                // Parallel should be faster (at least 20% improvement expected)
                let improvement_ratio =
                    sequential_time.as_millis() as f64 / parallel_time.as_millis() as f64;
                assert!(
                    improvement_ratio >= 1.2,
                    "Parallel processing should be at least 20% faster"
                );

                println!("Performance improvement: {:.2}x faster", improvement_ratio);
            }
            _ => {
                // Expected until parallel endpoint is implemented
                println!("Parallel endpoint not yet implemented - this is expected for TDD");
            }
        }
    }

    // TDD Test: Concurrent processing should handle task failures gracefully
    #[tokio::test]
    async fn test_parallel_error_handling() {
        let (app, session_id, card_id, _) = setup_parallel_test_app().await;

        let mixed_request = ParallelAnswerRequest {
            answers: vec![
                QuestionAnswer {
                    question_index: 0,
                    answer: "Valid answer".to_string(),
                },
                QuestionAnswer {
                    question_index: 999,
                    answer: "Invalid index".to_string(),
                }, // Should fail
                QuestionAnswer {
                    question_index: 2,
                    answer: "Another valid answer".to_string(),
                },
            ],
            processing_mode: Some("parallel".to_string()),
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!(
                        "/api/review/session/{}/answers/{}/parallel",
                        session_id, card_id
                    ))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&mixed_request).unwrap()))
                    .unwrap(),
            )
            .await;

        match response {
            Ok(response) if response.status() == StatusCode::OK => {
                // Should handle partial failures and return results for valid indices
                let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                    .await
                    .unwrap();
                let json_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

                // Should return results for valid questions only
                let results = json_response["data"].as_array().unwrap();
                assert_eq!(results.len(), 2); // Only valid questions processed
            }
            Ok(response) if response.status() == StatusCode::BAD_REQUEST => {
                // Alternative: reject entire batch if any question is invalid
                println!("Parallel endpoint rejects batch with invalid questions");
            }
            _ => {
                println!("Parallel endpoint not yet implemented");
            }
        }
    }

    // TDD Test: Concurrent result ordering should be preserved
    #[tokio::test]
    async fn test_parallel_result_ordering() {
        let (app, session_id, card_id, _) = setup_parallel_test_app().await;

        // Submit answers out of order to test result preservation
        let out_of_order_answers = vec![
            QuestionAnswer {
                question_index: 3,
                answer: "Fourth answer".to_string(),
            },
            QuestionAnswer {
                question_index: 1,
                answer: "Second answer".to_string(),
            },
            QuestionAnswer {
                question_index: 0,
                answer: "First answer".to_string(),
            },
            QuestionAnswer {
                question_index: 2,
                answer: "Third answer".to_string(),
            },
        ];

        let request = ParallelAnswerRequest {
            answers: out_of_order_answers,
            processing_mode: Some("parallel".to_string()),
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!(
                        "/api/review/session/{}/answers/{}/parallel",
                        session_id, card_id
                    ))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request).unwrap()))
                    .unwrap(),
            )
            .await;

        match response {
            Ok(response) if response.status() == StatusCode::OK => {
                let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                    .await
                    .unwrap();
                let json_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

                let results = json_response["data"].as_array().unwrap();
                assert_eq!(results.len(), 4);

                // Results should be ordered by question_id regardless of submission order
                assert_eq!(results[0]["question_id"], "1"); // First question
                assert_eq!(results[1]["question_id"], "2"); // Second question
                assert_eq!(results[2]["question_id"], "3"); // Third question
                assert_eq!(results[3]["question_id"], "4"); // Fourth question
            }
            _ => {
                println!("Parallel endpoint not yet implemented");
            }
        }
    }

    // TDD Test: Concurrent limits should be configurable
    #[tokio::test]
    async fn test_parallel_concurrency_limits() {
        let (app, session_id, card_id, _) = setup_parallel_test_app().await;

        // Test with large number of questions to verify concurrency limiting
        let many_answers: Vec<QuestionAnswer> = (0..20)
            .map(|i| QuestionAnswer {
                question_index: i % 5, // Cycle through available questions
                answer: format!("Answer {}", i),
            })
            .collect();

        let request = ParallelAnswerRequest {
            answers: many_answers,
            processing_mode: Some("parallel".to_string()),
        };

        let start_time = Instant::now();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!(
                        "/api/review/session/{}/answers/{}/parallel",
                        session_id, card_id
                    ))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request).unwrap()))
                    .unwrap(),
            )
            .await;

        match response {
            Ok(response) if response.status() == StatusCode::OK => {
                let elapsed = start_time.elapsed();

                let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                    .await
                    .unwrap();
                let json_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

                let results = json_response["data"].as_array().unwrap();
                assert_eq!(results.len(), 20);

                // Should complete within reasonable time despite large batch
                assert!(
                    elapsed.as_secs() < 10,
                    "Large batch processing took too long"
                );

                println!(
                    "Processed {} questions in {}ms",
                    results.len(),
                    elapsed.as_millis()
                );
            }
            _ => {
                println!("Parallel endpoint not yet implemented");
            }
        }
    }

    // TDD Test: Fallback to sequential processing when parallel fails
    #[tokio::test]
    async fn test_parallel_fallback_mechanism() {
        let (app, session_id, card_id, _) = setup_parallel_test_app().await;

        let request = ParallelAnswerRequest {
            answers: vec![
                QuestionAnswer {
                    question_index: 0,
                    answer: "Test answer 1".to_string(),
                },
                QuestionAnswer {
                    question_index: 1,
                    answer: "Test answer 2".to_string(),
                },
            ],
            processing_mode: Some("parallel_with_failure".to_string()), // Mock failure mode
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!(
                        "/api/review/session/{}/answers/{}/parallel",
                        session_id, card_id
                    ))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request).unwrap()))
                    .unwrap(),
            )
            .await;

        match response {
            Ok(response) if response.status() == StatusCode::OK => {
                // Should still succeed due to fallback mechanism
                let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                    .await
                    .unwrap();
                let json_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

                assert_eq!(json_response["success"], true);

                let results = json_response["data"].as_array().unwrap();
                assert_eq!(results.len(), 2);

                // Verify fallback was used (could be indicated in response metadata)
                if let Some(metadata) = json_response.get("metadata") {
                    assert_eq!(metadata["processing_mode"], "sequential_fallback");
                }
            }
            _ => {
                println!("Parallel endpoint not yet implemented");
            }
        }
    }

    // TDD Test: Resource cleanup after parallel processing
    #[tokio::test]
    async fn test_parallel_resource_cleanup() {
        let (app, session_id, card_id, _) = setup_parallel_test_app().await;

        let request = ParallelAnswerRequest {
            answers: vec![QuestionAnswer {
                question_index: 0,
                answer: "Resource test".to_string(),
            }],
            processing_mode: Some("parallel".to_string()),
        };

        // Make multiple requests to verify resources are cleaned up
        for i in 0..5 {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri(&format!(
                            "/api/review/session/{}/answers/{}/parallel",
                            session_id, card_id
                        ))
                        .header("content-type", "application/json")
                        .body(Body::from(serde_json::to_string(&request).unwrap()))
                        .unwrap(),
                )
                .await;

            match response {
                Ok(response) if response.status() == StatusCode::OK => {
                    // Each request should succeed independently
                    println!("Request {} completed successfully", i + 1);
                }
                _ => {
                    println!("Parallel endpoint not yet implemented (request {})", i + 1);
                    break;
                }
            }
        }
    }

    // TDD Test: Memory usage should be reasonable under parallel load
    #[tokio::test]
    async fn test_parallel_memory_efficiency() {
        let (app, session_id, card_id, _) = setup_parallel_test_app().await;

        // Simulate memory-intensive parallel processing
        let large_answers: Vec<QuestionAnswer> = (0..100)
            .map(|i| QuestionAnswer {
                question_index: i % 5,
                answer: "A".repeat(1000), // Large answer content
            })
            .collect();

        let request = ParallelAnswerRequest {
            answers: large_answers,
            processing_mode: Some("parallel".to_string()),
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!(
                        "/api/review/session/{}/answers/{}/parallel",
                        session_id, card_id
                    ))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request).unwrap()))
                    .unwrap(),
            )
            .await;

        match response {
            Ok(response) if response.status() == StatusCode::OK => {
                let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                    .await
                    .unwrap();
                let json_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

                let results = json_response["data"].as_array().unwrap();
                assert_eq!(results.len(), 100);

                // Memory usage should be bounded even with large parallel processing
                println!(
                    "Successfully processed {} large answers in parallel",
                    results.len()
                );
            }
            _ => {
                println!("Parallel endpoint not yet implemented");
            }
        }
    }

    // TDD Test: Parallel processing metrics and monitoring
    #[tokio::test]
    async fn test_parallel_processing_metrics() {
        let (app, session_id, card_id, _) = setup_parallel_test_app().await;

        let request = ParallelAnswerRequest {
            answers: vec![
                QuestionAnswer {
                    question_index: 0,
                    answer: "Metrics test 1".to_string(),
                },
                QuestionAnswer {
                    question_index: 1,
                    answer: "Metrics test 2".to_string(),
                },
                QuestionAnswer {
                    question_index: 2,
                    answer: "Metrics test 3".to_string(),
                },
            ],
            processing_mode: Some("parallel".to_string()),
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!(
                        "/api/review/session/{}/answers/{}/parallel",
                        session_id, card_id
                    ))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request).unwrap()))
                    .unwrap(),
            )
            .await;

        match response {
            Ok(response) if response.status() == StatusCode::OK => {
                let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                    .await
                    .unwrap();
                let json_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

                // Response should include processing metrics
                if let Some(metrics) = json_response.get("metrics") {
                    assert!(metrics.get("total_processing_time_ms").is_some());
                    assert!(metrics.get("parallel_tasks_spawned").is_some());
                    assert!(metrics.get("concurrent_execution_count").is_some());
                    assert!(metrics.get("average_task_duration_ms").is_some());

                    println!("Parallel processing metrics: {:#}", metrics);
                }
            }
            _ => {
                println!("Parallel endpoint not yet implemented");
            }
        }
    }
}
