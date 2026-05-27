//! Misconfiguration is rejected, not silently tolerated.
//!
//! These are pedagogical anti-examples: each shows a configuration the
//! framework refuses, and the `#[should_panic]` message is the diagnostic a
//! developer would see. A violated precondition is a defect in the calling
//! code, so the framework panics rather than producing a misleading verdict.

use feotest::criteria::Criterion;
use feotest::model::ThresholdOrigin;
use feotest::ptest::ProbabilisticTest;
use feotest::ptest::builder::ThresholdApproach;

use feotest_examples::service_contracts::PaymentGatewayServiceContract;
use feotest_examples::service_contracts::payment_gateway::standard_charges;

/// A normative pass rate must lie in the open interval (0, 1).
#[test]
#[should_panic(expected = "pass rate must be in (0, 1)")]
fn pass_rate_above_one_is_rejected() {
    let _ = Criterion::<String>::meeting().pass_rate(1.5);
}

/// Under the default verification intent, a sample count too small to verify
/// the SLA at 95% confidence is rejected before the run starts. (Contrast the
/// smoke-intent tests, which tolerate undersizing with a warning.)
#[test]
#[should_panic(expected = "feasib")]
fn verification_rejects_an_infeasible_sample_size() {
    let charges = standard_charges();
    let _ = ProbabilisticTest::for_contract(PaymentGatewayServiceContract::new())
        .inputs(&charges)
        .approach(ThresholdApproach::ThresholdFirst {
            samples: 3,
            min_pass_rate: 0.9999,
        })
        .threshold_origin(ThresholdOrigin::Sla)
        .run();
}
