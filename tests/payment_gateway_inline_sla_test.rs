//! Payment gateway: the inline contract form.
//!
//! When a single criterion suffices and the target is normative (no measured
//! baseline), the whole contract can be authored at the test site with the
//! `#[probabilistic_test]` attribute — no separate `ServiceContract` type. The
//! function body is the criterion's postcondition; the attribute carries the
//! sample count, the target, its origin, and the contract reference.

use feotest::model::ContractViolation;
use feotest::probabilistic_test;

use feotest_examples::payment::{MockPaymentGateway, PaymentGateway};

/// Inline SLA check: charge a card and require the transaction to succeed.
/// `Err(ContractViolation)` is a counted failure (a declined charge), not a
/// defect. Smoke-sized, like the named-contract SLA test.
#[probabilistic_test(
    samples = 3,
    threshold = 0.99,
    threshold_origin = "sla",
    contract_ref = "Payment Provider SLA v2.0 §4.1",
    intent = "smoke"
)]
fn payment_gateway_meets_sla_inline(_input: &str) -> Result<(), ContractViolation> {
    let gateway = MockPaymentGateway::new();
    let result = gateway.charge("tok_visa_4242", 1999);
    if result.is_success() {
        Ok(())
    } else {
        let code = result.error_code().unwrap_or("UNKNOWN");
        Err(ContractViolation::new(
            "transaction",
            format!("payment failed: {code}"),
        ))
    }
}
