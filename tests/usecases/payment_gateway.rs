//! Payment gateway use case: SLA-driven reliability testing.
//!
//! This use case wraps a payment processing service and tests it against
//! an SLA requirement. Unlike the shopping basket (where the threshold
//! is derived empirically), the payment gateway has an explicit
//! contractual threshold — making it a natural demonstration of
//! threshold-first testing with normative origins.
//!
//! The mock gateway achieves ~99.97% success, which is deliberately
//! below a hypothetical 99.99% SLA. This means sufficiently large
//! sample runs will detect the gap.

use std::fmt;
use std::time::Instant;

use feotest::model::{ContractViolation, TrialOutcome};

use feotest_examples::payment::{MockPaymentGateway, PaymentGateway};

/// A use case for charging a payment card and verifying the transaction.
///
/// All configuration is set at construction time. The use case is immutable
/// after construction — this preserves the i.i.d. assumption required for
/// valid statistical inference.
///
/// # Contract postcondition
///
/// The transaction must succeed (functional correctness only; latency
/// testing is not yet supported in feotest).
pub struct PaymentGatewayUseCase {
    gateway: Box<dyn PaymentGateway>,
    region: String,
}

impl PaymentGatewayUseCase {
    /// Creates a new payment gateway use case with the default mock.
    #[must_use]
    pub fn new() -> Self {
        Self {
            gateway: Box::new(MockPaymentGateway::new()),
            region: "US".to_string(),
        }
    }

    /// Creates a use case with a specific gateway implementation.
    #[must_use]
    pub fn gateway(gateway: Box<dyn PaymentGateway>) -> Self {
        Self {
            gateway,
            region: "US".to_string(),
        }
    }

    /// Sets the region at construction time.
    #[must_use]
    pub fn region(mut self, region: impl Into<String>) -> Self {
        self.region = region.into();
        self
    }

    /// Charges a card and evaluates the service contract.
    ///
    /// The single postcondition is that the transaction must succeed.
    /// A declined or errored transaction is a contract violation — a
    /// legitimate statistical observation, not a software defect.
    #[must_use]
    pub fn charge_card(&self, card_token: &str, amount_cents: u64) -> TrialOutcome {
        let start = Instant::now();
        let result = self.gateway.charge(card_token, amount_cents);
        let elapsed = start.elapsed();

        if result.is_success() {
            TrialOutcome::success(elapsed)
        } else {
            let error_code = result.error_code().unwrap_or("UNKNOWN");
            TrialOutcome::failure(
                ContractViolation::new("transaction", format!("payment failed: {error_code}")),
                elapsed,
            )
        }
    }
}

impl fmt::Display for PaymentGatewayUseCase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PaymentGateway (region={})", self.region)
    }
}

impl Default for PaymentGatewayUseCase {
    fn default() -> Self {
        Self::new()
    }
}