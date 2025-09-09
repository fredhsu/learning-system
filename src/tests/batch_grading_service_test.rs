#[cfg(test)]
mod batch_grading_service_tests {
    use super::*;
    use crate::{
        llm_service::LLMService,
        models::*,
    };
    use chrono::Utc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_batch_grading_request_construction() {
        let llm_service = LLMService::new_mock();

        // Create test data
        let card = Card {
            id: Uuid::new_v4(),
            zettel_id: "TEST001".to_string(),
            title: Some("Test Card".to_string()),
            content: "This is test content for batch grading".to_string(),
            creation_date: Utc::now(),
            last_reviewed: None,
            next_review: Utc::now(),
            difficulty: 0.3,
            stability: 2.0,
            retrievability: 0.9,
            reps: 0,
            lapses: 0,
            state: "New".to_string(),
            links: None,
        };

        let questions = vec![
            QuizQuestion {
                question: "What is this about?".to_string(),
                question_type: "short_answer".to_string(),
                options: None,
                correct_answer: Some("Test content".to_string()),
            },
            QuizQuestion {
                question: "Choose correct option".to_string(),
                question_type: "multiple_choice".to_string(),
                options: Some(vec!["A) Correct".to_string(), "B) Wrong".to_string()]),
                correct_answer: Some("A) Correct".to_string()),
            },
        ];

        let user_answers = vec!["Test content".to_string(), "A) Correct".to_string()];

        // Build batch grading requests
        let batch_requests: Vec<BatchGradingRequest> = questions
            .iter()
            .zip(user_answers.iter())
            .map(|(question, answer)| BatchGradingRequest {
                card_content: card.content.clone(),
                question: question.clone(),
                user_answer: answer.clone(),
            })
            .collect();

        assert_eq!(batch_requests.len(), 2);
        assert_eq!(batch_requests[0].card_content, card.content);
        assert_eq!(batch_requests[0].question.question, "What is this about?");
        assert_eq!(batch_requests[0].user_answer, "Test content");
        assert_eq!(batch_requests[1].question.question_type, "multiple_choice");
        assert_eq!(batch_requests[1].user_answer, "A) Correct");
    }

    #[tokio::test]
    async fn test_batch_grading_execution() {
        let llm_service = LLMService::new_mock();

        let batch_requests = vec![
            BatchGradingRequest {
                card_content: "Test card content".to_string(),
                question: QuizQuestion {
                    question: "What is the answer?".to_string(),
                    question_type: "short_answer".to_string(),
                    options: None,
                    correct_answer: Some("42".to_string()),
                },
                user_answer: "42".to_string(),
            },
            BatchGradingRequest {
                card_content: "Test card content".to_string(),
                question: QuizQuestion {
                    question: "Pick the right option".to_string(),
                    question_type: "multiple_choice".to_string(),
                    options: Some(vec!["A) Right".to_string(), "B) Wrong".to_string()]),
                    correct_answer: Some("A) Right".to_string()),
                },
                user_answer: "A) Right".to_string(),
            },
        ];

        let results = llm_service.grade_batch_answers(&batch_requests).await.unwrap();

        assert_eq!(results.len(), 2);

        // Verify each result has required fields
        for (i, result) in results.iter().enumerate() {
            assert_eq!(result.question_id, (i + 1).to_string());
            assert!(result.feedback.len() > 0);
            assert!(result.suggested_rating >= 1 && result.suggested_rating <= 4);
            // Mock service should return correct for exact matches
            assert!(result.is_correct);
        }
    }

    #[tokio::test]
    async fn test_batch_grading_empty_request() {
        let llm_service = LLMService::new_mock();
        let batch_requests: Vec<BatchGradingRequest> = vec![];

        let results = llm_service.grade_batch_answers(&batch_requests).await.unwrap();

        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_batch_grading_single_request() {
        let llm_service = LLMService::new_mock();

        let batch_requests = vec![BatchGradingRequest {
            card_content: "Single test content".to_string(),
            question: QuizQuestion {
                question: "Single question?".to_string(),
                question_type: "short_answer".to_string(),
                options: None,
                correct_answer: Some("Single answer".to_string()),
            },
            user_answer: "Single answer".to_string(),
        }];

        let results = llm_service.grade_batch_answers(&batch_requests).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].question_id, "1");
        assert!(results[0].is_correct);
        assert!(results[0].feedback.len() > 0);
    }

    #[tokio::test]
    async fn test_batch_grading_mixed_question_types() {
        let llm_service = LLMService::new_mock();

        let batch_requests = vec![
            BatchGradingRequest {
                card_content: "Mixed content".to_string(),
                question: QuizQuestion {
                    question: "Short answer question".to_string(),
                    question_type: "short_answer".to_string(),
                    options: None,
                    correct_answer: Some("Short answer".to_string()),
                },
                user_answer: "Short answer".to_string(),
            },
            BatchGradingRequest {
                card_content: "Mixed content".to_string(),
                question: QuizQuestion {
                    question: "Multiple choice question".to_string(),
                    question_type: "multiple_choice".to_string(),
                    options: Some(vec!["A) Option 1".to_string(), "B) Option 2".to_string()]),
                    correct_answer: Some("A) Option 1".to_string()),
                },
                user_answer: "A".to_string(), // Test abbreviated answer
            },
        ];

        let results = llm_service.grade_batch_answers(&batch_requests).await.unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].question_id, "1");
        assert_eq!(results[1].question_id, "2");

        // Both should be correct (mock service should handle abbreviations)
        assert!(results[0].is_correct);
        assert!(results[1].is_correct);
    }

    #[tokio::test]
    async fn test_batch_grading_incorrect_answers() {
        let llm_service = LLMService::new_mock_with_incorrect_answers();

        let batch_requests = vec![
            BatchGradingRequest {
                card_content: "Test content".to_string(),
                question: QuizQuestion {
                    question: "What is correct?".to_string(),
                    question_type: "short_answer".to_string(),
                    options: None,
                    correct_answer: Some("Correct answer".to_string()),
                },
                user_answer: "Wrong answer".to_string(),
            },
            BatchGradingRequest {
                card_content: "Test content".to_string(),
                question: QuizQuestion {
                    question: "Choose right option".to_string(),
                    question_type: "multiple_choice".to_string(),
                    options: Some(vec!["A) Right".to_string(), "B) Wrong".to_string()]),
                    correct_answer: Some("A) Right".to_string()),
                },
                user_answer: "B) Wrong".to_string(),
            },
        ];

        let results = llm_service.grade_batch_answers(&batch_requests).await.unwrap();

        assert_eq!(results.len(), 2);

        // Mock service configured for incorrect answers
        for result in results {
            assert!(!result.is_correct);
            assert!(result.suggested_rating >= 1 && result.suggested_rating <= 2); // Should be low for incorrect
            assert!(result.feedback.len() > 0);
        }
    }

    #[tokio::test]
    async fn test_batch_grading_result_ordering() {
        let llm_service = LLMService::new_mock();

        let batch_requests = vec![
            BatchGradingRequest {
                card_content: "Content 1".to_string(),
                question: QuizQuestion {
                    question: "First question".to_string(),
                    question_type: "short_answer".to_string(),
                    options: None,
                    correct_answer: Some("First answer".to_string()),
                },
                user_answer: "First answer".to_string(),
            },
            BatchGradingRequest {
                card_content: "Content 2".to_string(),
                question: QuizQuestion {
                    question: "Second question".to_string(),
                    question_type: "short_answer".to_string(),
                    options: None,
                    correct_answer: Some("Second answer".to_string()),
                },
                user_answer: "Second answer".to_string(),
            },
            BatchGradingRequest {
                card_content: "Content 3".to_string(),
                question: QuizQuestion {
                    question: "Third question".to_string(),
                    question_type: "short_answer".to_string(),
                    options: None,
                    correct_answer: Some("Third answer".to_string()),
                },
                user_answer: "Third answer".to_string(),
            },
        ];

        let results = llm_service.grade_batch_answers(&batch_requests).await.unwrap();

        assert_eq!(results.len(), 3);

        // Verify results maintain order
        assert_eq!(results[0].question_id, "1");
        assert_eq!(results[1].question_id, "2");
        assert_eq!(results[2].question_id, "3");
    }

    #[tokio::test]
    async fn test_batch_grading_long_content_truncation() {
        let llm_service = LLMService::new_mock();

        // Create content longer than 300 characters to test truncation
        let long_content = "A".repeat(500);

        let batch_requests = vec![BatchGradingRequest {
            card_content: long_content.clone(),
            question: QuizQuestion {
                question: "Question about long content".to_string(),
                question_type: "short_answer".to_string(),
                options: None,
                correct_answer: Some("Answer".to_string()),
            },
            user_answer: "Answer".to_string(),
        }];

        // This should not fail due to content length
        let results = llm_service.grade_batch_answers(&batch_requests).await.unwrap();

        assert_eq!(results.len(), 1);
        assert!(results[0].feedback.len() > 0);
    }

    #[tokio::test]
    async fn test_batch_grading_fallback_mechanism() {
        let llm_service = LLMService::new_mock_with_batch_failure();

        let batch_requests = vec![
            BatchGradingRequest {
                card_content: "Fallback test".to_string(),
                question: QuizQuestion {
                    question: "Fallback question".to_string(),
                    question_type: "short_answer".to_string(),
                    options: None,
                    correct_answer: Some("Fallback answer".to_string()),
                },
                user_answer: "Fallback answer".to_string(),
            },
        ];

        // Should fall back to individual grading
        let results = llm_service.grade_batch_answers(&batch_requests).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].question_id, "1");
        // Fallback should still provide valid results
        assert!(results[0].feedback.len() > 0);
    }

    #[tokio::test]
    async fn test_batch_grading_large_batch() {
        let llm_service = LLMService::new_mock();

        // Test with larger batch to verify scalability
        let batch_requests: Vec<BatchGradingRequest> = (0..20)
            .map(|i| BatchGradingRequest {
                card_content: format!("Content {}", i),
                question: QuizQuestion {
                    question: format!("Question {}", i),
                    question_type: "short_answer".to_string(),
                    options: None,
                    correct_answer: Some(format!("Answer {}", i)),
                },
                user_answer: format!("Answer {}", i),
            })
            .collect();

        let results = llm_service.grade_batch_answers(&batch_requests).await.unwrap();

        assert_eq!(results.len(), 20);

        for (i, result) in results.iter().enumerate() {
            assert_eq!(result.question_id, (i + 1).to_string());
            assert!(result.is_correct);
            assert!(result.feedback.len() > 0);
        }
    }
}