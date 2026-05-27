//! Deployable reliability sentinel.
//!
//! A sentinel is a standalone binary that carries reliability specifications
//! into a target environment and runs them on demand — a scheduled job, a
//! health check, a post-deploy gate. The same specs a developer runs locally
//! ship here unchanged.
//!
//! Build and invoke:
//!
//! ```text
//! cargo build --bin sentinel
//! cargo run --bin sentinel -- list                       # list registered specs
//! cargo run --bin sentinel -- run <spec>                 # run a probabilistic check
//! cargo run --bin sentinel -- measure --output <uri> <spec>   # establish a baseline
//! cargo run --bin sentinel -- check --baselines <dir>    # verify embedded baselines
//! ```
//!
//! Two specs are registered, mirroring the two example services: a normative
//! (SLA-origin) payment check that needs no baseline, and an empirical shopping
//! check paired with its calibrating measure experiment.

use feotest::{sentinel, sentinel_impl};

use feotest_examples::llm::ChatLlmProvider;
use feotest_examples::payment::{MockPaymentGateway, PaymentGateway};
use feotest_examples::service_contracts::ShoppingBasketServiceContract;
use feotest_examples::shopping::ShoppingActionValidator;

/// Payment gateway: a normative SLA check. The threshold is contractual, so no
/// baseline is resolved — the spec is self-contained.
#[sentinel(description = "Payment gateway SLA reliability check")]
#[derive(Default)]
pub struct PaymentGatewaySentinel;

#[sentinel_impl]
impl PaymentGatewaySentinel {
    /// One charge must succeed. Samples are i.i.d. charges of a representative
    /// card.
    #[probabilistic_test(origin = "sla", threshold = 0.99, samples = 3)]
    fn meets_contractual_sla(&self) -> bool {
        let _ = self;
        MockPaymentGateway::new()
            .charge("tok_visa_4242", 1999)
            .is_success()
    }
}

/// Shopping basket: an empirical check.
///
/// The calibrate experiment establishes the baseline that `meets_baseline` is
/// verified against — the measure ↔ test pairing that a deployed sentinel
/// re-establishes in its target environment.
#[sentinel(description = "Shopping basket empirical reliability check")]
#[derive(Default)]
pub struct ShoppingBasketSentinel;

#[sentinel_impl]
impl ShoppingBasketSentinel {
    /// Establishes the baseline (run once in the target environment).
    #[measure_experiment(baseline_for = "meets_baseline", samples = 30)]
    fn calibrate(&self) -> bool {
        let _ = self;
        translates_one_instruction()
    }

    /// Verifies current behaviour against the calibrated baseline.
    #[probabilistic_test(
        origin = "empirical",
        samples = 3,
        confidence = 0.90,
        baseline = "calibrate"
    )]
    fn meets_baseline(&self) -> bool {
        let _ = self;
        translates_one_instruction()
    }
}

/// One shopping sample: translate a representative instruction and check the
/// response parses into valid actions.
fn translates_one_instruction() -> bool {
    let llm = ChatLlmProvider::resolve();
    let response = llm.chat(
        ShoppingBasketServiceContract::default_system_prompt(),
        "Add 2 apples",
        "gpt-4o-mini",
        0.3,
    );
    ShoppingActionValidator::validate(&response).is_ok()
}

fn main() -> std::process::ExitCode {
    sentinel::run_cli()
}
