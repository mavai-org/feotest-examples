//! Mock LLM that simulates realistic chat completion behaviour.
//!
//! The mock produces responses that vary in quality based on the
//! `temperature` parameter:
//!
//! - **0.0**: ~99% valid responses (deterministic, highly reliable)
//! - **0.3**: ~95% valid responses (recommended default)
//! - **0.5**: ~90% valid responses (balanced)
//! - **1.0**: ~70% valid responses (creative, error-prone)
//!
//! Failure modes include malformed JSON, hallucinated field names,
//! invalid action types, and missing required fields — the same kinds
//! of failures observed with real LLMs.

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::sync::Mutex;

use crate::llm::ChatLlm;
use crate::llm::response::ChatResponse;

/// A mock LLM that simulates temperature-dependent response quality.
///
/// The mock analyses the system prompt to determine how well-specified
/// the expected output format is, then generates responses with a
/// deviation probability proportional to `temperature`.
///
/// # Examples
///
/// ```
/// use feotest_examples::llm::MockChatLlm;
/// use feotest_examples::llm::ChatLlm;
///
/// let mut llm = MockChatLlm::new();
///
/// // Low temperature: highly reliable
/// let response = llm.chat(
///     "Respond with JSON only.",
///     "Add 2 apples",
///     "gpt-4o-mini",
///     0.0,
/// );
/// assert!(response.content().starts_with('{'));
///
/// // Token tracking
/// assert!(llm.total_tokens_used() > 0);
/// llm.reset_token_count();
/// assert_eq!(llm.total_tokens_used(), 0);
/// ```
pub struct MockChatLlm {
    rng: Mutex<StdRng>,
    total_tokens: Mutex<u64>,
}

impl MockChatLlm {
    /// Creates a new mock LLM with a random seed.
    #[must_use]
    pub fn new() -> Self {
        Self {
            rng: Mutex::new(StdRng::from_entropy()),
            total_tokens: Mutex::new(0),
        }
    }

    /// Creates a new mock LLM with a fixed seed for reproducible tests.
    #[must_use]
    pub fn with_seed(seed: u64) -> Self {
        Self {
            rng: Mutex::new(StdRng::seed_from_u64(seed)),
            total_tokens: Mutex::new(0),
        }
    }

    /// Analyses the system prompt to determine how well the expected
    /// output format is specified. Returns a quality score in [0, 1]
    /// where 1 means the prompt leaves no ambiguity.
    fn analyse_prompt(system_message: &str) -> f64 {
        let lower = system_message.to_lowercase();
        let mut score = 0.0;
        let checks = [
            "json",
            "actions",
            "context",
            "name",
            "parameters",
            "add",
            "remove",
            "clear",
            "quantity",
            "item",
        ];
        for keyword in &checks {
            if lower.contains(keyword) {
                score += 1.0;
            }
        }
        #[allow(clippy::cast_precision_loss)]
        let result = (score / checks.len() as f64).min(1.0);
        result
    }

    /// Generates a response for the given instruction.
    ///
    /// The response quality depends on temperature and prompt specificity.
    #[allow(clippy::significant_drop_tightening)]
    fn generate_response(
        &self,
        system_message: &str,
        user_message: &str,
        temperature: f64,
    ) -> String {
        let (should_deviate, failure_mode) = {
            let mut rng = self.rng.lock().unwrap();
            let prompt_quality = Self::analyse_prompt(system_message);

            // Deviation probability: higher temperature and lower prompt quality
            // increase the chance of generating a bad response.
            let deviation_chance = temperature * 0.5 * prompt_quality.mul_add(-0.5, 1.0);
            let deviate: bool = rng.gen_bool(deviation_chance.clamp(0.0, 0.99));
            let mode: u32 = rng.gen_range(0..5);
            (deviate, mode)
        };

        if should_deviate {
            Self::generate_deviant_response(failure_mode)
        } else {
            Self::generate_valid_response(user_message)
        }
    }

    /// Generates a valid JSON response for the instruction.
    fn generate_valid_response(user_message: &str) -> String {
        let lower = user_message.to_lowercase();

        if lower.contains("clear") {
            return r#"{"actions": [{"context": "SHOP", "name": "clear", "parameters": []}]}"#
                .to_string();
        }

        if lower.contains("remove") {
            let item = extract_item(&lower, "remove");
            let qty = extract_quantity(&lower);
            return format!(
                r#"{{"actions": [{{"context": "SHOP", "name": "remove", "parameters": [{{"name": "item", "value": "{item}"}}, {{"name": "quantity", "value": "{qty}"}}]}}]}}"#,
            );
        }

        // Default: add action
        let item = extract_item(&lower, "add");
        let qty = extract_quantity(&lower);
        // Occasionally generate multi-action for compound instructions
        if lower.contains(" and ") {
            let parts: Vec<&str> = lower.split(" and ").collect();
            let actions: Vec<String> = parts
                .iter()
                .map(|part| {
                    let sub_item = extract_item(part, "add");
                    let sub_qty = extract_quantity(part);
                    let action_name = if part.contains("remove") {
                        "remove"
                    } else {
                        "add"
                    };
                    format!(
                        r#"{{"context": "SHOP", "name": "{action_name}", "parameters": [{{"name": "item", "value": "{sub_item}"}}, {{"name": "quantity", "value": "{sub_qty}"}}]}}"#,
                    )
                })
                .collect();
            return format!(r#"{{"actions": [{}]}}"#, actions.join(", "));
        }

        format!(
            r#"{{"actions": [{{"context": "SHOP", "name": "add", "parameters": [{{"name": "item", "value": "{item}"}}, {{"name": "quantity", "value": "{qty}"}}]}}]}}"#,
        )
    }

    /// Generates a deviant response that will fail validation.
    ///
    /// Failure modes mirror real LLM failure patterns:
    /// - Wrong JSON structure (e.g., `"operations"` instead of `"actions"`)
    /// - Invalid action names (e.g., `"purchase"`, `"buy"`)
    /// - Malformed JSON (truncated, wrong quotes)
    /// - Missing required fields
    fn generate_deviant_response(failure_mode: u32) -> String {
        match failure_mode {
            // Wrong top-level key
            0 => r#"{"operations": [{"action": "add", "item": "apples", "quantity": 2}]}"#
                .to_string(),
            // Invalid action name
            1 => {
                r#"{"actions": [{"context": "SHOP", "name": "purchase", "parameters": [{"name": "item", "value": "apples"}]}]}"#
                    .to_string()
            }
            // Malformed JSON (truncated)
            2 => r#"{"actions": [{"context": "SHOP", "name": "add", "para"#.to_string(),
            // Missing context field
            3 => {
                r#"{"actions": [{"name": "add", "parameters": [{"name": "item", "value": "apples"}]}]}"#
                    .to_string()
            }
            // Prose mixed with JSON
            _ => {
                r#"Sure! Here's the action: {"actions": [{"context": "SHOP", "name": "add", "parameters": []}]}"#
                    .to_string()
            }
        }
    }

    /// Estimates token count for a text string (~1.3 tokens per word).
    fn estimate_tokens(text: &str) -> u64 {
        let words = text.split_whitespace().count();
        #[allow(
            clippy::cast_sign_loss,
            clippy::cast_possible_truncation,
            clippy::cast_precision_loss
        )]
        let tokens = (words as f64 * 1.3).ceil() as u64;
        tokens.max(1)
    }
}

impl Default for MockChatLlm {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatLlm for MockChatLlm {
    fn chat(
        &self,
        system_message: &str,
        user_message: &str,
        _model: &str,
        temperature: f64,
    ) -> ChatResponse {
        let content = self.generate_response(system_message, user_message, temperature);

        let prompt_tokens =
            Self::estimate_tokens(system_message) + Self::estimate_tokens(user_message);
        let completion_tokens = Self::estimate_tokens(&content);

        {
            let mut total = self.total_tokens.lock().unwrap();
            *total += prompt_tokens + completion_tokens;
        }

        ChatResponse::new(content, prompt_tokens, completion_tokens)
    }

    fn total_tokens_used(&self) -> u64 {
        *self.total_tokens.lock().unwrap()
    }

    fn reset_token_count(&mut self) {
        *self.total_tokens.lock().unwrap() = 0;
    }
}

/// Extracts an item name from an instruction string.
fn extract_item(text: &str, action: &str) -> String {
    // Simple heuristic: take words after the action verb, skip quantity words
    let after_action = text
        .find(action)
        .map_or(text, |i| &text[i + action.len()..])
        .trim();

    let words: Vec<&str> = after_action
        .split_whitespace()
        .filter(|w| {
            !w.chars().all(|c| c.is_ascii_digit())
                && *w != "a"
                && *w != "an"
                && *w != "the"
                && *w != "some"
                && *w != "of"
                && *w != "from"
                && *w != "basket"
                && *w != "dozen"
        })
        .collect();

    if words.is_empty() {
        "item".to_string()
    } else {
        words.join(" ")
    }
}

/// Extracts a quantity from an instruction string, defaulting to 1.
fn extract_quantity(text: &str) -> String {
    // Look for a number
    for word in text.split_whitespace() {
        if let Ok(n) = word.parse::<u32>() {
            return n.to_string();
        }
    }
    // "a dozen" → 12
    if text.contains("dozen") {
        return "12".to_string();
    }
    "1".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn produces_valid_json_at_zero_temperature() {
        let mut llm = MockChatLlm::with_seed(42);
        for _ in 0..20 {
            let response = llm.chat(
                "Respond with JSON only. Use actions array with context, name, parameters.",
                "Add 2 apples",
                "gpt-4o-mini",
                0.0,
            );
            assert!(
                response.content().starts_with('{'),
                "Expected JSON, got: {}",
                response.content()
            );
        }
        llm.reset_token_count();
        assert_eq!(llm.total_tokens_used(), 0);
    }

    #[test]
    fn tracks_token_usage() {
        let llm = MockChatLlm::with_seed(42);
        llm.chat("system", "user", "model", 0.0);
        assert!(llm.total_tokens_used() > 0);
    }

    #[test]
    fn higher_temperature_produces_more_failures() {
        let llm_low = MockChatLlm::with_seed(42);
        let llm_high = MockChatLlm::with_seed(42);

        let prompt = "Respond with JSON. Use actions array.";
        let mut low_failures = 0;
        let mut high_failures = 0;

        for i in 0..100 {
            let msg = format!("Add {i} apples");
            let low = llm_low.chat(prompt, &msg, "m", 0.0);
            let high = llm_high.chat(prompt, &msg, "m", 1.0);
            if !low.content().contains("\"actions\"") {
                low_failures += 1;
            }
            if !high.content().contains("\"actions\"") {
                high_failures += 1;
            }
        }

        assert!(
            high_failures >= low_failures,
            "Expected more failures at high temp ({high_failures}) than low ({low_failures})"
        );
    }

    #[test]
    fn handles_clear_instructions() {
        let llm = MockChatLlm::with_seed(42);
        let response = llm.chat("JSON only.", "Clear the basket", "m", 0.0);
        assert!(response.content().contains("\"clear\""));
    }

    #[test]
    fn handles_remove_instructions() {
        let llm = MockChatLlm::with_seed(42);
        let response = llm.chat("JSON only.", "Remove the milk", "m", 0.0);
        assert!(response.content().contains("\"remove\""));
    }
}
