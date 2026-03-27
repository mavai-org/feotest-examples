//! Shopping basket use case: translating natural language to structured actions.
//!
//! This use case wraps an LLM call that translates instructions like
//! "Add 2 apples" into structured JSON actions. It demonstrates the full
//! feotest workflow:
//!
//! 1. Define a service contract with postconditions
//! 2. Execute the service and evaluate the contract
//! 3. Return a `TrialOutcome` for statistical analysis
//!
//! The use case is inherently stochastic: the same instruction may produce
//! different (and sometimes invalid) responses across invocations, depending
//! on the model and temperature.

use std::time::Instant;

use feotest::model::{ContractViolation, TrialOutcome};

use crate::llm::{ChatLlm, ChatLlmProvider};
use crate::shopping::ShoppingActionValidator;

/// The system prompt sent to the LLM. Instructs it to produce structured
/// JSON in the expected format.
const DEFAULT_SYSTEM_PROMPT: &str = "\
You are a shopping assistant. Translate the user's natural language instruction \
into a JSON object with an \"actions\" array. Each action has:\n\
- \"context\": always \"SHOP\"\n\
- \"name\": one of \"add\", \"remove\", or \"clear\"\n\
- \"parameters\": array of {\"name\": string, \"value\": string} pairs\n\n\
For add/remove actions, include \"item\" and \"quantity\" parameters.\n\
For clear actions, use an empty parameters array.\n\n\
Respond with JSON only. No prose, no markdown, no explanation.";

/// A use case for translating natural language shopping instructions into
/// structured actions via an LLM.
///
/// # Contract postconditions
///
/// 1. **Response has content**: the LLM returned a non-empty string.
/// 2. **Valid shopping action**: the response parses as valid JSON with
///    the expected `actions` array structure.
/// 3. **Contains valid actions**: each action's name is valid for its
///    context (e.g., `"add"` is valid for `Shop`, `"purchase"` is not).
///
/// # Configuration
///
/// - `model`: the LLM model identifier (default: `"gpt-4o-mini"`)
/// - `temperature`: controls response variability (default: `0.3`)
///
/// # Examples
///
/// ```
/// use feotest_examples::usecases::ShoppingBasketUseCase;
///
/// let mut use_case = ShoppingBasketUseCase::new();
///
/// // Execute a trial and get the outcome for statistical analysis
/// let outcome = use_case.translate_instruction("Add 2 apples");
///
/// // The outcome is either success or a contract violation —
/// // never a panic. This is the fundamental distinction between
/// // a contract failure (a statistical observation) and a defect.
/// if outcome.is_success() {
///     println!("Trial succeeded");
/// } else {
///     println!("Contract violation: {}", outcome.violation().unwrap());
/// }
/// ```
pub struct ShoppingBasketUseCase {
    llm: Box<dyn ChatLlm>,
    model: String,
    temperature: f64,
    system_prompt: String,
}

impl ShoppingBasketUseCase {
    /// Creates a new shopping basket use case with default configuration.
    ///
    /// Uses the LLM resolved by [`ChatLlmProvider`] (mock by default).
    #[must_use]
    pub fn new() -> Self {
        Self {
            llm: ChatLlmProvider::resolve(),
            model: "gpt-4o-mini".to_string(),
            temperature: 0.3,
            system_prompt: DEFAULT_SYSTEM_PROMPT.to_string(),
        }
    }

    /// Creates a use case with a specific LLM implementation.
    ///
    /// Useful for testing with a mock that has a fixed seed.
    #[must_use]
    pub fn with_llm(llm: Box<dyn ChatLlm>) -> Self {
        Self {
            llm,
            model: "gpt-4o-mini".to_string(),
            temperature: 0.3,
            system_prompt: DEFAULT_SYSTEM_PROMPT.to_string(),
        }
    }

    /// The current LLM model identifier.
    #[must_use]
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Sets the LLM model identifier.
    pub fn set_model(&mut self, model: impl Into<String>) {
        self.model = model.into();
    }

    /// The current temperature.
    #[must_use]
    pub const fn temperature(&self) -> f64 {
        self.temperature
    }

    /// Sets the temperature.
    pub const fn set_temperature(&mut self, temperature: f64) {
        self.temperature = temperature;
    }

    /// The current system prompt.
    #[must_use]
    pub fn system_prompt(&self) -> &str {
        &self.system_prompt
    }

    /// Sets the system prompt.
    pub fn set_system_prompt(&mut self, prompt: impl Into<String>) {
        self.system_prompt = prompt.into();
    }

    /// Translates a natural language instruction into structured shopping
    /// actions and evaluates the service contract.
    ///
    /// Returns a [`TrialOutcome`] suitable for consumption by feotest's
    /// execution engine. The outcome is either a success (all postconditions
    /// met) or a contract violation (one or more postconditions failed).
    ///
    /// # Contract evaluation
    ///
    /// The three postconditions are evaluated in order (fail-fast):
    ///
    /// 1. Response must have content (non-empty)
    /// 2. Response must parse as a valid shopping action JSON
    /// 3. All parsed actions must have valid names for their contexts
    ///
    /// The [`ShoppingActionValidator`] handles checks 2 and 3 as a
    /// single validation step, returning a `ContractViolation` if either
    /// fails.
    #[must_use]
    pub fn translate_instruction(&self, instruction: &str) -> TrialOutcome {
        let start = Instant::now();

        let response = self.llm.chat(
            &self.system_prompt,
            instruction,
            &self.model,
            self.temperature,
        );

        let elapsed = start.elapsed();

        // Postcondition 1: Response has content
        if response.content().trim().is_empty() {
            return TrialOutcome::failure(
                ContractViolation::new("content", "empty response from LLM"),
                elapsed,
            );
        }

        // Postconditions 2 & 3: Valid structure and valid actions
        match ShoppingActionValidator::validate(&response) {
            Ok(_actions) => TrialOutcome::success(elapsed),
            Err(violation) => TrialOutcome::failure(violation, elapsed),
        }
    }
}

impl Default for ShoppingBasketUseCase {
    fn default() -> Self {
        Self::new()
    }
}

/// The standard set of test instructions for the shopping basket use case.
///
/// These 10 instructions cover a range of complexity: simple additions,
/// removals, compound operations, and ambiguous natural language. They
/// are cycled round-robin when the sample count exceeds 10.
///
/// # Examples
///
/// ```
/// use feotest_examples::usecases::shopping_basket::standard_instructions;
///
/// let inputs = standard_instructions();
/// assert_eq!(inputs.len(), 10);
/// assert_eq!(inputs[0], "Add 2 apples");
/// ```
#[must_use]
pub fn standard_instructions() -> Vec<String> {
    vec![
        "Add 2 apples".to_string(),
        "Remove the milk".to_string(),
        "Add 1 loaf of bread".to_string(),
        "Add 3 oranges and 2 bananas".to_string(),
        "Add 5 tomatoes and remove the cheese".to_string(),
        "Clear the basket".to_string(),
        "Clear everything".to_string(),
        "Remove 2 eggs from the basket".to_string(),
        "Add a dozen eggs".to_string(),
        "I'd like to remove all the vegetables".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::MockChatLlm;

    #[test]
    fn succeeds_with_low_temperature() {
        let llm = MockChatLlm::with_seed(42);
        let uc = ShoppingBasketUseCase::with_llm(Box::new(llm));

        let outcome = uc.translate_instruction("Add 2 apples");
        assert!(outcome.is_success(), "Expected success at low temperature");
    }

    #[test]
    fn configuration_accessors() {
        let mut uc = ShoppingBasketUseCase::new();
        assert_eq!(uc.model(), "gpt-4o-mini");
        assert!((uc.temperature() - 0.3).abs() < 1e-10);

        uc.set_model("claude-sonnet-4-6");
        uc.set_temperature(0.7);
        assert_eq!(uc.model(), "claude-sonnet-4-6");
        assert!((uc.temperature() - 0.7).abs() < 1e-10);
    }

    #[test]
    fn standard_instructions_has_ten_entries() {
        assert_eq!(standard_instructions().len(), 10);
    }
}
