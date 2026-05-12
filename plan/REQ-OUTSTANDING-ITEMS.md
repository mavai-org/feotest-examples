# Outstanding Items

Features present in punit-examples that are not yet adopted in feotest-examples. Each item is categorised by whether it is blocked on feotest framework support or is an examples-only concern.

## Blocked on feotest framework

These items require new capabilities in the feotest core library before the examples can demonstrate them.

### Latency dimension testing

punit-examples demonstrates two-dimensional testing for the payment gateway: functional correctness *and* latency (percentile thresholds like p95 < 500ms, p99 < 1000ms). feotest does not yet support latency assertions.

**punit-examples coverage**: `PaymentGatewayReliability` tests with `@Latency(p95Ms=500, p99Ms=1000)`, `assertLatency()`, `assertAll()`.

### ~~Covariate-aware baseline selection~~ (Implemented)

Covariate-aware baseline selection is implemented in feotest. The `SpecResolver::resolve_with_covariates()` method selects the best-matching baseline from multiple candidates using a two-phase algorithm: hard-gate filtering on `Configuration` covariates, then soft-match scoring on remaining covariates. The `ProbabilisticTest` and `ProbabilisticTestBuilder` accept `.service_contract(&uc)` to provide covariate context. Demonstrated in `shopping_basket_covariate_test.rs` with two temperature-partitioned baselines.

### Budget controls (time and token)

punit-examples demonstrates time budgets, token budgets, and budget exhaustion behaviour (fail vs evaluate-partial). feotest has the `ExecutionConfig` types but they are not yet fully integrated with the probabilistic test builder.

**punit-examples coverage**: `ShoppingBasketBudgetTest` with `timeBudgetMs`, `tokenBudget`, `tokenCharge`, `TokenChargeRecorder`, `BudgetExhaustedBehavior.EVALUATE_PARTIAL`.

### Pacing / rate limiting

punit-examples demonstrates `@Pacing` constraints (requests per second, per minute, minimum delay between samples). feotest has `PacingConfig` types but they are not yet enforced end-to-end in probabilistic tests.

**punit-examples coverage**: `ShoppingBasketPacingTest` with various `@Pacing` configurations.

### Early termination

punit-examples demonstrates early termination when success is guaranteed or failure is inevitable. feotest's execution engine has a placeholder for this but does not yet implement the threshold-aware logic.

**punit-examples coverage**: `ShoppingBasketDiagnosticsTest.testShowingEarlyTermination`.

### Transparent statistics output

punit-examples demonstrates `transparentStats = true` for detailed statistical reasoning in verdict output. feotest has the `transparent_stats` flag on the builder but does not yet render the detailed output.

**punit-examples coverage**: `ShoppingBasketDiagnosticsTest` with three transparent-stats test methods.

### Confidence-first approach (end-to-end example)

The confidence-first approach (`confidence` + `minDetectableEffect` + `power` → computed sample size) is implemented in feotest and tested in the core library. However, there is no standalone example demonstrating it in a user-facing test.

**punit-examples coverage**: `ShoppingBasketThresholdApproachesTest.confidenceFirst`.

### Exception handling modes

punit-examples demonstrates `FAIL_SAMPLE` (count exception as failure, continue) vs `ABORT_TEST` (stop immediately). feotest's design decision is that panics are not caught — they are defects, not contract violations. This may mean no equivalent is needed, but the `maxExampleFailures` diagnostic feature is also absent.

**punit-examples coverage**: `ShoppingBasketExceptionTest` with `FAIL_SAMPLE`, `ABORT_TEST`, and `maxExampleFailures`.

---

## Examples-only concerns

These items can be implemented in feotest-examples without changes to the feotest core library.

### Optimize experiments

punit-examples includes two optimize experiments:
1. **Temperature optimization** (`ShoppingBasketOptimizeTemperature`): linear search from 1.0 down to 0.0 using `TemperatureMutator`.
2. **Prompt optimization** (`ShoppingBasketOptimizePrompt`): iterative prompt refinement using `ShoppingBasketPromptMutator` with mock and LLM-powered mutation strategies.

feotest supports `OptimizeExperiment` with `Scorer` and `FactorMutator` traits. The examples and their supporting infrastructure (scorers, mutators, mutation strategies) need to be ported.

**Status**: Deferred. The feotest `OptimizeExperiment` API is implemented and tested; only the example code is missing.

### Golden dataset / instance conformance

punit-examples uses a `fixtures/shopping-instructions.json` file containing instructions paired with expected JSON responses. The measure experiment checks each response against the expected value for instance conformance.

**Status**: The fixture file and conformance checking logic need to be ported.

### Sentinel reliability specifications

punit-examples provides `ShoppingBasketReliability` and `PaymentGatewayReliability` as `@Sentinel` classes that combine measure experiments and probabilistic tests in a single specification. feotest's `Reliability` trait and sentinel binary scaffold are described in the design document but not yet implemented.

**Status**: Blocked on feotest sentinel implementation.

### JUnit adapter pattern (dual consumption)

punit-examples demonstrates the one-line JUnit adapter pattern where a test class inherits from a sentinel specification. In Rust, this would be a `#[test]` function calling `feotest::run_reliability(...)`. The examples cannot demonstrate this until the sentinel infrastructure exists.

### Extended test classes with metadata

punit-examples shows JUnit `@DisplayName`, `@Tag`, and mixed standard/probabilistic tests in a single class. The Rust equivalent would use `#[test]` functions with descriptive names and `#[ignore]` where appropriate. This is a documentation concern rather than a capability gap.

### Verdict catalogue generation

punit-examples generates a markdown verdict catalogue from test execution (summary and verbose detail levels). This requires the reporting infrastructure to be more mature.

---

## Not applicable to Rust

These punit-examples features have no Rust equivalent because the underlying concern does not exist.

### `@RegisterExtension` / `ServiceContractProvider`

JUnit 5 extension mechanism for dependency injection. Rust uses direct construction and closures — no equivalent needed.

### Gradle task automation

punit-examples uses Gradle tasks (`explore`, `measure`, `optimize`) to drive experiment execution. In Rust, `cargo test --test <name>` serves the same purpose. No build system integration needed.

### Module separation (app / app-usecases / app-tests)

punit-examples splits into three Gradle subprojects to manage the JUnit dependency boundary. In Rust, `#[cfg(test)]` provides this boundary at zero cost (see feotest design document section 7). A single crate suffices.
