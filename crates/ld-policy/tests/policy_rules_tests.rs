use ld_policy::rules::{PolicyEngine, RiskLevel, ToolPermission};

#[test]
fn test_policy_engine_risk_levels_and_tool_allowlists() {
    let engine = PolicyEngine::for_brain("coding");

    assert_eq!(
        engine.evaluate_tool("read_file"),
        ToolPermission::Allow(RiskLevel::ReadOnly)
    );
    assert_eq!(
        engine.evaluate_tool("write_file"),
        ToolPermission::Allow(RiskLevel::Medium)
    );
    assert_eq!(
        engine.evaluate_tool("run_command"),
        ToolPermission::RequiresApproval(RiskLevel::High)
    );
    assert_eq!(
        engine.evaluate_tool("delete_database"),
        ToolPermission::Deny(RiskLevel::Dangerous)
    );
}

#[test]
fn test_policy_engine_unknown_tool_requires_approval() {
    let engine = PolicyEngine::for_brain("coding");
    assert_eq!(
        engine.evaluate_tool("custom_unknown_tool"),
        ToolPermission::RequiresApproval(RiskLevel::Medium)
    );
}
