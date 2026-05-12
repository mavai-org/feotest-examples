//! Covariate-aware baseline selection tests for the shopping basket service contract.
//!
//! Demonstrates UC05: when multiple covariate-partitioned baselines exist,
//! the framework selects the one that best matches the current runtime
//! covariate profile. The `.service_contract(&uc)` method on the test builder
//! provides the covariate context for selection.
//!
//! Two baselines exist in `tests/baselines/`:
//! - `shopping-basket-d833-47e1-4f4c-5bad-a769.yaml` — temperature=0.3
//! - `shopping-basket-d833-47e1-4f4c-5bad-f51e.yaml` — temperature=0.7
//!
//! Run with:
//! ```text
//! cargo test --test shopping_basket_covariate_test -- --nocapture
//! ```

mod service_contracts;

use feotest::model::ThresholdOrigin;
use feotest::ptest::ProbabilisticTest;
use service_contracts::shopping_basket::standard_instructions;
use service_contracts::ShoppingBasketServiceContract;

/// The framework selects the temperature=0.3 baseline when the service contract
/// is configured with temperature=0.3.
///
/// Both baselines have the same service contract ID and footprint, but differ
/// in the `temperature` covariate value. The selector matches on the
/// resolved profile and picks the correct partition.
#[test]
fn selects_baseline_matching_low_temperature() {
    let inputs = standard_instructions();
    let service_contract = ShoppingBasketServiceContract::new().temperature(0.3);

    // The .service_contract() call provides covariate context for baseline selection.
    // The framework resolves covariates (day-of-week, time-of-day, model,
    // temperature) and selects the baseline whose covariate profile matches
    // best.
    ProbabilisticTest::new("shopping-basket", &inputs, |input| {
        service_contract.translate_instruction(input)
    })
    .samples(100)
    .threshold(0.80)
    .threshold_origin(ThresholdOrigin::Empirical)
    .service_contract(&service_contract)
    .run();
}

/// The framework selects the temperature=0.7 baseline when the service contract
/// is configured with temperature=0.7.
///
/// The threshold is set lower than the temperature=0.3 test because
/// higher temperature produces more failures in the mock.
#[test]
fn selects_baseline_matching_high_temperature() {
    let inputs = standard_instructions();
    let service_contract = ShoppingBasketServiceContract::new().temperature(0.7);

    ProbabilisticTest::new("shopping-basket", &inputs, |input| {
        service_contract.translate_instruction(input)
    })
    .samples(100)
    .threshold(0.70)
    .threshold_origin(ThresholdOrigin::Empirical)
    .service_contract(&service_contract)
    .run();
}
