use serde_json::Value;
use tl_parse::{
    parse, parse_with_context, ContextualParseError, ContextualParseReport,
    ContextualParseSchemaVersion, ParseLimits, TL_SYNTAX_REVISION,
};
use tl_syntax::{
    OwnedSignalDeclaration, PropositionBinding, PropositionId, RequirementContextDocument,
    SemanticProfile, SignalCatalogDocument, SignalDomain, SignalId, SourceSpan,
};

fn catalog() -> SignalCatalogDocument {
    SignalCatalogDocument::new(
        vec![
            OwnedSignalDeclaration::new(SignalId(10), "brake".into(), SignalDomain::Boolean),
            OwnedSignalDeclaration::new(SignalId(11), "door".into(), SignalDomain::Boolean),
            OwnedSignalDeclaration::new(SignalId(12), "power".into(), SignalDomain::Boolean),
        ],
        vec![
            PropositionBinding::new(PropositionId(1), SignalId(10)),
            PropositionBinding::new(PropositionId(2), SignalId(11)),
            PropositionBinding::new(PropositionId(3), SignalId(12)),
        ],
    )
    .unwrap()
}

fn context(anchor: &str) -> RequirementContextDocument {
    RequirementContextDocument::new(
        "REQ-42".into(),
        "r7".into(),
        "3.2".into(),
        anchor.into(),
        SourceSpan::new(120, 144).unwrap(),
    )
    .unwrap()
}

fn bind(source: &str, context: Option<&RequirementContextDocument>) -> ContextualParseReport {
    parse_with_context(
        source,
        SemanticProfile::ClosedTraceV1,
        ParseLimits::default(),
        &catalog(),
        context,
    )
    .unwrap()
}

// Trace: TC-029, FR-007-AC-1, StR-003-VC-1
#[test]
fn binds_each_free_proposition_once_in_first_parser_node_order() {
    let caller_context = context("system.brake");
    let report = bind("p2 & (p1 | p2)", Some(&caller_context));

    assert_eq!(report.schema_version, ContextualParseSchemaVersion::V2);
    assert_eq!(report.tl_syntax_revision, TL_SYNTAX_REVISION);
    assert_eq!(report.bindings.len(), 2);
    assert_eq!(report.bindings[0].proposition, PropositionId(2));
    assert_eq!(report.bindings[0].signal, SignalId(11));
    assert_eq!(
        report.bindings[0].parser_span,
        SourceSpan::new(0, 2).unwrap()
    );
    assert_eq!(report.bindings[1].proposition, PropositionId(1));
    assert_eq!(report.bindings[1].signal, SignalId(10));
    assert_eq!(
        report.bindings[1].parser_span,
        SourceSpan::new(6, 8).unwrap()
    );
    assert_eq!(report.signal_catalog, catalog());
    assert_eq!(report.requirement_context, Some(caller_context));
}

// Trace: TC-030, FR-007-AC-2, StR-003-VC-2
#[test]
fn unresolved_proposition_returns_its_exact_parser_span() {
    let error = parse_with_context(
        "p3 | p1",
        SemanticProfile::ClosedTraceV1,
        ParseLimits::default(),
        &SignalCatalogDocument::new(
            vec![OwnedSignalDeclaration::new(
                SignalId(10),
                "brake".into(),
                SignalDomain::Boolean,
            )],
            vec![PropositionBinding::new(PropositionId(1), SignalId(10))],
        )
        .unwrap(),
        None,
    )
    .unwrap_err();
    assert_eq!(
        error,
        ContextualParseError::MissingProposition {
            proposition: PropositionId(3),
            parser_span: SourceSpan::new(0, 2).unwrap(),
        }
    );
}

// Trace: TC-031, FR-007-AC-3, StR-003-VC-1
#[test]
fn retains_shared_requirement_context_without_confusing_its_span() {
    let caller_context = context("power.available");
    let report = bind("p3", Some(&caller_context));
    assert_eq!(report.requirement_context, Some(caller_context));
    assert_eq!(
        report.bindings[0].parser_span,
        SourceSpan::new(0, 2).unwrap()
    );
}

// Trace: TC-032, FR-007-AC-4, StR-003-VC-2
#[test]
fn binding_identities_are_deterministic_and_change_with_declared_inputs() {
    let first_context = context("brake.available");
    let first = bind("p1", Some(&first_context));
    let repeated = bind("p1", Some(&first_context));
    let changed_context = bind("p1", Some(&context("brake.applied")));
    let changed_formula = bind("p2", Some(&first_context));

    assert_eq!(first, repeated);
    assert_ne!(first.request_sha256, changed_context.request_sha256);
    assert_ne!(first.request_sha256, changed_formula.request_sha256);
    assert_eq!(
        first.signal_catalog_sha256,
        changed_formula.signal_catalog_sha256
    );

    let changed_catalog = SignalCatalogDocument::new(
        vec![
            OwnedSignalDeclaration::new(SignalId(11), "door".into(), SignalDomain::Boolean),
            OwnedSignalDeclaration::new(SignalId(20), "brake-v2".into(), SignalDomain::Boolean),
        ],
        vec![
            PropositionBinding::new(PropositionId(1), SignalId(20)),
            PropositionBinding::new(PropositionId(2), SignalId(11)),
        ],
    )
    .unwrap();
    let changed = parse_with_context(
        "p1",
        SemanticProfile::ClosedTraceV1,
        ParseLimits::default(),
        &changed_catalog,
        Some(&first_context),
    )
    .unwrap();
    assert_ne!(first.signal_catalog_sha256, changed.signal_catalog_sha256);
    assert_ne!(first.request_sha256, changed.request_sha256);
}

// Trace: TC-033, FR-007-AC-5, StR-003-VC-2
#[test]
fn v1_parse_wire_remains_compatible_and_v2_binding_wire_is_closed() {
    let legacy = parse("p1", SemanticProfile::ClosedTraceV1, ParseLimits::default());
    let legacy_json = serde_json::to_string(&legacy).unwrap();
    assert_eq!(
        serde_json::from_str::<tl_parse::ParseReport>(&legacy_json).unwrap(),
        legacy
    );

    let report = bind("p1", None);
    let encoded = serde_json::to_string(&report).unwrap();
    assert!(encoded.contains("\"requirementContext\":null"));
    assert_eq!(
        serde_json::from_str::<ContextualParseReport>(&encoded).unwrap(),
        report
    );

    let value: Value = serde_json::from_str(&encoded).unwrap();
    for field in [
        "signalCatalogSha256",
        "requestSha256",
        "requirementContext",
        "formulaDocument",
        "signalCatalog",
    ] {
        let mut missing = value.clone();
        missing.as_object_mut().unwrap().remove(field);
        assert!(
            serde_json::from_value::<ContextualParseReport>(missing).is_err(),
            "{field}"
        );
    }
    let mut unknown = value.clone();
    unknown
        .as_object_mut()
        .unwrap()
        .insert("unknown".into(), Value::Bool(true));
    assert!(serde_json::from_value::<ContextualParseReport>(unknown).is_err());
    let mut wrong_version = value.clone();
    wrong_version["schemaVersion"] = Value::String("tl-parse.contextual-binding/v3".into());
    assert!(serde_json::from_value::<ContextualParseReport>(wrong_version).is_err());
    let mut wrong_revision = value;
    wrong_revision["tlSyntaxRevision"] = Value::String("not-this-pin".into());
    assert!(serde_json::from_value::<ContextualParseReport>(wrong_revision).is_err());

    let mut wrong_catalog_digest: Value = serde_json::from_str(&encoded).unwrap();
    wrong_catalog_digest["signalCatalogSha256"] = Value::String("0".repeat(64));
    assert!(serde_json::from_value::<ContextualParseReport>(wrong_catalog_digest).is_err());
    let mut wrong_request_digest: Value = serde_json::from_str(&encoded).unwrap();
    wrong_request_digest["requestSha256"] = Value::String("0".repeat(64));
    assert!(serde_json::from_value::<ContextualParseReport>(wrong_request_digest).is_err());
    let mut wrong_binding: Value = serde_json::from_str(&encoded).unwrap();
    wrong_binding["bindings"][0]["signal"] = Value::from(999_u32);
    assert!(serde_json::from_value::<ContextualParseReport>(wrong_binding).is_err());
}

// Trace: TC-034, FR-007-AC-6
#[test]
fn contextual_public_surface_uses_shared_types_not_assurance_runtime_types() {
    let manifest = include_str!("../Cargo.toml").to_ascii_lowercase();
    for forbidden in ["quire", "quoin", "engineering-assurance", "contract-ir"] {
        assert!(
            !manifest.contains(forbidden),
            "contextual parsing must not acquire a {forbidden} runtime dependency"
        );
    }
}
