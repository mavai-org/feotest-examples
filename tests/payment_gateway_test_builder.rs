//! Builder variants of the payment gateway probabilistic tests.
//!
//! These tests are functionally equivalent to the macro versions in
//! `payment_gateway_test.rs`. They demonstrate the builder API with
//! SLA-driven normative thresholds.
//!
//! Run with:
//! ```text
//! cargo test --test payment_gateway_test_builder -- --nocapture
//! ```

mod usecases;

use feotest::model::{TestIntent, ThresholdOrigin};
use feotest::ptest::ProbabilisticTest;
use usecases::PaymentGatewayUseCase;

/// SLA verification with the builder API.
///
/// Equivalent to the `sla_verification` macro test.
#[test]
fn sla_verification() {
    let inputs = vec!["tok_visa_4242:1999".to_string()];
    let use_case = PaymentGatewayUseCase::new();

    ProbabilisticTest::new("PaymentGatewayUseCase", &inputs, |_input| {
        use_case.charge_card("tok_visa_4242", 1999)
    })
    .samples(200)
    .threshold(0.99)
    .threshold_origin(ThresholdOrigin::Sla)
    .contract_ref("Payment Provider SLA v2.3, Section 4.1")
    .run();
}

/// Smoke test with the builder API.
///
/// Equivalent to the `sla_smoke_test` macro test.
#[test]
fn sla_smoke_test() {
    let inputs = vec!["tok_visa_4242:1999".to_string()];
    let use_case = PaymentGatewayUseCase::new();

    ProbabilisticTest::new("PaymentGatewayUseCase", &inputs, |_input| {
        use_case.charge_card("tok_visa_4242", 1999)
    })
    .samples(50)
    .threshold(0.99)
    .intent(TestIntent::Smoke)
    .threshold_origin(ThresholdOrigin::Sla)
    .run();
}
