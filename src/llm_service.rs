use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::models::{Card, QuizQuestion};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMRequest {
    pub model: String,
    pub messages: Vec<LLMMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub choices: Vec<LLMChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMChoice {
    pub message: LLMMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedQuiz {
    pub questions: Vec<QuizQuestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradingResult {
    pub is_correct: bool,
    pub feedback: String,
    pub suggested_rating: i32, // 1-4 for FSRS
}

#[derive(Clone)]
pub struct LLMService {
    client: Client,
    api_key: String,
    base_url: String,
}

impl LLMService {
    pub fn new(api_key: String, base_url: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
        }
    }

    pub async fn generate_quiz_questions(&self, card: &Card) -> Result<Vec<QuizQuestion>> {
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
                        "options": ["A", "B", "C", "D"] or null,
                        "correct_answer": "Correct answer or option letter"
                    }}
                ]
            }}

            Guidelines:
            - For multiple_choice, provide 4 options and indicate the correct option letter
            - For short_answer, provide the expected answer
            - For problem_solving, provide the solution approach
            - Make questions challenging but fair
            - Ensure questions test key concepts from the card"#,
            card.content
        );

        let request = LLMRequest {
            model: "gpt-5-mini".to_string(),
            messages: vec![
                LLMMessage {
                    role: "system".to_string(),
                    content: "You are an expert quiz generator. Always respond with valid JSON in the requested format.".to_string(),
                },
                LLMMessage {
                    role: "user".to_string(),
                    content: prompt,
                },
            ],
        };

        let response = self
            .client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        // Check if response is successful
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            eprintln!("API Error ({}): {}", status, error_text);
            return Err(anyhow::anyhow!("API request failed: {}", error_text));
        }

        let llm_response: LLMResponse = response.json().await?;

        if let Some(choice) = llm_response.choices.first() {
            // Extract JSON from markdown code blocks if present
            let content = &choice.message.content;
            eprintln!("LLM Response content: {}", content);
            let json_content = extract_json_from_response(content);
            eprintln!("Extracted JSON: {}", json_content);

            let generated_quiz: GeneratedQuiz = serde_json::from_str(&json_content)?;
            Ok(generated_quiz.questions)
        } else {
            Err(anyhow::anyhow!("No response from LLM"))
        }
    }

    pub async fn grade_answer(
        &self,
        card: &Card,
        question: &QuizQuestion,
        user_answer: &str,
    ) -> Result<GradingResult> {
        let prompt = format!(
            r#"Grade the following quiz answer based on the original card content.

            Card Content:
            {}

            Question: {}
            Question Type: {}
            Correct Answer: {}
            User's Answer: {}

            Please evaluate the answer and respond with a JSON object in this exact format:
            {{
                "is_correct": true|false,
                "feedback": "Detailed feedback explaining why the answer is correct/incorrect and providing the right information",
                "suggested_rating": 1|2|3|4
            }}

            Rating Guidelines:
            - 1 (Again): Completely wrong or no understanding shown
            - 2 (Hard): Partially correct but significant gaps or errors
            - 3 (Good): Mostly correct with minor issues or good understanding shown
            - 4 (Easy): Perfect or excellent answer demonstrating clear mastery

            Be constructive in feedback and help the user learn."#,
            card.content,
            question.question,
            question.question_type,
            question.correct_answer.as_deref().unwrap_or("N/A"),
            user_answer
        );

        let request = LLMRequest {
            model: "gpt-5-mini".to_string(),
            messages: vec![
                LLMMessage {
                    role: "system".to_string(),
                    content: "You are an expert teacher and grader. Always respond with valid JSON in the requested format.".to_string(),
                },
                LLMMessage {
                    role: "user".to_string(),
                    content: prompt,
                },
            ],
        };

        let response = self
            .client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let llm_response: LLMResponse = response.json().await?;

        if let Some(choice) = llm_response.choices.first() {
            // Extract JSON from markdown code blocks if present
            let content = &choice.message.content;
            let json_content = extract_json_from_response(content);

            let grading_result: GradingResult = serde_json::from_str(&json_content)?;
            Ok(grading_result)
        } else {
            Err(anyhow::anyhow!("No response from LLM"))
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
}

// Helper function to extract JSON from LLM responses that might be wrapped in markdown
fn extract_json_from_response(content: &str) -> String {
    // Try to find JSON within markdown code blocks
    if let Some(start) = content.find("```json") {
        if let Some(end) = content[start + 7..].find("```") {
            let json_start = start + 7;
            let json_end = json_start + end;
            return content[json_start..json_end].trim().to_string();
        }
    }

    // Try to find JSON within plain code blocks
    if let Some(start) = content.find("```") {
        if let Some(end) = content[start + 3..].find("```") {
            let json_start = start + 3;
            let json_end = json_start + end;
            let potential_json = content[json_start..json_end].trim();
            // Check if it looks like JSON
            if potential_json.starts_with('{') || potential_json.starts_with('[') {
                return potential_json.to_string();
            }
        }
    }

    // Try to find standalone JSON objects
    if let Some(start) = content.find('{') {
        if let Some(end) = content.rfind('}') {
            if end > start {
                return content[start..=end].to_string();
            }
        }
    }

    // Return original content if no JSON extraction patterns match
    content.trim().to_string()
}
