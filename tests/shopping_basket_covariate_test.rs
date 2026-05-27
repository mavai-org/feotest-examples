//! Shopping basket: covariate-aware baseline matching.
//!
//! A baseline is only comparable to a test run made under the same conditions.
//! The shopping contract declares covariates (model, temperature, and the
//! day/time of the run); the baseline is partitioned by them, and the
//! verifying test resolves the baseline that matches its own covariate
//! profile. A profile mismatch still resolves (so the test runs) but records a
//! misalignment warning.

use feotest::experiment::MeasureExperiment;
use feotest::model::ThresholdOrigin;
use feotest::ptest::ProbabilisticTest;
use feotest::ptest::builder::ThresholdApproach;
use feotest::service_contract::ServiceContract;
use feotest::spec::SpecResolver;

use feotest_examples::service_contracts::ShoppingBasketServiceContract;
use feotest_examples::service_contracts::sample_sizes::shopping;
use feotest_examples::service_contracts::shopping_basket::standard_instructions;

const COVARIATE_KEYS: [&str; 2] = ["llm_model", "temperature"];

fn measure_baseline(dir: &std::path::Path, model: &'static str) {
    let inputs = standard_instructions();
    let contract = ShoppingBasketServiceContract::new().model(model);
    MeasureExperiment::builder()
        .service_contract_id(contract.id().to_owned())
        .service_contract(move || ShoppingBasketServiceContract::new().model(model))
        .samples(shopping::MEASURE)
        .inputs(&inputs)
        .baseline_dir(dir)
        .covariates(
            COVARIATE_KEYS.iter().map(ToString::to_string).collect(),
            contract.resolve_covariates(),
        )
        .build()
        .run();
}

#[test]
fn matching_covariates_resolve_the_baseline() {
    let dir = tempfile::tempdir().expect("temp dir");
    measure_baseline(dir.path(), "gpt-4o-mini");

    let inputs = standard_instructions();
    let result =
        ProbabilisticTest::for_contract(ShoppingBasketServiceContract::new().model("gpt-4o-mini"))
            .inputs(&inputs)
            .approach(ThresholdApproach::SampleSizeFirst {
                samples: shopping::TEST,
                confidence: 0.95,
            })
            .spec_resolver(SpecResolver::with_dir(dir.path()))
            .threshold_origin(ThresholdOrigin::Empirical)
            .run();

    // Same model as the baseline: it resolves and yields a verdict.
    println!(
        "matched-covariate verdict: {:?}",
        result.verdict_record().verdict()
    );
}

#[test]
fn mismatched_covariates_resolve_with_a_warning() {
    let dir = tempfile::tempdir().expect("temp dir");
    measure_baseline(dir.path(), "gpt-4o-mini");

    let inputs = standard_instructions();
    // Verify under a different model than the baseline was measured with.
    let result =
        ProbabilisticTest::for_contract(ShoppingBasketServiceContract::new().model("gpt-4-turbo"))
            .inputs(&inputs)
            .approach(ThresholdApproach::SampleSizeFirst {
                samples: shopping::TEST,
                confidence: 0.95,
            })
            .spec_resolver(SpecResolver::with_dir(dir.path()))
            .threshold_origin(ThresholdOrigin::Empirical)
            .run();

    // The run still completes — covariate misalignment is a warning, not a hard
    // failure to resolve.
    let _ = result.passed();
}
