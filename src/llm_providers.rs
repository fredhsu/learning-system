use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

/// Common message structure for LLM requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMMessage {
    pub role: String,
    pub content: String,
}

/// Enum-based LLM provider implementation for better compatibility
#[derive(Debug, Clone)]
pub enum LLMProvider {
    OpenAI(OpenAIProvider),
    Gemini(GeminiProvider),
}

impl LLMProvider {
    /// Make a request to the LLM provider with optional system message
    pub async fn make_request(&self, system_message: Option<&str>, prompt: &str) -> Result<String> {
        match self {
            LLMProvider::OpenAI(provider) => provider.make_request(system_message, prompt).await,
            LLMProvider::Gemini(provider) => provider.make_request(system_message, prompt).await,
        }
    }
    
    /// Get the provider name for logging
    pub fn provider_name(&self) -> &'static str {
        match self {
            LLMProvider::OpenAI(provider) => provider.provider_name(),
            LLMProvider::Gemini(provider) => provider.provider_name(),
        }
    }
    
    /// Get the model name being used
    pub fn model_name(&self) -> &str {
        match self {
            LLMProvider::OpenAI(provider) => provider.model_name(),
            LLMProvider::Gemini(provider) => provider.model_name(),
        }
    }
}

/// OpenAI provider implementation
#[derive(Debug, Clone)]
pub struct OpenAIProvider {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
}

/// OpenAI-specific request structures
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<LLMMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIChoice {
    message: LLMMessage,
}

impl OpenAIProvider {
    pub fn new(api_key: String, base_url: Option<String>, model: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            model: model.unwrap_or_else(|| "gpt-4o-mini".to_string()),
        }
    }
}

impl OpenAIProvider {
    pub async fn make_request(&self, system_message: Option<&str>, prompt: &str) -> Result<String> {
        let mut messages = Vec::new();
        
        if let Some(sys_msg) = system_message {
            messages.push(LLMMessage {
                role: "system".to_string(),
                content: sys_msg.to_string(),
            });
        }
        
        messages.push(LLMMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        });

        let request_body = OpenAIRequest {
            model: self.model.clone(),
            messages,
        };

        info!(
            provider = self.provider_name(),
            model = %self.model,
            base_url = %self.base_url,
            prompt_length = prompt.len(),
            "Making LLM request"
        );

        let response = self
            .client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                provider = self.provider_name(),
                status = %status,
                error = %error_text,
                "LLM API request failed"
            );
            return Err(anyhow::anyhow!("OpenAI API request failed: {}", error_text));
        }

        let openai_response: OpenAIResponse = response.json().await?;
        
        if openai_response.choices.is_empty() {
            return Err(anyhow::anyhow!("No choices in OpenAI response"));
        }

        let response_content = openai_response.choices[0].message.content.clone();
        info!(
            provider = self.provider_name(),
            response_length = response_content.len(),
            "Successfully received LLM response"
        );

        Ok(response_content)
    }

    pub fn provider_name(&self) -> &'static str {
        "OpenAI"
    }

    pub fn model_name(&self) -> &str {
        &self.model
    }
}

/// Gemini provider implementation
#[derive(Debug, Clone)]
pub struct GeminiProvider {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
}

/// Gemini-specific request structures
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(rename = "generationConfig")]
    generation_config: GeminiGenerationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiPart {
    text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiGenerationConfig {
    temperature: f32,
    #[serde(rename = "topK")]
    top_k: i32,
    #[serde(rename = "topP")]
    top_p: f32,
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}

impl GeminiProvider {
    pub fn new(api_key: String, base_url: Option<String>, model: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1beta".to_string()),
            model: model.unwrap_or_else(|| "gemini-2.0-flash-exp".to_string()),
        }
    }
}

impl GeminiProvider {
    pub async fn make_request(&self, system_message: Option<&str>, prompt: &str) -> Result<String> {
        let full_prompt = match system_message {
            Some(sys_msg) => format!("{}\n\n{}", sys_msg, prompt),
            None => prompt.to_string(),
        };

        let request_body = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart {
                    text: full_prompt,
                }],
            }],
            generation_config: GeminiGenerationConfig {
                temperature: 0.7,
                top_k: 40,
                top_p: 0.9,
                max_output_tokens: 2048,
            },
        };

        let url = format!(
            "{}/models/{}:generateContent?key={}", 
            self.base_url, 
            self.model, 
            self.api_key
        );

        info!(
            provider = self.provider_name(),
            model = %self.model,
            base_url = %self.base_url,
            prompt_length = prompt.len(),
            "Making LLM request"
        );

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                provider = self.provider_name(),
                status = %status,
                error = %error_text,
                "LLM API request failed"
            );
            return Err(anyhow::anyhow!("Gemini API request failed: {}", error_text));
        }

        let gemini_response: GeminiResponse = response.json().await?;
        
        if gemini_response.candidates.is_empty() {
            return Err(anyhow::anyhow!("No candidates in Gemini response"));
        }

        if gemini_response.candidates[0].content.parts.is_empty() {
            return Err(anyhow::anyhow!("No parts in Gemini response"));
        }

        let response_content = gemini_response.candidates[0].content.parts[0].text.clone();
        info!(
            provider = self.provider_name(),
            response_length = response_content.len(),
            "Successfully received LLM response"
        );

        Ok(response_content)
    }

    pub fn provider_name(&self) -> &'static str {
        "Gemini"
    }

    pub fn model_name(&self) -> &str {
        &self.model
    }
}

/// Centralized JSON response parser with robust extraction logic
#[derive(Clone)]
pub struct JsonResponseParser;

impl JsonResponseParser {
    /// Extract JSON from LLM responses that might be wrapped in markdown or other formatting
    pub fn extract_json_from_response(content: &str) -> String {
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

        // Try to find standalone JSON arrays
        if let Some(start) = content.find('[') {
            if let Some(end) = content.rfind(']') {
                if end > start {
                    return content[start..=end].to_string();
                }
            }
        }

        // Return original content if no JSON extraction patterns match
        content.trim().to_string()
    }

    /// Parse JSON response into a specific type with error handling
    pub fn parse_json_response<T>(&self, content: &str) -> Result<T> 
    where 
        T: serde::de::DeserializeOwned 
    {
        let json_content = Self::extract_json_from_response(content);
        serde_json::from_str::<T>(&json_content)
            .map_err(|e| anyhow::anyhow!("Failed to parse JSON response: {}", e))
    }
}

/// Factory for creating LLM providers based on provider type
pub struct LLMProviderFactory;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum LLMProviderType {
    OpenAI,
    Gemini,
}

impl LLMProviderFactory {
    /// Create a new LLM provider instance based on provider type
    pub fn create_provider(
        provider_type: LLMProviderType,
        api_key: String,
        base_url: Option<String>,
        model: Option<String>,
    ) -> LLMProvider {
        match provider_type {
            LLMProviderType::OpenAI => LLMProvider::OpenAI(OpenAIProvider::new(api_key, base_url, model)),
            LLMProviderType::Gemini => LLMProvider::Gemini(GeminiProvider::new(api_key, base_url, model)),
        }
    }
}