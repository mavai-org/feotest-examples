//! Measure experiment for the payment gateway use case.
//!
//! Establishes a baseline for the gateway's reliability. With the mock's
//! 99.97% success rate, the derived threshold will be close to but
//! slightly below 1.0 — accounting for sampling variability.
//!
//! Run with:
//! ```text
//! cargo test --test payment_gateway_measure -- --nocapture
//! ```

#[path = "../usecases/mod.rs"]
mod usecases;

use feotest::measure_experiment;
use usecases::PaymentGatewayUseCase;

#[measure_experiment(
    use_case = "PaymentGatewayUseCase",
    samples = 200,
    inputs = ["tok_visa_4242:1999"],
    experiment_id = "baseline-v1",
    spec_dir = "target/test-specs"
)]
fn measure_payment_baseline(_input: &str) -> TrialOutcome {
    PaymentGatewayUseCase::new().charge_card("tok_visa_4242", 1999)
}
