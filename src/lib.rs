//! # feotest-examples
//!
//! Example experiments and probabilistic tests demonstrating the
//! [`feotest`] framework.
//!
//! This crate provides two worked examples of probabilistic testing
//! for stochastic services:
//!
//! - **Shopping basket** — an LLM-backed service that translates natural
//!   language instructions into structured shopping actions. The service
//!   is inherently stochastic: the same instruction may produce different
//!   (and sometimes invalid) JSON structures across invocations.
//!
//! - **Payment gateway** — a payment processing service with an SLA-driven
//!   reliability target. The service exhibits low but non-zero failure rates,
//!   demonstrating threshold-first testing against contractual requirements.
//!
//! ## LLM mode
//!
//! The shopping basket example can run against real LLM providers or a
//! built-in mock. The mode is selected via environment variable:
//!
//! ```text
//! # Mock mode (default) — no API keys needed
//! FEOTEST_LLM_MODE=mock cargo test
//!
//! # Real mode — requires API keys
//! FEOTEST_LLM_MODE=real OPENAI_API_KEY=sk-... cargo test
//! ```
//!
//! The mock produces realistic behaviour including temperature-dependent
//! failure rates, making it suitable for demonstrating the framework
//! without incurring API costs.

pub mod llm;
pub mod payment;
pub mod shopping;
