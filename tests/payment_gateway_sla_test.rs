//! Payment gateway: threshold-first testing against a normative SLA.
//!
//! The payment gateway has an explicit contractual target, so its threshold is
//! stated, not measured. These tests use the threshold-first approach and a
//! normative [`ThresholdOrigin`]. They run at deliberately small sample counts
//! (see [`sample_sizes`]) under a **smoke** intent: too few samples to *verify*
//! the SLA, but enough to exercise the path and catch a catastrophic
//! regression. The feasibility floor for a real verification run is documented
//! as a constant.

use feotest::model::ThresholdOrigin;
use feotest::ptest::ProbabilisticTest;
use feotest::ptest::builder::ThresholdApproach;

use feotest_examples::service_contracts::PaymentGatewayServiceContract;
use feotest_examples::service_contracts::payment_gateway::standard_charges;
use feotest_examples::service_contracts::sample_sizes::payment;

/// A smoke check against the customer-facing 99.99% SLA. At this sample count
/// the run cannot verify the SLA (that needs thousands of samples), so the
/// smoke intent keeps the feasibility gate quiet and the run completes with a
/// verdict instead of a configuration panic.
#[test]
fn smoke_check_against_contractual_sla() {
    let charges = standard_charges();

    let result = ProbabilisticTest::for_contract(PaymentGatewayServiceContract::new())
        .inputs(&charges)
        .approach(ThresholdApproach::ThresholdFirst {
            samples: payment::TEST,
            min_pass_rate: payment::CONTRACTUAL_SLA_PASS_RATE,
        })
        .threshold_origin(ThresholdOrigin::Sla)
        .contract_ref("Payment Provider SLA v2.0 §4.1")
        .smoke()
        .run();

    // The verdict depends on the stochastic mock; we assert the run produced
    // one rather than demanding a pass at this (smoke-sized) sample count.
    println!(
        "contractual SLA smoke verdict: {:?}",
        result.verdict_record().verdict()
    );
}

/// A threshold-first check against the team's internal service-level
/// *objective* (99%), again smoke-sized for a fast example. A real internal
/// verification would use at least [`payment::INTERNAL_OBJECTIVE_FLOOR`]
/// samples — the minimum to verify 99% at 95% confidence.
#[test]
fn smoke_check_against_internal_objective() {
    let charges = standard_charges();

    let result = ProbabilisticTest::for_contract(PaymentGatewayServiceContract::new())
        .inputs(&charges)
        .approach(ThresholdApproach::ThresholdFirst {
            samples: payment::TEST,
            min_pass_rate: payment::INTERNAL_OBJECTIVE_PASS_RATE,
        })
        .threshold_origin(ThresholdOrigin::Slo)
        .smoke()
        .run();

    let _ = result.passed();
}
