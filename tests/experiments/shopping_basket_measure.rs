//! Measure experiment for the shopping basket use case.
//!
//! Establishes an empirical baseline by running a large number of trials
//! and recording the observed success rate. The resulting spec file
//! captures the Wilson lower bound as the derived minimum pass rate,
//! which subsequent probabilistic tests use as their threshold.
//!
//! Run with:
//! ```text
//! cargo test --test shopping_basket_measure -- --nocapture
//! ```

#[path = "../usecases/mod.rs"]
mod usecases;

use feotest::measure_experiment;
use usecases::ShoppingBasketUseCase;

/// Establishes a baseline for the shopping basket use case.
///
/// Runs 1000 trials cycling through 10 representative instructions.
/// At the default temperature (0.3) and the default mock, this typically
/// yields a ~95% success rate, producing a derived threshold around 0.93.
///
/// The spec is written to `target/test-specs/`. In a real project, you
/// would write to `specs/` and commit the file.
#[measure_experiment(
    use_case = "ShoppingBasketUseCase",
    samples = 1000,
    inputs = [
        "Add 2 apples",
        "Remove the milk",
        "Add 1 loaf of bread",
        "Add 3 oranges and 2 bananas",
        "Add 5 tomatoes and remove the cheese",
        "Clear the basket",
        "Clear everything",
        "Remove 2 eggs from the basket",
        "Add a dozen eggs",
        "I'd like to remove all the vegetables"
    ],
    experiment_id = "baseline-v1",
    spec_dir = "target/test-specs"
)]
fn measure_shopping_basket_baseline(input: &str) -> TrialOutcome {
    ShoppingBasketUseCase::new().translate_instruction(input)
}
