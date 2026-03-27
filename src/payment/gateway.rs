//! Mock payment gateway with SLA-bound reliability.

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::sync::Mutex;
use std::time::Duration;

use crate::payment::PaymentGateway;
use crate::payment::result::PaymentResult;

/// Failure rate: 0.03% (99.97% success), intentionally below a 99.99% SLA.
const FAILURE_RATE: f64 = 0.0003;

/// Error codes returned by the mock gateway on failure.
const ERROR_CODES: &[&str] = &[
    "DECLINED",
    "TIMEOUT",
    "NETWORK_ERROR",
    "INSUFFICIENT_FUNDS",
    "CARD_EXPIRED",
    "FRAUD_SUSPECTED",
];

/// A mock payment gateway that simulates production-like reliability.
///
/// The mock achieves a ~99.97% success rate — intentionally below a
/// hypothetical 99.99% SLA target. This makes it possible to observe
/// occasional failures in large sample runs, demonstrating how
/// probabilistic tests handle low-probability events.
///
/// Each transaction incurs a simulated latency of 50–200ms.
///
/// # Examples
///
/// ```
/// use feotest_examples::payment::{MockPaymentGateway, PaymentResult};
/// use feotest_examples::payment::PaymentGateway;
///
/// let gateway = MockPaymentGateway::new();
/// let result = gateway.charge("tok_visa_4242", 1999);
///
/// // At 99.97% success rate, this almost certainly succeeds
/// // (but not guaranteed — that's the point of probabilistic testing)
/// println!("Success: {}", result.is_success());
/// ```
pub struct MockPaymentGateway {
    rng: Mutex<StdRng>,
}

impl MockPaymentGateway {
    /// Creates a new mock gateway with a random seed.
    #[must_use]
    pub fn new() -> Self {
        Self {
            rng: Mutex::new(StdRng::from_entropy()),
        }
    }

    /// Creates a new mock gateway with a fixed seed for reproducible tests.
    #[must_use]
    pub fn with_seed(seed: u64) -> Self {
        Self {
            rng: Mutex::new(StdRng::seed_from_u64(seed)),
        }
    }

    /// Generates a realistic transaction ID.
    fn generate_transaction_id(rng: &mut StdRng) -> String {
        let id: u64 = rng.r#gen();
        format!("txn_{id:016x}")
    }
}

impl Default for MockPaymentGateway {
    fn default() -> Self {
        Self::new()
    }
}

impl PaymentGateway for MockPaymentGateway {
    #[allow(clippy::significant_drop_tightening)]
    fn charge(&self, _card_token: &str, _amount_cents: u64) -> PaymentResult {
        let (latency_ms, fails, error_idx, txn_id) = {
            let mut rng = self.rng.lock().unwrap();
            let latency: u64 = rng.gen_range(50..=200);
            let fail = rng.gen_bool(FAILURE_RATE);
            let err_idx = rng.gen_range(0..ERROR_CODES.len());
            let id = Self::generate_transaction_id(&mut rng);
            (latency, fail, err_idx, id)
        };

        // Simulate latency (50–200ms)
        std::thread::sleep(Duration::from_millis(latency_ms));

        if fails {
            PaymentResult::failure(ERROR_CODES[error_idx])
        } else {
            PaymentResult::success(txn_id)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mostly_succeeds() {
        let gateway = MockPaymentGateway::with_seed(42);
        let mut successes = 0;
        for _ in 0..100 {
            if gateway.charge("tok_visa_4242", 1999).is_success() {
                successes += 1;
            }
        }
        // With 99.97% success rate, expect at least 95 out of 100
        assert!(successes >= 95, "Expected >= 95 successes, got {successes}");
    }

    #[test]
    fn successful_result_has_transaction_id() {
        let gateway = MockPaymentGateway::with_seed(42);
        let result = gateway.charge("tok_visa_4242", 1999);
        if result.is_success() {
            assert!(result.transaction_id().is_some());
            assert!(result.transaction_id().unwrap().starts_with("txn_"));
        }
    }
}
