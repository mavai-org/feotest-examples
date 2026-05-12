//! Covariate declarations: declaring what external conditions affect a service contract.
//!
//! Covariates are environmental variables that the developer does not control
//! but that influence outcomes: the day of the week, the deployment region,
//! the LLM model behind an endpoint. A service contract *declares* which covariates
//! matter; it does not capture their current values (that is a separate
//! concern).
//!
//! Declarations serve three purposes:
//!
//! 1. **Baseline partitioning** — a service contract that performs differently on
//!    weekdays versus weekends can have separate baselines for each condition.
//! 2. **Variance explanation** — when a test fails, the declared covariates
//!    provide context ("this ran on a Sunday in the EU region").
//! 3. **Experiment design** — knowing which covariates matter helps decide
//!    whether to measure under controlled conditions or across the full space.
//!
//! This test file demonstrates how to declare covariates on service contracts and
//! how the framework accesses them. It uses the two example service contracts:
//!
//! - **Shopping basket** (LLM-backed): four covariates mixing built-in
//!   temporal dimensions with custom external-dependency dimensions.
//! - **Payment gateway** (SLA-driven): a single built-in infrastructure
//!   covariate (region).
//!
//! Run with:
//! ```text
//! cargo test --test covariate_declaration_test -- --nocapture
//! ```

mod service_contracts;

use feotest::service_contract::{CovariateCategory, CovariateDeclaration, ServiceContract, validate_covariates};
use service_contracts::{PaymentGatewayServiceContract, ShoppingBasketServiceContract};

// ---------------------------------------------------------------------------
// Covariate resolution: declared covariates must be resolvable
// ---------------------------------------------------------------------------

/// Every declared covariate must have a corresponding resolved value.
/// The shopping basket resolves all four of its declared covariates.
#[test]
fn shopping_basket_resolves_all_declared_covariates() {
    let service_contract = ShoppingBasketServiceContract::new();
    let declarations = service_contract.covariates();
    let profile = service_contract.resolve_covariates();
    let entries = profile.entries();

    assert_eq!(
        entries.len(),
        declarations.len(),
        "every declared covariate must have a resolved value"
    );

    for (decl, (key, _value)) in declarations.iter().zip(entries.iter()) {
        assert_eq!(decl.key(), key);
    }
}

/// The payment gateway resolves its single declared covariate (region).
#[test]
fn payment_gateway_resolves_region() {
    let service_contract = PaymentGatewayServiceContract::new();
    let profile = service_contract.resolve_covariates();
    let entries = profile.entries();

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].0, "region");
    assert_eq!(entries[0].1, "US");
}

/// Resolved values reflect instance state. Two instances configured
/// differently produce the same declarations but different profiles.
#[test]
fn resolved_values_reflect_instance_configuration() {
    let us = PaymentGatewayServiceContract::new().region("US");
    let eu = PaymentGatewayServiceContract::new().region("EU");

    // Same declarations...
    assert_eq!(us.covariates(), eu.covariates());

    // ...different resolved values.
    let us_entries = us.resolve_covariates();
    let eu_entries = eu.resolve_covariates();
    assert_ne!(us_entries.entries(), eu_entries.entries());
}

// ---------------------------------------------------------------------------
// Accessing covariate declarations through the ServiceContract trait
// ---------------------------------------------------------------------------

/// The shopping basket declares four covariates: two built-in temporal
/// dimensions and two custom dimensions for the LLM model and temperature.
#[test]
fn shopping_basket_declares_covariates() {
    let service_contract = ShoppingBasketServiceContract::new();
    let covariates = service_contract.covariates();

    assert_eq!(covariates.len(), 4);

    // Built-in helpers produce standard keys and categories.
    assert_eq!(covariates[0].key(), "day-of-week");
    assert_eq!(covariates[0].category(), CovariateCategory::Temporal);

    assert_eq!(covariates[1].key(), "time-of-day");
    assert_eq!(covariates[1].category(), CovariateCategory::Temporal);

    // Custom covariates use explicit keys and categories.
    assert_eq!(covariates[2].key(), "llm_model");
    assert_eq!(covariates[2].category(), CovariateCategory::ExternalDependency);

    assert_eq!(covariates[3].key(), "temperature");
    assert_eq!(covariates[3].category(), CovariateCategory::ExternalDependency);
}

/// The payment gateway declares a single covariate: the deployment region.
/// Payment processing reliability varies by region (different acquirers,
/// different infrastructure), but not meaningfully by day of week.
#[test]
fn payment_gateway_declares_region_covariate() {
    let service_contract = PaymentGatewayServiceContract::new();
    let covariates = service_contract.covariates();

    assert_eq!(covariates.len(), 1);
    assert_eq!(covariates[0].key(), "region");
    assert_eq!(covariates[0].category(), CovariateCategory::Infrastructure);
}

// ---------------------------------------------------------------------------
// Validation: the framework rejects duplicate covariate keys
// ---------------------------------------------------------------------------

/// Each covariate key must be unique within a service contract. The validation
/// function catches duplicates early, before they cause subtle baseline
/// matching errors downstream.
#[test]
fn covariate_declarations_have_unique_keys() {
    let shopping = ShoppingBasketServiceContract::new();
    validate_covariates(&shopping.covariates());

    let payment = PaymentGatewayServiceContract::new();
    validate_covariates(&payment.covariates());
}

// ---------------------------------------------------------------------------
// Building covariate declarations: built-in helpers vs custom
// ---------------------------------------------------------------------------

/// Built-in helpers create declarations with standard keys that the
/// framework recognises for automatic temporal and infrastructure
/// partitioning. Using them avoids typos and ensures consistent naming
/// across service contracts.
#[test]
fn built_in_helpers_produce_standard_keys() {
    let temporal = vec![
        CovariateDeclaration::day_of_week(),
        CovariateDeclaration::time_of_day(),
    ];
    for cov in &temporal {
        assert_eq!(cov.category(), CovariateCategory::Temporal);
    }

    let infrastructure = vec![
        CovariateDeclaration::region(),
        CovariateDeclaration::timezone(),
    ];
    for cov in &infrastructure {
        assert_eq!(cov.category(), CovariateCategory::Infrastructure);
    }
}

/// Custom covariates use any key and one of the six taxonomy categories.
#[test]
fn custom_covariates_span_the_full_taxonomy() {
    let covariates = vec![
        CovariateDeclaration::new("feature-flag", CovariateCategory::Configuration),
        CovariateDeclaration::new("season", CovariateCategory::Temporal),
        CovariateDeclaration::new("cloud-provider", CovariateCategory::Infrastructure),
        CovariateDeclaration::new("load-level", CovariateCategory::Operational),
        CovariateDeclaration::new("upstream-api-version", CovariateCategory::ExternalDependency),
        CovariateDeclaration::new("training-data-vintage", CovariateCategory::DataState),
    ];

    assert_eq!(covariates.len(), 6);
    validate_covariates(&covariates);
}

// ---------------------------------------------------------------------------
// Declarations are pure metadata
// ---------------------------------------------------------------------------

/// Covariate declarations do not affect service contract behaviour. Two instances
/// with different configurations produce the same declarations — the
/// declarations describe what CAN vary, not what IS varying right now.
#[test]
fn declarations_are_independent_of_instance_state() {
    let low_temp = ShoppingBasketServiceContract::new().temperature(0.1);
    let high_temp = ShoppingBasketServiceContract::new().temperature(0.9);

    assert_eq!(low_temp.covariates(), high_temp.covariates());
}

/// A service contract that declares no covariates returns an empty vector.
/// This is the default for the ServiceContract trait — only override when
/// the service contract genuinely has environmental sensitivities.
#[test]
fn default_is_no_covariates() {
    struct Deterministic;
    impl ServiceContract for Deterministic {
        fn id(&self) -> &str {
            "deterministic-service"
        }
    }

    assert!(Deterministic.covariates().is_empty());
}
