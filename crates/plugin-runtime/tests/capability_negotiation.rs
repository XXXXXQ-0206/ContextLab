//! Contract tests for local plugin-to-MCP capability negotiation.

use contextlab_mcp::{
    CapabilityDescriptor, CapabilityKind, LifecycleContract, LifecyclePhase, PluginId,
    PluginManifest, Version,
};
use contextlab_plugin_runtime::{CapabilityNegotiationDiagnosticCode, McpCapabilityNegotiator};

fn manifest(capabilities: Vec<CapabilityDescriptor>) -> PluginManifest {
    PluginManifest::new(
        1,
        PluginId::new("example.plugin").expect("plugin id"),
        "Example plugin",
        Version::new(1, 0, 0),
        capabilities,
        LifecycleContract::new(
            Version::new(1, 0, 0),
            [
                LifecyclePhase::Initialize,
                LifecyclePhase::Activate,
                LifecyclePhase::Deactivate,
                LifecyclePhase::Shutdown,
            ],
        )
        .expect("lifecycle"),
    )
    .expect("manifest")
}

fn capability(id: &str, kind: CapabilityKind, version: Version) -> CapabilityDescriptor {
    CapabilityDescriptor::new(id, kind, version).expect("capability")
}

#[test]
fn exact_match_negotiates_in_canonical_order() {
    let manifest = manifest(vec![
        capability("zeta.tool", CapabilityKind::Tool, Version::new(2, 4, 6)),
        capability(
            "alpha.resource",
            CapabilityKind::Resource,
            Version::new(1, 2, 3),
        ),
    ]);

    let report = McpCapabilityNegotiator::negotiate(
        "local-mcp.json",
        &manifest,
        r#"{
            "schema_version": "1.0.0",
            "plugin_id": "example.plugin",
            "capabilities": [
                {"id": "zeta.tool", "kind": "tool", "version": "2.4.6"},
                {"id": "alpha.resource", "kind": "resource", "version": "1.2.3"}
            ]
        }"#,
    );

    assert!(report.diagnostics().is_empty());
    assert_eq!(
        report
            .capabilities()
            .iter()
            .map(CapabilityDescriptor::id)
            .collect::<Vec<_>>(),
        vec!["alpha.resource", "zeta.tool"]
    );
}

#[test]
fn malformed_and_unsupported_descriptions_fail_closed_with_structured_diagnostics() {
    let manifest = manifest(Vec::new());

    let malformed = McpCapabilityNegotiator::negotiate(
        "malformed.json",
        &manifest,
        r#"{"schema_version":"1.0.0","plugin_id":"bad plugin","capabilities":[]}"#,
    );
    assert!(malformed.capabilities().is_empty());
    assert_eq!(malformed.diagnostics().len(), 1);
    assert_eq!(malformed.diagnostics()[0].source(), "malformed.json");
    assert_eq!(
        malformed.diagnostics()[0].code(),
        CapabilityNegotiationDiagnosticCode::MalformedDescriptor
    );
    assert!(malformed.diagnostics()[0].capability_id().is_none());

    let unsupported = McpCapabilityNegotiator::negotiate(
        "future.json",
        &manifest,
        r#"{"schema_version":"2.0.0","plugin_id":"example.plugin","capabilities":[]}"#,
    );
    assert!(unsupported.capabilities().is_empty());
    assert_eq!(unsupported.diagnostics().len(), 1);
    assert_eq!(
        unsupported.diagnostics()[0].code(),
        CapabilityNegotiationDiagnosticCode::UnsupportedDescriptorVersion
    );
}

#[test]
fn malformed_descriptor_diagnostics_do_not_echo_untrusted_identifier_contents() {
    let manifest = manifest(Vec::new());
    let sensitive_marker = "credential_should_not_escape";
    let report = McpCapabilityNegotiator::negotiate(
        "malformed.json",
        &manifest,
        &format!(
            r#"{{"schema_version":"1.0.0","plugin_id":"bad {sensitive_marker}","capabilities":[]}}"#
        ),
    );

    assert!(report.capabilities().is_empty());
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].code(),
        CapabilityNegotiationDiagnosticCode::MalformedDescriptor
    );
    assert!(!report.diagnostics()[0].message().contains(sensitive_marker));
}

#[test]
fn mismatched_descriptions_report_canonical_diagnostics_and_negotiate_nothing() {
    let manifest = manifest(vec![
        capability("alpha.tool", CapabilityKind::Tool, Version::new(1, 0, 0)),
        capability(
            "kind.resource",
            CapabilityKind::Resource,
            Version::new(1, 0, 0),
        ),
        capability("missing.tool", CapabilityKind::Tool, Version::new(1, 0, 0)),
        capability("version.tool", CapabilityKind::Tool, Version::new(1, 2, 3)),
    ]);

    let report = McpCapabilityNegotiator::negotiate(
        "mismatch.json",
        &manifest,
        r#"{
            "schema_version": "1.0.0",
            "plugin_id": "example.plugin",
            "capabilities": [
                {"id": "unexpected.tool", "kind": "tool", "version": "1.0.0"},
                {"id": "version.tool", "kind": "tool", "version": "1.2.2"},
                {"id": "kind.resource", "kind": "tool", "version": "1.0.0"},
                {"id": "alpha.tool", "kind": "tool", "version": "1.0.0"}
            ]
        }"#,
    );

    assert!(report.capabilities().is_empty());
    assert_eq!(
        report
            .diagnostics()
            .iter()
            .map(|diagnostic| (
                diagnostic.capability_id().map(|id| id.as_str()),
                diagnostic.code(),
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                Some("kind.resource"),
                CapabilityNegotiationDiagnosticCode::CapabilityKindMismatch,
            ),
            (
                Some("missing.tool"),
                CapabilityNegotiationDiagnosticCode::MissingCapability,
            ),
            (
                Some("unexpected.tool"),
                CapabilityNegotiationDiagnosticCode::UnexpectedCapability,
            ),
            (
                Some("version.tool"),
                CapabilityNegotiationDiagnosticCode::CapabilityVersionMismatch,
            ),
        ]
    );
}

#[test]
fn plugin_identity_mismatch_fails_closed_before_capability_comparison() {
    let manifest = manifest(Vec::new());
    let report = McpCapabilityNegotiator::negotiate(
        "other.json",
        &manifest,
        r#"{"schema_version":"1.0.0","plugin_id":"other.plugin","capabilities":[]}"#,
    );

    assert!(report.capabilities().is_empty());
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].code(),
        CapabilityNegotiationDiagnosticCode::PluginIdMismatch
    );
}
