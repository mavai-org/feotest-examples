//! LLM provider resolution based on environment configuration.
//!
//! The provider factory selects between mock and real LLM implementations
//! based on the `FEOTEST_LLM_MODE` environment variable. This enables
//! the same test code to run against either a cost-free mock (for
//! framework demonstration) or real LLM providers (for production baselines).

use crate::llm::ChatLlm;
use crate::llm::mock::MockChatLlm;
use crate::llm::real::RoutingChatLlm;

/// The resolved LLM operating mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlmMode {
    /// Use the built-in mock (default). No API keys required.
    Mock,
    /// Use real LLM providers. Requires API keys in environment variables.
    Real,
}

impl std::fmt::Display for LlmMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mock => write!(f, "mock"),
            Self::Real => write!(f, "real"),
        }
    }
}

/// Factory for creating LLM instances based on environment configuration.
///
/// Resolution order:
/// 1. Environment variable `FEOTEST_LLM_MODE`
/// 2. Default: `mock`
///
/// # Examples
///
/// ```
/// use feotest_examples::llm::{ChatLlmProvider, LlmMode};
///
/// // In default configuration, resolves to mock mode
/// let mode = ChatLlmProvider::resolved_mode();
/// // mode is Mock unless FEOTEST_LLM_MODE=real is set
///
/// let llm = ChatLlmProvider::resolve();
/// let response = llm.chat("system", "hello", "gpt-4o-mini", 0.3);
/// assert!(!response.content().is_empty());
/// ```
pub struct ChatLlmProvider;

impl ChatLlmProvider {
    /// Resolves the configured LLM mode.
    #[must_use]
    pub fn resolved_mode() -> LlmMode {
        match std::env::var("FEOTEST_LLM_MODE") {
            Ok(mode) if mode.eq_ignore_ascii_case("real") => LlmMode::Real,
            _ => LlmMode::Mock,
        }
    }

    /// Whether the resolved mode is mock.
    #[must_use]
    pub fn is_mock_mode() -> bool {
        Self::resolved_mode() == LlmMode::Mock
    }

    /// Whether the resolved mode is real.
    #[must_use]
    pub fn is_real_mode() -> bool {
        Self::resolved_mode() == LlmMode::Real
    }

    /// Creates a new LLM instance based on the resolved mode.
    ///
    /// - **Mock mode**: returns a [`MockChatLlm`] with a random seed.
    /// - **Real mode**: returns a [`RoutingChatLlm`] that routes to
    ///   `OpenAI` or Anthropic based on model name.
    ///
    /// # Panics
    ///
    /// In real mode, panics if the HTTP client cannot be constructed.
    #[must_use]
    pub fn resolve() -> Box<dyn ChatLlm> {
        match Self::resolved_mode() {
            LlmMode::Mock => Box::new(MockChatLlm::new()),
            LlmMode::Real => Box::new(RoutingChatLlm::new()),
        }
    }
}
