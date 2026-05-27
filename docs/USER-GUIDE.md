# feotest-examples — User Guide

Rust developers set a high bar. The ecosystem's emphasis on correctness —
ownership, type safety, `unsafe` as an explicit opt-in, `clippy` as a cultural
norm — reflects a community that treats quality as an engineering discipline,
not an afterthought.

But there is a category of system where even Rust's rigour leaves a gap.
Services backed by large language models, ranking algorithms, classifiers, and
other stochastic components produce different outputs on each invocation. For
these systems the correctness of a single execution is not a meaningful concept
— correctness is a statistical property of behaviour observed over many
executions under controlled conditions.

The Rust ecosystem has strong tools for deterministic testing (`cargo test`,
`cargo-nextest`), property-based testing (`proptest`, `quickcheck`), fuzzing,
and benchmarking (`criterion`). `feotest` does not compete with any of them. It
complements them: statistically sound verdicting of stochastic service
behaviour, grounded in confidence bounds rather than ad hoc retry counts or
hard-coded tolerances.

This project is a worked tour of that discipline. For the framework concepts
themselves — Wilson bounds, threshold derivation, the verdict model — see
[`feotest`'s own user guide](../../feotest/docs/USER-GUIDE.md); this guide shows
them applied.

## The two example services

### Shopping basket (LLM-powered)

A user issues natural-language instructions like _"Add 2 apples"_. An LLM
translates each into structured JSON a basket API can execute:

```json
{"actions": [{"context": "SHOP", "name": "add",
  "parameters": [{"name": "item", "value": "apples"}, {"name": "quantity", "value": "2"}]}]}
```

Valid `SHOP` actions are `add`, `remove`, `clear`. A translation succeeds when
the LLM returns valid JSON that deserialises into valid actions. Because the LLM
may hallucinate fields, emit malformed JSON, or invent actions, success is
probabilistic — a natural fit for feotest's **empirical** approach, where the
threshold is *derived from a measured baseline*.

### Payment gateway (SLA-driven)

A payment gateway with a contractual 99.99% SLA. The mock underperforms it
slightly (~99.97%) so the statistics have realistic data. The threshold is known
upfront from the contract — feotest's **normative** approach.

## Project layout

```
src/
  llm/ shopping/ payment/        application code (the domain)
  service_contracts/             the units under test, on feotest's ServiceContract trait
    shopping_basket.rs           empirical: invoke + criteria (non-empty + transforming validate)
    payment_gateway.rs           normative: meeting().pass_rate + a p95 latency commitment
    sample_sizes.rs              centralised, example-tiny sizing/threshold policy
  bin/sentinel.rs                deployable reliability sentinel (run_cli)
tests/
  *_test.rs                      probabilistic tests (one file per scenario)
  experiment_*.rs                measure / explore / optimize experiments
  common/                        shared baseline helper
```

Service contracts live in `src/` (not `tests/`) so the same contract drives the
probabilistic tests, the experiments, and the deployable sentinel.

## Authoring a service contract

A contract implements `ServiceContract`: `invoke` produces the **raw** response,
and `criteria` judge it. A malformed response is not a defect — it is a criterion
failure. See `src/service_contracts/shopping_basket.rs` and `payment_gateway.rs`
in full; the shape is:

```rust
impl ServiceContract for PaymentGatewayServiceContract {
    type Input = Charge;
    type Output = PaymentResult;
    fn id(&self) -> &str { "payment-gateway" }
    fn invoke(&self, charge: &Charge, _cost: &mut Cost) -> Result<PaymentResult, Defect> {
        Ok(self.gateway.charge(&charge.card_token, charge.amount_cents))
    }
    fn criteria(&self) -> Criteria<PaymentResult> {
        Criteria::of([Criterion::meeting().pass_rate(0.99)
            .name("transaction-succeeds")
            .satisfies("transaction succeeds", |r: &PaymentResult| {
                if r.is_success() { Ok(()) } else { Err(ContractViolation::new("transaction", "declined")) }
            })
            .build()])
    }
    fn latency(&self) -> Option<LatencyCriterion> {
        Some(LatencyCriterion::meeting().at_most(Percentile::P95, Duration::from_secs(1)))
    }
}
```

## Running probabilistic tests

```bash
cargo test                          # everything (mock LLM, no API keys)
cargo test --test payment_gateway_sla_test
```

Each scenario is its own file under `tests/`:

| File | Demonstrates |
|------|--------------|
| `payment_gateway_sla_test` | threshold-first against a normative SLA, smoke intent |
| `payment_gateway_inline_sla_test` | the inline `#[probabilistic_test]` form (no separate contract) |
| `shopping_basket_test` | the empirical pair: measure a baseline, then verify it |
| `shopping_basket_covariate_test` | covariate-matched vs mismatched baselines |
| `shopping_basket_budget_test` | time / token budgets and exhaustion behaviour |
| `shopping_basket_pacing_test` | requests-per-second / per-minute pacing |
| `shopping_basket_threshold_approaches_test` | sample-size-first and confidence-first |
| `shopping_basket_diagnostics_test` | transparent statistics |
| `shopping_basket_conformance_test` | reproducible verdicts under a seeded mock |
| `invalid_configuration_test` | configurations the framework rejects |

The empirical (shopping) tests measure a baseline into a temp directory first,
because the shopping criterion is empirical and needs one. The normative
(payment) tests need no baseline.

## Running experiments

Experiments establish or compare baselines. Each is a `#[test]` writing to a
temporary directory (so the suite leaves no artifacts):

| File | Experiment |
|------|------------|
| `experiment_measure` | establish baselines for both services |
| `experiment_explore` | sweep a model × temperature grid |
| `experiment_optimize` | temperature cool-down search (custom `Scorer` + `FactorMutator`) |

## The sentinel

`src/bin/sentinel.rs` is a deployable binary carrying both services' reliability
specs into a target environment:

```bash
cargo run --bin sentinel -- list                    # registered specs
cargo run --bin sentinel -- run <spec>              # a probabilistic check
cargo run --bin sentinel -- measure --output <uri> <spec>
cargo run --bin sentinel -- check --baselines <dir>
```

## LLM mode

The shopping example runs against a built-in mock by default — no API keys:

```bash
cargo test                                  # mock (default)
FEOTEST_LLM_MODE=real OPENAI_API_KEY=sk-... cargo test   # real provider
```

The mock produces realistic, temperature-dependent behaviour, so the examples
demonstrate the framework without API cost. Individual sample failures within a
run are expected — that is the nature of probabilistic testing; the verdict, not
any single sample, is the result.
