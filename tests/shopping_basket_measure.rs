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

use feotest::experiment::MeasureExperiment;
use feotest::spec::SpecResolver;
use feotest_examples::usecases::ShoppingBasketUseCase;
use feotest_examples::usecases::shopping_basket::standard_instructions;

/// Establishes a baseline for the shopping basket use case.
///
/// Runs 1000 trials cycling through 10 representative instructions.
/// At the default temperature (0.3) and the default mock, this typically
/// yields a ~95% success rate, producing a derived threshold around 0.93.
///
/// The spec is written to a temporary directory. In a real project, you
/// would write to `specs/` and commit the file.
#[test]
fn measure_shopping_basket_baseline() {
    let dir = tempfile::tempdir().unwrap();
    let resolver = SpecResolver::with_dir(dir.path());
    let inputs = standard_instructions();

    let mut use_case = ShoppingBasketUseCase::new();

    let result = MeasureExperiment::new("ShoppingBasketUseCase", 1000, &inputs, |instruction| {
        use_case.translate_instruction(instruction)
    })
    .with_experiment_id("baseline-v1")
    .with_spec_resolver(resolver)
    .run();

    let spec = result.spec();

    println!("=== Shopping Basket Baseline ===");
    println!("Samples executed:  {}", spec.execution.samples_executed);
    println!("Successes:         {}", spec.statistics.successes);
    println!("Failures:          {}", spec.statistics.failures);
    println!(
        "Observed rate:     {:.4}",
        spec.statistics.success_rate.observed
    );
    println!("Derived threshold: {:.4}", spec.requirements.min_pass_rate);
    println!(
        "95% CI:            [{:.4}, {:.4}]",
        spec.statistics.success_rate.confidence_interval95[0],
        spec.statistics.success_rate.confidence_interval95[1]
    );

    if let Some(path) = result.spec_path() {
        println!("Spec written to:   {}", path.display());
    }

    // Sanity checks
    assert_eq!(spec.statistics.successes + spec.statistics.failures, 1000);
    assert!(
        spec.requirements.min_pass_rate > 0.5,
        "Derived threshold should be reasonable, got {}",
        spec.requirements.min_pass_rate
    );
}
