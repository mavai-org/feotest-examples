# feotest-examples

Example experiments and probabilistic tests demonstrating the [feotest](https://github.com/javai-org/feotest) framework.

## Use cases

### Shopping basket

An LLM-backed service that translates natural language instructions ("Add 2 apples") into structured JSON shopping actions. The service is inherently stochastic: the same instruction may produce different (and sometimes invalid) responses across invocations.

Demonstrates:
- **Measure experiment**: establishing an empirical baseline (1000 samples)
- **Explore experiment**: comparing model configurations
- **Probabilistic tests**: threshold-first, spec-driven, and smoke intents

### Payment gateway

A payment processing service with an SLA-driven reliability target (99% success). The mock gateway achieves ~99.97% — above the SLA, but with observable failures in large sample runs.

Demonstrates:
- **SLA verification**: threshold-first testing with normative threshold origin
- **Smoke testing**: lightweight checks with fewer samples
- **Measure experiment**: establishing a payment baseline

## Running

```bash
# Run all tests (mock LLM, no API keys needed)
cargo test -- --nocapture

# Run a specific experiment
cargo test --test shopping_basket_measure -- --nocapture

# Run with real LLM providers (requires API keys)
FEOTEST_LLM_MODE=real OPENAI_API_KEY=sk-... cargo test -- --nocapture
```

## LLM mode

The `FEOTEST_LLM_MODE` environment variable controls whether the shopping basket uses a mock or real LLM:

| Value | Behaviour |
|-------|-----------|
| `mock` (default) | Built-in mock with temperature-dependent reliability |
| `real` | Routes to OpenAI or Anthropic based on model name |

Real mode requires API keys:
- OpenAI: `OPENAI_API_KEY`
- Anthropic: `ANTHROPIC_API_KEY`

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
plan/
└── REQ-OUTSTANDING-ITEMS.md     # Features not yet adopted from punit-examples
```
