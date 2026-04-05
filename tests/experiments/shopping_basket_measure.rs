//! Measure experiment for the shopping basket use case.
//!
//! Establishes an empirical baseline by running a large number of trials.
//! The use case provides identity and covariate information; the baseline
//! spec is written to `tests/baselines/` by default.
//!
//! Run with:
//! ```text
//! cargo test --test shopping_basket_measure -- --nocapture
//! ```

#[path = "../usecases/mod.rs"]
mod usecases;

use feotest::experiment::MeasureExperiment;
use usecases::ShoppingBasketUseCase;
use usecases::shopping_basket::standard_instructions;

/// Establishes a baseline for the shopping basket use case.
///
/// Runs 1000 trials cycling through representative instructions.
/// The spec is written to `tests/baselines/` by default.
#[test]
fn measure_shopping_basket_baseline() {
    let inputs = standard_instructions();
    let uc = ShoppingBasketUseCase::new();

    MeasureExperiment::new(&uc, 1000, &inputs, |input| {
        uc.translate_instruction(input)
    })
    .experiment_id("baseline-v1")
    .run();
}
