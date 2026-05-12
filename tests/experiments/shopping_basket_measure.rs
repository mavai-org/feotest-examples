//! Measure experiment for the shopping basket service contract.
//!
//! Establishes an empirical baseline by running a large number of trials.
//! The service contract provides identity and covariate information; the baseline
//! spec is written to `tests/baselines/` by default.
//!
//! Run with:
//! ```text
//! cargo test --test shopping_basket_measure -- --nocapture
//! ```

#[path = "../service_contracts/mod.rs"]
mod service_contracts;

use feotest::experiment::MeasureExperiment;
use service_contracts::ShoppingBasketServiceContract;
use service_contracts::shopping_basket::standard_instructions;

/// Establishes a baseline for the shopping basket service contract.
///
/// Runs 1000 trials cycling through representative instructions.
/// The spec is written to `tests/baselines/` by default.
#[test]
fn measure_shopping_basket_baseline() {
    let inputs = standard_instructions();
    let uc = ShoppingBasketServiceContract::new();

    MeasureExperiment::new(&uc, 1000, &inputs, |input| {
        uc.translate_instruction(input)
    })
    .experiment_id("baseline-v1")
    .run();
}
