//! Shopping basket: the three threshold approaches.
//!
//! feotest offers three ways to fix the relationship between sample count,
//! confidence, and threshold:
//!
//! - **sample-size-first** — fix samples + confidence; derive the threshold
//!   from the baseline (its Wilson lower bound).
//! - **confidence-first** — fix confidence + effect size + power; derive the
//!   required sample count via power analysis on the baseline.
//! - **threshold-first** — fix samples + an explicit threshold; no baseline
//!   needed.
//!
//! The first two are baseline-derived, so they measure first. They apply to
//! the shopping basket, whose criterion is empirical. The third — threshold-
//! first — needs no baseline and so applies to a *normative* contract; it is
//! demonstrated on the payment gateway (see `payment_gateway_sla_test`).

use feotest::experiment::MeasureExperiment;
use feotest::model::ThresholdOrigin;
use feotest::ptest::ProbabilisticTest;
use feotest::ptest::builder::ThresholdApproach;
use feotest::spec::SpecResolver;

use feotest_examples::service_contracts::ShoppingBasketServiceContract;
use feotest_examples::service_contracts::sample_sizes::shopping;
use feotest_examples::service_contracts::shopping_basket::standard_instructions;

fn measure(dir: &std::path::Path) {
    let inputs = standard_instructions();
    MeasureExperiment::builder()
        .service_contract_id("shopping-basket")
        .service_contract(ShoppingBasketServiceContract::new)
        .samples(shopping::MEASURE)
        .inputs(&inputs)
        .baseline_dir(dir)
        .build()
        .run();
}

#[test]
fn sample_size_first() {
    let dir = tempfile::tempdir().expect("temp dir");
    measure(dir.path());
    let inputs = standard_instructions();

    let result = ProbabilisticTest::for_contract(ShoppingBasketServiceContract::new())
        .inputs(&inputs)
        .approach(ThresholdApproach::SampleSizeFirst {
            samples: shopping::TEST,
            confidence: 0.95,
        })
        .spec_resolver(SpecResolver::with_dir(dir.path()))
        .threshold_origin(ThresholdOrigin::Empirical)
        .run();
    let _ = result.passed();
}

#[test]
fn confidence_first() {
    let dir = tempfile::tempdir().expect("temp dir");
    measure(dir.path());
    let inputs = standard_instructions();

    // Derive the sample count needed to detect a 10-point drop at 80% power.
    let result = ProbabilisticTest::for_contract(ShoppingBasketServiceContract::new())
        .inputs(&inputs)
        .approach(ThresholdApproach::ConfidenceFirst {
            confidence: 0.95,
            min_detectable_effect: 0.10,
            power: 0.80,
        })
        .spec_resolver(SpecResolver::with_dir(dir.path()))
        .threshold_origin(ThresholdOrigin::Empirical)
        .run();
    let _ = result.passed();
}
