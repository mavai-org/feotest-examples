//! Measure experiments: establishing empirical baselines.
//!
//! A measure run exercises a contract many times under one fixed condition and
//! writes a baseline spec — the observed pass rate, its Wilson lower bound, and
//! the latency distribution. A paired probabilistic test later verifies current
//! behaviour against that baseline.
//!
//! These examples write to a temporary directory so they leave no artifacts; a
//! real measure run targets a committed baseline directory.

use feotest::experiment::MeasureExperiment;

use feotest_examples::service_contracts::payment_gateway::standard_charges;
use feotest_examples::service_contracts::sample_sizes::shopping;
use feotest_examples::service_contracts::shopping_basket::standard_instructions;
use feotest_examples::service_contracts::{
    PaymentGatewayServiceContract, ShoppingBasketServiceContract,
};

#[test]
fn measure_shopping_baseline() {
    let dir = tempfile::tempdir().expect("temp dir");
    let inputs = standard_instructions();

    let result = MeasureExperiment::builder()
        .service_contract_id("shopping-basket")
        .service_contract(ShoppingBasketServiceContract::new)
        .samples(shopping::MEASURE)
        .inputs(&inputs)
        .baseline_dir(dir.path())
        .build()
        .run();

    // The run wrote a baseline spec to the target directory.
    assert!(result.spec_path().is_some());
}

#[test]
fn measure_payment_baseline() {
    let dir = tempfile::tempdir().expect("temp dir");
    let charges = standard_charges();

    let result = MeasureExperiment::builder()
        .service_contract_id("payment-gateway")
        .service_contract(PaymentGatewayServiceContract::new)
        .samples(10)
        .inputs(&charges)
        .baseline_dir(dir.path())
        .build()
        .run();

    assert!(result.spec_path().is_some());
}
