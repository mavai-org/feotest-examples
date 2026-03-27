//! Validation of LLM responses into structured shopping actions.

use feotest::model::ContractViolation;

use crate::llm::ChatResponse;
use crate::shopping::action::ShoppingAction;

/// Validates an LLM chat response, parsing it into a list of shopping actions.
///
/// Expects a JSON response with the structure:
///
/// ```json
/// {"actions": [{"context": "SHOP", "name": "add", "parameters": [...]}]}
/// ```
///
/// # Examples
///
/// ```
/// use feotest_examples::llm::ChatResponse;
/// use feotest_examples::shopping::ShoppingActionValidator;
///
/// let response = ChatResponse::new(
///     r#"{"actions": [{"context": "SHOP", "name": "add", "parameters": []}]}"#,
///     10, 5,
/// );
///
/// let result = ShoppingActionValidator::validate(&response);
/// assert!(result.is_ok());
///
/// let actions = result.unwrap();
/// assert_eq!(actions.len(), 1);
/// assert_eq!(actions[0].name(), "add");
/// ```
pub struct ShoppingActionValidator;

/// Intermediate structure for deserializing the actions wrapper.
#[derive(serde::Deserialize)]
struct ActionsWrapper {
    actions: Vec<ShoppingAction>,
}

impl ShoppingActionValidator {
    /// Validates a chat response, returning the parsed actions or a
    /// contract violation describing what went wrong.
    ///
    /// # Errors
    ///
    /// Returns `Err(ContractViolation)` if:
    /// - The response is empty or blank
    /// - The JSON cannot be parsed
    /// - The response lacks an `"actions"` array
    /// - Any action has an invalid name for its context
    pub fn validate(response: &ChatResponse) -> Result<Vec<ShoppingAction>, ContractViolation> {
        let content = response.content().trim();

        if content.is_empty() {
            return Err(ContractViolation::new("content", "empty response"));
        }

        // Parse the outer JSON structure
        let wrapper: ActionsWrapper = serde_json::from_str(content).map_err(|e| {
            ContractViolation::new("parse", format!("failed to parse response as JSON: {e}"))
        })?;

        if wrapper.actions.is_empty() {
            return Err(ContractViolation::new("actions", "actions array is empty"));
        }

        // Validate each action's name against its context
        for action in &wrapper.actions {
            if !action.is_valid() {
                return Err(ContractViolation::new(
                    "action",
                    format!(
                        "invalid action '{}' for context {:?}",
                        action.name(),
                        action.context()
                    ),
                ));
            }
        }

        Ok(wrapper.actions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_well_formed_response() {
        let response = ChatResponse::new(
            r#"{"actions": [{"context": "SHOP", "name": "add", "parameters": [{"name": "item", "value": "apples"}]}]}"#,
            10,
            5,
        );
        let result = ShoppingActionValidator::validate(&response);
        assert!(result.is_ok());
        let actions = result.unwrap();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].name(), "add");
    }

    #[test]
    fn rejects_empty_response() {
        let response = ChatResponse::new("", 0, 0);
        let result = ShoppingActionValidator::validate(&response);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().check(), "content");
    }

    #[test]
    fn rejects_malformed_json() {
        let response = ChatResponse::new("not json at all", 5, 3);
        let result = ShoppingActionValidator::validate(&response);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().check(), "parse");
    }

    #[test]
    fn rejects_wrong_structure() {
        let response = ChatResponse::new(r#"{"operations": [{"action": "add"}]}"#, 5, 3);
        let result = ShoppingActionValidator::validate(&response);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_invalid_action_name() {
        let response = ChatResponse::new(
            r#"{"actions": [{"context": "SHOP", "name": "purchase", "parameters": []}]}"#,
            10,
            5,
        );
        let result = ShoppingActionValidator::validate(&response);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().check(), "action");
    }

    #[test]
    fn accepts_multiple_valid_actions() {
        let response = ChatResponse::new(
            r#"{"actions": [{"context": "SHOP", "name": "add", "parameters": []}, {"context": "SHOP", "name": "remove", "parameters": []}]}"#,
            10,
            10,
        );
        let result = ShoppingActionValidator::validate(&response);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }
}
