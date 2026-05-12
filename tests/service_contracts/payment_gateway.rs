//! Payment gateway service contract: SLA-driven reliability testing.
//!
//! This service contract wraps a payment processing service and tests it against
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
use feotest::spec::namer::CovariateProfile;
use feotest::service_contract::{CovariateDeclaration, ServiceContract};

use feotest_examples::payment::{MockPaymentGateway, PaymentGateway};

/// A service contract for charging a payment card and verifying the transaction.
///
/// All configuration is set at construction time. The service contract is immutable
/// after construction — this preserves the i.i.d. assumption required for
/// valid statistical inference.
///
/// # Contract postcondition
///
/// The transaction must succeed (functional correctness).
pub struct PaymentGatewayServiceContract {
    gateway: Box<dyn PaymentGateway>,
    region: String,
}

impl PaymentGatewayServiceContract {
    /// Creates a new payment gateway service contract with the default mock.
    #[must_use]
    pub fn new() -> Self {
        Self {
            gateway: Box::new(MockPaymentGateway::new()),
            region: "US".to_string(),
        }
    }

    /// Creates a service contract with a specific gateway implementation.
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

impl ServiceContract for PaymentGatewayServiceContract {
    fn id(&self) -> &str {
        "payment-gateway"
    }

    fn description(&self) -> &str {
        "Charges a payment card and verifies transaction success against an SLA"
    }

    fn covariates(&self) -> Vec<CovariateDeclaration> {
        vec![CovariateDeclaration::region()]
    }

    fn resolve_covariates(&self) -> CovariateProfile {
        CovariateProfile::builder()
            .put("region", &self.region)
            .build()
    }
}

impl fmt::Display for PaymentGatewayServiceContract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PaymentGateway (region={})", self.region)
    }
}

impl Default for PaymentGatewayServiceContract {
    fn default() -> Self {
        Self::new()
    }
}