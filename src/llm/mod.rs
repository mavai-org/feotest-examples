//! LLM infrastructure: chat interface, mock, and real provider routing.
//!
//! The LLM subsystem provides a uniform interface for chat-based language
//! model interaction. Two modes are supported:
//!
//! - **Mock** (default): a deterministic mock that simulates realistic LLM
//!   behaviour including temperature-dependent failure rates. No API keys
//!   required.
//! - **Real**: routes requests to `OpenAI` or Anthropic based on model name.
//!   Requires API keys via environment variables.
//!
//! The mode is selected via `FEOTEST_LLM_MODE` (or the system property
//! equivalent). When unset, mock mode is used.

mod mock;
mod provider;
mod real;
mod response;

pub use mock::MockChatLlm;
pub use provider::{ChatLlmProvider, LlmMode};
pub use response::ChatResponse;

/// A chat-based language model.
///
/// Implementations accept a system message, user message, model identifier,
/// and temperature, and return a [`ChatResponse`] containing the generated
/// text and token usage metadata.
///
/// # Examples
///
/// ```
/// use feotest_examples::llm::{ChatLlmProvider, ChatResponse};
///
/// let llm = ChatLlmProvider::resolve();
/// let response = llm.chat("You are a helpful assistant.", "Hello!", "gpt-4o-mini", 0.3);
/// assert!(!response.content().is_empty());
/// ```
pub trait ChatLlm: Send + Sync {
    /// Sends a chat completion request and returns the response.
    fn chat(
        &self,
        system_message: &str,
        user_message: &str,
        model: &str,
        temperature: f64,
    ) -> ChatResponse;

    /// Returns the cumulative number of tokens consumed.
    fn total_tokens_used(&self) -> u64;

    /// Resets the token counter to zero.
    fn reset_token_count(&mut self);
}
