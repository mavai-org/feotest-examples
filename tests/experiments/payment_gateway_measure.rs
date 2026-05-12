//! Measure experiment for the payment gateway service contract.
//!
//! Establishes a baseline for the gateway's reliability. With the mock's
//! 99.97% success rate, the derived threshold will be close to but
//! slightly below 1.0 — accounting for sampling variability.
//!
//! Run with:
//! ```text
//! cargo test --test payment_gateway_measure -- --nocapture
//! ```

#[path = "../service_contracts/mod.rs"]
mod service_contracts;

use feotest::experiment::MeasureExperiment;
use service_contracts::PaymentGatewayServiceContract;

/// Establishes a baseline for the payment gateway service contract.
#[test]
fn measure_payment_baseline() {
    let inputs = vec!["tok_visa_4242:1999".to_string()];
    let uc = PaymentGatewayServiceContract::new();

    MeasureExperiment::new(&uc, 200, &inputs, |_input| {
        uc.charge_card("tok_visa_4242", 1999)
    })
    .experiment_id("baseline-v1")
    .run();
}
