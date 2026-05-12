//! Explore experiment: how does temperature affect shopping basket accuracy?
//!
//! Compares two LLM temperature configurations to see which produces
//! more reliable structured action translations. This is the "look
//! before you measure" phase — descriptive statistics only.
//!
//! Produces one YAML file per configuration under the output directory.
//! These files are designed to be diffed against one another:
//!
//! ```text
//! diff tests/explorations/low-temp.yaml tests/explorations/high-temp.yaml
//! ```
//!
//! Run with:
//! ```text
//! cargo test --test explore_shopping_temperatures -- --nocapture
//! ```

#[path = "../service_contracts/mod.rs"]
mod service_contracts;

use feotest::experiment::ExploreExperiment;
use service_contracts::ShoppingBasketServiceContract;
use service_contracts::shopping_basket::standard_instructions;

/// Compares low vs high temperature for shopping basket translation.
#[test]
fn explore_shopping_temperatures() {
    let inputs = standard_instructions();

    let uc_low = ShoppingBasketServiceContract::new()
        .model("gpt-4o-mini")
        .temperature(0.0);

    let uc_high = ShoppingBasketServiceContract::new()
        .model("gpt-4o-mini")
        .temperature(1.0);

    let result = ExploreExperiment::new(
        &uc_low,
        10,
        &inputs,
        ShoppingBasketServiceContract::translate_instruction,
    )
    .config_named("high-temp", &uc_high)
    .output_dir("tests/explorations")
    .run();

    for path in result.spec_paths().unwrap_or_default() {
        println!("{}", path.display());
    }
}
