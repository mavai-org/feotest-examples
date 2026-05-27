//! Shopping basket: translating natural language into structured actions.
//!
//! An LLM turns an instruction like "Add 2 apples" into a JSON object with an
//! `actions` array. The service is stochastic — the same instruction may
//! produce a different (and sometimes invalid) response each time — so its
//! reliability is established empirically and then verified against that
//! measured baseline.

use std::fmt;

use feotest::controls::Cost;
use feotest::criteria::{Criteria, Criterion};
use feotest::model::{ContractViolation, Defect};
use feotest::service_contract::{CovariateCategory, CovariateDeclaration, ServiceContract};
use feotest::spec::namer::CovariateProfile;

use crate::llm::{ChatLlm, ChatLlmProvider, ChatResponse};
use crate::shopping::ShoppingActionValidator;

/// The system prompt sent to the LLM, instructing it to emit structured JSON.
const DEFAULT_SYSTEM_PROMPT: &str = "\
You are a shopping assistant. Translate the user's natural language instruction \
into a JSON object with an \"actions\" array. Each action has:\n\
- \"context\": always \"SHOP\"\n\
- \"name\": one of \"add\", \"remove\", or \"clear\"\n\
- \"parameters\": array of {\"name\": string, \"value\": string} pairs\n\n\
For add/remove actions, include \"item\" and \"quantity\" parameters.\n\
For clear actions, use an empty parameters array.\n\n\
Respond with JSON only. No prose, no markdown, no explanation.";

/// Translates natural-language shopping instructions into structured actions
/// via an LLM.
///
/// Configuration is fixed at construction: the contract is immutable once
/// built, which keeps every sample identically distributed.
pub struct ShoppingBasketServiceContract {
    llm: Box<dyn ChatLlm>,
    model: String,
    temperature: f64,
    system_prompt: String,
}

impl ShoppingBasketServiceContract {
    /// Creates a contract using the LLM resolved by [`ChatLlmProvider`]
    /// (a mock by default — no API keys required).
    #[must_use]
    pub fn new() -> Self {
        Self::with_llm(ChatLlmProvider::resolve())
    }

    /// Creates a contract backed by a specific LLM (e.g. a seeded mock).
    #[must_use]
    pub fn with_llm(llm: Box<dyn ChatLlm>) -> Self {
        Self {
            llm,
            model: "gpt-4o-mini".to_string(),
            temperature: 0.3,
            system_prompt: DEFAULT_SYSTEM_PROMPT.to_string(),
        }
    }

    /// Sets the model identifier.
    #[must_use]
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Sets the sampling temperature.
    #[must_use]
    pub const fn temperature(mut self, temperature: f64) -> Self {
        self.temperature = temperature;
        self
    }

    /// Sets the system prompt.
    #[must_use]
    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = prompt.into();
        self
    }

    /// The default system prompt, exposed for tests that call the LLM directly.
    #[must_use]
    pub const fn default_system_prompt() -> &'static str {
        DEFAULT_SYSTEM_PROMPT
    }
}

impl ServiceContract for ShoppingBasketServiceContract {
    type Input = String;
    type Output = ChatResponse;

    fn id(&self) -> &'static str {
        "shopping-basket"
    }

    fn description(&self) -> &'static str {
        "Translates natural language shopping instructions into structured actions via an LLM"
    }

    fn warmup(&self) -> u32 {
        3
    }

    /// Calls the LLM and returns its raw response. The response is judged by
    /// the criteria; a malformed response is not a defect, so `invoke` only
    /// records the token cost and returns the response.
    fn invoke(&self, instruction: &String, cost: &mut Cost) -> Result<ChatResponse, Defect> {
        let response = self.llm.chat(
            &self.system_prompt,
            instruction,
            &self.model,
            self.temperature,
        );
        cost.record_tokens(response.total_tokens());
        Ok(response)
    }

    /// One empirical criterion: the response must be non-empty and parse into
    /// valid shopping actions. The parse is a `transforming` step, so a
    /// malformed or contextually-invalid response is a counted failure with a
    /// precise reason, never an abort.
    fn criteria(&self) -> Criteria<ChatResponse> {
        Criteria::of([Criterion::empirical()
            .pass_rate()
            .name("valid-shopping-actions")
            .satisfies("response not empty", |response: &ChatResponse| {
                if response.content().trim().is_empty() {
                    Err(ContractViolation::new("content", "empty response"))
                } else {
                    Ok(())
                }
            })
            .transforming(ShoppingActionValidator::validate)
            .build()])
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

impl Default for ShoppingBasketServiceContract {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ShoppingBasketServiceContract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (temperature={})", self.model, self.temperature)
    }
}

/// A tuning configuration for the shopping service — the factor varied by
/// explore and optimize experiments. `Serialize` lets the framework name each
/// configuration's artifacts by a hash of the factor bundle.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LlmTuning {
    /// The model identifier.
    pub model: String,
    /// The sampling temperature.
    pub temperature: f64,
}

impl LlmTuning {
    /// Creates a tuning bundle.
    #[must_use]
    pub fn new(model: impl Into<String>, temperature: f64) -> Self {
        Self {
            model: model.into(),
            temperature,
        }
    }

    /// Builds a contract configured by this tuning — the factory body for
    /// explore and optimize experiments.
    #[must_use]
    pub fn contract(&self) -> ShoppingBasketServiceContract {
        ShoppingBasketServiceContract::new()
            .model(self.model.clone())
            .temperature(self.temperature)
    }
}

impl fmt::Display for LlmTuning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.model, self.temperature)
    }
}

/// The canonical instruction set.
///
/// Shared by experiments and tests so a baseline and the test that verifies it
/// sample the same inputs.
#[must_use]
pub fn standard_instructions() -> Vec<String> {
    [
        "Add 2 apples",
        "Remove the milk",
        "Add 1 loaf of bread",
        "Add 3 oranges and 2 bananas",
        "Add 5 tomatoes and remove the cheese",
        "Clear the basket",
        "Clear everything",
        "Remove 2 eggs from the basket",
        "Add a dozen eggs",
        "I'd like to remove all the vegetables",
    ]
    .iter()
    .map(ToString::to_string)
    .collect()
}
