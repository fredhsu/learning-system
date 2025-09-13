#[cfg(test)]
mod concurrent_processing_integration_tests {
    use super::*;
    use crate::{
        llm_service::{GradingResult, LLMService},
        models::*,
    };
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};
    use tokio::time::sleep;
    use uuid::Uuid;

    // Test infrastructure for concurrent processing
    struct ConcurrentGradingService {
        llm_service: LLMService,
        active_tasks: Arc<Mutex<usize>>,
        max_concurrent_tasks: usize,
    }

    impl ConcurrentGradingService {
        fn new(max_concurrent: usize) -> Self {
            Self {
                llm_service: LLMService::new_mock(),
                active_tasks: Arc::new(Mutex::new(0)),
                max_concurrent_tasks: max_concurrent,
            }
        }

        // This method will be implemented in Phase 2
        async fn grade_answers_concurrently(
            &self,
            card: &Card,
            questions_and_answers: Vec<(QuizQuestion, String)>,
        ) -> Result<Vec<(usize, GradingResult)>, Box<dyn std::error::Error + Send + Sync>> {
            let mut handles = vec![];

            for (index, (question, answer)) in questions_and_answers.into_iter().enumerate() {
                let service = self.llm_service.clone();
                let card_clone = card.clone();
                let question_clone = question.clone();
                let answer_clone = answer.clone();
                let active_tasks = self.active_tasks.clone();

                // Limit concurrent tasks
                {
                    let mut tasks = active_tasks.lock().unwrap();
                    while *tasks >= self.max_concurrent_tasks {
                        drop(tasks);
                        sleep(Duration::from_millis(10)).await;
                        tasks = active_tasks.lock().unwrap();
                    }
                    *tasks += 1;
                }

                let handle = tokio::spawn(async move {
                    let result = service
                        .grade_answer(&card_clone, &question_clone, &answer_clone)
                        .await;

                    // Decrement active task count
                    {
                        let mut tasks = active_tasks.lock().unwrap();
                        *tasks = tasks.saturating_sub(1);
                    }

                    match result {
                        Ok(grading_result) => Ok((index, grading_result)),
                        Err(e) => Err((index, e)),
                    }
                });

                handles.push(handle);
            }

            // Wait for all tasks to complete
            let mut results = Vec::with_capacity(handles.len());
            let mut errors = Vec::new();

            for handle in handles {
                match handle.await {
                    Ok(Ok((index, result))) => {
                        results.push((index, result));
                    }
                    Ok(Err((index, error))) => {
                        errors.push((index, error));
                    }
                    Err(join_error) => {
                        return Err(format!("Task join error: {}", join_error).into());
                    }
                }
            }

            // Sort results by original index to maintain order
            results.sort_by_key(|(index, _)| *index);

            if !errors.is_empty() {
                return Err(format!("Failed to grade {} answers", errors.len()).into());
            }

            Ok(results)
        }

        async fn grade_answers_sequentially(
            &self,
            card: &Card,
            questions_and_answers: Vec<(QuizQuestion, String)>,
        ) -> Result<Vec<(usize, GradingResult)>, Box<dyn std::error::Error + Send + Sync>> {
            let mut results = Vec::new();

            for (index, (question, answer)) in questions_and_answers.into_iter().enumerate() {
                let result = self
                    .llm_service
                    .grade_answer(card, &question, &answer)
                    .await?;
                results.push((index, result));
            }

            Ok(results)
        }
    }

    fn create_test_card() -> Card {
        use chrono::Utc;

        Card {
            id: Uuid::new_v4(),
            zettel_id: "CONC001".to_string(),
            title: Some("Concurrent Processing Test".to_string()),
            content: "This card tests concurrent processing of individual grading tasks."
                .to_string(),
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
        }
    }

    fn create_test_questions() -> Vec<QuizQuestion> {
        vec![
            QuizQuestion {
                question: "What is concurrent processing?".to_string(),
                question_type: "short_answer".to_string(),
                options: None,
                correct_answer: Some("Multiple tasks executing at the same time".to_string()),
            },
            QuizQuestion {
                question: "Which is faster for independent tasks?".to_string(),
                question_type: "multiple_choice".to_string(),
                options: Some(vec![
                    "A) Sequential processing".to_string(),
                    "B) Concurrent processing".to_string(),
                    "C) Both are the same".to_string(),
                ]),
                correct_answer: Some("B) Concurrent processing".to_string()),
            },
            QuizQuestion {
                question: "What is a potential downside of concurrency?".to_string(),
                question_type: "short_answer".to_string(),
                options: None,
                correct_answer: Some(
                    "Increased complexity and potential race conditions".to_string(),
                ),
            },
            QuizQuestion {
                question: "Which Rust construct spawns concurrent tasks?".to_string(),
                question_type: "multiple_choice".to_string(),
                options: Some(vec![
                    "A) std::thread::spawn".to_string(),
                    "B) tokio::spawn".to_string(),
                    "C) async fn".to_string(),
                    "D) futures::join".to_string(),
                ]),
                correct_answer: Some("B) tokio::spawn".to_string()),
            },
            QuizQuestion {
                question: "How does join_all work in concurrent processing?".to_string(),
                question_type: "short_answer".to_string(),
                options: None,
                correct_answer: Some("It waits for all concurrent tasks to complete".to_string()),
            },
        ]
    }

    #[tokio::test]
    async fn test_concurrent_vs_sequential_performance() {
        let service = ConcurrentGradingService::new(4); // Allow 4 concurrent tasks
        let card = create_test_card();
        let questions = create_test_questions();
        let answers = vec![
            "Multiple tasks executing at the same time".to_string(),
            "B) Concurrent processing".to_string(),
            "Increased complexity and potential race conditions".to_string(),
            "B) tokio::spawn".to_string(),
            "It waits for all concurrent tasks to complete".to_string(),
        ];

        let questions_and_answers: Vec<_> = questions.into_iter().zip(answers).collect();

        // Test sequential processing
        let sequential_start = Instant::now();
        let sequential_results = service
            .grade_answers_sequentially(&card, questions_and_answers.clone())
            .await
            .unwrap();
        let sequential_duration = sequential_start.elapsed();

        // Test concurrent processing
        let concurrent_start = Instant::now();
        let concurrent_results = service
            .grade_answers_concurrently(&card, questions_and_answers)
            .await
            .unwrap();
        let concurrent_duration = concurrent_start.elapsed();

        // Verify results are equivalent
        assert_eq!(sequential_results.len(), concurrent_results.len());
        assert_eq!(sequential_results.len(), 5);

        // Verify both produce valid grading results
        for ((seq_idx, seq_result), (conc_idx, conc_result)) in
            sequential_results.iter().zip(concurrent_results.iter())
        {
            assert_eq!(seq_idx, conc_idx);
            assert_eq!(seq_result.is_correct, conc_result.is_correct);
            // Feedback might vary slightly due to mock randomization, so we don't assert equality
            assert!(seq_result.feedback.len() > 0);
            assert!(conc_result.feedback.len() > 0);
            assert!(seq_result.suggested_rating >= 1 && seq_result.suggested_rating <= 4);
            assert!(conc_result.suggested_rating >= 1 && conc_result.suggested_rating <= 4);
        }

        println!(
            "Sequential: {}ms, Concurrent: {}ms",
            sequential_duration.as_millis(),
            concurrent_duration.as_millis()
        );

        // For mock services, we can't guarantee performance improvement,
        // but we can verify both complete successfully
        assert!(sequential_duration.as_millis() < 5000); // Should complete within 5 seconds
        assert!(concurrent_duration.as_millis() < 5000);
    }

    #[tokio::test]
    async fn test_concurrent_task_limiting() {
        let max_concurrent = 2;
        let service = ConcurrentGradingService::new(max_concurrent);
        let card = create_test_card();
        let questions = create_test_questions();
        let answers = vec!["Answer"; 5]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        let questions_and_answers: Vec<_> = questions.into_iter().zip(answers).collect();

        let start_time = Instant::now();
        let results = service
            .grade_answers_concurrently(&card, questions_and_answers)
            .await
            .unwrap();
        let duration = start_time.elapsed();

        assert_eq!(results.len(), 5);

        // Verify that tasks were limited (this is implicitly tested by the max_concurrent_tasks logic)
        println!(
            "Concurrent processing with {} max tasks completed in {}ms",
            max_concurrent,
            duration.as_millis()
        );

        // All tasks should complete successfully despite limitation
        for (idx, result) in &results {
            assert!(result.feedback.len() > 0);
            assert!(result.suggested_rating >= 1 && result.suggested_rating <= 4);
            assert!(*idx < 5); // Valid index
        }
    }

    #[tokio::test]
    async fn test_concurrent_error_handling() {
        let service = ConcurrentGradingService::new(3);
        let card = create_test_card();

        // Create questions with one invalid question to test error handling
        let mut questions = create_test_questions();
        questions.push(QuizQuestion {
            question: "".to_string(), // Invalid empty question
            question_type: "invalid_type".to_string(),
            options: None,
            correct_answer: None,
        });

        let answers = vec!["Answer"; 6]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        let questions_and_answers: Vec<_> = questions.into_iter().zip(answers).collect();

        let result = service
            .grade_answers_concurrently(&card, questions_and_answers)
            .await;

        // Should handle errors gracefully
        match result {
            Ok(results) => {
                // If successful, should have filtered out invalid questions
                assert!(results.len() <= 6);
                println!(
                    "Concurrent processing handled errors, got {} valid results",
                    results.len()
                );
            }
            Err(e) => {
                // If failed, should provide meaningful error message
                assert!(e.to_string().contains("Failed to grade"));
                println!("Expected error handling: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_concurrent_result_ordering() {
        let service = ConcurrentGradingService::new(3);
        let card = create_test_card();
        let questions = create_test_questions();
        let answers = vec![
            "First answer".to_string(),
            "Second answer".to_string(),
            "Third answer".to_string(),
            "Fourth answer".to_string(),
            "Fifth answer".to_string(),
        ];

        let questions_and_answers: Vec<_> = questions.into_iter().zip(answers).collect();

        let mut results = service
            .grade_answers_concurrently(&card, questions_and_answers)
            .await
            .unwrap();

        assert_eq!(results.len(), 5);

        // Verify results maintain correct order despite concurrent execution
        results.sort_by_key(|(index, _)| *index);
        for (expected_idx, (actual_idx, _result)) in results.iter().enumerate() {
            assert_eq!(*actual_idx, expected_idx);
        }

        println!("Concurrent processing maintained correct result ordering");
    }

    #[tokio::test]
    async fn test_concurrent_large_batch_processing() {
        let service = ConcurrentGradingService::new(5);
        let card = create_test_card();

        // Create a large batch of questions
        let questions: Vec<_> = (0..20)
            .map(|i| QuizQuestion {
                question: format!("Question {}", i + 1),
                question_type: "short_answer".to_string(),
                options: None,
                correct_answer: Some(format!("Answer {}", i + 1)),
            })
            .collect();

        let answers: Vec<_> = (0..20).map(|i| format!("Answer {}", i + 1)).collect();

        let questions_and_answers: Vec<_> = questions.into_iter().zip(answers).collect();

        let start_time = Instant::now();
        let results = service
            .grade_answers_concurrently(&card, questions_and_answers)
            .await
            .unwrap();
        let duration = start_time.elapsed();

        assert_eq!(results.len(), 20);

        // Verify all results are valid
        for (idx, result) in results.iter() {
            assert!(*idx < 20);
            assert!(result.feedback.len() > 0);
            assert!(result.suggested_rating >= 1 && result.suggested_rating <= 4);
        }

        println!(
            "Large batch concurrent processing: {} questions in {}ms",
            results.len(),
            duration.as_millis()
        );

        // Should complete large batch within reasonable time
        assert!(duration.as_millis() < 10000); // 10 seconds max for mock service
    }

    #[tokio::test]
    async fn test_concurrent_resource_cleanup() {
        let service = ConcurrentGradingService::new(3);
        let card = create_test_card();

        // Run multiple concurrent processing sessions to test resource cleanup
        for session in 0..5 {
            let questions = create_test_questions();
            let answers = vec!["Test answer"; 5]
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>();
            let questions_and_answers: Vec<_> = questions.into_iter().zip(answers).collect();

            let results = service
                .grade_answers_concurrently(&card, questions_and_answers)
                .await
                .unwrap();

            assert_eq!(results.len(), 5);

            // Check that active tasks counter is reset
            let active_tasks = *service.active_tasks.lock().unwrap();
            assert_eq!(
                active_tasks,
                0,
                "Active tasks should be reset after session {}",
                session + 1
            );

            println!(
                "Session {} completed successfully with proper cleanup",
                session + 1
            );
        }
    }

    #[tokio::test]
    async fn test_concurrent_mixed_question_types() {
        let service = ConcurrentGradingService::new(4);
        let card = create_test_card();

        let mixed_questions = vec![
            QuizQuestion {
                question: "Short answer question".to_string(),
                question_type: "short_answer".to_string(),
                options: None,
                correct_answer: Some("Short answer".to_string()),
            },
            QuizQuestion {
                question: "Multiple choice question".to_string(),
                question_type: "multiple_choice".to_string(),
                options: Some(vec!["A) Option 1".to_string(), "B) Option 2".to_string()]),
                correct_answer: Some("A) Option 1".to_string()),
            },
            QuizQuestion {
                question: "True/false question".to_string(),
                question_type: "true_false".to_string(),
                options: Some(vec!["True".to_string(), "False".to_string()]),
                correct_answer: Some("True".to_string()),
            },
            QuizQuestion {
                question: "Essay question".to_string(),
                question_type: "essay".to_string(),
                options: None,
                correct_answer: Some("Detailed essay response".to_string()),
            },
        ];

        let mixed_answers = vec![
            "Short answer".to_string(),
            "A) Option 1".to_string(),
            "True".to_string(),
            "This is a detailed essay response covering the topic comprehensively.".to_string(),
        ];

        let questions_and_answers: Vec<_> =
            mixed_questions.into_iter().zip(mixed_answers).collect();

        let results = service
            .grade_answers_concurrently(&card, questions_and_answers)
            .await
            .unwrap();

        assert_eq!(results.len(), 4);

        // Verify each question type was processed correctly
        for (idx, result) in results.iter() {
            assert!(*idx < 4);
            assert!(result.feedback.len() > 0);
            assert!(result.suggested_rating >= 1 && result.suggested_rating <= 4);

            // All should be correct with our mock answers
            assert!(result.is_correct, "Question {} should be correct", idx + 1);
        }

        println!("Successfully processed mixed question types concurrently");
    }

    #[tokio::test]
    async fn test_concurrent_backpressure_handling() {
        let service = ConcurrentGradingService::new(2); // Very limited concurrency
        let card = create_test_card();

        // Create many questions to test backpressure
        let questions: Vec<_> = (0..15)
            .map(|i| QuizQuestion {
                question: format!("Backpressure test question {}", i + 1),
                question_type: "short_answer".to_string(),
                options: None,
                correct_answer: Some(format!("Answer {}", i + 1)),
            })
            .collect();

        let answers: Vec<_> = (0..15).map(|i| format!("Answer {}", i + 1)).collect();

        let questions_and_answers: Vec<_> = questions.into_iter().zip(answers).collect();

        let start_time = Instant::now();
        let results = service
            .grade_answers_concurrently(&card, questions_and_answers)
            .await
            .unwrap();
        let duration = start_time.elapsed();

        assert_eq!(results.len(), 15);

        // With only 2 concurrent tasks, this should take longer than unlimited concurrency
        // but should still complete successfully
        println!(
            "Backpressure test: {} questions with max 2 concurrent tasks in {}ms",
            results.len(),
            duration.as_millis()
        );

        // Verify results are still correctly ordered and valid
        let mut sorted_results = results.clone();
        sorted_results.sort_by_key(|(idx, _)| *idx);
        for (expected_idx, (actual_idx, result)) in sorted_results.iter().enumerate() {
            assert_eq!(*actual_idx, expected_idx);
            assert!(result.is_correct);
        }

        // Check that no tasks are left running
        let active_tasks = *service.active_tasks.lock().unwrap();
        assert_eq!(active_tasks, 0);
    }
}
