//! Use case adapters for probabilistic testing.
//!
//! Each use case encapsulates a service invocation, its configuration
//! surface, and its contract (postconditions). Use cases are consumed by
//! experiments and probabilistic tests via feotest's builder and macro APIs.

// This module is compiled independently by each test binary, which may
// use only a subset. Suppress warnings for the unused portions.
#![allow(dead_code, unused_imports)]

mod payment_gateway;
pub mod shopping_basket;

pub use payment_gateway::PaymentGatewayUseCase;
pub use shopping_basket::ShoppingBasketUseCase;
