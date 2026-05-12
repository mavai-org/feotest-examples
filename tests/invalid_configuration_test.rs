//! Invalid configuration examples for feotest.
//!
//! Each test is intentionally misconfigured. The framework must reject it
//! before executing any samples. These examples serve as:
//!
//! 1. Documentation of common misconfigurations and their fixes.
//! 2. Regression tests that feotest rejects invalid inputs promptly.
//! 3. Cross-language parity evidence (see punit-examples'
//!    `InvalidProbabilisticTestExamplesTest.java`).
//!
//! Run with:
//! ```text
//! cargo test --test invalid_configuration_test
//! ```

use std::time::Duration;

use feotest::model::{TestIntent, TrialOutcome};
use feotest::ptest::ProbabilisticTest;
use feotest::ptest::ProbabilisticTestBuilder;
use feotest::ptest::builder::ThresholdApproach;

fn always_succeeds(_input: &str) -> TrialOutcome {
    TrialOutcome::success(Duration::from_millis(1))
}

// ─── Category 1: No approach specified ──────────────────────────────

/// Error: No operational approach is selected.
///
/// What it says: "Run 100 samples."
/// Why invalid: samples alone is a loop count, not a statistical test.
///   The framework needs to know what to derive — a threshold from
///   baseline data (Sample-Size-First) or implied confidence from an
///   explicit threshold (Threshold-First).
/// Fix: add `min_pass_rate` (Threshold-First) or `confidence`
///   (Sample-Size-First).
#[test]
#[should_panic(expected = "threshold approach must be set")]
fn no_approach_samples_only() {
    let inputs = vec!["input".to_string()];
    ProbabilisticTest::new("no-approach", &inputs, always_succeeds)
        // No .approach() call
        .run();
}

/// Error: No parameters specified at all via the simplified API.
///
/// What it says: "Test this service contract."
/// Why invalid: no samples, no threshold, no confidence — nothing to
///   compute or verify.
/// Fix: provide at least two of the parameter triangle:
///   `.samples(n).threshold(rate)`, `.samples(n).confidence(level)`,
///   or `.confidence(level).min_detectable_effect(mde).power(p)`.
#[test]
#[should_panic(expected = "UNDER-SPECIFIED")]
fn no_parameters_at_all() {
    let inputs = vec!["input".to_string()];
    ProbabilisticTest::new("no-params", &inputs, always_succeeds).run();
}

/// Error: Only samples specified via the simplified API.
///
/// What it says: "Run 100 samples."
/// Why invalid: same as builder variant — samples alone is a loop count.
/// Fix: add `.threshold(rate)` or `.confidence(level)`.
#[test]
#[should_panic(expected = "UNDER-SPECIFIED")]
fn under_specified_samples_only() {
    let inputs = vec!["input".to_string()];
    ProbabilisticTest::new("under-spec", &inputs, always_succeeds)
        .samples(100)
        .run();
}

// ─── Category 2: Incomplete Confidence-First ────────────────────────

/// Error: Partial Confidence-First — missing `min_detectable_effect`.
///
/// What it says: "Use 99% confidence and 80% power."
/// Why invalid: Confidence-First requires all three parameters:
///   confidence + power + `min_detectable_effect`. Without the effect
///   size, the framework cannot compute the required sample count.
/// Fix: add `.min_detectable_effect(0.05)`, or switch to
///   `.samples(n).threshold(rate)`.
#[test]
#[should_panic(expected = "INCOMPLETE")]
fn incomplete_confidence_first_missing_mde() {
    let inputs = vec!["input".to_string()];
    ProbabilisticTest::new("incomplete", &inputs, always_succeeds)
        .confidence(0.99)
        .power(0.80)
        // Missing: .min_detectable_effect(0.05)
        .run();
}

/// Error: Partial Confidence-First — missing `power`.
///
/// What it says: "Use 99% confidence to detect a 5% degradation."
/// Why invalid: without power, the framework cannot bound the
///   false-negative risk.
/// Fix: add `.power(0.80)`.
#[test]
#[should_panic(expected = "INCOMPLETE")]
fn incomplete_confidence_first_missing_power() {
    let inputs = vec!["input".to_string()];
    ProbabilisticTest::new("incomplete", &inputs, always_succeeds)
        .confidence(0.99)
        .min_detectable_effect(0.05)
        // Missing: .power(0.80)
        .run();
}

/// Error: Partial Confidence-First — missing `confidence`.
///
/// What it says: "Detect a 5% drop with 80% power."
/// Why invalid: without confidence, the false-positive risk is
///   undefined.
/// Fix: add `.confidence(0.95)`.
#[test]
#[should_panic(expected = "INCOMPLETE")]
fn incomplete_confidence_first_missing_confidence() {
    let inputs = vec!["input".to_string()];
    ProbabilisticTest::new("incomplete", &inputs, always_succeeds)
        .min_detectable_effect(0.05)
        .power(0.80)
        // Missing: .confidence(0.95)
        .run();
}

// ─── Category 3: Infeasible verification ────────────────────────────

/// Error: VERIFICATION intent with infeasibly small sample size.
///
/// What it says: "Verify 95% reliability with 5 samples."
/// Why invalid: the minimum feasible N for a 0.95 target at α=0.05 is
///   ~52. Five samples cannot produce a Wilson lower bound ≥ 0.95
///   even with a perfect observation (5/5 successes).
/// Fix: increase samples to at least 52, or use Smoke intent.
#[test]
#[should_panic(expected = "Infeasible")]
fn infeasible_verification() {
    let inputs = vec!["input".to_string()];
    ProbabilisticTest::new("infeasible", &inputs, always_succeeds)
        .approach(ThresholdApproach::ThresholdFirst {
            samples: 5,
            min_pass_rate: 0.95,
        })
        .intent(TestIntent::Verification)
        .run();
}

/// Error: Infeasible verification via the simplified API.
///
/// Same misconfiguration, different API surface. The framework must
/// reject this regardless of which builder is used.
#[test]
#[should_panic(expected = "Infeasible")]
fn infeasible_verification_simplified_api() {
    let inputs = vec!["input".to_string()];
    ProbabilisticTest::new("infeasible", &inputs, always_succeeds)
        .samples(5)
        .threshold(0.95)
        .intent(TestIntent::Verification)
        .run();
}

// ─── Category 4: Over-specification ─────────────────────────────────

/// Error: Over-specified — threshold + confidence both pinned.
///
/// What it says: "Use 10 samples at 99.99% pass rate with 99% confidence."
/// Why invalid: pins two of the three degrees of freedom by different
///   mechanisms, leaving the framework nothing to derive and no way to
///   verify internal consistency. Sample size, confidence, and threshold
///   are mathematically linked — you choose two, statistics derives the
///   third.
/// Fix: use Threshold-First (remove confidence) or Sample-Size-First
///   (remove threshold).
#[test]
#[should_panic(expected = "OVER-SPECIFIED")]
fn over_specified_threshold_plus_confidence() {
    let inputs = vec!["input".to_string()];
    ProbabilisticTest::new("over-spec", &inputs, always_succeeds)
        .confidence(0.99)
        .samples(10)
        .threshold(0.9999)
        .run();
}

// ─── Category 5: Approach requires baseline ─────────────────────────

/// Error: Sample-Size-First without a baseline.
///
/// What it says: "Use 100 samples at 95% confidence."
/// Why invalid: the framework must derive the threshold from baseline
///   data (via Wilson score interval). With no baseline, there is no
///   basis for derivation.
/// Fix: provide a baseline spec (via `.baseline()` or `.baseline_dir()`),
///   or switch to Threshold-First with an explicit pass rate.
#[test]
#[should_panic(expected = "REQUIRES_BASELINE")]
fn sample_size_first_without_baseline_builder() {
    let inputs = vec!["input".to_string()];
    ProbabilisticTest::new("no-baseline", &inputs, always_succeeds)
        .approach(ThresholdApproach::SampleSizeFirst {
            samples: 100,
            confidence: 0.95,
        })
        // No .spec_resolver() or .baseline_spec()
        .run();
}

/// Error: Confidence-First without a baseline.
///
/// What it says: "Use 95% confidence, detect 5% drops with 80% power."
/// Why invalid: power analysis requires a baseline success rate (p₀)
///   to compute the required sample size. Without it, the effect size
///   has no reference point.
/// Fix: provide a baseline spec, or add an explicit threshold as a
///   normative baseline rate.
#[test]
#[should_panic(expected = "REQUIRES_BASELINE_RATE")]
fn confidence_first_without_baseline() {
    let inputs = vec!["input".to_string()];
    ProbabilisticTest::new("no-baseline", &inputs, always_succeeds)
        .approach(ThresholdApproach::ConfidenceFirst {
            confidence: 0.95,
            min_detectable_effect: 0.05,
            power: 0.80,
        })
        // No .spec_resolver() or .baseline_spec()
        .run();
}

// ─── Category 6: Range violations ───────────────────────────────────

/// Error: `min_pass_rate` = 1.5 is outside [0, 1].
///
/// Why invalid: a pass rate is a probability — it must be between
///   0 and 1 inclusive.
/// Fix: use a value in [0, 1] (e.g., 0.95).
#[test]
#[should_panic(expected = "min_pass_rate must be in [0, 1]")]
fn min_pass_rate_above_one() {
    let inputs = vec!["input".to_string()];
    ProbabilisticTest::new("bad-threshold", &inputs, always_succeeds)
        .approach(ThresholdApproach::ThresholdFirst {
            samples: 10,
            min_pass_rate: 1.5,
        })
        .run();
}

/// Error: samples = 0 with Threshold-First.
///
/// Why invalid: zero samples means zero observations — no evidence can
///   be gathered. This is a programming error, not a runtime condition.
/// Fix: use a positive sample count (e.g., 100).
#[test]
#[should_panic(expected = "samples must be greater than 0")]
fn zero_samples_threshold_first() {
    let inputs = vec!["input".to_string()];
    ProbabilisticTest::new("zero-samples", &inputs, always_succeeds)
        .approach(ThresholdApproach::ThresholdFirst {
            samples: 0,
            min_pass_rate: 0.90,
        })
        .run();
}

/// Error: samples = 0 with Sample-Size-First.
///
/// Why invalid: same as above — zero observations, no evidence.
/// Fix: use a positive sample count.
#[test]
#[should_panic(expected = "samples must be greater than 0")]
fn zero_samples_sample_size_first() {
    let inputs = vec!["input".to_string()];
    ProbabilisticTest::new("zero-samples", &inputs, always_succeeds)
        .approach(ThresholdApproach::SampleSizeFirst {
            samples: 0,
            confidence: 0.95,
        })
        .run();
}

/// Error: inputs is empty.
///
/// Why invalid: a probabilistic test must have at least one input to
///   cycle through during trials.
/// Fix: provide at least one input string.
#[test]
#[should_panic(expected = "inputs must not be empty")]
fn empty_inputs() {
    let inputs: Vec<String> = vec![];
    ProbabilisticTest::new("empty", &inputs, always_succeeds)
        .approach(ThresholdApproach::ThresholdFirst {
            samples: 50,
            min_pass_rate: 0.90,
        })
        .run();
}
