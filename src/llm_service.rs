use anyhow::Result;
use std::collections::HashMap;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::llm_providers::{LLMProvider, LLMProviderFactory, LLMProviderType, JsonResponseParser};
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
        model: Option<String>
    ) -> Self {
        let provider = LLMProviderFactory::create_provider(
            provider_type,
            api_key,
            base_url,
            model,
        );

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
            model.or_else(|| Some("gemini-2.0-flash-exp".to_string()))
        )
    }

    #[allow(dead_code)]
    async fn make_llm_request(&self, prompt: &str) -> Result<String> {
        self.make_llm_request_with_system(None, prompt).await
    }

    async fn make_llm_request_with_system(&self, system_message: Option<&str>, prompt: &str) -> Result<String> {
        self.provider.make_request(system_message, prompt).await
    }

    /// Get the provider name for logging and testing
    pub fn provider_name(&self) -> &'static str {
        self.provider.provider_name()
    }

    /// Get the model name being used
    pub fn model_name(&self) -> &str {
        self.provider.model_name()
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
            - Ensure questions test key concepts from the card"#,
            card.content
        );

        let system_message = "You are a university professor. Always respond with valid JSON in the requested format.";
        let response_text = self.make_llm_request_with_system(Some(system_message), &prompt).await?;

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

            match self.json_parser.parse_json_response::<GeneratedQuiz>(&response_text) {
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

    pub async fn generate_batch_quiz_questions(&self, cards: &[Card]) -> Result<HashMap<Uuid, Vec<QuizQuestion>>> {
        if cards.is_empty() {
            return Ok(HashMap::new());
        }

        info!(
            card_count = cards.len(),
            card_ids = ?cards.iter().map(|c| c.id).collect::<Vec<_>>(),
            "Generating batch quiz questions for multiple cards"
        );

        // Create card summaries for the prompt
        let card_summaries = cards.iter().enumerate().map(|(i, card)| {
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
        }).collect::<Vec<_>>().join("\n\n");

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
            cards.get(1).map(|c| c.id.to_string()).unwrap_or(cards.first().map(|c| c.id.to_string()).unwrap_or_default())
        );

        let system_message = "You are a university professor creating quiz questions. Always respond with valid JSON in the exact requested format. Use the provided card IDs as keys.";
        let response_text = self.make_llm_request_with_system(Some(system_message), &prompt).await?;

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

            match self.json_parser.parse_json_response::<BatchGeneratedQuiz>(&response_text) {
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

    async fn fallback_to_individual_generation(&self, cards: &[Card]) -> Result<HashMap<Uuid, Vec<QuizQuestion>>> {
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
        let response_text = self.make_llm_request_with_system(Some(system_message), &prompt).await?;

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

            match self.json_parser.parse_json_response::<GradingResult>(&response_text) {
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
    pub async fn grade_batch_answers(&self, grading_requests: &[BatchGradingRequest]) -> Result<Vec<BatchGradingResult>> {
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
        let response_text = self.make_llm_request_with_system(Some(system_message), &prompt).await?;

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

        match self.json_parser.parse_json_response::<Vec<BatchGradingResult>>(&response_text) {
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
    async fn fallback_to_individual_grading(&self, grading_requests: &[BatchGradingRequest]) -> Result<Vec<BatchGradingResult>> {
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

            match self.grade_answer(&temp_card, &req.question, &req.user_answer).await {
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
                        feedback: "Unable to grade this answer due to technical issues.".to_string(),
                        suggested_rating: 2,
                    });
                }
            }
        }
        Ok(results)
    }
}
