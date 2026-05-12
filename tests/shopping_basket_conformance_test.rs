//! Expected-output matching tests for the shopping basket service contract.
//!
//! Demonstrates CT04 (expected-output matching) by comparing LLM
//! responses against a golden dataset of known-correct expected outputs.
//!
//! Unlike postcondition-based tests (which check abstract properties like
//! "is the response valid JSON?"), expected-output matching checks a stronger
//! property: "does this response match the specific expected output for
//! this specific input?"
//!
//! Run with:
//! ```text
//! cargo test --test shopping_basket_conformance_test -- --nocapture
//! ```

mod service_contracts;

use feotest::contract::conformance::{StringMatcher, VerificationMatcher};
use feotest::contract::json_matcher::JsonMatcher;
use feotest::contract::{MatchResult, ServiceContract, ServiceContractOutcome};
use feotest::model::ContractViolation;

use feotest_examples::llm::ChatLlmProvider;
use serde::Deserialize;
use service_contracts::ShoppingBasketServiceContract;

/// A golden dataset entry: an instruction paired with its expected output.
#[derive(Debug, Clone, Deserialize)]
struct GoldenInput {
    instruction: String,
    expected: String,
}

/// Loads the golden dataset from the fixtures directory.
fn load_golden_dataset() -> Vec<GoldenInput> {
    let content = include_str!("fixtures/shopping-instructions.json");
    serde_json::from_str(content).expect("golden dataset should be valid JSON")
}

// ---------------------------------------------------------------------------
// Expected-output matching with golden dataset using JsonMatcher
// ---------------------------------------------------------------------------

/// Demonstrates expected-output matching against a golden dataset.
///
/// Each golden input has a known expected JSON output. The test sends the
/// instruction to the LLM, captures the raw response, evaluates it against
/// postconditions, and then compares it against the expected output using
/// semantic JSON comparison (property order and whitespace are ignored).
///
/// With the mock LLM at temperature 0.0, responses are deterministic and
/// match rates are high. At higher temperatures, mismatches are expected —
/// that is the point of measuring them.
#[test]
fn golden_dataset_json_conformance() {
    let golden = load_golden_dataset();
    let llm = ChatLlmProvider::resolve();
    let system_prompt = ShoppingBasketServiceContract::default_system_prompt();
    let matcher = JsonMatcher::new();

    let contract = ServiceContract::<String, String>::builder()
        .ensure("has content", |_input, response: &String| {
            if response.trim().is_empty() {
                Err(ContractViolation::new("content", "empty response"))
            } else {
                Ok(())
            }
        })
        .build();

    let mut total = 0u32;
    let mut expected_matches = 0u32;

    for entry in &golden {
        // Check the LLM response against the contract's postconditions.
        // The closure calls the LLM directly so the raw response string
        // is captured as the outcome's response — available for matching
        // against expected output in the next step.
        let contract_outcome = ServiceContractOutcome::evaluate(
            &contract,
            &entry.instruction,
            || llm.chat(&system_prompt, &entry.instruction, "gpt-4o-mini", 0.0)
                .content()
                .to_string(),
        );

        if contract_outcome.violation().is_some() {
            continue;
        }

        // Match the actual LLM response against the golden expected output
        let outcome = contract_outcome.conforms_to(
            &entry.expected,
            |response| response.clone(),
            &matcher,
        );

        total += 1;
        if outcome.matches_expected() {
            expected_matches += 1;
        }
    }

    println!(
        "Golden dataset: {expected_matches}/{total} matched expected output ({} entries)",
        golden.len()
    );
    assert!(total > 0, "at least one golden input should produce a response");
}

// ---------------------------------------------------------------------------
// Matcher strategy demonstrations
// ---------------------------------------------------------------------------

/// Demonstrates the exact string matcher.
#[test]
fn exact_string_matching() {
    let matcher = StringMatcher::exact();
    let result = matcher.verify("hello world", "hello world");
    assert!(result.is_match());

    let result = matcher.verify("hello world", "Hello World");
    assert!(result.is_mismatch());
    println!("Exact mismatch diff: {}", result.diff());
}

/// Demonstrates the case-insensitive string matcher.
#[test]
fn case_insensitive_matching() {
    let matcher = StringMatcher::ignore_case();
    let result = matcher.verify("Hello World", "hello world");
    assert!(result.is_match());

    let result = matcher.verify("hello", "goodbye");
    assert!(result.is_mismatch());
}

/// Demonstrates the whitespace-normalising string matcher.
#[test]
fn whitespace_normalised_matching() {
    let matcher = StringMatcher::normalize_whitespace();
    let result = matcher.verify("hello   world", "  hello world  ");
    assert!(result.is_match());
}

/// Demonstrates semantic JSON comparison.
///
/// JSON matching ignores property ordering and insignificant whitespace,
/// making it ideal for comparing structured LLM outputs where the content
/// is correct but the serialisation order varies between invocations.
#[test]
fn json_structural_matching() {
    let matcher = JsonMatcher::new();

    // Same content, different property order — should match
    let result = matcher.verify(
        r#"{"name": "Alice", "age": 30}"#,
        r#"{"age": 30, "name": "Alice"}"#,
    );
    assert!(result.is_match());

    // Different values — should mismatch with a descriptive diff
    let result = matcher.verify(r#"{"name": "Alice"}"#, r#"{"name": "Bob"}"#);
    assert!(result.is_mismatch());
    println!("JSON mismatch diff: {}", result.diff());
    assert!(result.diff().contains("/name"));
}

/// Demonstrates a custom matcher implementation.
///
/// Custom matchers implement the `VerificationMatcher` trait, enabling
/// domain-specific comparison logic beyond the built-in strategies.
#[test]
fn custom_matcher_implementation() {
    /// A matcher that checks whether two shopping action JSON strings
    /// have the same number of actions, regardless of the action details.
    struct ActionCountMatcher;

    impl VerificationMatcher<str> for ActionCountMatcher {
        fn verify(&self, expected: &str, actual: &str) -> MatchResult {
            let expected_count = expected.matches("\"name\"").count();
            let actual_count = actual.matches("\"name\"").count();

            if expected_count == actual_count {
                MatchResult::matched()
            } else {
                MatchResult::mismatch(format!(
                    "expected {expected_count} actions, got {actual_count}"
                ))
            }
        }
    }

    let matcher = ActionCountMatcher;

    // Same number of actions
    let result = matcher.verify(
        r#"{"actions":[{"name":"add"},{"name":"remove"}]}"#,
        r#"{"actions":[{"name":"clear"},{"name":"add"}]}"#,
    );
    assert!(result.is_match());

    // Different number of actions
    let result = matcher.verify(
        r#"{"actions":[{"name":"add"}]}"#,
        r#"{"actions":[{"name":"add"},{"name":"remove"}]}"#,
    );
    assert!(result.is_mismatch());
    println!("Custom matcher diff: {}", result.diff());
}

// ---------------------------------------------------------------------------
// Expected-output matching on ServiceContractOutcome
// ---------------------------------------------------------------------------

/// Demonstrates attaching an expected-output match to a `ServiceContractOutcome`
/// and verifying that all three aspects — postconditions, duration, and
/// expected-output matching — contribute independently to the overall
/// success verdict.
#[test]
fn multi_aspect_outcome() {
    use std::time::Duration;

    let contract = ServiceContract::<String, String>::builder()
        .ensure("not empty", |_input, response: &String| {
            if response.is_empty() {
                Err(ContractViolation::new("content", "empty"))
            } else {
                Ok(())
            }
        })
        .ensure_duration_below(Duration::from_secs(5))
        .build();

    // All three aspects pass
    let contract_outcome = ServiceContractOutcome::from_response(
        &contract,
        &"Add apples".to_string(),
        r#"{"actions":[{"context":"SHOP","name":"add","parameters":[]}]}"#.to_string(),
        Duration::from_millis(50),
    );

    // Postconditions and duration have been checked.
    // This outcome has not yet been matched against an expected value.
    assert!(contract_outcome.violation().is_none());
    assert!(contract_outcome.within_duration_limit());
    assert!(contract_outcome.conformance_result().is_none());

    // Add the third aspect: match against expected output
    let outcome = contract_outcome.conforms_to(
        &r#"{"actions":[{"context":"SHOP","name":"add","parameters":[]}]}"#.to_string(),
        |r| r.clone(),
        &JsonMatcher::new(),
    );

    // All three aspects now present and passing
    assert!(outcome.is_success());
    assert!(outcome.matches_expected());
    assert!(outcome.within_duration_limit());
    assert!(outcome.violation().is_none());

    // Match against a different expected value.
    // The expected output deliberately differs from the response
    // to demonstrate that the mismatch is detected.
    let contract_outcome = ServiceContractOutcome::from_response(
        &contract,
        &"Add apples".to_string(),
        r#"{"actions":[{"context":"SHOP","name":"add","parameters":[]}]}"#.to_string(),
        Duration::from_millis(50),
    );

    // Postconditions and duration pass, but not yet matched against an expected value
    assert!(contract_outcome.violation().is_none());
    assert!(contract_outcome.within_duration_limit());
    assert!(contract_outcome.conformance_result().is_none());

    // Match against expected output (deliberately different)
    let outcome = contract_outcome.conforms_to(
        &r#"{"actions":[{"context":"SHOP","name":"remove","parameters":[]}]}"#.to_string(),
        |r| r.clone(),
        &JsonMatcher::new(),
    );

    // For demo purposes the following is_success() is false because matches_expected()
    // is false: the actual response contains "add" but the expected value contains "remove".
    // Postconditions and duration still pass independently.
    assert!(!outcome.is_success());
    assert!(!outcome.matches_expected());
    assert!(outcome.within_duration_limit());
    assert!(outcome.violation().is_none());

    let cr = outcome.conformance_result().unwrap();
    assert!(cr.match_result().is_mismatch());
    println!("Conformance diff: {}", cr.match_result().diff());
}
