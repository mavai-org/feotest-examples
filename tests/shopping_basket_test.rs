//! Probabilistic tests for the shopping basket use case.
//!
//! These tests verify that the shopping basket service meets its reliability
//! threshold. They demonstrate the threshold-first and sample-size-first
//! operational approaches with different test intents, using the
//! `#[probabilistic_test]` macro for compact declaration.
//!
//! The spec-driven test (`spec_driven_sample_size_first`) loads a committed
//! baseline spec from `tests/baselines/ShoppingBasketUseCase-49a9.yaml`. This reflects the
//! recommended workflow: measure once (rarely), commit the spec, then test
//! against it repeatedly.
//!
//! Run with:
//! ```text
//! cargo test --test shopping_basket_test -- --nocapture
//! ```

mod usecases;

use feotest::probabilistic_test;
use usecases::ShoppingBasketUseCase;

/// Threshold-first test with an explicit pass rate.
///
/// "I know the pass rate must be at least 80%. Run 100 samples to verify."
///
/// This is the simplest approach: the developer specifies both the sample
/// count and the threshold. The framework evaluates whether the observed
/// rate meets the threshold and computes the implied confidence.
#[probabilistic_test(samples = 100, threshold = 0.80, threshold_origin = "empirical")]
fn threshold_first_verification(input: &str) -> bool {
    ShoppingBasketUseCase::new()
        .translate_instruction(input)
        .is_success()
}

/// Smoke test with a smaller sample size.
///
/// Smoke tests are lightweight checks that do not claim evidential
/// weight. They accept undersized configurations and are intended
/// for early-warning monitoring rather than rigorous verification.
#[probabilistic_test(samples = 20, threshold = 0.70, intent = "smoke")]
fn smoke_test(input: &str) -> bool {
    ShoppingBasketUseCase::new()
        .translate_instruction(input)
        .is_success()
}

/// Tests with a controlled single instruction.
///
/// Demonstrates testing with a single hardcoded input rather than a varied
/// set. Useful for isolating the performance of a specific instruction type.
#[probabilistic_test(samples = 50, threshold = 0.75)]
fn controlled_single_instruction() -> bool {
    ShoppingBasketUseCase::new()
        .translate_instruction("Add 2 apples")
        .is_success()
}

/// Spec-driven test using a committed baseline.
///
/// This is the recommended workflow:
/// 1. Run a measure experiment to establish a baseline (see `shopping_basket_measure`)
/// 2. Review and commit the spec to `tests/baselines/`
/// 3. Run probabilistic tests against the committed spec — repeatedly, in CI,
///    across releases
///
/// The threshold is derived from the baseline's Wilson lower bound. The
/// developer specifies only the sample count and confidence level; the
/// framework does the rest.
#[probabilistic_test(
    samples = 500,
    confidence = 0.95,
    spec = "tests/baselines/ShoppingBasketUseCase-49a9.yaml",
    threshold_origin = "empirical"
)]
fn spec_driven_sample_size_first(input: &str) -> bool {
    ShoppingBasketUseCase::new()
        .translate_instruction(input)
        .is_success()
}
