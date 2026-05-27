//! Shopping basket: time and token budgets.
//!
//! A run can be capped by wall-clock time or by token cost. When a budget is
//! exhausted mid-run, the [`BudgetExhaustedBehavior`] decides what happens:
//! `Fail` rejects the incomplete run, `EvaluatePartial` renders a verdict from
//! the samples gathered so far (with a warning). The shopping contract records
//! token cost per sample, so it is the natural home for the token budget.

mod common;

use std::time::Duration;

use feotest::model::{BudgetExhaustedBehavior, ThresholdOrigin};
use feotest::ptest::ProbabilisticTest;
use feotest::ptest::builder::ThresholdApproach;
use feotest::spec::SpecResolver;

use feotest_examples::service_contracts::ShoppingBasketServiceContract;
use feotest_examples::service_contracts::sample_sizes::shopping;
use feotest_examples::service_contracts::shopping_basket::standard_instructions;

/// A generous time budget the small run completes well within — configured but
/// not hit.
#[test]
fn time_budget_not_exhausted() {
    let dir = tempfile::tempdir().expect("temp dir");
    common::measure_shopping_baseline(dir.path());
    let inputs = standard_instructions();

    let result = ProbabilisticTest::for_contract(ShoppingBasketServiceContract::new())
        .inputs(&inputs)
        .approach(ThresholdApproach::SampleSizeFirst {
            samples: shopping::TEST,
            confidence: 0.95,
        })
        .spec_resolver(SpecResolver::with_dir(dir.path()))
        .threshold_origin(ThresholdOrigin::Empirical)
        .time_budget(Duration::from_secs(30))
        .on_budget_exhausted(BudgetExhaustedBehavior::Fail)
        .run();

    let _ = result.passed();
}

/// A token budget with `EvaluatePartial`: if the token cost is exceeded before
/// all samples run, the verdict is rendered from what was collected.
#[test]
fn token_budget_evaluates_partial() {
    let dir = tempfile::tempdir().expect("temp dir");
    common::measure_shopping_baseline(dir.path());
    let inputs = standard_instructions();

    let result = ProbabilisticTest::for_contract(ShoppingBasketServiceContract::new())
        .inputs(&inputs)
        .approach(ThresholdApproach::SampleSizeFirst {
            samples: shopping::TEST,
            confidence: 0.95,
        })
        .spec_resolver(SpecResolver::with_dir(dir.path()))
        .threshold_origin(ThresholdOrigin::Empirical)
        .token_budget(10_000)
        .on_budget_exhausted(BudgetExhaustedBehavior::EvaluatePartial)
        .run();

    println!(
        "token-budget verdict: {:?}",
        result.verdict_record().verdict()
    );
}
