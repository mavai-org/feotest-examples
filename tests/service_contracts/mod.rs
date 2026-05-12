//! Service contract adapters for probabilistic testing.
//!
//! Each service contract encapsulates a service invocation, its configuration
//! surface, and its service contract. Service contracts are consumed by
//! experiments and probabilistic tests via feotest's builder and macro APIs.

// This module is compiled independently by each test binary, which may
// use only a subset. Suppress warnings for the unused portions.
#![allow(dead_code, unused_imports)]

mod payment_gateway;
pub mod shopping_basket;

pub use payment_gateway::PaymentGatewayServiceContract;
pub use shopping_basket::ShoppingBasketServiceContract;
