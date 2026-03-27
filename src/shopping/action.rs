//! Shopping action domain types.

use serde::Deserialize;

/// The application context in which a shopping action operates.
///
/// Each context defines a set of valid action names.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AppContext {
    /// Shopping context. Valid actions: `add`, `remove`, `clear`.
    Shop,
    /// Recipe context. Valid actions: `suggest`, `filter`, `save`, `share`.
    Recipe,
}

impl AppContext {
    /// Returns whether the given action name is valid for this context.
    ///
    /// # Examples
    ///
    /// ```
    /// use feotest_examples::shopping::AppContext;
    ///
    /// assert!(AppContext::Shop.is_valid_action("add"));
    /// assert!(AppContext::Shop.is_valid_action("remove"));
    /// assert!(AppContext::Shop.is_valid_action("clear"));
    /// assert!(!AppContext::Shop.is_valid_action("purchase"));
    ///
    /// assert!(AppContext::Recipe.is_valid_action("suggest"));
    /// assert!(!AppContext::Recipe.is_valid_action("add"));
    /// ```
    #[must_use]
    pub fn is_valid_action(&self, name: &str) -> bool {
        match self {
            Self::Shop => matches!(name, "add" | "remove" | "clear"),
            Self::Recipe => matches!(name, "suggest" | "filter" | "save" | "share"),
        }
    }
}

/// A structured shopping action parsed from an LLM response.
///
/// Each action has a context (e.g., `Shop`), a name (e.g., `"add"`),
/// and a list of parameters (e.g., item name and quantity).
///
/// # Examples
///
/// ```
/// use feotest_examples::shopping::{AppContext, ShoppingAction, ShoppingActionParameter};
///
/// let action = ShoppingAction::new(
///     AppContext::Shop,
///     "add".to_string(),
///     vec![
///         ShoppingActionParameter::new("item", "apples"),
///         ShoppingActionParameter::new("quantity", "2"),
///     ],
/// );
///
/// assert_eq!(action.name(), "add");
/// assert!(action.is_valid());
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct ShoppingAction {
    context: AppContext,
    name: String,
    #[serde(default)]
    parameters: Vec<ShoppingActionParameter>,
}

impl ShoppingAction {
    /// Creates a new shopping action.
    #[must_use]
    pub const fn new(
        context: AppContext,
        name: String,
        parameters: Vec<ShoppingActionParameter>,
    ) -> Self {
        Self {
            context,
            name,
            parameters,
        }
    }

    /// The application context.
    #[must_use]
    pub const fn context(&self) -> &AppContext {
        &self.context
    }

    /// The action name (e.g., `"add"`, `"remove"`, `"clear"`).
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The action parameters.
    #[must_use]
    pub fn parameters(&self) -> &[ShoppingActionParameter] {
        &self.parameters
    }

    /// Whether this action's name is valid for its context.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.context.is_valid_action(&self.name)
    }
}

/// A name-value parameter attached to a shopping action.
///
/// # Examples
///
/// ```
/// use feotest_examples::shopping::ShoppingActionParameter;
///
/// let param = ShoppingActionParameter::new("quantity", "3");
/// assert_eq!(param.name(), "quantity");
/// assert_eq!(param.value(), "3");
/// assert_eq!(param.value_as_int(), Some(3));
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct ShoppingActionParameter {
    name: String,
    value: String,
}

impl ShoppingActionParameter {
    /// Creates a new parameter.
    #[must_use]
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }

    /// The parameter name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The parameter value as a string.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Attempts to parse the value as an integer.
    #[must_use]
    pub fn value_as_int(&self) -> Option<i64> {
        self.value.parse().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shop_context_validates_actions() {
        assert!(AppContext::Shop.is_valid_action("add"));
        assert!(AppContext::Shop.is_valid_action("remove"));
        assert!(AppContext::Shop.is_valid_action("clear"));
        assert!(!AppContext::Shop.is_valid_action("purchase"));
        assert!(!AppContext::Shop.is_valid_action("buy"));
    }

    #[test]
    fn recipe_context_validates_actions() {
        assert!(AppContext::Recipe.is_valid_action("suggest"));
        assert!(!AppContext::Recipe.is_valid_action("add"));
    }

    #[test]
    fn action_validity_checks_context() {
        let valid = ShoppingAction::new(AppContext::Shop, "add".into(), vec![]);
        assert!(valid.is_valid());

        let invalid = ShoppingAction::new(AppContext::Shop, "purchase".into(), vec![]);
        assert!(!invalid.is_valid());
    }

    #[test]
    fn deserializes_action_from_json() {
        let json = r#"{"context": "SHOP", "name": "add", "parameters": [{"name": "item", "value": "apples"}]}"#;
        let action: ShoppingAction = serde_json::from_str(json).unwrap();
        assert_eq!(action.context(), &AppContext::Shop);
        assert_eq!(action.name(), "add");
        assert_eq!(action.parameters().len(), 1);
    }

    #[test]
    fn parameter_value_as_int() {
        let p = ShoppingActionParameter::new("qty", "3");
        assert_eq!(p.value_as_int(), Some(3));

        let p = ShoppingActionParameter::new("item", "apples");
        assert!(p.value_as_int().is_none());
    }
}
