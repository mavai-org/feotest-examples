//! Shopping basket: transparent statistics.
//!
//! Turning on transparent stats makes the framework emit the statistical
//! reasoning behind the verdict — observed rate, Wilson bound, threshold —
//! so a reader can see *why* a run passed or failed, not just the outcome.

mod common;

use feotest::model::ThresholdOrigin;
use feotest::ptest::ProbabilisticTest;
use feotest::ptest::builder::ThresholdApproach;
use feotest::spec::SpecResolver;

use feotest_examples::service_contracts::ShoppingBasketServiceContract;
use feotest_examples::service_contracts::sample_sizes::shopping;
use feotest_examples::service_contracts::shopping_basket::standard_instructions;

#[test]
fn transparent_stats_are_emitted() {
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
        .transparent_stats(true)
        .run();

    println!(
        "diagnostics verdict: {:?}",
        result.verdict_record().verdict()
    );
}
