//! Explore experiment for the shopping basket use case.
//!
//! Rapidly compares multiple LLM model configurations to identify which
//! performs best before committing to a full measurement. Each configuration
//! is a pre-built, immutable use case instance — the framework never mutates
//! a use case during sampling.
//!
//! Run with:
//! ```text
//! cargo test --test shopping_basket_explore -- --nocapture
//! ```

#[path = "../usecases/mod.rs"]
mod usecases;

use feotest::experiment::ExploreExperiment;
use usecases::ShoppingBasketUseCase;
use usecases::shopping_basket::standard_instructions;

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

    // Each configuration is a fully constructed, immutable use case.
    let mut uc_low = ShoppingBasketUseCase::new();
    uc_low.set_model("gpt-4o-mini");
    uc_low.set_temperature(0.1);

    let mut uc_high = ShoppingBasketUseCase::new();
    uc_high.set_model("gpt-4o-mini");
    uc_high.set_temperature(0.5);

    let result =
        ExploreExperiment::new("ShoppingBasketUseCase", 20, &inputs, |uc: &ShoppingBasketUseCase, input| {
            uc.translate_instruction(input)
        })
        .config("gpt-4o-mini (temp=0.1)", &uc_low)
        .config("gpt-4o-mini (temp=0.5)", &uc_high)
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
