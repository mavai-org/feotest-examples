//! Real LLM provider implementations (`OpenAI`, Anthropic).
//!
//! These implementations make actual HTTP requests to provider APIs.
//! They require API keys configured via environment variables:
//!
//! - `OpenAI`: `OPENAI_API_KEY`
//! - Anthropic: `ANTHROPIC_API_KEY`
//!
//! Model routing is automatic based on the model name:
//! - `gpt-*`, `o1-*`, `o3-*` → `OpenAI`
//! - `claude-*` → Anthropic

use std::sync::Mutex;
use std::time::Duration;

use crate::llm::ChatLlm;
use crate::llm::response::ChatResponse;

/// An LLM client that routes requests to the appropriate provider
/// based on the model name.
///
/// Supports `OpenAI` and Anthropic models. Requires API keys in
/// environment variables.
pub struct RoutingChatLlm {
    client: reqwest::blocking::Client,
    total_tokens: Mutex<u64>,
}

impl RoutingChatLlm {
    /// Creates a new routing LLM client.
    ///
    /// # Panics
    ///
    /// Panics if the HTTP client cannot be constructed.
    #[must_use]
    pub fn new() -> Self {
        let timeout = std::env::var("FEOTEST_LLM_TIMEOUT")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(30);

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(timeout))
            .build()
            .expect("failed to create HTTP client");

        Self {
            client,
            total_tokens: Mutex::new(0),
        }
    }

    /// Sends a request to `OpenAI`'s chat completions API.
    fn call_openai(
        &self,
        system_message: &str,
        user_message: &str,
        model: &str,
        temperature: f64,
    ) -> ChatResponse {
        let api_key =
            std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set for real LLM mode");

        let body = serde_json::json!({
            "model": model,
            "temperature": temperature,
            "messages": [
                {"role": "system", "content": system_message},
                {"role": "user", "content": user_message}
            ]
        });

        let resp = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {api_key}"))
            .json(&body)
            .send()
            .expect("OpenAI API request failed");

        let json: serde_json::Value = resp.json().expect("failed to parse OpenAI response");

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let prompt_tokens = json["usage"]["prompt_tokens"].as_u64().unwrap_or(0);
        let completion_tokens = json["usage"]["completion_tokens"].as_u64().unwrap_or(0);

        ChatResponse::new(content, prompt_tokens, completion_tokens)
    }

    /// Sends a request to Anthropic's messages API.
    fn call_anthropic(
        &self,
        system_message: &str,
        user_message: &str,
        model: &str,
        temperature: f64,
    ) -> ChatResponse {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .expect("ANTHROPIC_API_KEY must be set for real LLM mode");

        let body = serde_json::json!({
            "model": model,
            "max_tokens": 1024,
            "temperature": temperature,
            "system": system_message,
            "messages": [
                {"role": "user", "content": user_message}
            ]
        });

        let resp = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .expect("Anthropic API request failed");

        let json: serde_json::Value = resp.json().expect("failed to parse Anthropic response");

        let content = json["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let prompt_tokens = json["usage"]["input_tokens"].as_u64().unwrap_or(0);
        let completion_tokens = json["usage"]["output_tokens"].as_u64().unwrap_or(0);

        ChatResponse::new(content, prompt_tokens, completion_tokens)
    }

    /// Determines whether a model name should be routed to `OpenAI`.
    fn is_openai_model(model: &str) -> bool {
        model.starts_with("gpt-")
            || model.starts_with("o1-")
            || model.starts_with("o3-")
            || model.starts_with("text-")
    }
}

impl Default for RoutingChatLlm {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatLlm for RoutingChatLlm {
    fn chat(
        &self,
        system_message: &str,
        user_message: &str,
        model: &str,
        temperature: f64,
    ) -> ChatResponse {
        let response = if Self::is_openai_model(model) {
            self.call_openai(system_message, user_message, model, temperature)
        } else {
            self.call_anthropic(system_message, user_message, model, temperature)
        };

        let mut total = self.total_tokens.lock().unwrap();
        *total += response.total_tokens();

        response
    }

    fn total_tokens_used(&self) -> u64 {
        *self.total_tokens.lock().unwrap()
    }

    fn reset_token_count(&mut self) {
        *self.total_tokens.lock().unwrap() = 0;
    }
}
