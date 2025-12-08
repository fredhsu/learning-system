use anyhow::Result;
use futures_util::future;
use std::collections::HashMap;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::llm_providers::{JsonResponseParser, LLMProvider, LLMProviderFactory, LLMProviderType};
use crate::models::{BatchGradingRequest, BatchGradingResult, Card, QuizQuestion};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedQuiz {
    pub questions: Vec<QuizQuestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchGeneratedQuiz {
    pub results: HashMap<String, Vec<QuizQuestion>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradingResult {
    pub is_correct: bool,
    pub feedback: String,
    pub suggested_rating: i32, // 1-4 for FSRS
}

#[derive(Clone)]
pub struct LLMService {
    provider: LLMProvider,
    json_parser: JsonResponseParser,
}

impl LLMService {
    #[allow(dead_code)]
    pub fn new(api_key: String, base_url: Option<String>) -> Self {
        Self::new_with_provider(api_key, base_url, LLMProviderType::OpenAI, None)
    }

    pub fn new_with_provider(
        api_key: String,
        base_url: Option<String>,
        provider_type: LLMProviderType,
        model: Option<String>,
    ) -> Self {
        let provider = LLMProviderFactory::create_provider(provider_type, api_key, base_url, model);

        Self {
            provider,
            json_parser: JsonResponseParser,
        }
    }

    #[allow(dead_code)]
    pub fn new_gemini(api_key: String, model: Option<String>) -> Self {
        Self::new_with_provider(
            api_key,
            None,
            LLMProviderType::Gemini,
            model.or_else(|| Some("gemini-2.0-flash-exp".to_string())),
        )
    }

    #[allow(dead_code)]
    async fn make_llm_request(&self, prompt: &str) -> Result<String> {
        self.make_llm_request_with_system(None, prompt).await
    }

    async fn make_llm_request_with_system(
        &self,
        system_message: Option<&str>,
        prompt: &str,
    ) -> Result<String> {
        self.provider.make_request(system_message, prompt).await
    }

    /// Get the provider name for logging and testing
    #[allow(dead_code)]
    pub fn provider_name(&self) -> &'static str {
        self.provider.provider_name()
    }

    /// Get the model name being used
    #[allow(dead_code)]
    pub fn model_name(&self) -> &str {
        self.provider.model_name()
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub fn new_mock() -> Self {
        Self::new_mock_internal(true, false)
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub fn new_mock_with_incorrect_answers() -> Self {
        Self::new_mock_internal(false, false)
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub fn new_mock_with_batch_failure() -> Self {
        Self::new_mock_internal(true, true)
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub fn new_mock_with_mixed_results() -> Self {
        use crate::llm_providers::MockProvider;

        let provider = LLMProvider::Mock(MockProvider::new_mixed());
        let json_parser = JsonResponseParser::new();

        Self {
            provider,
            json_parser,
        }
    }

    #[cfg(test)]
    #[allow(dead_code)]
    fn new_mock_internal(correct_answers: bool, batch_fails: bool) -> Self {
        use crate::llm_providers::MockProvider;

        let provider = LLMProvider::Mock(MockProvider::new(correct_answers, batch_fails));
        let json_parser = JsonResponseParser::new();

        Self {
            provider,
            json_parser,
        }
    }

    pub async fn generate_quiz_questions(&self, card: &Card) -> Result<Vec<QuizQuestion>> {
        info!(
            card_id = %card.id,
            card_zettel_id = %card.zettel_id,
            content_length = card.content.len(),
            "Generating quiz questions for card"
        );

        let prompt = format!(
            r#"Based on the following learning card content, generate 2-3 quiz questions to test understanding.
            The questions should be varied in type (multiple choice, short answer, or problem-solving).

            Card Content:
            {}

            Please respond with a JSON object in this exact format:
            {{
                "questions": [
                    {{
                        "question": "Question text here",
                        "question_type": "multiple_choice|short_answer|problem_solving",
                        "options": ["Option text 1", "Option text 2", "Option text 3", "Option text 4"] or null,
                        "correct_answer": "Correct answer or option letter"
                    }}
                ]
            }}

            Guidelines:
            - For multiple_choice, provide 4 option texts WITHOUT any letter prefixes (A., B., etc.) - just the option content
            - The frontend will automatically add A., B., C., D. prefixes when displaying
            - For short_answer, provide the expected answer
            - For problem_solving, provide the solution approach
            - Make questions challenging but fair
            - The questions should emphasize practical, real world applications of the concepts, particurlarly when applied to machine learning or computer science.
            - Ensure questions test key concepts from the card"#,
            card.content
        );

        let system_message = "You are a university professor. Always respond with valid JSON in the requested format.";
        let response_text = self
            .make_llm_request_with_system(Some(system_message), &prompt)
            .await?;

        {
            debug!(
                card_id = %card.id,
                response_content = %response_text,
                "Raw LLM response for quiz generation"
            );

            let json_content = JsonResponseParser::extract_json_from_response(&response_text);
            debug!(
                card_id = %card.id,
                extracted_json = %json_content,
                "Extracted JSON from LLM response"
            );

            match self
                .json_parser
                .parse_json_response::<GeneratedQuiz>(&response_text)
            {
                Ok(generated_quiz) => {
                    info!(
                        card_id = %card.id,
                        question_count = generated_quiz.questions.len(),
                        "Successfully generated quiz questions"
                    );
                    Ok(generated_quiz.questions)
                }
                Err(e) => {
                    error!(
                        card_id = %card.id,
                        error = %e,
                        json_content = %json_content,
                        "Failed to parse quiz generation JSON response"
                    );
                    Err(anyhow::anyhow!("Failed to parse quiz JSON: {}", e))
                }
            }
        }
    }

    pub async fn generate_batch_quiz_questions(
        &self,
        cards: &[Card],
    ) -> Result<HashMap<Uuid, Vec<QuizQuestion>>> {
        if cards.is_empty() {
            return Ok(HashMap::new());
        }

        info!(
            card_count = cards.len(),
            card_ids = ?cards.iter().map(|c| c.id).collect::<Vec<_>>(),
            "Generating batch quiz questions for multiple cards"
        );

        // Create card summaries for the prompt
        let card_summaries = cards
            .iter()
            .enumerate()
            .map(|(i, card)| {
                format!(
                    "Card {}: ID={}, Zettel_ID={}, Content={}",
                    i + 1,
                    card.id,
                    card.zettel_id,
                    // Truncate content to avoid overly long prompts
                    if card.content.len() > 500 {
                        format!("{}...", &card.content[..500])
                    } else {
                        card.content.clone()
                    }
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        let prompt = format!(
            r#"Generate 2-3 quiz questions for each of the following learning cards. The questions should be varied in type (multiple choice, short answer, or problem-solving) and test key concepts.

Cards:
{}

Please respond with a JSON object in this exact format:
{{
    "results": {{
        "{}": [
            {{
                "question": "Question text here",
                "question_type": "multiple_choice|short_answer|problem_solving",
                "options": ["Option text 1", "Option text 2", "Option text 3", "Option text 4"] or null,
                "correct_answer": "Correct answer or option letter"
            }}
        ],
        "{}": [
            {{
                "question": "Question text here",
                "question_type": "multiple_choice|short_answer|problem_solving", 
                "options": ["Option text 1", "Option text 2", "Option text 3", "Option text 4"] or null,
                "correct_answer": "Correct answer or option letter"
            }}
        ]
    }}
}}

Guidelines:
- For multiple_choice, provide 4 option texts WITHOUT any letter prefixes (A., B., etc.) - just the option content
- The frontend will automatically add A., B., C., D. prefixes when displaying
- For short_answer, provide the expected answer
- For problem_solving, provide the solution approach
- Make questions challenging but fair
- Ensure questions test key concepts from each card
- Use the exact card IDs provided above as keys in the results object"#,
            card_summaries,
            cards.first().map(|c| c.id.to_string()).unwrap_or_default(),
            cards
                .get(1)
                .map(|c| c.id.to_string())
                .unwrap_or(cards.first().map(|c| c.id.to_string()).unwrap_or_default())
        );

        let system_message = "You are a university professor creating quiz questions. Always respond with valid JSON in the exact requested format. Use the provided card IDs as keys.";
        let response_text = self
            .make_llm_request_with_system(Some(system_message), &prompt)
            .await?;

        {
            debug!(
                card_count = cards.len(),
                response_content = %response_text,
                "Raw LLM response for batch quiz generation"
            );

            let json_content = JsonResponseParser::extract_json_from_response(&response_text);
            debug!(
                card_count = cards.len(),
                extracted_json = %json_content,
                "Extracted JSON from batch quiz response"
            );

            match self
                .json_parser
                .parse_json_response::<BatchGeneratedQuiz>(&response_text)
            {
                Ok(batch_quiz) => {
                    // Convert string keys to UUIDs
                    let mut result = HashMap::new();
                    for (card_id_str, questions) in batch_quiz.results {
                        if let Ok(card_id) = Uuid::parse_str(&card_id_str) {
                            result.insert(card_id, questions);
                        } else {
                            warn!(
                                card_id_str = %card_id_str,
                                "Failed to parse card ID from batch quiz response"
                            );
                        }
                    }

                    info!(
                        card_count = cards.len(),
                        generated_count = result.len(),
                        total_questions = result.values().map(|q| q.len()).sum::<usize>(),
                        "Successfully generated batch quiz questions"
                    );
                    Ok(result)
                }
                Err(e) => {
                    error!(
                        card_count = cards.len(),
                        error = %e,
                        json_content = %json_content,
                        "Failed to parse batch quiz generation JSON response"
                    );
                    // Fallback to individual generation
                    self.fallback_to_individual_generation(cards).await
                }
            }
        }
    }

    async fn fallback_to_individual_generation(
        &self,
        cards: &[Card],
    ) -> Result<HashMap<Uuid, Vec<QuizQuestion>>> {
        info!(
            card_count = cards.len(),
            "Falling back to individual question generation for cards"
        );

        let mut result = HashMap::new();
        for card in cards {
            match self.generate_quiz_questions(card).await {
                Ok(questions) => {
                    result.insert(card.id, questions);
                }
                Err(e) => {
                    warn!(
                        card_id = %card.id,
                        error = %e,
                        "Failed to generate questions for card, using fallback"
                    );
                    // Use local fallback for this card
                    match self.generate_quiz_questions_local(card, "").await {
                        Ok(fallback_questions) => {
                            result.insert(card.id, fallback_questions);
                        }
                        Err(fallback_e) => {
                            error!(
                                card_id = %card.id,
                                error = %fallback_e,
                                "Both primary and local fallback failed for card"
                            );
                            // Insert empty questions rather than failing entirely
                            result.insert(card.id, vec![]);
                        }
                    }
                }
            }
        }
        Ok(result)
    }

    pub async fn grade_answer(
        &self,
        card: &Card,
        question: &QuizQuestion,
        user_answer: &str,
    ) -> Result<GradingResult> {
        info!(
            card_id = %card.id,
            card_zettel_id = %card.zettel_id,
            question_type = %question.question_type,
            question_text = %question.question.chars().take(100).collect::<String>(),
            user_answer = %user_answer,
            "Grading quiz answer"
        );
        let prompt = format!(
            r#"Grade the following quiz answer based on semantic understanding and conceptual accuracy, not just literal text matching.

            Card Content:
            {}

            Question: {}
            Question Type: {}
            Correct Answer: {}
            User's Answer: {}

            GRADING PRINCIPLES:
            - Accept semantically equivalent answers (synonyms, paraphrasing, different valid explanations)
            - For multiple choice: Accept the correct option letter OR the full option text
            - For numerical answers: Accept equivalent forms (0.5 = 1/2 = 50%)
            - For short answers: Focus on key concepts rather than exact wording
            - Consider context from the card content when evaluating answers
            - Give credit for partially correct answers that show understanding

            EXAMPLES OF EQUIVALENT ANSWERS:
            - "Quick" = "Fast" = "Rapid" (synonyms)
            - "Machine Learning" = "ML" (abbreviations)  
            - "Because it increases efficiency" = "It makes things more efficient" (paraphrasing)
            - "Option A" = "A" = "[Full text of option A]" (multiple choice formats)

            GRADING CRITERIA:
            - CORRECT (is_correct: true): Answer demonstrates understanding of key concepts, even if wording differs
            - INCORRECT (is_correct: false): Answer shows fundamental misunderstanding or is completely wrong

            Please respond with a JSON object in this exact format:
            {{
                "is_correct": true|false,
                "feedback": "Specific feedback explaining the evaluation, mentioning what was correct/incorrect and providing the complete correct information",
                "suggested_rating": 1|2|3|4
            }}

            Rating Guidelines (be generous for conceptually correct answers):
            - 1 (Again): Fundamentally wrong or no understanding demonstrated
            - 2 (Hard): Shows some understanding but with significant conceptual errors
            - 3 (Good): Correct understanding with minor wording differences or small omissions
            - 4 (Easy): Perfect or excellent answer with clear mastery

            Focus on conceptual understanding rather than exact text matching. When in doubt between correct/incorrect, lean toward giving credit if the core concept is understood."#,
            card.content,
            question.question,
            question.question_type,
            question.correct_answer.as_deref().unwrap_or("N/A"),
            user_answer
        );

        let system_message = "You are an expert teacher focused on fair, understanding-based grading. Prioritize semantic meaning over exact text matching. Accept equivalent answers that demonstrate understanding. Always respond with valid JSON in the requested format.";
        let response_text = self
            .make_llm_request_with_system(Some(system_message), &prompt)
            .await?;

        {
            debug!(
                card_id = %card.id,
                response_content = %response_text,
                "Raw LLM response for answer grading"
            );

            let json_content = JsonResponseParser::extract_json_from_response(&response_text);
            debug!(
                card_id = %card.id,
                extracted_json = %json_content,
                "Extracted JSON from grading response"
            );

            match self
                .json_parser
                .parse_json_response::<GradingResult>(&response_text)
            {
                Ok(grading_result) => {
                    info!(
                        card_id = %card.id,
                        is_correct = grading_result.is_correct,
                        suggested_rating = grading_result.suggested_rating,
                        feedback = %grading_result.feedback.chars().take(100).collect::<String>(),
                        "Successfully graded quiz answer"
                    );
                    Ok(grading_result)
                }
                Err(e) => {
                    error!(
                        card_id = %card.id,
                        error = %e,
                        json_content = %json_content,
                        "Failed to parse grading JSON response"
                    );
                    Err(anyhow::anyhow!("Failed to parse grading JSON: {}", e))
                }
            }
        }
    }

    // Alternative method for local/offline LLM integration
    pub async fn generate_quiz_questions_local(
        &self,
        card: &Card,
        _local_endpoint: &str,
    ) -> Result<Vec<QuizQuestion>> {
        // This would integrate with a local LLM like Ollama
        // For now, return a simple hardcoded question as fallback
        Ok(vec![QuizQuestion {
            question: format!(
                "What is the main concept described in this card: '{}'?",
                card.content.chars().take(50).collect::<String>()
            ),
            question_type: "short_answer".to_string(),
            options: None,
            correct_answer: Some("Based on the card content".to_string()),
        }])
    }

    #[allow(dead_code)]
    pub async fn grade_batch_answers(
        &self,
        grading_requests: &[BatchGradingRequest],
    ) -> Result<Vec<BatchGradingResult>> {
        if grading_requests.is_empty() {
            return Ok(Vec::new());
        }

        info!(
            request_count = grading_requests.len(),
            "Grading batch of quiz answers"
        );

        let questions_and_answers = grading_requests.iter().enumerate().map(|(i, req)| {
            format!(
                "{}. Card Content: {}\n   Question: {}\n   Question Type: {}\n   Correct Answer: {}\n   User Answer: {}",
                i + 1,
                // Truncate card content for prompt efficiency
                if req.card_content.len() > 300 {
                    format!("{}...", &req.card_content[..300])
                } else {
                    req.card_content.clone()
                },
                req.question.question,
                req.question.question_type,
                req.question.correct_answer.as_deref().unwrap_or("N/A"),
                req.user_answer
            )
        }).collect::<Vec<_>>().join("\n\n");

        let prompt = format!(
            r#"Grade the following quiz answers based on semantic understanding and conceptual accuracy, not just literal text matching.

Questions and Answers:
{}

GRADING PRINCIPLES:
- Accept semantically equivalent answers (synonyms, paraphrasing, different valid explanations)
- For multiple choice: Accept the correct option letter OR the full option text
- For numerical answers: Accept equivalent forms (0.5 = 1/2 = 50%)
- For short answers: Focus on key concepts rather than exact wording
- Consider context from the card content when evaluating answers
- Give credit for partially correct answers that show understanding

EXAMPLES OF EQUIVALENT ANSWERS:
- "Quick" = "Fast" = "Rapid" (synonyms)
- "Machine Learning" = "ML" (abbreviations)
- "Because it increases efficiency" = "It makes things more efficient" (paraphrasing)
- "Option A" = "A" = "[Full text of option A]" (multiple choice formats)

Please respond with a JSON array in this exact format:
[
    {{
        "question_id": "1",
        "is_correct": true|false,
        "feedback": "Specific feedback explaining the evaluation, mentioning what was correct/incorrect",
        "suggested_rating": 1|2|3|4
    }},
    {{
        "question_id": "2",
        "is_correct": true|false,
        "feedback": "Specific feedback...",
        "suggested_rating": 1|2|3|4
    }}
]

Rating Guidelines (be generous for conceptually correct answers):
- 1 (Again): Fundamentally wrong or no understanding demonstrated
- 2 (Hard): Shows some understanding but with significant conceptual errors
- 3 (Good): Correct understanding with minor wording differences or small omissions
- 4 (Easy): Perfect or excellent answer with clear mastery

Focus on conceptual understanding rather than exact text matching."#,
            questions_and_answers
        );

        let system_message = "You are an expert teacher focused on fair, understanding-based grading. Prioritize semantic meaning over exact text matching. Accept equivalent answers that demonstrate understanding. Always respond with valid JSON array in the requested format.";
        let response_text = match self
            .make_llm_request_with_system(Some(system_message), &prompt)
            .await
        {
            Ok(text) => text,
            Err(e) => {
                error!(
                    request_count = grading_requests.len(),
                    error = %e,
                    "Batch grading LLM request failed, falling back to individual grading"
                );
                return self.fallback_to_individual_grading(grading_requests).await;
            }
        };

        debug!(
            request_count = grading_requests.len(),
            response_content = %response_text,
            "Raw LLM response for batch grading"
        );

        let json_content = JsonResponseParser::extract_json_from_response(&response_text);
        debug!(
            request_count = grading_requests.len(),
            extracted_json = %json_content,
            "Extracted JSON from batch grading response"
        );

        match self
            .json_parser
            .parse_json_response::<Vec<BatchGradingResult>>(&response_text)
        {
            Ok(results) => {
                info!(
                    request_count = grading_requests.len(),
                    result_count = results.len(),
                    "Successfully graded batch answers"
                );
                Ok(results)
            }
            Err(e) => {
                error!(
                    request_count = grading_requests.len(),
                    error = %e,
                    json_content = %json_content,
                    "Failed to parse batch grading JSON response"
                );
                // Fallback to individual grading
                self.fallback_to_individual_grading(grading_requests).await
            }
        }
    }

    #[allow(dead_code)]
    async fn fallback_to_individual_grading(
        &self,
        grading_requests: &[BatchGradingRequest],
    ) -> Result<Vec<BatchGradingResult>> {
        info!(
            request_count = grading_requests.len(),
            "Falling back to individual grading for answers"
        );

        let mut results = Vec::new();
        for (i, req) in grading_requests.iter().enumerate() {
            // Create a temporary card for the grading call
            let temp_card = Card {
                id: Uuid::new_v4(),
                zettel_id: format!("temp-{}", i),
                title: None,
                content: req.card_content.clone(),
                creation_date: chrono::Utc::now(),
                last_reviewed: None,
                next_review: chrono::Utc::now(),
                difficulty: 0.0,
                stability: 0.0,
                retrievability: 0.0,
                reps: 0,
                lapses: 0,
                state: "New".to_string(),
                links: None,
            };

            match self
                .grade_answer(&temp_card, &req.question, &req.user_answer)
                .await
            {
                Ok(grading_result) => {
                    results.push(BatchGradingResult {
                        question_id: (i + 1).to_string(),
                        is_correct: grading_result.is_correct,
                        feedback: grading_result.feedback,
                        suggested_rating: grading_result.suggested_rating,
                    });
                }
                Err(e) => {
                    warn!(
                        question_index = i,
                        error = %e,
                        "Failed to grade individual answer in batch fallback"
                    );
                    // Provide default grading for failed individual calls
                    results.push(BatchGradingResult {
                        question_id: (i + 1).to_string(),
                        is_correct: false,
                        feedback: "Unable to grade this answer due to technical issues."
                            .to_string(),
                        suggested_rating: 2,
                    });
                }
            }
        }
        Ok(results)
    }

    // Phase 2: Concurrent individual grading methods

    /// Grade answers using true parallel processing with concurrent individual LLM calls
    pub async fn grade_answers_concurrently(
        &self,
        card: &Card,
        questions_and_answers: Vec<(QuizQuestion, String)>,
        max_concurrent_tasks: Option<usize>,
    ) -> Result<Vec<BatchGradingResult>> {
        if questions_and_answers.is_empty() {
            return Ok(Vec::new());
        }

        let max_concurrent = max_concurrent_tasks.unwrap_or(5); // Default to 5 concurrent tasks
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(max_concurrent));

        info!(
            card_id = %card.id,
            question_count = questions_and_answers.len(),
            max_concurrent = max_concurrent,
            "Starting concurrent answer grading"
        );

        let mut tasks = Vec::new();

        for (index, (question, answer)) in questions_and_answers.into_iter().enumerate() {
            let service = self.clone();
            let card_clone = card.clone();
            let question_clone = question.clone();
            let answer_clone = answer.clone();
            let semaphore_clone = semaphore.clone();

            let task = tokio::spawn(async move {
                let _permit = semaphore_clone.acquire().await.unwrap();

                let start_time = std::time::Instant::now();
                let result = service
                    .grade_answer(&card_clone, &question_clone, &answer_clone)
                    .await;
                let duration = start_time.elapsed();

                match result {
                    Ok(grading_result) => {
                        debug!(
                            question_index = index,
                            duration_ms = duration.as_millis() as u64,
                            is_correct = grading_result.is_correct,
                            "Concurrent grading task completed successfully"
                        );

                        Ok((
                            index,
                            BatchGradingResult {
                                question_id: (index + 1).to_string(),
                                is_correct: grading_result.is_correct,
                                feedback: grading_result.feedback,
                                suggested_rating: grading_result.suggested_rating,
                            },
                            duration,
                        ))
                    }
                    Err(e) => {
                        error!(
                            question_index = index,
                            error = %e,
                            duration_ms = duration.as_millis() as u64,
                            "Concurrent grading task failed"
                        );
                        Err((index, e))
                    }
                }
            });

            tasks.push(task);
        }

        // Wait for all concurrent tasks to complete
        let task_results = future::join_all(tasks).await;

        let mut results = Vec::new();
        let mut errors = Vec::new();
        let mut durations = Vec::new();

        for task_result in task_results {
            match task_result {
                Ok(Ok((index, batch_result, duration))) => {
                    results.push((index, batch_result));
                    durations.push(duration);
                }
                Ok(Err((index, error))) => {
                    errors.push((index, error));
                }
                Err(join_error) => {
                    error!(error = %join_error, "Task join error in concurrent grading");
                    return Err(anyhow::anyhow!("Concurrent grading failed: {}", join_error));
                }
            }
        }

        // Handle any errors by providing default responses
        for (index, error) in errors {
            warn!(
                question_index = index,
                error = %error,
                "Providing default response for failed concurrent grading task"
            );

            results.push((
                index,
                BatchGradingResult {
                    question_id: (index + 1).to_string(),
                    is_correct: false,
                    feedback: "Unable to grade this answer due to technical issues.".to_string(),
                    suggested_rating: 2,
                },
            ));
        }

        // Sort results by original index to maintain order
        results.sort_by_key(|(index, _)| *index);
        let final_results: Vec<BatchGradingResult> =
            results.into_iter().map(|(_, result)| result).collect();

        let avg_duration = if !durations.is_empty() {
            durations.iter().sum::<std::time::Duration>().as_millis() / durations.len() as u128
        } else {
            0
        };

        info!(
            card_id = %card.id,
            results_count = final_results.len(),
            avg_task_duration_ms = avg_duration,
            concurrent_tasks_used = max_concurrent,
            "Concurrent answer grading completed"
        );

        Ok(final_results)
    }

    /// Grade answers with automatic processing mode selection and fallback
    pub async fn grade_answers_with_fallback(
        &self,
        card: &Card,
        questions_and_answers: Vec<(QuizQuestion, String)>,
        processing_mode: Option<&str>,
        max_concurrent_tasks: Option<usize>,
    ) -> Result<(Vec<BatchGradingResult>, String, Option<String>)> {
        let start_time = std::time::Instant::now();

        let requested_mode = processing_mode.unwrap_or("parallel");

        // Try parallel processing first if requested
        if requested_mode == "parallel" {
            match self
                .grade_answers_concurrently(
                    card,
                    questions_and_answers.clone(),
                    max_concurrent_tasks,
                )
                .await
            {
                Ok(results) => {
                    let duration = start_time.elapsed();
                    info!(
                        card_id = %card.id,
                        duration_ms = duration.as_millis() as u64,
                        processing_mode = "parallel",
                        "Successfully completed parallel answer grading"
                    );
                    return Ok((results, "parallel".to_string(), None));
                }
                Err(e) => {
                    warn!(
                        card_id = %card.id,
                        error = %e,
                        "Parallel processing failed, falling back to batch processing"
                    );
                }
            }
        }

        // Try batch processing as fallback
        if requested_mode == "parallel" || requested_mode == "batch" {
            // Convert to batch grading requests
            let batch_requests: Vec<BatchGradingRequest> = questions_and_answers
                .iter()
                .map(|(question, answer)| BatchGradingRequest {
                    card_content: card.content.clone(),
                    question: question.clone(),
                    user_answer: answer.clone(),
                })
                .collect();

            match self.grade_batch_answers(&batch_requests).await {
                Ok(results) => {
                    let duration = start_time.elapsed();
                    let fallback_reason = if requested_mode == "parallel" {
                        Some("Parallel processing unavailable, used batch processing".to_string())
                    } else {
                        None
                    };

                    info!(
                        card_id = %card.id,
                        duration_ms = duration.as_millis() as u64,
                        processing_mode = "batch_fallback",
                        "Successfully completed batch answer grading"
                    );
                    return Ok((results, "batch_fallback".to_string(), fallback_reason));
                }
                Err(e) => {
                    warn!(
                        card_id = %card.id,
                        error = %e,
                        "Batch processing failed, falling back to sequential processing"
                    );
                }
            }
        }

        // Final fallback: sequential individual grading
        let mut results = Vec::new();

        for (index, (question, answer)) in questions_and_answers.into_iter().enumerate() {
            match self.grade_answer(card, &question, &answer).await {
                Ok(grading_result) => {
                    results.push(BatchGradingResult {
                        question_id: (index + 1).to_string(),
                        is_correct: grading_result.is_correct,
                        feedback: grading_result.feedback,
                        suggested_rating: grading_result.suggested_rating,
                    });
                }
                Err(e) => {
                    warn!(
                        card_id = %card.id,
                        question_index = index,
                        error = %e,
                        "Sequential grading failed for individual question"
                    );
                    results.push(BatchGradingResult {
                        question_id: (index + 1).to_string(),
                        is_correct: false,
                        feedback: "Unable to grade this answer due to technical issues."
                            .to_string(),
                        suggested_rating: 2,
                    });
                }
            }
        }

        let duration = start_time.elapsed();
        let fallback_reason = Some(
            "Both parallel and batch processing failed, used sequential processing".to_string(),
        );

        info!(
            card_id = %card.id,
            duration_ms = duration.as_millis() as u64,
            processing_mode = "sequential_fallback",
            "Completed sequential answer grading fallback"
        );

        Ok((results, "sequential_fallback".to_string(), fallback_reason))
    }
}
