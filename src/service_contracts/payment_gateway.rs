//! Payment gateway: SLA-driven reliability.
//!
//! A payment charge either succeeds or fails. Unlike the shopping basket
//! (whose target is measured), this service has an explicit contractual
//! target — a normative pass rate and a latency ceiling — so it is the
//! natural home for threshold-first testing against an SLA.
//!
//! The mock succeeds ~99.97% of the time, deliberately just under a 99.99%
//! SLA, so a large enough run will detect the shortfall.

use std::fmt;
use std::time::Duration;

use feotest::controls::Cost;
use feotest::criteria::{Criteria, Criterion};
use feotest::latency::{LatencyCriterion, Percentile};
use feotest::model::{ContractViolation, Defect};
use feotest::service_contract::{CovariateDeclaration, ServiceContract};
use feotest::spec::namer::CovariateProfile;

use crate::payment::{MockPaymentGateway, PaymentGateway, PaymentResult};

/// A card charge: the input a payment sample is drawn from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Charge {
    /// The tokenised card identifier.
    pub card_token: String,
    /// The amount to charge, in minor units (cents).
    pub amount_cents: u64,
}

impl Charge {
    /// Creates a charge.
    #[must_use]
    pub fn new(card_token: impl Into<String>, amount_cents: u64) -> Self {
        Self {
            card_token: card_token.into(),
            amount_cents,
        }
    }
}

/// Charges a payment card and verifies the transaction succeeds.
pub struct PaymentGatewayServiceContract {
    gateway: Box<dyn PaymentGateway>,
    region: String,
}

impl PaymentGatewayServiceContract {
    /// Creates a contract backed by the default mock gateway.
    #[must_use]
    pub fn new() -> Self {
        Self::with_gateway(Box::new(MockPaymentGateway::new()))
    }

    /// Creates a contract backed by a specific gateway.
    #[must_use]
    pub fn with_gateway(gateway: Box<dyn PaymentGateway>) -> Self {
        Self {
            gateway,
            region: "US".to_string(),
        }
    }

    /// Sets the region covariate.
    #[must_use]
    pub fn region(mut self, region: impl Into<String>) -> Self {
        self.region = region.into();
        self
    }
}

impl ServiceContract for PaymentGatewayServiceContract {
    type Input = Charge;
    type Output = PaymentResult;

    fn id(&self) -> &'static str {
        "payment-gateway"
    }

    fn description(&self) -> &'static str {
        "Charges a payment card and verifies transaction success against an SLA"
    }

    fn warmup(&self) -> u32 {
        3
    }

    /// Charges the card and returns the gateway's result. A declined or errored
    /// transaction is a legitimate observation carried in the result, not a
    /// defect, so `invoke` returns `Ok` for every response the gateway gives.
    fn invoke(&self, charge: &Charge, _cost: &mut Cost) -> Result<PaymentResult, Defect> {
        Ok(self.gateway.charge(&charge.card_token, charge.amount_cents))
    }

    /// One normative criterion: the transaction must succeed, at the SLA pass
    /// rate.
    fn criteria(&self) -> Criteria<PaymentResult> {
        Criteria::of([Criterion::meeting()
            .pass_rate(
                crate::service_contracts::sample_sizes::payment::INTERNAL_OBJECTIVE_PASS_RATE,
            )
            .name("transaction-succeeds")
            .satisfies("transaction succeeds", |result: &PaymentResult| {
                if result.is_success() {
                    Ok(())
                } else {
                    let code = result.error_code().unwrap_or("UNKNOWN");
                    Err(ContractViolation::new(
                        "transaction",
                        format!("payment failed: {code}"),
                    ))
                }
            })
            .build()])
    }

    /// The SLA latency commitment: p95 at or under one second.
    fn latency(&self) -> Option<LatencyCriterion> {
        Some(LatencyCriterion::meeting().at_most(Percentile::P95, Duration::from_secs(1)))
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

impl Default for PaymentGatewayServiceContract {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for PaymentGatewayServiceContract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PaymentGateway (region={})", self.region)
    }
}

/// The canonical charge set, shared by experiments, tests, and the sentinel.
#[must_use]
pub fn standard_charges() -> Vec<Charge> {
    vec![
        Charge::new("tok_visa_4242", 1999),
        Charge::new("tok_mastercard_5555", 2499),
        Charge::new("tok_amex_3782", 3499),
        Charge::new("tok_discover_6011", 999),
        Charge::new("tok_visa_4000", 14999),
    ]
}
