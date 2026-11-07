use ld_schemas::*;
use std::str::FromStr;

#[test]
fn test_serde_turn_and_completion() {
    let turn = Turn {
        role: Role::User,
        content: "Explain auth flow".to_string(),
        tool_calls: vec![],
        timestamp: "2026-11-07T12:00:00Z".to_string(),
    };
    let json = serde_json::to_string(&turn).unwrap();
    let deserialized: Turn = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.role, Role::User);
    assert_eq!(deserialized.content, "Explain auth flow");
    assert_eq!(deserialized.timestamp, "2026-11-07T12:00:00Z");
    assert!(deserialized.tool_calls.is_empty());
}

#[test]
fn test_turn_with_tool_calls() {
    let tool_call = ToolCall::new("call_123", "read_file", r#"{"path":"Cargo.toml"}"#);
    let mut turn = Turn::new(Role::Assistant, "Checking file...");
    turn.tool_calls.push(tool_call.clone());

    let json = serde_json::to_string(&turn).unwrap();
    let deserialized: Turn = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.role, Role::Assistant);
    assert_eq!(deserialized.tool_calls.len(), 1);
    assert_eq!(deserialized.tool_calls[0].id, "call_123");
    assert_eq!(deserialized.tool_calls[0].name, "read_file");
    assert_eq!(deserialized.tool_calls[0].arguments, r#"{"path":"Cargo.toml"}"#);
}

#[test]
fn test_brain_type_roundtrip_and_aliases() {
    let brains = [BrainType::Coding, BrainType::Design, BrainType::General];
    for b in &brains {
        let json = serde_json::to_string(b).unwrap();
        let de: BrainType = serde_json::from_str(&json).unwrap();
        assert_eq!(*b, de);
        assert_eq!(b.to_string(), format!("{}", b));
        assert_eq!(BrainType::from_str(&b.to_string()).unwrap(), *b);
    }

    // Test case insensitive & uppercase deserialization
    assert_eq!(serde_json::from_str::<BrainType>("\"coding\"").unwrap(), BrainType::Coding);
    assert_eq!(serde_json::from_str::<BrainType>("\"Coding\"").unwrap(), BrainType::Coding);
    assert_eq!(serde_json::from_str::<BrainType>("\"CODING\"").unwrap(), BrainType::Coding);
    assert_eq!(serde_json::from_str::<BrainType>("\"design\"").unwrap(), BrainType::Design);
    assert_eq!(serde_json::from_str::<BrainType>("\"general\"").unwrap(), BrainType::General);
}

#[test]
fn test_role_roundtrip_and_aliases() {
    let roles = [Role::System, Role::User, Role::Assistant, Role::Tool];
    for r in &roles {
        let json = serde_json::to_string(r).unwrap();
        let de: Role = serde_json::from_str(&json).unwrap();
        assert_eq!(*r, de);
        assert_eq!(Role::from_str(&r.to_string()).unwrap(), *r);
    }

    assert_eq!(serde_json::from_str::<Role>("\"system\"").unwrap(), Role::System);
    assert_eq!(serde_json::from_str::<Role>("\"System\"").unwrap(), Role::System);
    assert_eq!(serde_json::from_str::<Role>("\"user\"").unwrap(), Role::User);
    assert_eq!(serde_json::from_str::<Role>("\"assistant\"").unwrap(), Role::Assistant);
    assert_eq!(serde_json::from_str::<Role>("\"tool\"").unwrap(), Role::Tool);
}

#[test]
fn test_privacy_class_roundtrip() {
    let classes = [
        PrivacyClass::Local,
        PrivacyClass::RemoteApproved,
        PrivacyClass::RemoteUnapproved,
    ];
    for c in &classes {
        let json = serde_json::to_string(c).unwrap();
        let de: PrivacyClass = serde_json::from_str(&json).unwrap();
        assert_eq!(*c, de);
        assert_eq!(PrivacyClass::from_str(&c.to_string()).unwrap(), *c);
    }

    assert_eq!(
        serde_json::from_str::<PrivacyClass>("\"local\"").unwrap(),
        PrivacyClass::Local
    );
    assert_eq!(
        serde_json::from_str::<PrivacyClass>("\"remote_approved\"").unwrap(),
        PrivacyClass::RemoteApproved
    );
    assert_eq!(
        serde_json::from_str::<PrivacyClass>("\"remoteApproved\"").unwrap(),
        PrivacyClass::RemoteApproved
    );
}

#[test]
fn test_health_status_roundtrip() {
    let statuses = [
        HealthStatus::Healthy,
        HealthStatus::Degraded,
        HealthStatus::Unhealthy,
        HealthStatus::Unknown,
    ];
    for s in &statuses {
        let json = serde_json::to_string(s).unwrap();
        let de: HealthStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(*s, de);
        assert_eq!(HealthStatus::from_str(&s.to_string()).unwrap(), *s);
    }

    assert_eq!(
        serde_json::from_str::<HealthStatus>("\"healthy\"").unwrap(),
        HealthStatus::Healthy
    );
    assert_eq!(
        serde_json::from_str::<HealthStatus>("\"degraded\"").unwrap(),
        HealthStatus::Degraded
    );
}

#[test]
fn test_model_capabilities_roundtrip() {
    let caps = ModelCapabilities {
        alias: "coder-local".to_string(),
        endpoint: "http://127.0.0.1:8741".to_string(),
        model_name: "ldrago-coder-v1".to_string(),
        capabilities: vec!["completion".to_string(), "code_editing".to_string()],
        context_window: 8192,
        privacy: PrivacyClass::Local,
        brain: Some(BrainType::Coding),
    };

    let json = serde_json::to_string_pretty(&caps).unwrap();
    let deserialized: ModelCapabilities = serde_json::from_str(&json).unwrap();
    assert_eq!(caps, deserialized);
    assert_eq!(deserialized.context_window, 8192);
    assert_eq!(deserialized.brain, Some(BrainType::Coding));
}

#[test]
fn test_completion_request_roundtrip() {
    let mut req = CompletionRequest::new("Write a fibonacci function in Rust", BrainType::Coding);
    req.max_tokens = Some(256);
    req.temperature = Some(0.2);
    req.stream = true;

    let json = serde_json::to_string(&req).unwrap();
    let deserialized: CompletionRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(req, deserialized);
    assert_eq!(deserialized.max_tokens, Some(256));
    assert_eq!(deserialized.stream, true);

    // Test minimal json deserialization defaults
    let minimal_json = r#"{"prompt":"hello","brain":"design"}"#;
    let minimal_req: CompletionRequest = serde_json::from_str(minimal_json).unwrap();
    assert_eq!(minimal_req.prompt, "hello");
    assert_eq!(minimal_req.brain, BrainType::Design);
    assert_eq!(minimal_req.max_tokens, None);
    assert_eq!(minimal_req.temperature, None);
    assert_eq!(minimal_req.stream, false);
}

#[test]
fn test_completion_response_roundtrip() {
    let mut resp = CompletionResponse::new("fn fib(n: u64) -> u64 { ... }", BrainType::Coding);
    resp.tokens_generated = Some(42);

    let json = serde_json::to_string(&resp).unwrap();
    let deserialized: CompletionResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(resp, deserialized);
    assert_eq!(deserialized.tokens_generated, Some(42));
}

#[test]
fn test_completion_chunk_roundtrip() {
    let chunk = CompletionChunk::new("let x = 10;", BrainType::Coding, false);
    let json = serde_json::to_string(&chunk).unwrap();
    let deserialized: CompletionChunk = serde_json::from_str(&json).unwrap();
    assert_eq!(chunk, deserialized);
    assert_eq!(deserialized.done, false);

    let end_chunk = CompletionChunk::new("", BrainType::Coding, true);
    let end_json = serde_json::to_string(&end_chunk).unwrap();
    let end_deserialized: CompletionChunk = serde_json::from_str(&end_json).unwrap();
    assert_eq!(end_chunk, end_deserialized);
    assert_eq!(end_deserialized.done, true);
}
