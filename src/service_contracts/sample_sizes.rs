//! Centralised sample-size and threshold policy for the worked examples.
//!
//! The examples deliberately use **tiny** sample counts so the whole suite
//! runs cheaply. Each constant documents the count a real deployment would
//! use — orders of magnitude larger — alongside the example value. Tests,
//! experiments, and sentinels read these constants so dev-time and the
//! documented production sizing stay commensurate.

/// Sizing for the shopping-basket (empirical) example.
pub mod shopping {
    /// Samples for a baseline-establishing measure run.
    /// Example: 30. A real baseline uses 200+ for a tight interval.
    pub const MEASURE: u32 = 30;

    /// Samples for a baseline-verifying probabilistic test.
    /// Example: 3. A real verification uses 20+.
    pub const TEST: u32 = 3;

    /// Samples per configuration in an explore sweep.
    /// Example: 5. A real comparison uses 50+.
    pub const EXPLORE_PER_CONFIG: u32 = 5;

    /// Samples per iteration in an optimize run.
    /// Example: 5. A real optimisation uses 30+.
    pub const OPTIMIZE_PER_ITERATION: u32 = 5;
}

/// Sizing and thresholds for the payment-gateway (normative) example.
pub mod payment {
    /// Samples for a sentinel/probabilistic check.
    /// Example: 3. A real check uses far more.
    pub const TEST: u32 = 3;

    /// The internal service-level *objective* the team holds itself to.
    pub const INTERNAL_OBJECTIVE_PASS_RATE: f64 = 0.99;

    /// Minimum samples to verify the internal objective at 95% confidence.
    pub const INTERNAL_OBJECTIVE_FLOOR: u32 = 268;

    /// The customer-facing contractual SLA pass rate.
    pub const CONTRACTUAL_SLA_PASS_RATE: f64 = 0.9999;

    /// A deliberately undersized smoke check against the 99.99% SLA.
    ///
    /// Too few samples to *verify* the SLA (that runs to the thousands), but
    /// enough to catch a catastrophic regression. Pair with a smoke intent so
    /// the feasibility gate stays quiet.
    pub const CONTRACTUAL_SLA_SMOKE: u32 = 50;
}
