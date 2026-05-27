//! Explore experiment: comparing configurations.
//!
//! An explore run sweeps a grid of tuning configurations, sampling each, so a
//! team can compare (say) models or temperatures on the same inputs before
//! committing to a baseline. Each configuration yields its own result row,
//! named by a hash of its factor bundle.

use feotest::experiment::ExploreExperiment;

use feotest_examples::service_contracts::sample_sizes::shopping;
use feotest_examples::service_contracts::shopping_basket::{LlmTuning, standard_instructions};

#[test]
fn explore_models_and_temperatures() {
    let inputs = standard_instructions();
    let grid = vec![
        LlmTuning::new("gpt-4o-mini", 0.1),
        LlmTuning::new("gpt-4o-mini", 0.7),
        LlmTuning::new("gpt-4o", 0.1),
        LlmTuning::new("claude-haiku", 0.1),
    ];
    let grid_size = grid.len();

    let result = ExploreExperiment::builder()
        .service_contract_id("shopping-basket")
        .factors(grid)
        .service_contract(|tuning: &LlmTuning| tuning.contract())
        .inputs(&inputs)
        .samples_per_config(shopping::EXPLORE_PER_CONFIG)
        .build()
        .run();

    // One result row per configuration in the grid.
    assert_eq!(result.configs().len(), grid_size);
}
