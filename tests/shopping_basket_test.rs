//! Probabilistic tests for the shopping basket use case.
//!
//! These tests verify that the shopping basket service meets its reliability
//! threshold. They demonstrate the three operational approaches and
//! different test intents.
//!
//! Run with:
//! ```text
//! cargo test --test shopping_basket_test -- --nocapture
//! ```

use feotest::model::{TestIntent, ThresholdOrigin};
use feotest::ptest::ProbabilisticTestBuilder;
use feotest::ptest::builder::ThresholdApproach;
use feotest::verdict::Verdict;
use feotest_examples::llm::MockChatLlm;
use feotest_examples::usecases::ShoppingBasketUseCase;
use feotest_examples::usecases::shopping_basket::standard_instructions;

/// Threshold-first test with an explicit pass rate.
///
/// "I know the pass rate must be at least 80%. Run 100 samples to verify."
///
/// This is the simplest approach: the developer specifies both the sample
/// count and the threshold. The framework evaluates whether the observed
/// rate meets the threshold and computes the implied confidence.
#[test]
fn threshold_first_verification() {
    let inputs = standard_instructions();
    let mut use_case = ShoppingBasketUseCase::new();

    let result = ProbabilisticTestBuilder::new("ShoppingBasketUseCase", &inputs, |instruction| {
        use_case.translate_instruction(instruction)
    })
    .approach(ThresholdApproach::ThresholdFirst {
        samples: 100,
        min_pass_rate: 0.80,
    })
    .intent(TestIntent::Verification)
    .threshold_origin(ThresholdOrigin::Empirical)
    .run();

    let record = result.verdict_record();
    println!("=== Threshold-First Verification ===");
    println!("Verdict:       {}", record.verdict());
    println!(
        "Pass rate:     {:.4} ({}/{})",
        record.functional().pass_rate(),
        record.functional().successes(),
        record.functional().successes() + record.functional().failures(),
    );

    if let Some(stats) = record.statistical_analysis() {
        println!("Threshold:     {:.4}", stats.threshold());
        println!(
            "95% CI:        [{:.4}, {:.4}]",
            stats.ci_lower(),
            stats.ci_upper()
        );
        if let Some(p) = stats.p_value() {
            println!("p-value:       {p:.4}");
        }
    }

    // With mock at 0.3 temperature, expect ~95% pass rate → should pass 80% threshold
    assert_eq!(record.verdict(), Verdict::Pass);
}

/// Smoke test with a smaller sample size.
///
/// Smoke tests are lightweight checks that do not claim evidential
/// weight. They accept undersized configurations and are intended
/// for early-warning monitoring rather than rigorous verification.
#[test]
fn smoke_test() {
    let inputs = standard_instructions();
    let mut use_case = ShoppingBasketUseCase::new();

    let result = ProbabilisticTestBuilder::new("ShoppingBasketUseCase", &inputs, |instruction| {
        use_case.translate_instruction(instruction)
    })
    .approach(ThresholdApproach::ThresholdFirst {
        samples: 20,
        min_pass_rate: 0.80,
    })
    .intent(TestIntent::Smoke)
    .run();

    let record = result.verdict_record();
    println!("=== Smoke Test ===");
    println!(
        "Verdict: {} (intent: {})",
        record.verdict(),
        record.intent()
    );
    println!("Samples: {}", record.execution().samples_executed());

    assert_eq!(record.intent(), TestIntent::Smoke);
}

/// Spec-driven test using a baseline established by a measure experiment.
///
/// This is the recommended workflow:
/// 1. Run a measure experiment to establish a baseline (see `shopping_basket_measure`)
/// 2. Run a probabilistic test with `SampleSizeFirst`, which derives the
///    threshold from the baseline's Wilson lower bound.
///
/// This test creates a temporary baseline and immediately verifies against it.
/// It uses a seeded mock for deterministic behaviour.
#[test]
fn spec_driven_sample_size_first() {
    let dir = tempfile::tempdir().unwrap();
    let inputs = standard_instructions();

    // Step 1: Establish a baseline with a large sample (seeded for determinism)
    let mut baseline_uc = ShoppingBasketUseCase::with_llm(Box::new(MockChatLlm::with_seed(42)));
    let _measure = feotest::experiment::MeasureExperiment::new(
        "ShoppingBasketUseCase",
        500,
        &inputs,
        |instruction| baseline_uc.translate_instruction(instruction),
    )
    .with_spec_resolver(feotest::spec::SpecResolver::with_dir(dir.path()))
    .run();

    // Step 2: Run a probabilistic test against the baseline (fresh seeded mock)
    let mut test_uc = ShoppingBasketUseCase::with_llm(Box::new(MockChatLlm::with_seed(42)));
    let resolver = feotest::spec::SpecResolver::with_dir(dir.path());
    let result = ProbabilisticTestBuilder::new("ShoppingBasketUseCase", &inputs, |instruction| {
        test_uc.translate_instruction(instruction)
    })
    .approach(ThresholdApproach::SampleSizeFirst {
        samples: 100,
        confidence: 0.95,
    })
    .spec_resolver(resolver)
    .threshold_origin(ThresholdOrigin::Empirical)
    .run();

    let record = result.verdict_record();
    println!("=== Spec-Driven (Sample-Size-First) ===");
    println!("Verdict: {}", record.verdict());
    if let Some(stats) = record.statistical_analysis() {
        println!("Threshold (from baseline): {:.4}", stats.threshold());
    }
    if let Some(prov) = record.spec_provenance() {
        if let Some(file) = prov.spec_filename() {
            println!("Baseline spec: {file}");
        }
    }

    assert_eq!(record.verdict(), Verdict::Pass);
}

/// Tests with a controlled single instruction.
///
/// Demonstrates testing with a single input rather than a varied set.
/// Useful for isolating the performance of a specific instruction type.
#[test]
fn controlled_single_instruction() {
    let inputs = vec!["Add 2 apples".to_string()];
    let use_case = ShoppingBasketUseCase::with_llm(Box::new(MockChatLlm::with_seed(99)));

    let result = ProbabilisticTestBuilder::new("ShoppingBasketUseCase", &inputs, |instruction| {
        use_case.translate_instruction(instruction)
    })
    .approach(ThresholdApproach::ThresholdFirst {
        samples: 50,
        min_pass_rate: 0.75,
    })
    .run();

    println!("=== Controlled Single Instruction ===");
    println!(
        "Verdict: {} (pass rate: {:.4})",
        result.verdict_record().verdict(),
        result.verdict_record().functional().pass_rate(),
    );

    assert_eq!(result.verdict_record().verdict(), Verdict::Pass);
}
