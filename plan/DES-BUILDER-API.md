# DES: Builder API Design

## Context

feotest offers two ways to declare experiments and probabilistic tests:

1. **Macros** (`#[probabilistic_test]`, `#[measure_experiment]`) — compact and
   declarative, ideal when inputs are static and configuration is simple.
2. **Builder API** (`ProbabilisticTestBuilder`, `MeasureExperiment`,
   `ExploreExperiment`) — required when inputs are dynamic, shared across
   tests, or generated at runtime.

The macros expand to builder calls internally. The DX gap is that the builder
API is significantly more verbose than the macro equivalent, with boilerplate
that obscures intent. This design aims to close that gap.

## Design goals

1. **Zero unnecessary boilerplate** — if the macro doesn't need it, the builder
   shouldn't need it either.
2. **Same defaults** — the builder and macro should share identical defaults
   (spec directory, intent, threshold origin).
3. **Composable** — advanced configuration (budgets, pacing, latency) should
   layer on cleanly without polluting the simple case.
4. **Discoverable** — a developer reading the builder API should understand
   which parameters are required and which are optional without consulting
   documentation.
5. **No assumptions about service contract construction** — the framework must not
   assume service contracts can be instantiated with a no-arg constructor. In real
   applications, service contracts may require dependency injection, configuration
   from external sources, or complex setup. The trial closure captures a
   service contract instance that the developer has already constructed — feotest
   never creates service contracts itself.

## Current state

### Measure experiment (current)

```rust
let inputs = standard_instructions();
let mut service_contract = ShoppingBasketServiceContract::new();

let result = MeasureExperiment::new(
    "ShoppingBasketServiceContract",
    1000,
    &inputs,
    |instruction| service_contract.translate_instruction(instruction),
)
.with_experiment_id("baseline-v1")
.with_spec_resolver(SpecResolver::with_dir(dir.path()))
.run();
```

Issues:
- `with_spec_resolver(SpecResolver::with_dir(...))` is boilerplate; the macro
  defaults to `tests/baselines`.
- `with_` prefix on every method is visual noise.
- No way to discover that `.run()` writes a spec unless you know about
  `with_spec_resolver`.

### Probabilistic test — threshold-first (current)

```rust
let inputs = standard_instructions();
let mut service_contract = ShoppingBasketServiceContract::new();

let result = ProbabilisticTestBuilder::new(
    "ShoppingBasketServiceContract",
    &inputs,
    |instruction| service_contract.translate_instruction(instruction),
)
.approach(ThresholdApproach::ThresholdFirst {
    samples: 100,
    min_pass_rate: 0.80,
})
.intent(TestIntent::Verification)
.threshold_origin(ThresholdOrigin::Empirical)
.run();

assert_eq!(result.verdict_record().verdict(), Verdict::Pass);
```

Issues:
- `ThresholdApproach::ThresholdFirst { samples: 100, min_pass_rate: 0.80 }`
  is structurally heavy for the simplest approach.
- The developer must import `ThresholdApproach`, `TestIntent`,
  `ThresholdOrigin` — three enum types for a straightforward test.
- The trial closure `|instruction| service_contract.translate_instruction(instruction)`
  is often just forwarding a method call.

### Probabilistic test — spec-driven (current)

```rust
let result = ProbabilisticTestBuilder::new(
    "ShoppingBasketServiceContract",
    &inputs,
    |instruction| service_contract.translate_instruction(instruction),
)
.approach(ThresholdApproach::SampleSizeFirst {
    samples: 100,
    confidence: 0.95,
})
.spec_resolver(SpecResolver::with_dir(dir.path()))
.threshold_origin(ThresholdOrigin::Empirical)
.run();
```

Issues:
- `spec_resolver(SpecResolver::with_dir(...))` should be unnecessary; the
  framework knows the default location.
- The developer must explicitly choose `SampleSizeFirst` and pair it with a
  resolver — the framework could infer this when a spec is available.

### Explore experiment (current)

```rust
let service_contract = Arc::new(Mutex::new(ShoppingBasketServiceContract::new()));
let uc_a = Arc::clone(&service_contract);
let uc_b = Arc::clone(&service_contract);
let uc_trial = Arc::clone(&service_contract);

let result = ExploreExperiment::new(
    "ShoppingBasketServiceContract",
    20,
    &inputs,
    move |instruction| {
        uc_trial.lock().unwrap().translate_instruction(instruction)
    },
)
.config("gpt-4o-mini (temp=0.1)", move || {
    let mut uc = uc_a.lock().unwrap();
    uc.set_model("gpt-4o-mini");
    uc.set_temperature(0.1);
})
.config("gpt-4o-mini (temp=0.5)", move || {
    let mut uc = uc_b.lock().unwrap();
    uc.set_model("gpt-4o-mini");
    uc.set_temperature(0.5);
})
.with_experiment_id("model-comparison")
.run();
```

Issues:
- The `Arc<Mutex<>>` boilerplate with multiple clones is the developer's
  problem to solve. This is the most hostile API surface in the project.
- `with_experiment_id` vs `config` — inconsistent prefix convention.

## Proposed design

### Principle: drop `with_` prefixes

Rust builder convention has moved towards bare method names. The `with_` prefix
was common in early Rust but is now considered noise. Libraries like `reqwest`,
`clap`, `tracing`, and `tokio` all use bare names.

Current: `.with_experiment_id("baseline-v1").with_spec_resolver(resolver)`
Proposed: `.experiment_id("baseline-v1").spec_dir("path")`

This applies uniformly to all builders.

### Principle: shared defaults with the macros

| Setting | Default | Override method |
|---------|---------|-----------------|
| `spec_dir` | `tests/baselines` | `.spec_dir("custom/path")` |
| `intent` | `Verification` | `.intent(TestIntent::Smoke)` |
| `threshold_origin` | `Unspecified` | `.threshold_origin(ThresholdOrigin::Sla)` |
| `transparent_stats` | `false` | `.transparent_stats(true)` |

The builder should write specs to `tests/baselines/` by default, just as the
macro does. The developer opts out, not in.

### Service contract construction

The builder API never instantiates service contracts. The developer constructs the
service contract — however complex that may be — and the trial closure captures it.

In the simplest case, this is a one-liner:

```rust
let service_contract = ShoppingBasketServiceContract::new();
```

In a real application, construction may involve dependency injection,
configuration, or service wiring:

```rust
let config = AppConfig::from_env();
let llm_client = LlmClient::new(&config.api_key, config.model);
let service_contract = ShoppingBasketServiceContract::new(llm_client, config.temperature);
```

The trial closure captures the service contract by reference (or by move, depending
on ownership needs). feotest is agnostic to how the service contract was created.

### Measure experiment (proposed)

**Minimal (common case):**

```rust
let inputs = standard_instructions();
let service_contract = ShoppingBasketServiceContract::new();

MeasureExperiment::new("ShoppingBasketServiceContract", 1000, &inputs, |input| {
    service_contract.translate_instruction(input)
})
.run();
```

This writes to `tests/baselines/ShoppingBasketServiceContract.yaml` automatically.

**With options:**

```rust
let inputs = standard_instructions();
let service_contract = ShoppingBasketServiceContract::new();

MeasureExperiment::new("ShoppingBasketServiceContract", 1000, &inputs, |input| {
    service_contract.translate_instruction(input)
})
.experiment_id("baseline-v1")
.time_budget(Duration::from_secs(300))
.run();
```

Changes from current API:
- Default spec writing (no resolver setup needed).
- Drop `with_` prefix from all builder methods.
- `with_config(ExecutionConfig)` replaced by individual methods that build
  the config internally (`.time_budget()`, `.token_budget()`, `.pacing()`).
  The `ExecutionConfig` type remains as an internal implementation detail.
- Warmup removed from the builder (it is a service contract property).

### The parameter triangle

A probabilistic test has three interdependent statistical parameters: **sample
size**, **threshold**, and **confidence**. The developer fixes two; feotest
computes the third. Attempting to fix all three is statistically nonsensical
and is rejected at runtime.

This is a single API with one constructor. The approach is detected from which
combination of parameters the developer sets — exactly as the macro already
does.

Where a threshold is needed but not explicitly set, feotest resolves it from
a committed baseline spec. By default it looks for
`tests/baselines/{use_case_id}.yaml`. The developer can override this with
`.baseline("path/to/spec.yaml")`.

### Probabilistic test (proposed)

**Fix samples + threshold → framework computes confidence:**

```rust
let inputs = standard_instructions();
let service_contract = ShoppingBasketServiceContract::new();

ProbabilisticTest::new("ShoppingBasketServiceContract", &inputs, |input| {
    service_contract.translate_instruction(input)
})
.samples(100)
.threshold(0.80)
.run();
```

**Fix samples + confidence → framework derives threshold from baseline:**

```rust
let inputs = standard_instructions();
let service_contract = ShoppingBasketServiceContract::new();

ProbabilisticTest::new("ShoppingBasketServiceContract", &inputs, |input| {
    service_contract.translate_instruction(input)
})
.samples(200)
.confidence(0.95)
.run();
```

The baseline is resolved automatically from the service contract ID. To override:

```rust
.confidence(0.95)
.baseline("tests/baselines/ShoppingBasketServiceContract.yaml")
```

**Fix confidence + MDE + power → framework computes sample size:**

```rust
let inputs = standard_instructions();
let service_contract = ShoppingBasketServiceContract::new();

ProbabilisticTest::new("ShoppingBasketServiceContract", &inputs, |input| {
    service_contract.translate_instruction(input)
})
.confidence(0.95)
.min_detectable_effect(0.05)
.power(0.80)
.run();
```

The framework computes the required sample size and resolves the baseline
automatically for the reference rate.

**With provenance metadata:**

```rust
let inputs = vec!["tok_visa_4242:1999".to_string()];
let service_contract = PaymentGatewayServiceContract::new();

ProbabilisticTest::new("PaymentGatewayServiceContract", &inputs, |_input| {
    service_contract.charge_card("tok_visa_4242", 1999)
})
.samples(200)
.threshold(0.99)
.intent(TestIntent::Verification)
.threshold_origin(ThresholdOrigin::Sla)
.contract_ref("Payment Provider SLA v2.3, Section 4.1")
.run();
```

**Validation rules (enforced at runtime):**

| Parameters set | Result |
|---|---|
| `samples` + `threshold` | Valid: framework computes confidence |
| `samples` + `confidence` | Valid: framework derives threshold from baseline |
| `confidence` + `min_detectable_effect` + `power` | Valid: framework computes samples, derives threshold from baseline |
| `samples` + `threshold` + `confidence` | **Error: over-specified** |
| `threshold` only (no samples or confidence) | **Error: under-specified** |
| `confidence` only (no samples, MDE, or power) | **Error: under-specified** |
| `samples` + `confidence` without baseline | **Error: no baseline to derive threshold** |

Changes from current API:
- Single constructor `ProbabilisticTest::new()` replaces
  `ProbabilisticTestBuilder::new()` + `approach(ThresholdApproach::...)`.
- No `ThresholdApproach` enum in the public API.
- Approach detected from parameter combination, not declared upfront.
- Baseline auto-resolution from service contract ID and default directory.
- `.threshold()` replaces `.min_pass_rate()` — aligns with punit terminology.
- `.baseline()` replaces `.spec()` / `.spec_resolver()` — clearer intent.

### Immutable service contract principle

A unifying design principle across all experiment and test types:

> **The service contract is immutable during sampling.**

Each configuration gets its own service contract instance. The framework never mutates
a service contract between or during trials. This is not merely a DX convenience — it
is a direct expression of the fixed experimental condition that the Bernoulli
model requires. Mutating the service contract during sampling changes the condition,
which undermines the i.i.d. assumption and invalidates the statistics.

The principle is realised differently depending on who decides the
configurations:

| Type | Who decides configs | How service contract is provided |
|------|---------------------|------------------------|
| Measure | N/A (single config) | Developer passes one instance |
| Explore | Developer enumerates upfront | Developer passes pre-built instances |
| Optimize | Framework searches dynamically | Developer provides a factory |
| Probabilistic test | N/A (single config) | Developer passes one instance via closure |

Every row: immutable service contract during sampling. No shared mutable state
anywhere. No `Arc`, no `Mutex`, no `Send` bound.

### Explore experiment (proposed)

Explore configurations are **fully known upfront**. The developer constructs
each configuration before any execution happens, then hands them to the
framework as immutable references.

The trial function is declared once on the constructor. Each `.config()` adds
a name and a reference to a pre-built service contract. No closures in `.config()`,
no mutation, no shared state.

```rust
let inputs = standard_instructions();

let uc_low = ShoppingBasketServiceContract::new()
    .model("gpt-4o-mini")
    .temperature(0.1);

let uc_high = ShoppingBasketServiceContract::new()
    .model("gpt-4o-mini")
    .temperature(0.5);

ExploreExperiment::new("ShoppingBasketServiceContract", 20, &inputs, |uc, input| {
    uc.translate_instruction(input)
})
.config(&uc_low)
.config(&uc_high)
.experiment_id("model-comparison")
.run();
```

The framework's job is pure iteration: for each config, run the shared trial
function with that config's service contract reference and the cycling inputs.

### Optimize experiment (proposed)

Optimize configurations are **determined dynamically** by the framework's
search algorithm. The developer cannot enumerate them upfront, but the same
immutable-during-sampling principle applies.

The developer provides a **factory closure** that constructs a fresh service contract
for each point in the parameter space. The framework calls the factory, runs
all trials for that configuration against an immutable borrow of the returned
instance, then drops it and moves to the next point.

Service contract construction is typically microseconds (wiring a client, setting a
parameter). Trial execution involves network calls, LLM invocations, or
substantial computation — orders of magnitude more expensive. The cost of
constructing a fresh instance per configuration is negligible.

```rust
let inputs = standard_instructions();

OptimizeExperiment::new("ShoppingBasketServiceContract", &inputs, |uc, input| {
    uc.translate_instruction(input)
})
.parameter("temperature", 0.1..=0.9, 0.1)
.factory(|params| {
    ShoppingBasketServiceContract::new()
        .temperature(params.get("temperature"))
})
.experiment_id("temperature-sweep")
.run();
```

The factory returns an owned, immutable service contract. The framework borrows it
immutably for trials, then drops it. Same statistical guarantee as explore:
the experimental condition is fixed during sampling.

The service contract's `Display` implementation labels each configuration point
automatically — the framework never asks the developer for labels. If the
factory produces a service contract that displays as `"gpt-4o-mini (temperature=0.3)"`,
that string appears in the results and reports.

### Assertion convenience

Currently the developer must write:

```rust
assert_eq!(result.verdict_record().verdict(), Verdict::Pass);
```

Proposed: `.run()` always asserts. It captures the verdict (for reporting)
and then panics if the verdict is Fail. On Pass, it returns the
`VerdictRecord` for optional inspection:

```rust
// Asserts automatically — panics on Fail, returns VerdictRecord on Pass
let record = ProbabilisticTest::new(...)
    .samples(100)
    .threshold(0.80)
    .run();

// Optional: inspect after a passing test
println!("Pass rate: {}", record.functional().pass_rate());
```

## Method inventory

### `MeasureExperiment`

| Method | Required | Default | Purpose |
|--------|----------|---------|---------|
| `::new(service_contract, samples, inputs, trial_fn)` | yes | — | Constructor |
| `.experiment_id(id)` | no | none | Trace-back identifier |
| `.baseline_dir(path)` | no | `tests/baselines` | Override output directory |
| `.time_budget(duration)` | no | none | Wall-clock cap |
| `.token_budget(n)` | no | none | Token cap |
| `.pacing(config)` | no | none | Rate limiting |
| `.run()` | — | — | Execute and write spec |

### `ProbabilisticTest`

**Constructor:**

| Method | Purpose |
|--------|---------|
| `::new(service_contract, inputs, trial_fn)` | Single constructor for all approaches |

**Parameter triangle** (set exactly two of the first three, or confidence + MDE + power):

| Method | Purpose |
|--------|---------|
| `.samples(n)` | Fix the sample count |
| `.threshold(rate)` | Fix the minimum pass rate |
| `.confidence(level)` | Fix the confidence level |
| `.min_detectable_effect(mde)` | Minimum detectable effect (with confidence + power) |
| `.power(p)` | Statistical power (with confidence + MDE) |

**Baseline resolution** (needed when threshold is not explicitly set):

| Method | Default | Purpose |
|--------|---------|---------|
| `.baseline(path)` | auto-resolved from service contract ID | Explicit baseline spec file |
| `.baseline_dir(path)` | `tests/baselines` | Override baseline directory |

**Optional configuration:**

| Method | Default | Purpose |
|--------|---------|---------|
| `.intent(intent)` | `Verification` | Test intent |
| `.threshold_origin(origin)` | `Unspecified` | Threshold provenance |
| `.contract_ref(ref)` | none | Human-readable contract reference |
| `.transparent_stats(bool)` | `false` | Include detailed statistics |
| `.time_budget(duration)` | none | Wall-clock cap |
| `.token_budget(n)` | none | Token cap |
| `.pacing(config)` | none | Rate limiting |

**Terminal methods:**

| Method | Purpose |
|--------|---------|
| `.run()` | Execute and assert verdict |

### `ExploreExperiment`

Requires `T: Display`. The service contract's `Display` implementation provides
the configuration label automatically.

| Method | Required | Default | Purpose |
|--------|----------|---------|---------|
| `::new(service_contract, samples, inputs, trial_fn)` | yes | — | Constructor with shared trial function |
| `.config(&service_contract_instance)` | yes (1+) | — | Add a pre-built configuration (label from `Display`) |
| `.config_named(name, &service_contract_instance)` | — | — | Add a configuration with an explicit label |
| `.experiment_id(id)` | no | none | Trace-back identifier |
| `.time_budget(duration)` | no | none | Wall-clock cap |
| `.run()` | — | — | Execute all configurations |


## Resolved questions

1. **`ProbabilisticTest` vs `ProbabilisticTestBuilder`** — `ProbabilisticTest`
   is preferred. The builder pattern is implicit.

2. **Explore experiment shared state** — resolved. Each `.config()` takes a
   pre-built, immutable service contract reference. No shared mutable state, no
   `Arc<Mutex<>>`, no `Send` bound. This is both better DX and statistically
   sounder — the immutability of each configuration during its trial run is
   a direct expression of the fixed experimental condition that the Bernoulli
   model requires. Optimize experiments (which have dynamic parameter spaces)
   will use a factory-based approach instead.

3. **Auto-assertion in `.run()`** — `.run()` always asserts. This is the
   raison d'être of the probabilistic test. There is no `.execute()` variant.
   If a developer needs to inspect the verdict without asserting, they use
   the builder API directly (which is an advanced, non-default path).

4. **Warmup** — warmup is a property of the **service contract**, not of a test or
   experiment. It belongs on the service contract because experimentally derived
   baselines and probabilistic tests using that baseline must match execution
   conditions. Changing warmup between a measure and a subsequent test
   undermines the Bernoulli (and especially latency) statistics. Warmup is
   therefore removed from `ProbabilisticTest` and `MeasureExperiment`
   builders.

   Time and token budgets remain on the builder. A **global budget** mechanism
   will be introduced that overrides local definitions when present, allowing
   the developer to maintain control over budgets across all experiments and
   tests (e.g., via environment variable or a configuration file).

5. **Parameter triangle validation** — runtime, with clear panic messages
   that name the conflicting parameters. Keeping the type signature clean and
   simple is more important than compile-time enforcement here.
