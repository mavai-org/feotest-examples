//! Payment domain: gateway interface and mock implementation.
//!
//! This module models a payment processing service with an SLA-driven
//! reliability target. The [`MockPaymentGateway`] simulates realistic
//! payment behaviour:
//!
//! - **99.97% success rate** (intentionally below a 99.99% SLA target,
//!   making it possible to observe failures in large sample runs)
//! - **50–200ms latency** per transaction
//! - Realistic error codes: `DECLINED`, `TIMEOUT`, `NETWORK_ERROR`, etc.

mod gateway;
mod result;

pub use gateway::MockPaymentGateway;
pub use result::PaymentResult;

/// A payment processing service.
///
/// Implementations accept a card token and amount, and return a
/// [`PaymentResult`] indicating success or failure.
pub trait PaymentGateway: Send + Sync {
    /// Charges the given card for the specified amount.
    fn charge(&self, card_token: &str, amount_cents: u64) -> PaymentResult;
}
