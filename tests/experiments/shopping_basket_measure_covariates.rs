//! Measure experiment verifying covariate encoding in baseline output.
//!
//! The `ShoppingBasketUseCase` declares four covariates:
//! `day-of-week`, `time-of-day`, `llm_model`, `temperature`.
//! The baseline filename and YAML body must reflect all four.

#[path = "../usecases/mod.rs"]
mod usecases;

use feotest::experiment::MeasureExperiment;
use feotest::spec::baseline::BaselineSpec;
use usecases::ShoppingBasketUseCase;
use usecases::shopping_basket::standard_instructions;

/// Verifies that the baseline filename encodes all four covariates
/// and the YAML body contains a covariates block with four entries.
#[test]
fn baseline_encodes_covariates() {
    let inputs = standard_instructions();
    let uc = ShoppingBasketUseCase::new();

    let result = MeasureExperiment::for_use_case(&uc, 100, &inputs, |input| {
        uc.translate_instruction(input)
    })
    .run();

    // The spec should have been written
    let spec_path = result.spec_path().expect("spec should be written");
    assert!(spec_path.exists());

    // Filename: {UseCaseId}-{4-char footprint}-{4 x 4-char cov hashes}.yaml
    let filename = spec_path.file_name().unwrap().to_str().unwrap();
    let stem = filename.trim_end_matches(".yaml");
    let parts: Vec<&str> = stem.split('-').collect();
    // "shopping" + "basket" from the ID... actually the ID is "shopping-basket"
    // which contains hyphens. The sanitizer replaces dots/slashes but keeps hyphens.
    // So the filename starts with "shopping-basket-" then footprint then cov hashes.
    // Let's just count: we expect a footprint (4 chars) + 4 cov hashes (4 chars each).
    // Total hash segments = 5, each 4 chars.
    let hash_parts: Vec<&str> = parts.iter()
        .filter(|p: &&&str| p.len() == 4 && p.chars().all(|c: char| c.is_ascii_hexdigit()))
        .copied()
        .collect();
    assert_eq!(
        hash_parts.len(), 5,
        "expected 1 footprint + 4 covariate hashes, got {hash_parts:?} in filename '{filename}'"
    );

    // YAML body: must contain footprint and covariates block
    let yaml = std::fs::read_to_string(&spec_path).unwrap();
    let spec = BaselineSpec::from_yaml(&yaml).unwrap();

    assert!(spec.footprint.is_some(), "footprint should be present");
    assert_eq!(spec.footprint.as_ref().unwrap().len(), 8, "footprint should be 8 chars");

    assert_eq!(spec.covariates.len(), 4, "should have 4 covariates");
    assert!(spec.covariates.contains_key("day-of-week"));
    assert!(spec.covariates.contains_key("time-of-day"));
    assert!(spec.covariates.contains_key("llm_model"));
    assert!(spec.covariates.contains_key("temperature"));

    // llm_model and temperature should match the use case's configuration
    assert_eq!(spec.covariates["llm_model"], "gpt-4o-mini");
    assert_eq!(spec.covariates["temperature"], "0.3");

    // Content fingerprint should be present
    assert!(spec.content_fingerprint.is_some(), "content fingerprint should be present");

    println!("Baseline written: {filename}");
    println!("Covariates: {:?}", spec.covariates);
}
