//! Contract tests for provider-free MCP manifests and capability discovery.

use contextlab_mcp::{
    CapabilityAvailability, CapabilityAvailabilityProjection, CapabilityCompatibility,
    CapabilityDescriptor, CapabilityKind, CapabilityRegistry, CompatibilityError,
    DiscoveryCandidate, LifecycleContract, LifecyclePhase, PluginId, PluginManifest,
    RuntimeCompatibility, Version,
};
use serde_json::json;

fn version(major: u64, minor: u64, patch: u64) -> Version {
    Version::new(major, minor, patch)
}

fn lifecycle(version: Version) -> LifecycleContract {
    LifecycleContract::new(
        version,
        [
            LifecyclePhase::Shutdown,
            LifecyclePhase::Activate,
            LifecyclePhase::Initialize,
            LifecyclePhase::Deactivate,
        ],
    )
    .expect("valid lifecycle contract")
}

fn manifest(
    id: &str,
    manifest_version: u16,
    lifecycle_version: Version,
    capabilities: Vec<CapabilityDescriptor>,
) -> PluginManifest {
    PluginManifest::new(
        manifest_version,
        PluginId::new(id).expect("valid plugin id"),
        format!("{id} plugin"),
        version(1, 0, 0),
        capabilities,
        lifecycle(lifecycle_version),
    )
    .expect("valid plugin manifest")
}

fn capability(id: &str, kind: CapabilityKind) -> CapabilityDescriptor {
    CapabilityDescriptor::new(id, kind, version(1, 0, 0)).expect("valid capability")
}

#[test]
fn registry_projects_canonical_v1_available_capabilities() {
    let mut registry = CapabilityRegistry::new();
    registry
        .register(manifest(
            "zeta.plugin",
            1,
            version(1, 0, 0),
            vec![capability("zeta.tool", CapabilityKind::Tool)],
        ))
        .expect("zeta manifest registers");
    registry
        .register(manifest(
            "alpha.plugin",
            1,
            version(1, 0, 0),
            vec![capability("alpha.tool", CapabilityKind::Tool)],
        ))
        .expect("alpha manifest registers");

    let projection = registry.capability_availability();

    assert_eq!(projection.version(), Version::new(1, 0, 0));
    assert_eq!(
        projection
            .entries()
            .iter()
            .map(|entry| (
                entry.plugin_id().as_str(),
                entry.capability_id().as_str(),
                entry.capability_version(),
                entry.availability(),
                entry.compatibility(),
                entry.diagnostic_code(),
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                "alpha.plugin",
                "alpha.tool",
                Version::new(1, 0, 0),
                CapabilityAvailability::Available,
                CapabilityCompatibility::Compatible,
                None,
            ),
            (
                "zeta.plugin",
                "zeta.tool",
                Version::new(1, 0, 0),
                CapabilityAvailability::Available,
                CapabilityCompatibility::Compatible,
                None,
            ),
        ]
    );
}

#[test]
fn capability_availability_projection_rejects_non_v1_schema() {
    let error = serde_json::from_value::<CapabilityAvailabilityProjection>(json!({
        "version": "2.0.0",
        "entries": [],
    }))
    .expect_err("external adapters must reject unsupported projection schemas");

    assert!(error.to_string().contains("unsupported"));
}

#[test]
fn manifest_parser_is_versioned_and_canonicalizes_capability_order() {
    let manifest = manifest(
        "example.plugin",
        1,
        version(1, 0, 0),
        vec![
            capability("zeta", CapabilityKind::Tool),
            capability("alpha", CapabilityKind::Resource),
        ],
    );

    assert_eq!(
        manifest
            .capabilities()
            .iter()
            .map(|item| item.id())
            .collect::<Vec<_>>(),
        vec!["alpha", "zeta"]
    );

    let encoded = serde_json::to_value(&manifest).expect("manifest serializes");
    let decoded = serde_json::from_value::<PluginManifest>(encoded).expect("manifest parses");

    assert_eq!(decoded, manifest);
    assert_eq!(decoded.manifest_version(), 1);
}

#[test]
fn discovery_is_source_sorted_and_keeps_invalid_candidates_isolated() {
    let valid_alpha = manifest(
        "alpha.plugin",
        1,
        version(1, 0, 0),
        vec![capability("alpha.tool", CapabilityKind::Tool)],
    );
    let valid_beta = manifest(
        "beta.plugin",
        1,
        version(1, 0, 0),
        vec![capability("beta.tool", CapabilityKind::Tool)],
    );
    let duplicate_beta = valid_beta.clone();
    let future = manifest(
        "future.plugin",
        2,
        version(1, 0, 0),
        vec![capability("future.tool", CapabilityKind::Tool)],
    );
    let invalid = DiscoveryCandidate::parse(
        "00-invalid.json",
        &json!({
            "manifest_version": 1,
            "id": "bad plugin",
            "name": "Invalid",
            "version": "1.0.0",
            "capabilities": [],
            "lifecycle": {
                "version": "1.0.0",
                "required_phases": ["initialize", "activate"]
            }
        })
        .to_string(),
    );

    let report = CapabilityRegistry::discover(
        vec![
            DiscoveryCandidate::from_manifest("20-beta.json", valid_beta),
            DiscoveryCandidate::from_manifest("10-alpha.json", valid_alpha),
            DiscoveryCandidate::from_manifest("15-future.json", future),
            DiscoveryCandidate::from_manifest("30-duplicate.json", duplicate_beta),
            invalid,
        ],
        &RuntimeCompatibility::current(),
    );

    assert_eq!(
        report
            .accepted_plugins()
            .iter()
            .map(PluginId::as_str)
            .collect::<Vec<_>>(),
        vec!["alpha.plugin", "beta.plugin"]
    );
    assert_eq!(report.registry().plugins().count(), 2);
    assert_eq!(report.diagnostics().len(), 3);
    assert_eq!(report.diagnostics()[0].source(), "00-invalid.json");
    assert_eq!(report.diagnostics()[1].source(), "15-future.json");
    assert_eq!(report.diagnostics()[2].source(), "30-duplicate.json");
}

#[test]
fn compatibility_rejects_new_lifecycle_contracts_before_loading() {
    let manifest = manifest(
        "future.lifecycle",
        1,
        version(2, 0, 0),
        vec![capability("future.tool", CapabilityKind::Tool)],
    );

    let error = RuntimeCompatibility::current()
        .check_manifest(&manifest)
        .expect_err("new lifecycle contract must fail closed");

    assert!(matches!(
        error,
        CompatibilityError::LifecycleMajorMismatch { .. }
    ));
}
