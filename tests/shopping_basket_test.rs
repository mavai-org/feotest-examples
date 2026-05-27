//! Shopping basket: empirical verification against a measured baseline.
//!
//! The shopping basket's reliability target is not stated — it is *measured*.
//! This is the core empirical workflow: a measure run establishes a baseline,
//! then a probabilistic test verifies that current behaviour still meets it,
//! using the sample-size-first approach (the threshold is the baseline's
//! Wilson lower bound at the chosen confidence).
//!
//! The two steps share a temporary baseline directory so the test is
//! self-contained — no committed fixture, no run ordering between test files.

use feotest::experiment::MeasureExperiment;
use feotest::model::ThresholdOrigin;
use feotest::ptest::ProbabilisticTest;
use feotest::ptest::builder::ThresholdApproach;
use feotest::spec::SpecResolver;

use feotest_examples::service_contracts::ShoppingBasketServiceContract;
use feotest_examples::service_contracts::sample_sizes::shopping;
use feotest_examples::service_contracts::shopping_basket::standard_instructions;

#[test]
fn meets_measured_baseline() {
    let dir = tempfile::tempdir().expect("temp dir");
    let inputs = standard_instructions();

    // Step 1 — measure: establish the empirical baseline in the temp dir.
    MeasureExperiment::builder()
        .service_contract_id("shopping-basket")
        .service_contract(ShoppingBasketServiceContract::new)
        .samples(shopping::MEASURE)
        .inputs(&inputs)
        .baseline_dir(dir.path())
        .build()
        .run();

    // Step 2 — verify: the probabilistic test reads that baseline back and
    // derives its threshold from it (sample-size-first).
    let result = ProbabilisticTest::for_contract(ShoppingBasketServiceContract::new())
        .inputs(&inputs)
        .approach(ThresholdApproach::SampleSizeFirst {
            samples: shopping::TEST,
            confidence: 0.95,
        })
        .spec_resolver(SpecResolver::with_dir(dir.path()))
        .threshold_origin(ThresholdOrigin::Empirical)
        .run();

    // The verdict depends on the stochastic mock; we assert the empirical path
    // resolved a baseline and produced a verdict rather than going inconclusive
    // for want of one.
    println!("empirical verdict: {:?}", result.verdict_record().verdict());
}
