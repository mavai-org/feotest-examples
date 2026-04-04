//! Explore experiment: how does temperature affect shopping basket accuracy?
//!
//! Compares two LLM temperature configurations to see which produces
//! more reliable structured action translations. This is the "look
//! before you measure" phase — descriptive statistics only.
//!
//! Run with:
//! ```text
//! cargo test --test explore_shopping_temperatures -- --nocapture
//! cargo test --test explore_shopping_temperatures -- --nocapture > results.yaml
//! ```

#[path = "../usecases/mod.rs"]
mod usecases;

use feotest::experiment::ExploreExperiment;
use usecases::ShoppingBasketUseCase;
use usecases::shopping_basket::standard_instructions;

/// Compares low vs high temperature for shopping basket translation.
#[test]
fn explore_shopping_temperatures() {
    let inputs = standard_instructions();

    let uc_low = ShoppingBasketUseCase::new()
        .model("gpt-4o-mini")
        .temperature(0.1);

    let uc_high = ShoppingBasketUseCase::new()
        .model("gpt-4o-mini")
        .temperature(0.5);

    let result = ExploreExperiment::new(
        "ShoppingBasketUseCase",
        10,
        &inputs,
        ShoppingBasketUseCase::translate_instruction,
    )
    .config_named("low-temp", &uc_low)
    .config_named("high-temp", &uc_high)
    .run();

    print!("{}", result.to_yaml());
}
