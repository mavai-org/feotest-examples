//! Shopping basket service contract: translating natural language to structured actions.
//!
//! This service contract wraps an LLM call that translates instructions like
//! "Add 2 apples" into structured JSON actions. It demonstrates the full
//! feotest workflow:
//!
//! 1. Define a service contract with postconditions
//! 2. Execute the service and evaluate the contract
//! 3. Return a `TrialOutcome` for statistical analysis
//!
//! The service contract is inherently stochastic: the same instruction may produce
//! different (and sometimes invalid) responses across invocations, depending
//! on the model and temperature.

use std::fmt;
use std::time::Instant;

use feotest::model::{ContractViolation, TrialOutcome};
use feotest::spec::namer::CovariateProfile;
use feotest::service_contract::{CovariateCategory, CovariateDeclaration, ServiceContract};

use feotest_examples::llm::{ChatLlm, ChatLlmProvider};
use feotest_examples::shopping::ShoppingActionValidator;

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

/// A service contract for translating natural language shopping instructions into
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
/// All configuration is set at construction time. The service contract is immutable
/// after construction — this is a deliberate design choice that preserves
/// the i.i.d. assumption required for valid statistical inference.
///
/// - `model`: the LLM model identifier (default: `"gpt-4o-mini"`)
/// - `temperature`: controls response variability (default: `0.3`)
pub struct ShoppingBasketServiceContract {
    llm: Box<dyn ChatLlm>,
    model: String,
    temperature: f64,
    system_prompt: String,
}

impl ShoppingBasketServiceContract {
    /// Returns the default system prompt used by this service contract.
    ///
    /// Exposed for tests that call the LLM directly (e.g., conformance
    /// tests that need the raw response string).
    #[must_use]
    pub fn default_system_prompt() -> &'static str {
        DEFAULT_SYSTEM_PROMPT
    }

    /// Creates a new shopping basket service contract with default configuration.
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

    /// Creates a service contract with a specific LLM implementation.
    ///
    /// Useful for testing with a mock that has a fixed seed.
    #[must_use]
    pub fn llm(llm: Box<dyn ChatLlm>) -> Self {
        Self {
            llm,
            model: "gpt-4o-mini".to_string(),
            temperature: 0.3,
            system_prompt: DEFAULT_SYSTEM_PROMPT.to_string(),
        }
    }

    /// Sets the LLM model identifier at construction time.
    #[must_use]
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Sets the temperature at construction time.
    #[must_use]
    pub fn temperature(mut self, temperature: f64) -> Self {
        self.temperature = temperature;
        self
    }

    /// Sets the system prompt at construction time.
    #[must_use]
    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = prompt.into();
        self
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

        let content = response.content().to_string();

        // Postcondition 1: Response has content
        if content.trim().is_empty() {
            return TrialOutcome::failure(
                ContractViolation::new("content", "empty response from LLM"),
                elapsed,
            )
            .content(&content)
            .postcondition("has content", "failed");
        }

        // Postconditions 2 & 3: Valid structure and valid actions
        match ShoppingActionValidator::validate(&response) {
            Ok(_actions) => TrialOutcome::success(elapsed)
                .content(&content)
                .postcondition("has content", "passed")
                .postcondition("valid structure", "passed")
                .postcondition("valid actions", "passed"),
            Err(violation) => {
                let check = violation.check().to_string();
                let mut outcome =
                    TrialOutcome::failure(violation, elapsed).content(&content);

                for &name in &["has content", "valid structure", "valid actions"] {
                    if name == check {
                        outcome = outcome.postcondition(name, "failed");
                        break;
                    }
                    outcome = outcome.postcondition(name, "passed");
                }
                outcome
            }
        }
    }
}

impl ServiceContract for ShoppingBasketServiceContract {
    fn id(&self) -> &str {
        "shopping-basket"
    }

    fn description(&self) -> &str {
        "Translates natural language shopping instructions into structured actions via an LLM"
    }

    fn warmup(&self) -> u32 {
        3
    }

    fn covariates(&self) -> Vec<CovariateDeclaration> {
        vec![
            CovariateDeclaration::day_of_week(),
            CovariateDeclaration::time_of_day(),
            CovariateDeclaration::new("llm_model", CovariateCategory::ExternalDependency),
            CovariateDeclaration::new("temperature", CovariateCategory::ExternalDependency),
        ]
    }

    fn resolve_covariates(&self) -> CovariateProfile {
        CovariateProfile::builder()
            .put("day-of-week", CovariateProfile::resolve_day_of_week())
            .put("time-of-day", CovariateProfile::resolve_time_of_day())
            .put("llm_model", &self.model)
            .put("temperature", self.temperature.to_string())
            .build()
    }
}

impl fmt::Display for ShoppingBasketServiceContract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (temperature={})", self.model, self.temperature)
    }
}

impl Default for ShoppingBasketServiceContract {
    fn default() -> Self {
        Self::new()
    }
}

/// The standard set of test instructions for the shopping basket service contract.
///
/// These 10 instructions cover a range of complexity: simple additions,
/// removals, compound operations, and ambiguous natural language. They
/// are cycled round-robin when the sample count exceeds 10.
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