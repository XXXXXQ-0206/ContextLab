//! Contract tests for the immutable provider-free capability registry snapshot.

use contextlab_mcp::{
    CAPABILITY_REGISTRY_SNAPSHOT_V1, CapabilityDescriptor, CapabilityKind, CapabilityRegistry,
    CapabilityRegistrySnapshotV1, LifecycleContract, LifecyclePhase, PluginId, PluginManifest,
    Version,
};

fn manifest(
    plugin_id: &'static str,
    capabilities: impl IntoIterator<Item = (&'static str, CapabilityKind, Version)>,
) -> PluginManifest {
    let lifecycle = LifecycleContract::new(
        Version::new(1, 0, 0),
        [
            LifecyclePhase::Initialize,
            LifecyclePhase::Activate,
            LifecyclePhase::Deactivate,
            LifecyclePhase::Shutdown,
        ],
    )
    .expect("lifecycle");
    PluginManifest::new(
        1,
        PluginId::new(plugin_id).expect("plugin id"),
        plugin_id,
        Version::new(1, 0, 0),
        capabilities
            .into_iter()
            .map(|(id, kind, version)| {
                CapabilityDescriptor::new(id, kind, version).expect("capability")
            })
            .collect(),
        lifecycle,
    )
    .expect("manifest")
}

fn registry(manifests: impl IntoIterator<Item = PluginManifest>) -> CapabilityRegistry {
    let mut registry = CapabilityRegistry::new();
    for manifest in manifests {
        registry.register(manifest).expect("unique manifest");
    }
    registry
}

#[test]
fn snapshot_is_immutable_and_has_a_versioned_fingerprint() {
    let mut registry = registry([manifest(
        "plugin.alpha",
        [(
            "workflow.alpha",
            CapabilityKind::Tool,
            Version::new(1, 2, 0),
        )],
    )]);
    let snapshot: CapabilityRegistrySnapshotV1 = registry.snapshot();

    assert_eq!(snapshot.schema_version(), CAPABILITY_REGISTRY_SNAPSHOT_V1);
    assert!(!snapshot.fingerprint().is_empty());
    assert!(snapshot.registry().capability("workflow.alpha").is_some());

    registry
        .register(manifest(
            "plugin.beta",
            [(
                "knowledge.citations",
                CapabilityKind::Resource,
                Version::new(1, 0, 0),
            )],
        ))
        .expect("mutate source registry after snapshot");

    assert!(
        snapshot
            .registry()
            .capability("knowledge.citations")
            .is_none()
    );
    assert_eq!(snapshot.fingerprint(), snapshot.fingerprint());
}

#[test]
fn snapshot_fingerprint_and_availability_are_registration_order_independent() {
    let first = registry([
        manifest(
            "plugin.zeta",
            [(
                "workflow.zeta",
                CapabilityKind::McpServer,
                Version::new(2, 0, 0),
            )],
        ),
        manifest(
            "plugin.alpha",
            [(
                "knowledge.citations",
                CapabilityKind::Resource,
                Version::new(1, 1, 0),
            )],
        ),
    ])
    .snapshot();
    let second = registry([
        manifest(
            "plugin.alpha",
            [(
                "knowledge.citations",
                CapabilityKind::Resource,
                Version::new(1, 1, 0),
            )],
        ),
        manifest(
            "plugin.zeta",
            [(
                "workflow.zeta",
                CapabilityKind::McpServer,
                Version::new(2, 0, 0),
            )],
        ),
    ])
    .snapshot();

    assert_eq!(first.fingerprint(), second.fingerprint());
    assert_eq!(first.availability(), second.availability());
}
