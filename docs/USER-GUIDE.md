# feotest-examples — User Guide

Rust developers set a high bar. The ecosystem's emphasis on correctness —
ownership, type safety, `unsafe` as an explicit opt-in, `clippy` as a
cultural norm — reflects a community that treats quality as an engineering
discipline, not an afterthought. That commitment is one of the reasons Rust
codebases tend to be among the most reliable in production.

But there is a category of system where even Rust's rigour leaves a gap.
Services backed by large language models, ranking algorithms, classifiers, and
other stochastic components produce different outputs on each invocation. For
these systems, the correctness of a single execution is not a meaningful
concept — correctness is a statistical property of behaviour observed over many
executions under controlled conditions.

The Rust ecosystem has strong tools for deterministic testing (`cargo test`,
`cargo-nextest`), property-based testing (`proptest`, `quickcheck`), fuzzing,
and benchmarking (`criterion`). `feotest` does not compete with any of them.
It complements them by addressing the problem of non-determinism with an
engineering discipline that has not been present to date: statistically sound
verdicting of stochastic service behaviour, grounded in confidence bounds
rather than ad hoc retry counts or hard-coded tolerances.

This guide walks through the example application, explains how to run the
experiments and tests, and covers LLM configuration.

## The example application

The examples exercise two simulated services, each representing a different
flavour of non-determinism.

### Shopping basket (LLM-powered)

A user issues natural language instructions like _"Add 2 apples"_ or _"Clear
the basket"_. An LLM translates each instruction into a structured JSON action
that a shopping basket API can execute:

```json
{
  "actions": [
    {
      "context": "SHOP",
      "name": "add",
      "parameters": [
        {"name": "item", "value": "apples"},
        {"name": "quantity", "value": "2"}
      ]
    }
  ]
}
```

Valid actions for the `SHOP` context are `add`, `remove`, and `clear`. A
translation is considered successful when the LLM returns valid JSON that
deserialises into valid actions for the given context.

Because the LLM is inherently non-deterministic — it may hallucinate field
names, produce malformed JSON, or invent actions that do not exist — success
rates are probabilistic. This makes the shopping basket a natural fit for
feotest's **empirical approach**, where acceptable thresholds are derived from
measured baselines.

### Payment gateway (SLA-driven)

The payment gateway simulates an external service with a contractual SLA of 99.99%
availability. The mock gateway intentionally underperforms its SLA slightly
(~99.97% actual availability) so that feotest's statistical machinery has
realistic data to work with. Unlike the shopping basket, the thresholds are
known upfront from the contract — this is feotest's **normative approach**.

### Architecture

```
tests/              ← experiments and probabilistic tests
    │
    ▼
src/usecases/       ← feotest use case adapters
    │
    ▼
src/llm/            ← application code (no feotest dependency)
src/shopping/
src/payment/
```

The `llm`, `shopping`, and `payment` modules contain the application code. The
`usecases` module wraps these in feotest use case adapters. The integration
tests in `tests/` exercise the use cases through feotest experiments and
probabilistic tests.

## Prerequisites

- **Rust 1.85** or later (edition 2024)
- A local checkout of `feotest` in the sibling directory `../feotest`

No API keys are required for the default mock mode.

## Running experiments

Experiments gather empirical data about how the system behaves. feotest
provides several experiment types, and this project includes examples of each.

### Explore — compare configurations

Before committing to a model or temperature, explore how different
configurations perform:

```bash
cargo test --test shopping_basket_explore -- --nocapture
```

This runs a small number of samples per configuration and reports the results,
giving you a quick signal about which settings are worth measuring in depth.

### Measure — establish a baseline

Once you have chosen a configuration, run a measurement experiment to establish
a statistical baseline:

```bash
cargo test --test shopping_basket_measure -- --nocapture
```

This runs 1000 samples (by default), computes a Wilson score confidence
interval for the true success probability, and writes a spec file. Probabilistic
tests derive their pass/fail thresholds from this baseline.

## Running tests

Probabilistic tests verify that the system's observed behaviour meets
expectations. The shopping basket tests can use a baseline established by the
measure experiment. The payment gateway tests use inline thresholds from the
SLA.

```bash
# Run all tests
cargo test -- --nocapture

# Run a specific test file
cargo test --test shopping_basket_test -- --nocapture
cargo test --test payment_gateway_test -- --nocapture
```

Individual sample failures are expected — that is the nature of probabilistic
testing. feotest aggregates the results and applies statistical analysis to
determine the verdict.

## Understanding the test types

### Threshold-first

You specify the sample count and the minimum pass rate. feotest runs the
samples, computes a confidence interval for the true success probability, and
determines whether the lower bound exceeds your threshold.

```rust
ProbabilisticTestBuilder::new("shopping_basket")
    .approach(ThresholdApproach::ThresholdFirst {
        samples: 100,
        min_pass_rate: 0.80,
    })
    .intent(TestIntent::Verification)
    // ...
    .run();
```

This is the simplest approach and works well when you have a clear threshold —
either from an SLA (normative) or from a prior measurement (empirical).

### Sample-size-first with spec

You specify the sample count and confidence level. feotest loads a baseline
spec from a prior measurement and derives the threshold automatically. This
separates the "what is acceptable" question (answered by measurement) from the
"does the service still meet it" question (answered by the test).

```rust
// Step 1: measure to establish baseline
MeasureExperiment::new("shopping_basket")
    .sample_count(500)
    // ...
    .run();

// Step 2: test against the baseline
ProbabilisticTestBuilder::new("shopping_basket")
    .approach(ThresholdApproach::SampleSizeFirst {
        samples: 100,
        confidence: 0.95,
    })
    .spec_resolver(resolver)
    // ...
    .run();
```

### Smoke

A lightweight check with a deliberately small sample size. feotest suppresses
the statistical rigour warnings that would normally flag an undersized sample,
making this suitable for quick health checks in CI without the overhead of a
full verification run.

```rust
ProbabilisticTestBuilder::new("shopping_basket")
    .approach(ThresholdApproach::ThresholdFirst {
        samples: 20,
        min_pass_rate: 0.70,
    })
    .intent(TestIntent::Smoke)
    // ...
    .run();
```

## LLM configuration

### Mock mode (default)

By default, all LLM calls use a built-in mock that requires no API keys, no
network access, and costs nothing. The mock simulates realistic LLM behaviour
including:

- Temperature-sensitive reliability (lower temperature = more reliable
  structured output)
- Realistic failure modes (malformed JSON, hallucinated fields, invalid values)
- Approximate token counting

This means you can run every experiment and test in this project out of the box.

### Real mode

To call real LLM providers, set the mode and provide API keys:

```bash
export FEOTEST_LLM_MODE=real
export OPENAI_API_KEY=sk-...
export ANTHROPIC_API_KEY=sk-ant-...
```

In real mode, the provider is selected based on the model name:

| Model pattern           | Provider  |
|-------------------------|-----------|
| `gpt-*`, `o1-*`, `o3-*` | OpenAI    |
| `claude-*`              | Anthropic |

Providers are initialised lazily — if an experiment only uses OpenAI models, no
Anthropic API key is required (and vice versa).

**Real mode will incur costs on your provider accounts.** The measurement
experiment runs 1000 samples by default. Be aware of your provider's rate limits
and pricing before running large experiments.

### Configuration reference

| Setting           | Environment variable | Default |
|-------------------|----------------------|---------|
| LLM mode          | `FEOTEST_LLM_MODE`   | `mock`  |
| OpenAI API key    | `OPENAI_API_KEY`     | —       |
| Anthropic API key | `ANTHROPIC_API_KEY`  | —       |

## Typical workflow

A typical workflow for the shopping basket use case:

1. **Explore** — compare models and temperatures to find the best configuration:
   ```bash
   cargo test --test shopping_basket_explore -- --nocapture
   ```

2. **Measure** — establish a baseline with your chosen configuration:
   ```bash
   cargo test --test shopping_basket_measure -- --nocapture
   ```

3. **Test** — run probabilistic tests against the baseline:
   ```bash
   cargo test --test shopping_basket_test -- --nocapture
   ```

For the payment gateway, no baseline is needed — the SLA threshold is specified
directly in the test:

```bash
cargo test --test payment_gateway_test -- --nocapture
```

## Further reading

- [feotest README](https://github.com/javai-org/feotest) — full framework
  documentation and statistical foundations
