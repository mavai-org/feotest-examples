//! Service contracts: the units of work under test.
//!
//! Each contract wraps a stochastic service behind feotest's
//! [`ServiceContract`](feotest::service_contract::ServiceContract) trait —
//! producing a raw response from [`invoke`](feotest::service_contract::ServiceContract::invoke)
//! and judging it with named [`criteria`](feotest::service_contract::ServiceContract::criteria).
//! They live in the library (not under `tests/`) so the same contract drives
//! probabilistic tests, experiments, and a deployable sentinel.
//!
//! - [`ShoppingBasketServiceContract`] — an LLM that translates natural-language
//!   instructions into structured shopping actions; its target is derived from a
//!   measured baseline (empirical).
//! - [`PaymentGatewayServiceContract`] — a payment charge judged against a
//!   normative SLA pass rate and a latency ceiling.

pub mod payment_gateway;
pub mod sample_sizes;
pub mod shopping_basket;

pub use payment_gateway::{Charge, PaymentGatewayServiceContract};
pub use shopping_basket::ShoppingBasketServiceContract;
