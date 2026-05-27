//! Optimize experiment: searching for a better configuration.
//!
//! An optimize run iterates: sample the current configuration, score it, then
//! mutate to the next configuration. Here a cool-down mutator walks the
//! temperature down from 1.0, and the scorer is the iteration's pass rate —
//! structured-output reliability is expected to improve as temperature falls.
//!
//! The `Scorer` and `FactorMutator` are traits the author implements; there is
//! no built-in scorer or closure shortcut.

use feotest::experiment::{
    ContractExecutionResult, FactorMutator, IterationRecord, OptimizeExperiment, Scorer,
};

use feotest_examples::service_contracts::sample_sizes::shopping;
use feotest_examples::service_contracts::shopping_basket::{LlmTuning, standard_instructions};

/// Scores an iteration by its observed pass rate.
struct PassRate;

impl Scorer for PassRate {
    fn score(&self, result: &ContractExecutionResult) -> f64 {
        result.aggregate().success_rate()
    }
}

/// Walks the temperature down by 0.1 each iteration, model fixed.
struct CoolDown;

impl FactorMutator<LlmTuning> for CoolDown {
    fn mutate(&self, current: &LlmTuning, _history: &[IterationRecord<LlmTuning>]) -> LlmTuning {
        LlmTuning::new(current.model.clone(), (current.temperature - 0.1).max(0.0))
    }
}

#[test]
fn optimize_temperature_cooldown() {
    let inputs = standard_instructions();

    let result = OptimizeExperiment::builder()
        .service_contract_id("shopping-basket")
        .initial_factor(LlmTuning::new("gpt-4o-mini", 1.0))
        .service_contract(|tuning: &LlmTuning| tuning.contract())
        .scorer(PassRate)
        .mutator(CoolDown)
        .inputs(&inputs)
        .samples_per_iteration(shopping::OPTIMIZE_PER_ITERATION)
        .max_iterations(5)
        .build()
        .run();

    // The run recorded an iteration history and a best-seen configuration.
    assert!(!result.history().is_empty());
    assert!(result.best_factor().is_some());
}
