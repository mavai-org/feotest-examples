//! Builder variants of the shopping basket probabilistic tests.
//!
//! These tests are functionally equivalent to the macro versions in
//! `shopping_basket_test.rs`. They demonstrate the builder API for
//! cases where inputs are dynamic, shared, or generated at runtime.
//!
//! Run with:
//! ```text
//! cargo test --test shopping_basket_test_builder -- --nocapture
//! ```

mod service_contracts;

use feotest::model::{TestIntent, ThresholdOrigin};
use feotest::ptest::ProbabilisticTest;
use service_contracts::ShoppingBasketServiceContract;
use service_contracts::shopping_basket::standard_instructions;

/// Threshold-first test with the builder API.
///
/// Equivalent to the `threshold_first_verification` macro test.
/// The builder is useful here when the inputs come from a shared
/// function rather than being inlined.
#[test]
fn threshold_first_verification() {
    let inputs = standard_instructions();
    let service_contract = ShoppingBasketServiceContract::new();

    ProbabilisticTest::new("ShoppingBasketServiceContract", &inputs, |input| {
        service_contract.translate_instruction(input)
    })
    .samples(100)
    .threshold(0.80)
    .threshold_origin(ThresholdOrigin::Empirical)
    .run();
}

/// Smoke test with the builder API.
///
/// Equivalent to the `smoke_test` macro test.
#[test]
fn smoke_test() {
    let inputs = standard_instructions();
    let service_contract = ShoppingBasketServiceContract::new();

    ProbabilisticTest::new("ShoppingBasketServiceContract", &inputs, |input| {
        service_contract.translate_instruction(input)
    })
    .samples(20)
    .threshold(0.70)
    .intent(TestIntent::Smoke)
    .run();
}

/// Spec-driven test with the builder API.
///
/// Equivalent to the `spec_driven_sample_size_first` macro test.
/// The builder resolves the baseline automatically from the service contract
/// ID — no explicit path needed.
#[test]
fn spec_driven_sample_size_first() {
    let inputs = standard_instructions();
    let service_contract = ShoppingBasketServiceContract::new();

    ProbabilisticTest::new("ShoppingBasketServiceContract", &inputs, |input| {
        service_contract.translate_instruction(input)
    })
    .samples(500)
    .confidence(0.95)
    .threshold_origin(ThresholdOrigin::Empirical)
    .run();
}
