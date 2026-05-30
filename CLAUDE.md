# feotest-examples

Example project demonstrating the [feotest](../feotest) probabilistic testing framework.

## Terminology

The canonical glossary for the mavai project family is at
[`../mavai-R/docs/GLOSSARY.md`](../mavai-R/docs/GLOSSARY.md). All code
comments, documentation, and discussions should use terms consistently with
that glossary.

## Build and test

```bash
# Build
cargo build

# Run all tests (mock LLM, no API keys needed)
cargo test

# Run tests with output shown
cargo test -- --nocapture

# Run a specific test by name
cargo test test_name

# Run a specific integration test file
cargo test --test shopping_basket_measure

# Check without building
cargo check

# Lint
cargo clippy

# Format
cargo fmt
```

## Testing notes

Many tests demonstrate feotest features where the verdict depends on the
stochastic mock's behaviour. Individual sample failures within a test run are
expected — that is the nature of probabilistic testing. The key indicator of
correctness is successful **compilation** and that verdicts align with
statistical expectations, not a 100% sample pass rate.

## Project structure

- **src/llm/** — LLM infrastructure: `ChatLlm` trait, mock with temperature-dependent reliability, real provider routing
- **src/shopping/** — Shopping domain types, action model, and response validator
- **src/payment/** — Payment gateway trait and mock with configurable failure rate
- **src/service_contracts/** — The units under test on feotest's `ServiceContract` trait (`invoke` + `criteria`), plus the centralised `sample_sizes` policy. In `src/` so the same contract drives tests, experiments, and the sentinel.
- **src/bin/sentinel.rs** — Deployable reliability sentinel binary (`feotest::sentinel::run_cli`).
- **tests/*_test.rs** — Probabilistic tests, one file per scenario (SLA, inline, covariate, budget, pacing, threshold-approaches, diagnostics, conformance, invalid-config).
- **tests/experiment_*.rs** — Measure / explore / optimize experiments.
- **tests/common/** — Shared baseline-measuring helper (the shopping criterion is empirical, so its tests need a baseline).

## feotest dependency

The project uses a path dependency (`path = "../feotest"`) to reference the
local feotest checkout. This means the sibling `feotest` directory must be
present for the project to compile.

## Conventions

### Language and toolchain

- Rust edition 2024, minimum supported Rust version 1.85.
- All code must pass `cargo clippy` with the lint configuration in `Cargo.toml` (clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo all at warn level).
- All code must be formatted with `cargo fmt` (default rustfmt settings).
- `unsafe` code is forbidden (`unsafe_code = "forbid"` in `Cargo.toml`).

### Code style

- **Idiomatic Rust**: prefer standard library types and patterns. Use `Option` for optional values. Prefer iterators over manual loops. Use `impl Trait` in argument position for flexibility, concrete types in return position for clarity.
- **Explicit naming**: names should be self-documenting. Avoid abbreviations except where universally understood. Public APIs should read like a domain model.
- **Type-driven design**: use newtypes and enums to make invalid states unrepresentable. Derive standard traits (`Debug`, `Clone`, `PartialEq`) where appropriate.
- **`Result` is for genuine runtime uncertainty only**: a violated precondition is a programming error — assert and abort. Reserve `Result` for conditions outside the program's control (network calls, user-provided files, stochastic service responses).
- **`unwrap()` in library code**: acceptable only where failure is logically impossible. Freely acceptable in tests.

### Testing

- Integration tests live in `tests/` and exercise feotest experiments and probabilistic test workflows.
- Test names should read as sentences: `fn sla_verification()`, `fn measure_shopping_basket_baseline()`.
- Use `assert!`, `assert_eq!`, `assert_ne!` from the standard library.

### Documentation tone

- This is a Rust project. Documentation and comments should be written for a Rust audience. Do not reference Java, JUnit, or punit in code comments or doc strings.

### Dependencies

- Add dependencies deliberately. Every dependency must justify its inclusion.
- Pin major versions in `Cargo.toml` to avoid unexpected breakage.

### Git

- Commit messages should be concise and describe the *why*, not just the *what*.
- Keep commits focused: one logical change per commit.
