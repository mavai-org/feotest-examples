//! Shopping basket: pacing for rate-limited services.
//!
//! LLM endpoints are often rate-limited. A [`PacingConfig`] throttles the
//! sample loop to a maximum requests-per-second and/or requests-per-minute;
//! the most restrictive constraint governs each sample.

mod common;

use feotest::controls::PacingConfig;
use feotest::model::ThresholdOrigin;
use feotest::ptest::ProbabilisticTest;
use feotest::ptest::builder::ThresholdApproach;
use feotest::spec::SpecResolver;

use feotest_examples::service_contracts::ShoppingBasketServiceContract;
use feotest_examples::service_contracts::sample_sizes::shopping;
use feotest_examples::service_contracts::shopping_basket::standard_instructions;

#[test]
fn paced_at_requests_per_second() {
    let dir = tempfile::tempdir().expect("temp dir");
    common::measure_shopping_baseline(dir.path());
    let inputs = standard_instructions();

    let result = ProbabilisticTest::for_contract(ShoppingBasketServiceContract::new())
        .inputs(&inputs)
        .approach(ThresholdApproach::SampleSizeFirst {
            samples: shopping::TEST,
            confidence: 0.95,
        })
        .spec_resolver(SpecResolver::with_dir(dir.path()))
        .threshold_origin(ThresholdOrigin::Empirical)
        .pacing(PacingConfig::new().max_requests_per_second(20.0))
        .run();

    let _ = result.passed();
}

#[test]
fn paced_with_burst_and_sustained_limits() {
    let dir = tempfile::tempdir().expect("temp dir");
    common::measure_shopping_baseline(dir.path());
    let inputs = standard_instructions();

    // A burst limit (per-second) and a sustained limit (per-minute) together;
    // whichever binds first paces a given sample.
    let result = ProbabilisticTest::for_contract(ShoppingBasketServiceContract::new())
        .inputs(&inputs)
        .approach(ThresholdApproach::SampleSizeFirst {
            samples: shopping::TEST,
            confidence: 0.95,
        })
        .spec_resolver(SpecResolver::with_dir(dir.path()))
        .threshold_origin(ThresholdOrigin::Empirical)
        .pacing(
            PacingConfig::new()
                .max_requests_per_second(50.0)
                .max_requests_per_minute(600.0),
        )
        .run();

    let _ = result.passed();
}
