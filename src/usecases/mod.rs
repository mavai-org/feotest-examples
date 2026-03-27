//! Use case implementations for probabilistic testing.
//!
//! Each use case encapsulates a service invocation, its configuration
//! surface, and its contract (postconditions). Use cases are consumed by
//! experiments and probabilistic tests via feotest's builder APIs.

mod payment_gateway;
pub mod shopping_basket;

pub use payment_gateway::PaymentGatewayUseCase;
pub use shopping_basket::ShoppingBasketUseCase;
