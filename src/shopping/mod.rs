//! Shopping domain types and validation.
//!
//! This module models a service that translates natural language shopping
//! instructions into structured actions. The LLM receives an instruction
//! like "Add 2 apples" and produces a JSON response containing an array
//! of [`ShoppingAction`]s, each with a context, action name, and parameters.
//!
//! The [`ShoppingActionValidator`] parses and validates LLM responses,
//! producing either a list of valid actions or a contract violation
//! describing what went wrong.

mod action;
mod validator;

pub use action::{AppContext, ShoppingAction, ShoppingActionParameter};
pub use validator::ShoppingActionValidator;
