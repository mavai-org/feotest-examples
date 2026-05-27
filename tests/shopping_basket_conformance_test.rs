//! Shopping basket: reproducible (conformant) verdicts under a seeded mock.
//!
//! Probabilistic verdicts are only auditable if they are reproducible: the same
//! inputs and the same (seeded) service must yield the same verdict. This test
//! drives the full measure → verify pipeline twice with a fixed-seed mock LLM
//! and asserts the two verdicts conform. (With the default unseeded mock, the
//! verdict varies run to run — that is the stochastic case the other tests
//! exercise.)

use feotest::experiment::MeasureExperiment;
use feotest::model::ThresholdOrigin;
use feotest::ptest::ProbabilisticTest;
use feotest::ptest::builder::ThresholdApproach;
use feotest::spec::SpecResolver;
use feotest::verdict::Verdict;

use feotest_examples::llm::MockChatLlm;
use feotest_examples::service_contracts::ShoppingBasketServiceContract;
use feotest_examples::service_contracts::sample_sizes::shopping;
use feotest_examples::service_contracts::shopping_basket::standard_instructions;

const SEED: u64 = 42;

/// Runs measure → verify with a seed-`SEED` mock and returns the verdict.
fn seeded_run() -> Verdict {
    let dir = tempfile::tempdir().expect("temp dir");
    let inputs = standard_instructions();

    MeasureExperiment::builder()
        .service_contract_id("shopping-basket")
        .service_contract(|| {
            ShoppingBasketServiceContract::with_llm(Box::new(MockChatLlm::with_seed(SEED)))
        })
        .samples(shopping::MEASURE)
        .inputs(&inputs)
        .baseline_dir(dir.path())
        .build()
        .run();

    let result = ProbabilisticTest::for_contract(ShoppingBasketServiceContract::with_llm(
        Box::new(MockChatLlm::with_seed(SEED)),
    ))
    .inputs(&inputs)
    .approach(ThresholdApproach::SampleSizeFirst {
        samples: shopping::TEST,
        confidence: 0.95,
    })
    .spec_resolver(SpecResolver::with_dir(dir.path()))
    .threshold_origin(ThresholdOrigin::Empirical)
    .run();

    result.verdict_record().verdict()
}

#[test]
fn seeded_runs_produce_a_conformant_verdict() {
    assert_eq!(
        seeded_run(),
        seeded_run(),
        "a fixed seed must yield a reproducible verdict"
    );
}
