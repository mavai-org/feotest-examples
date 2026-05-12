//! Explore experiment for the shopping basket service contract.
//!
//! Rapidly compares multiple LLM model configurations to identify which
//! performs best before committing to a full measurement. Each configuration
//! is a pre-built, immutable service contract instance — the framework never mutates
//! a service contract during sampling.
//!
//! Run with:
//! ```text
//! cargo test --test shopping_basket_explore -- --nocapture
//! ```

#[path = "../service_contracts/mod.rs"]
mod service_contracts;

use feotest::experiment::ExploreExperiment;
use service_contracts::ShoppingBasketServiceContract;
use service_contracts::shopping_basket::standard_instructions;

/// Compares different model configurations for the shopping basket.
///
/// Each configuration is constructed upfront with its own temperature
/// and model settings. The trial function is declared once and shared
/// across all configurations.
///
/// In mock mode, all configurations use the same mock — the comparison
/// demonstrates the framework's exploration mechanics rather than real
/// model differences. In real mode (`FEOTEST_LLM_MODE=real`), this
/// would compare actual model performance.
#[test]
fn explore_model_configurations() {
    let inputs = standard_instructions();

    // Each configuration is fully constructed and immutable.
    let uc_low = ShoppingBasketServiceContract::new()
        .model("gpt-4o-mini")
        .temperature(0.1);

    let uc_high = ShoppingBasketServiceContract::new()
        .model("gpt-4o-mini")
        .temperature(0.5);

    let result =
        ExploreExperiment::new(&uc_low, 20, &inputs, |uc: &ShoppingBasketServiceContract, input| {
            uc.translate_instruction(input)
        })
        .config(&uc_high)
        .experiment_id("model-comparison")
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
