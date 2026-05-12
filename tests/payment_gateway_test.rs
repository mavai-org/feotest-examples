//! Probabilistic tests for the payment gateway service contract.
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

mod service_contracts;

use feotest::probabilistic_test;
use service_contracts::PaymentGatewayServiceContract;

/// SLA verification: "The payment gateway must succeed at least 99% of the time."
///
/// The mock gateway achieves ~99.97% success, comfortably above the
/// 99% threshold. This test should pass reliably.
///
/// `threshold_origin = "sla"` marks this as a normative threshold,
/// meaning the framework enforces feasibility checks when the intent
/// is `Verification`.
#[probabilistic_test(
    samples = 200,
    threshold = 0.99,
    threshold_origin = "sla",
    contract_ref = "Payment Provider SLA v2.3, Section 4.1"
)]
fn sla_verification() -> bool {
    PaymentGatewayServiceContract::new()
        .charge_card("tok_visa_4242", 1999)
        .is_success()
}

/// Smoke test: lightweight SLA check with fewer samples.
///
/// Smoke tests trade statistical rigour for speed. They are useful
/// for frequent monitoring (e.g., in a sentinel deployment) where
/// a quick signal is more valuable than a slow proof.
#[probabilistic_test(
    samples = 50,
    threshold = 0.99,
    intent = "smoke",
    threshold_origin = "sla"
)]
fn sla_smoke_test() -> bool {
    PaymentGatewayServiceContract::new()
        .charge_card("tok_visa_4242", 1999)
        .is_success()
}