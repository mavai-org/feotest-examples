//! Explore experiment for the shopping basket use case.
//!
//! Rapidly compares multiple LLM model configurations to identify which
//! performs best before committing to a full measurement. Each configuration
//! is tested with a small number of samples — enough to spot clear winners,
//! not enough for statistical rigour.
//!
//! Run with:
//! ```text
//! cargo test --test shopping_basket_explore -- --nocapture
//! ```

#[path = "../usecases/mod.rs"]
mod usecases;

use feotest::experiment::ExploreExperiment;
use std::sync::{Arc, Mutex};
use usecases::ShoppingBasketUseCase;
use usecases::shopping_basket::standard_instructions;

/// Compares different model configurations for the shopping basket.
///
/// In mock mode, all configurations use the same mock — the comparison
/// demonstrates the framework's exploration mechanics rather than real
/// model differences. In real mode (`FEOTEST_LLM_MODE=real`), this
/// would compare actual model performance.
#[test]
fn explore_model_configurations() {
    let inputs = standard_instructions();

    // Shared use case behind a mutex so the explore closures can mutate it
    let use_case = Arc::new(Mutex::new(ShoppingBasketUseCase::new()));

    let uc_a = Arc::clone(&use_case);
    let uc_b = Arc::clone(&use_case);
    let uc_trial = Arc::clone(&use_case);

    let result = ExploreExperiment::new("ShoppingBasketUseCase", 20, &inputs, move |instruction| {
        uc_trial.lock().unwrap().translate_instruction(instruction)
    })
    .config("gpt-4o-mini (temp=0.1)", move || {
        let mut uc = uc_a.lock().unwrap();
        uc.set_model("gpt-4o-mini");
        uc.set_temperature(0.1);
    })
    .config("gpt-4o-mini (temp=0.5)", move || {
        let mut uc = uc_b.lock().unwrap();
        uc.set_model("gpt-4o-mini");
        uc.set_temperature(0.5);
    })
    .with_experiment_id("model-comparison")
    .run();

    println!("=== Shopping Basket Exploration ===");
    println!("Configurations tested: {}", result.configs().len());
    println!();

    for config in result.configs() {
        let summary = config.execution().summary();
        let rate = summary.observed_pass_rate();
        println!(
            "  {:<35} pass rate: {:.2}%  ({}/{} samples)",
            config.name(),
            rate * 100.0,
            summary.successes(),
            summary.samples_executed(),
        );
    }

    // Verify all configurations ran
    assert_eq!(result.configs().len(), 2);
    for config in result.configs() {
        assert_eq!(config.execution().summary().samples_executed(), 20);
    }
}
