//! Builder variant of the shopping basket measure experiment.
//!
//! Functionally equivalent to the macro version in
//! `shopping_basket_measure.rs`. Demonstrates the builder API for cases
//! where inputs are dynamic or shared across experiments.
//!
//! Run with:
//! ```text
//! cargo test --test shopping_basket_measure_builder -- --nocapture
//! ```

#[path = "../usecases/mod.rs"]
mod usecases;

use feotest::experiment::MeasureExperiment;
use usecases::ShoppingBasketUseCase;
use usecases::shopping_basket::standard_instructions;

/// Establishes a baseline for the shopping basket use case.
///
/// Uses the builder API with dynamic inputs from `standard_instructions()`.
/// The spec is written to `tests/baselines/` by default.
#[test]
fn measure_shopping_basket_baseline() {
    let inputs = standard_instructions();
    let use_case = ShoppingBasketUseCase::new();

    MeasureExperiment::new("ShoppingBasketUseCase", 1000, &inputs, |input| {
        use_case.translate_instruction(input)
    })
    .experiment_id("baseline-v1")
    .run();
}
