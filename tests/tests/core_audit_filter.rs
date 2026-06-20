// T-CORE-AF-01~11: AuditFilter
use aidaguard_core::storage_types::AuditFilter;

#[test] fn test_audit_filter_new_empty() {
    let f = AuditFilter::new();
    assert!(f.rule_id.is_none());
    assert!(f.path.is_none());
    assert!(f.date_from_ms.is_none());
    assert!(f.date_to_ms.is_none());
    assert!(f.strategy.is_none());
    assert!(f.tool_name.is_none());
}

#[test] fn test_audit_filter_by_rule() {
    let f = AuditFilter::by_rule("R001");
    assert_eq!(f.rule_id.as_deref(), Some("R001"));
    assert!(f.path.is_none());
}

#[test] fn test_audit_filter_by_date_range() {
    let f = AuditFilter::by_date_range(1000, 2000);
    assert_eq!(f.date_from_ms, Some(1000));
    assert_eq!(f.date_to_ms, Some(2000));
    assert!(f.rule_id.is_none());
}

#[test] fn test_audit_filter_with_rule_chained() {
    let f = AuditFilter::new().with_rule("R002");
    assert_eq!(f.rule_id.as_deref(), Some("R002"));
}

#[test] fn test_audit_filter_with_path_chained() {
    let f = AuditFilter::new().with_path("/v1/chat");
    assert_eq!(f.path.as_deref(), Some("/v1/chat"));
}

#[test] fn test_audit_filter_with_date_range_chained() {
    let f = AuditFilter::new().with_date_range(3000, 4000);
    assert_eq!(f.date_from_ms, Some(3000));
    assert_eq!(f.date_to_ms, Some(4000));
}

#[test] fn test_audit_filter_with_strategy() {
    let f = AuditFilter::new().with_strategy("mask");
    assert_eq!(f.strategy.as_deref(), Some("mask"));
}

#[test] fn test_audit_filter_with_tool() {
    let f = AuditFilter::new().with_tool("cursor");
    assert_eq!(f.tool_name.as_deref(), Some("cursor"));
}

#[test] fn test_audit_filter_full_builder_chain() {
    let f = AuditFilter::new()
        .with_rule("R003")
        .with_path("/api")
        .with_date_range(5000, 6000)
        .with_strategy("placeholder")
        .with_tool("cline");
    assert_eq!(f.rule_id.as_deref(), Some("R003"));
    assert_eq!(f.path.as_deref(), Some("/api"));
    assert_eq!(f.date_from_ms, Some(5000));
    assert_eq!(f.date_to_ms, Some(6000));
    assert_eq!(f.strategy.as_deref(), Some("placeholder"));
    assert_eq!(f.tool_name.as_deref(), Some("cline"));
}

#[test] fn test_audit_filter_by_rule_then_with_rule_override() {
    let f = AuditFilter::by_rule("R_OLD").with_rule("R_NEW");
    assert_eq!(f.rule_id.as_deref(), Some("R_NEW"));
}

#[test] fn test_audit_filter_default_matches_new() {
    let f_new = AuditFilter::new();
    let f_default = AuditFilter::default();
    assert_eq!(f_new.rule_id.is_none(), f_default.rule_id.is_none());
    assert_eq!(f_new.path.is_none(), f_default.path.is_none());
    assert_eq!(f_new.date_from_ms.is_none(), f_default.date_from_ms.is_none());
    assert_eq!(f_new.date_to_ms.is_none(), f_default.date_to_ms.is_none());
    assert_eq!(f_new.strategy.is_none(), f_default.strategy.is_none());
    assert_eq!(f_new.tool_name.is_none(), f_default.tool_name.is_none());
}
