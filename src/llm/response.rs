//! Chat response type bundling content with token usage metadata.

/// The response from a chat completion request.
///
/// Contains the generated text and token usage counts for cost tracking.
#[derive(Debug, Clone)]
pub struct ChatResponse {
    content: String,
    prompt_tokens: u64,
    completion_tokens: u64,
}

impl ChatResponse {
    /// Creates a new chat response.
    #[must_use]
    pub fn new(content: impl Into<String>, prompt_tokens: u64, completion_tokens: u64) -> Self {
        Self {
            content: content.into(),
            prompt_tokens,
            completion_tokens,
        }
    }

    /// The generated text content.
    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Number of tokens in the input (system + user messages).
    #[must_use]
    pub const fn prompt_tokens(&self) -> u64 {
        self.prompt_tokens
    }

    /// Number of tokens in the generated response.
    #[must_use]
    pub const fn completion_tokens(&self) -> u64 {
        self.completion_tokens
    }

    /// Total tokens consumed (prompt + completion).
    #[must_use]
    pub const fn total_tokens(&self) -> u64 {
        self.prompt_tokens + self.completion_tokens
    }
}

impl std::fmt::Display for ChatResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ChatResponse[content={}, promptTokens={}, completionTokens={}]",
            self.content, self.prompt_tokens, self.completion_tokens
        )
    }
}
