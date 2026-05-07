// T-CORE-19~20: DetectionEngine trait
use aidaguard_core::detector::Detector;
use aidaguard_core::DetectionEngine;

#[test]
fn test_detector_impls_engine() {
    fn accept_engine(_engine: &dyn DetectionEngine) {}
    let detector = Detector::new();
    accept_engine(&detector);
}

#[test]
fn test_engine_trait_object() {
    let mut engine: Box<dyn DetectionEngine> = Box::new(Detector::new());
    assert_eq!(engine.rule_count(), 0);

    let hits = engine.detect("no sensitive data");
    assert!(hits.is_empty());

    let result = engine.reload(std::path::Path::new("/nonexistent_dir_xyz"));
    assert!(result.is_err());
}
