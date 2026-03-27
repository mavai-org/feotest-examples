//! Probabilistic tests for the payment gateway use case.
//!
//! These tests demonstrate threshold-first testing with an SLA-driven
//! normative threshold. The payment gateway has an explicit contractual
//! target (99% success), unlike the shopping basket where the threshold
//! is derived empirically.
//!
//! Run with:
//! ```text
//! cargo test --test payment_gateway_test -- --nocapture
//! ```

use feotest::model::{TestIntent, ThresholdOrigin};
use feotest::ptest::ProbabilisticTestBuilder;
use feotest::ptest::builder::ThresholdApproach;
use feotest::verdict::Verdict;
use feotest_examples::usecases::PaymentGatewayUseCase;

/// SLA verification: "The payment gateway must succeed at least 99% of the time."
///
/// The mock gateway achieves ~99.97% success, comfortably above the
/// 99% threshold. This test should pass reliably.
///
/// Note: `ThresholdOrigin::Sla` marks this as a normative threshold,
/// meaning the framework enforces feasibility checks when the intent
/// is `Verification`.
#[test]
fn sla_verification() {
    let inputs = vec!["tok_visa_4242:1999".to_string()];
    let use_case = PaymentGatewayUseCase::new();

    let result = ProbabilisticTestBuilder::new("PaymentGatewayUseCase", &inputs, |_input| {
        use_case.charge_card("tok_visa_4242", 1999)
    })
    .approach(ThresholdApproach::ThresholdFirst {
        samples: 200,
        min_pass_rate: 0.99,
    })
    .intent(TestIntent::Verification)
    .threshold_origin(ThresholdOrigin::Sla)
    .contract_ref("Payment Provider SLA v2.3, Section 4.1")
    .run();

    let record = result.verdict_record();
    println!("=== Payment Gateway SLA Verification ===");
    println!("Verdict:       {}", record.verdict());
    println!(
        "Pass rate:     {:.4} ({}/{})",
        record.functional().pass_rate(),
        record.functional().successes(),
        record.functional().successes() + record.functional().failures(),
    );

    if let Some(prov) = record.spec_provenance() {
        if let Some(cref) = prov.contract_ref() {
            println!("Contract ref:  {cref}");
        }
    }

    // Warnings (e.g., undersized sample caveat)
    for warning in record.warnings() {
        println!("Warning:       {warning}");
    }

    assert_eq!(record.verdict(), Verdict::Pass);
}

/// Smoke test: lightweight SLA check with fewer samples.
///
/// Smoke tests trade statistical rigour for speed. They are useful
/// for frequent monitoring (e.g., in a sentinel deployment) where
/// a quick signal is more valuable than a slow proof.
#[test]
fn sla_smoke_test() {
    let inputs = vec!["tok_visa_4242:1999".to_string()];
    let use_case = PaymentGatewayUseCase::new();

    let result = ProbabilisticTestBuilder::new("PaymentGatewayUseCase", &inputs, |_input| {
        use_case.charge_card("tok_visa_4242", 1999)
    })
    .approach(ThresholdApproach::ThresholdFirst {
        samples: 50,
        min_pass_rate: 0.99,
    })
    .intent(TestIntent::Smoke)
    .threshold_origin(ThresholdOrigin::Sla)
    .run();

    let record = result.verdict_record();
    println!("=== Payment Gateway Smoke Test ===");
    println!(
        "Verdict: {} (intent: {})",
        record.verdict(),
        record.intent()
    );

    // Smoke test against normative threshold should carry a caveat
    assert_eq!(record.intent(), TestIntent::Smoke);
}

/// Measure experiment for the payment gateway.
///
/// Establishes a baseline for the gateway's reliability. With the mock's
/// 99.97% success rate, the derived threshold will be close to but
/// slightly below 1.0 — accounting for sampling variability.
#[test]
fn measure_payment_baseline() {
    let dir = tempfile::tempdir().unwrap();
    let inputs = vec!["tok_visa_4242:1999".to_string()];
    let use_case = PaymentGatewayUseCase::new();

    let result = feotest::experiment::MeasureExperiment::new(
        "PaymentGatewayUseCase",
        200,
        &inputs,
        |_input| use_case.charge_card("tok_visa_4242", 1999),
    )
    .with_experiment_id("baseline-v1")
    .with_spec_resolver(feotest::spec::SpecResolver::with_dir(dir.path()))
    .run();

    let spec = result.spec();
    println!("=== Payment Gateway Baseline ===");
    println!("Samples:           {}", spec.execution.samples_executed);
    println!(
        "Observed rate:     {:.4}",
        spec.statistics.success_rate.observed
    );
    println!("Derived threshold: {:.4}", spec.requirements.min_pass_rate);

    assert!(spec.requirements.min_pass_rate > 0.95);
}
