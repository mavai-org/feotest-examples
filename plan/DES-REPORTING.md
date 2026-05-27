# DES: Reporting Design

## Context

feotest computes rich statistical information during every probabilistic test
and experiment. Today, almost none of it reaches the developer. The macro tests
produce silence on pass and a terse panic message on fail. The builder tests
can access the `VerdictRecord` programmatically, but there is no built-in
rendering.

punit (the Java sibling) has a mature reporting pipeline: console output during
test runs, JUnit XML for CI, and a standalone HTML report for human review.
feotest should match or exceed this capability, taking advantage of Rust's
simpler test harness — where there is no divergence between the harness verdict
and the statistical verdict.

### Rust's advantage over JUnit

In JUnit 5, each sample is a test event. A sample failure is reported as a test
failure to the framework, which CI interprets as a broken build. punit had to
work around this by reporting statistically insignificant sample failures as
"skipped" to prevent CI breakage. The HTML report consequently needs two
columns — "JUnit" (what JUnit thinks) and "PUnit" (what the statistics say) —
because the two can disagree.

In Rust, a `#[test]` function is opaque. `cargo test` doesn't know or care what
happens inside it. Individual sample failures are invisible to the harness. The
probabilistic test runs all samples internally, computes the verdict, and either
returns (pass) or panics (fail). This means:

- **No "skipped" hack needed.** Sample failures don't leak to CI.
- **No harness-vs-verdict divergence.** There is one result: the statistical
  verdict. No need for two columns in a report.
- **The test harness and the statistical verdict are always aligned.** A test
  passes in `cargo test` if and only if the statistical verdict is Pass.

This is a genuine DX advantage. The reporting design should leverage it rather
than replicate punit's workarounds.

## Requirements mapping

The reporting requirements from the feature inventory are:

| ID | Name | Status | Priority |
|----|------|--------|----------|
| RP01 | Verdict Record | Implemented | — |
| RP02 | JUnit/Surefire XML | Implemented | — |
| RP03 | Custom XML Schema | N/A for Rust | — |
| RP04 | HTML Report | Not implemented | 2 |
| RP05 | Console Renderer | Not implemented | **1** |
| RP06 | Verdict Catalogue | Not implemented | **1** |

RP05 and RP06 are the highest priority because they provide immediate feedback
during development. RP04 (HTML) builds on RP06 and is the CI/operator
deliverable.

## Design

### Architecture

```
                 Trial execution
                       |
                       v
              VerdictRecord (RP01)
                       |
           +-----------+-----------+
           |           |           |
           v           v           v
     ConsoleRenderer  VerdictCatalogue  JunitXmlWriter
        (RP05)          (RP06)           (RP02)
                         |
                    +----+----+
                    |         |
                    v         v
              HtmlReport   JSON export
               (RP04)
```

`VerdictRecord` is the single source of truth. All renderers consume it. No
renderer has access to information that another does not.

### RP05: Console renderer

The console renderer is the primary feedback channel during local development.
It must produce useful output during `cargo test -- --nocapture` without
requiring any setup.

#### Output model

Each probabilistic test produces one verdict block. Each experiment produces
one summary block. After all tests, a suite summary line is emitted.

#### Verdict block (probabilistic test)

```
── ShoppingBasketServiceContract::threshold_first_verification ── PASS ──

  Observed:   0.9400 (94/100) >= threshold: 0.8000
  Confidence: 95.0%   CI: [0.8743, 0.9753]
  p-value:    0.9998   z: 3.52
  Elapsed:    52ms

```

On failure:

```
── ShoppingBasketServiceContract::sla_verification ── FAIL ──

  Observed:   0.9650 (193/200) >= threshold: 0.9900
  Confidence: 95.0%   CI: [0.9295, 0.9832]
  p-value:    0.0031   z: -2.74
  Elapsed:    26410ms

  Caveats:
    - Observed rate 0.9650 is below SLA threshold 0.9900.

```

Inconclusive (when baseline is misaligned or sample insufficient):

```
── ShoppingBasketServiceContract::spec_driven ── INCONCLUSIVE ──

  Observed:   0.9200 (92/100) >= threshold: 0.9047
  Confidence: 95.0%   CI: [0.8488, 0.9592]
  Elapsed:    48ms

  Caveats:
    - Baseline spec outdated: generated 2026-01-15, service updated 2026-03-28.

```

#### Verdict block structure

Line 1: identity + verdict (coloured when terminal supports ANSI)
- PASS: green
- FAIL: red
- INCONCLUSIVE: amber/purple

Body: key statistics, always shown. Fits within 80 columns.

Caveats: only shown when present. Includes warnings from the VerdictRecord.

Statistical detail (when `transparent_stats = true`): expanded block with
SE, test statistic derivation, power analysis. Mirrors punit's Level 3 detail.

#### Experiment summary (measure)

```
── measure: ShoppingBasketServiceContract (baseline-v1) ──

  Samples:    1000 (1000 planned)
  Pass rate:  0.9200   CI: [0.9015, 0.9353]
  Threshold:  0.9047 (Wilson lower bound)
  Spec:       tests/baselines/ShoppingBasketServiceContract.yaml (written)
  Elapsed:    3214ms

```

#### Experiment summary (explore)

```
── explore: ShoppingBasketServiceContract (model-comparison) ──

  gpt-4o-mini (temp=0.1)     0.9700 (97/100)
  gpt-4o-mini (temp=0.5)     0.8200 (82/100)

  Best: gpt-4o-mini (temp=0.1)
  Elapsed: 210ms

```

#### Suite summary

After all tests and experiments:

```
── feotest: 6 tests, 4 pass, 1 fail, 1 inconclusive (52.4s) ──
```

#### Colour support

- Detect terminal capability via the `NO_COLOR` environment variable
  (de facto standard, see https://no-color.org).
- When colour is available, use ANSI codes for verdict colouring.
- When colour is unavailable, use text markers: `[PASS]`, `[FAIL]`,
  `[INCONCLUSIVE]`.
- Dependency: `owo-colors` or `termcolor` (lightweight, well-maintained).
  Alternatively, emit ANSI codes directly — only three colours are needed.

#### Integration with `cargo test`

`cargo test` captures stdout by default. Output is only visible with
`--nocapture`. This is standard Rust behaviour and developers expect it.

The renderer writes to stdout. Warnings and errors write to stderr (visible
even without `--nocapture`).

For the macro-generated tests, the macro expansion should call the console
renderer automatically before asserting. This means every macro test produces
visible output when run with `--nocapture`, and on failure the output is
shown by `cargo test` (which displays captured output for failed tests).

For the builder API, `.run()` should call the console renderer before
asserting. `.execute()` does not render (the developer is handling output
themselves).

#### Verbosity

| Level | Env var | Behaviour |
|-------|---------|-----------|
| Normal | (default) | Verdict block with key statistics |
| Verbose | `FEOTEST_VERBOSE=1` | Full statistical detail per test |
| Quiet | `FEOTEST_QUIET=1` | Suite summary only |

### RP06: Verdict catalogue

The verdict catalogue aggregates all verdicts from a test run into a single
structure. It is the data source for the HTML report and the suite summary
line.

#### Data model

```rust
pub struct VerdictCatalogue {
    suite_name: String,
    timestamp: DateTime<Utc>,
    duration: Duration,
    verdicts: Vec<VerdictRecord>,
}
```

#### Derived properties

| Property | Description |
|----------|-------------|
| `total()` | Number of tests |
| `pass_count()` | Tests with Pass verdict |
| `fail_count()` | Tests with Fail verdict |
| `inconclusive_count()` | Tests with Inconclusive verdict |
| `suite_pass_rate()` | pass_count / total |
| `mean_pass_rate()` | Average observed pass rate across all tests |
| `weakest_test()` | Test with lowest observed pass rate |
| `closest_margin()` | Test closest to its threshold |
| `failure_digest()` | Failed + inconclusive tests with detail |

#### Serialisation

Derives `serde::Serialize` for JSON export. This enables:
- Trend analysis across builds (persist JSON per CI run)
- Custom dashboards consuming JSON
- Comparison between environments

#### Collection mechanism

The challenge in Rust is that each `#[test]` function runs independently —
there is no test lifecycle hook to collect verdicts across tests in the same
binary.

Proposed approach: **file-based accumulation**.

1. Each test writes its `VerdictRecord` as a JSON file to
   `target/feotest/verdicts/{test_name}.json`.
2. After `cargo test` completes, a post-processing step reads all verdict
   files and assembles the catalogue.
3. The post-processing can be:
   - A `cargo-feotest` subcommand (`cargo feotest report`)
   - A build script or CI step
   - Invoked manually

This is the same pattern used by `cargo-nextest` (writes per-test results,
assembles report post-hoc) and `cargo-llvm-cov` (writes per-test coverage,
merges after).

Alternative: use a shared file with file locking (append-only JSONL). Simpler
but risks contention with `cargo test`'s parallel execution.

The per-file approach is preferred: no contention, no locking, easy to debug.

### RP04: HTML report

The HTML report is a standalone, self-contained file with embedded CSS. No
JavaScript dependencies, no external resources. Opens in any browser.

#### Content

1. **Header**: suite name, timestamp, duration
2. **Summary bar**: total tests, pass/fail/inconclusive counts (coloured)
3. **Statistical assumptions disclosure** (collapsible): explains Bernoulli
   assumptions, warns about violations. Matches punit's approach.
4. **Results table**, grouped by service contract:
   - Test name (collapsible detail)
   - Verdict (coloured)
   - Functional summary (pass/fail counts)
   - Latency percentiles (when available)
   - Sample count
   - Elapsed time
5. **Detail panel** (within collapsible):
   - Level 2: observed rate, threshold, contract summary, elapsed
   - Level 3: full statistical analysis (confidence, SE, CI, z, p-value),
     caveats, provenance
6. **Footer**: generation timestamp, feotest version

#### Differences from punit's HTML report

- **No JUnit column.** There is no harness-vs-verdict divergence in Rust.
  The verdict column is the only verdict.
- **No "inconclusive due to misalignment" banner** unless feotest implements
  covariate tracking (future feature). If implemented, include it.
- **Latency columns** shown only when latency data is present. No empty
  columns for tests that don't measure latency.

#### Implementation approach

**Template engine**: `askama` (compile-time Jinja2-like templates). This is
the idiomatic Rust choice for HTML generation:
- Templates are compiled into the binary — no runtime file loading.
- Type-safe: template variables are checked at compile time.
- Zero runtime dependencies beyond the generated code.

Alternative: string formatting with `write!`. Simpler, no dependency, but
harder to maintain as the template grows. Given the report's complexity
(collapsible sections, conditional columns, grouped rows), a template engine
is justified.

**CSS**: embedded inline in `<style>` tag. The punit report's CSS is a good
starting point — clean, minimal, dark-on-light, readable.

#### Generation

The HTML report is generated from the `VerdictCatalogue`:

```rust
pub struct HtmlReportWriter;

impl HtmlReportWriter {
    /// Writes the report to a file.
    pub fn write_to_file(catalogue: &VerdictCatalogue, path: &Path) -> io::Result<()>;

    /// Renders the report to a string.
    pub fn render(catalogue: &VerdictCatalogue) -> String;
}
```

Default output location: `target/feotest/reports/index.html`.

### Reporting pipeline — end to end

The full pipeline for a typical CI run:

```
cargo test -- --nocapture
  |
  +-- each test writes VerdictRecord JSON to target/feotest/verdicts/
  +-- each test prints verdict block to stdout (RP05)
  |
  v
cargo feotest report                    (or: a CI step)
  |
  +-- reads target/feotest/verdicts/*.json
  +-- assembles VerdictCatalogue (RP06)
  +-- writes target/feotest/reports/index.html (RP04)
  +-- prints suite summary to stdout (RP05)
```

For local development, the per-test console output (step 1) is sufficient.
The HTML report is a CI artifact.

### Open questions

1. **`cargo-feotest` subcommand vs library function** — should the report
   generation be a standalone CLI tool or a library function that tests can
   invoke? The CLI approach is cleaner (no test code running report logic)
   but adds an installation step. The library approach is zero-setup but
   muddies the test/report boundary.

2. **Verdict file cleanup** — should `target/feotest/verdicts/` be cleared
   before each `cargo test` run? If not, stale verdicts from previous runs
   will appear in the report. A `cargo feotest clean` command or automatic
   cleanup at report generation time could handle this.

3. **Experiment results in the report** — should measure and explore
   experiments appear in the HTML report alongside probabilistic tests? They
   have different output structures (no verdict per se). punit's report
   includes only probabilistic tests. Experiments could have a separate
   section or a separate report.

4. **Template dependency** — `askama` adds a build dependency. Is this
   acceptable, or should the HTML be generated with pure `write!` formatting?
   The template is complex enough that `askama` would significantly improve
   maintainability.

5. **Per-test file I/O overhead** — writing a JSON file per test adds I/O.
   For fast tests (mock-based, <1ms), this could be noticeable. An alternative
   is to write only when `FEOTEST_REPORT=1` is set, making it opt-in for CI
   and silent for local development. The console renderer would still work
   regardless.

6. **Colour in CI logs** — some CI systems (GitHub Actions, GitLab) support
   ANSI colours in logs. Should feotest auto-detect this, or respect only
   `NO_COLOR`? The `supports-color` crate can detect CI environments, but
   adds a dependency for a cosmetic feature.
