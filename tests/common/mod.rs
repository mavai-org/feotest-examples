//! Shared helpers for the probabilistic-test examples.
//!
//! Each test binary compiles this module independently and uses only the parts
//! it needs, so unused items are expected per binary.
#![allow(dead_code)]

use std::path::Path;

use feotest::experiment::MeasureExperiment;

use feotest_examples::service_contracts::ShoppingBasketServiceContract;
use feotest_examples::service_contracts::sample_sizes::shopping;
use feotest_examples::service_contracts::shopping_basket::standard_instructions;

/// Establishes a shopping-basket baseline in `dir` so an empirical
/// probabilistic test has something to verify against. The shopping contract's
/// single criterion is empirical, so every shopping test needs a baseline.
pub fn measure_shopping_baseline(dir: &Path) {
    let inputs = standard_instructions();
    MeasureExperiment::builder()
        .service_contract_id("shopping-basket")
        .service_contract(ShoppingBasketServiceContract::new)
        .samples(shopping::MEASURE)
        .inputs(&inputs)
        .baseline_dir(dir)
        .build()
        .run();
}
