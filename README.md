# feotest-examples

Worked examples of probabilistic testing with the
[feotest](https://github.com/javai-org/feotest) framework.

## Why this project exists

Rust has excellent tools for deterministic testing, property-based testing, and
benchmarking. What it does not yet have is a disciplined approach to testing
services whose behaviour is inherently **non-deterministic** — LLM-backed
endpoints, ranking systems, classifiers, or any service where the same input
may produce different outputs on each invocation.

`feotest` fills that gap by treating repeated service calls as Bernoulli trials,
applying statistical inference to determine whether observed behaviour meets a
specified quality threshold, and producing verdicts grounded in confidence
bounds rather than ad hoc retry logic or hard-coded tolerances.

This project demonstrates that workflow end to end, using two example services
that represent distinct flavours of non-determinism.

## The example services

### Shopping basket (LLM-powered)

A user issues natural language instructions like _"Add 2 apples"_ or _"Clear
the basket"_. An LLM translates each instruction into structured JSON:

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

Because the LLM is inherently non-deterministic — it may hallucinate field
names, produce malformed JSON, or invent actions that do not exist — success
rates are probabilistic. This makes the shopping basket a natural fit for
feotest's **empirical approach**, where acceptable thresholds are derived from
measured baselines.

### Payment gateway (SLA-driven)

The payment gateway simulates an external service with a contractual SLA of 99.99%
availability. The mock gateway achieves ~99.97% — above the SLA, but with
observable failures in large sample runs. Unlike the shopping basket, the
threshold is known upfront from the contract — this is feotest's **normative
approach**.

## Prerequisites

- **Rust 1.85** or later (edition 2024)
- A local checkout of `feotest` in the sibling directory `../feotest`

## Running

Everything runs out of the box with no API keys — the built-in mock LLM
produces realistic behaviour including temperature-dependent failure rates.

```bash
# Run all tests
cargo test -- --nocapture

# Run a specific experiment
cargo test --test shopping_basket_measure -- --nocapture

# Run with a real LLM provider (incurs API costs)
FEOTEST_LLM_MODE=real OPENAI_API_KEY=sk-... cargo test -- --nocapture
```

## Typical workflow

### 1. Explore — compare configurations

Before committing to a model or temperature, explore how different
configurations perform:

```bash
cargo test --test shopping_basket_explore -- --nocapture
```

### 2. Measure — establish a baseline

Once you have chosen a configuration, run a measurement experiment to establish
a statistical baseline:

```bash
cargo test --test shopping_basket_measure -- --nocapture
```

This runs a large number of samples and writes a spec file. Probabilistic tests
derive their pass/fail thresholds from this baseline.

### 3. Test — verify against the baseline

Run probabilistic tests that compare current behaviour against the established
baseline:

```bash
cargo test --test shopping_basket_test -- --nocapture
```

For the payment gateway, no baseline is needed — the SLA threshold is specified
directly:

```bash
cargo test --test payment_gateway_test -- --nocapture
```

Individual sample failures are expected — that is the nature of probabilistic
testing. feotest aggregates the results and applies statistical analysis to
determine the verdict.

## LLM configuration

### Mock mode (default)

All LLM calls use a built-in mock that requires no API keys, no network access,
and costs nothing. The mock simulates realistic LLM behaviour including
temperature-sensitive reliability and realistic failure modes (malformed JSON,
hallucinated fields, invalid values).

### Real mode

To call real LLM providers, set the mode and provide API keys:

```bash
export FEOTEST_LLM_MODE=real
export OPENAI_API_KEY=sk-...
export ANTHROPIC_API_KEY=sk-ant-...
```

The routing logic selects a provider based on model name:

| Model pattern           | Provider  |
|-------------------------|-----------|
| `gpt-*`, `o1-*`, `o3-*` | OpenAI    |
| `claude-*`              | Anthropic |

**Real mode incurs costs on your provider accounts.** Be aware of rate limits
and pricing before running large experiments.

## Project structure

```
src/
├── lib.rs              # Crate root
├── llm/                # LLM infrastructure (trait, mock, real providers)
├── shopping/           # Shopping domain types and validator
├── payment/            # Payment domain and mock gateway
└── usecases/           # Use case implementations with service contracts
tests/
├── shopping_basket_measure.rs   # Measure experiment (baseline)
├── shopping_basket_explore.rs   # Explore experiment (model comparison)
├── shopping_basket_test.rs      # Probabilistic tests
└── payment_gateway_test.rs      # SLA verification tests
```

## Licence

Apache-2.0
