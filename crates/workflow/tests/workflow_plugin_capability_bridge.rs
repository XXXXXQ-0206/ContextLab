//! Contract tests for deriving Workflow capability snapshots from the MCP registry.

use contextlab_mcp::{
    CapabilityDescriptor, CapabilityKind, CapabilityRegistry, CapabilityRegistrySnapshotV1,
    LifecycleContract, LifecyclePhase, PluginId, PluginManifest, Version,
};
use contextlab_workflow::{
    WorkflowCapabilityId, WorkflowCapabilityRequirement, WorkflowCapabilityVersion,
    WorkflowPluginCapabilityBridge, WorkflowPluginCapabilityBridgeError,
    WorkflowPluginCapabilityRequirement,
};

fn version(major: u64, minor: u64, patch: u64) -> Version {
    Version::new(major, minor, patch)
}

fn workflow_version(major: u64, minor: u64, patch: u64) -> WorkflowCapabilityVersion {
    WorkflowCapabilityVersion::new(major, minor, patch)
}

fn manifest(
    plugin_id: &str,
    capabilities: impl IntoIterator<Item = (&'static str, CapabilityKind, Version)>,
) -> PluginManifest {
    PluginManifest::new(
        1,
        PluginId::new(plugin_id).expect("valid plugin ID"),
        plugin_id,
        version(1, 0, 0),
        capabilities
            .into_iter()
            .map(|(id, kind, capability_version)| {
                CapabilityDescriptor::new(id, kind, capability_version)
                    .expect("valid capability descriptor")
            })
            .collect(),
        LifecycleContract::new(
            version(1, 0, 0),
            [LifecyclePhase::Initialize, LifecyclePhase::Activate],
        )
        .expect("valid lifecycle"),
    )
    .expect("valid plugin manifest")
}

fn registry(manifests: impl IntoIterator<Item = PluginManifest>) -> CapabilityRegistry {
    let mut registry = CapabilityRegistry::new();
    for manifest in manifests {
        registry.register(manifest).expect("unique plugin manifest");
    }
    registry
}

fn requirement(
    id: &str,
    minimum_version: WorkflowCapabilityVersion,
    expected_kind: CapabilityKind,
) -> WorkflowPluginCapabilityRequirement {
    WorkflowPluginCapabilityRequirement::new(
        WorkflowCapabilityRequirement::new(id, minimum_version)
            .expect("valid Workflow requirement"),
        expected_kind,
    )
}

#[test]
fn bridge_resolves_registry_versions_into_the_existing_snapshot() {
    let registry = registry([
        manifest(
            "plugin.zeta",
            [("workflow.zeta", CapabilityKind::McpServer, version(2, 4, 1))],
        ),
        manifest(
            "plugin.alpha",
            [("workflow.alpha", CapabilityKind::Tool, version(1, 3, 2))],
        ),
    ]);
    let alpha_id = WorkflowCapabilityId::new("workflow.alpha").expect("valid capability ID");
    let zeta_id = WorkflowCapabilityId::new("workflow.zeta").expect("valid capability ID");

    let snapshot = WorkflowPluginCapabilityBridge::snapshot(
        &registry,
        [
            requirement(
                "workflow.zeta",
                workflow_version(2, 1, 0),
                CapabilityKind::McpServer,
            ),
            requirement(
                "workflow.alpha",
                workflow_version(1, 0, 0),
                CapabilityKind::Tool,
            ),
        ],
    )
    .expect("compatible registry derives a Workflow snapshot");

    assert_eq!(snapshot.version(&alpha_id), Some(workflow_version(1, 3, 2)));
    assert_eq!(snapshot.version(&zeta_id), Some(workflow_version(2, 4, 1)));
}

#[test]
fn bridge_resolves_only_the_immutable_registry_snapshot() {
    let mut registry = registry([manifest(
        "plugin.alpha",
        [("workflow.alpha", CapabilityKind::Tool, version(1, 0, 0))],
    )]);
    let sealed: CapabilityRegistrySnapshotV1 = registry.snapshot();
    registry
        .register(manifest(
            "plugin.late",
            [("workflow.late", CapabilityKind::Tool, version(1, 0, 0))],
        ))
        .expect("late registration mutates only the source registry");

    let error = WorkflowPluginCapabilityBridge::snapshot_from_registry_snapshot(
        &sealed,
        [requirement(
            "workflow.late",
            workflow_version(1, 0, 0),
            CapabilityKind::Tool,
        )],
    )
    .expect_err("a sealed bridge input must fail closed for later registrations");

    assert_eq!(
        error,
        WorkflowPluginCapabilityBridgeError::MissingCapability {
            capability: WorkflowCapabilityId::new("workflow.late").expect("valid capability ID"),
            required: workflow_version(1, 0, 0),
        }
    );
}

#[test]
fn bridge_fails_closed_when_a_capability_is_missing() {
    let registry = CapabilityRegistry::new();

    let error = WorkflowPluginCapabilityBridge::snapshot(
        &registry,
        [requirement(
            "workflow.missing",
            workflow_version(1, 2, 0),
            CapabilityKind::Tool,
        )],
    )
    .expect_err("missing capability must fail closed");

    assert_eq!(
        error,
        WorkflowPluginCapabilityBridgeError::MissingCapability {
            capability: WorkflowCapabilityId::new("workflow.missing").expect("valid capability ID"),
            required: workflow_version(1, 2, 0),
        }
    );
}

#[test]
fn bridge_fails_closed_when_registry_version_is_incompatible() {
    let registry = registry([manifest(
        "plugin.tool",
        [("workflow.tool", CapabilityKind::Tool, version(1, 1, 9))],
    )]);

    let error = WorkflowPluginCapabilityBridge::snapshot(
        &registry,
        [requirement(
            "workflow.tool",
            workflow_version(1, 2, 0),
            CapabilityKind::Tool,
        )],
    )
    .expect_err("older registry capability must fail closed");

    assert_eq!(
        error,
        WorkflowPluginCapabilityBridgeError::IncompatibleCapability {
            capability: WorkflowCapabilityId::new("workflow.tool").expect("valid capability ID"),
            required: workflow_version(1, 2, 0),
            provided: workflow_version(1, 1, 9),
        }
    );
}

#[test]
fn bridge_fails_closed_when_capability_kind_is_wrong() {
    let registry = registry([manifest(
        "plugin.resource",
        [(
            "workflow.lookup",
            CapabilityKind::Resource,
            version(1, 4, 0),
        )],
    )]);

    let error = WorkflowPluginCapabilityBridge::snapshot(
        &registry,
        [requirement(
            "workflow.lookup",
            workflow_version(1, 0, 0),
            CapabilityKind::Tool,
        )],
    )
    .expect_err("wrong capability kind must fail closed");

    assert_eq!(
        error,
        WorkflowPluginCapabilityBridgeError::WrongCapabilityKind {
            capability: WorkflowCapabilityId::new("workflow.lookup").expect("valid capability ID"),
            expected: CapabilityKind::Tool,
            provided: CapabilityKind::Resource,
        }
    );
}

#[test]
fn bridge_consolidates_compatible_duplicate_requirements_at_the_highest_minimum() {
    let registry = registry([manifest(
        "plugin.tool",
        [("workflow.tool", CapabilityKind::Tool, version(1, 5, 3))],
    )]);
    let capability_id = WorkflowCapabilityId::new("workflow.tool").expect("valid capability ID");

    let snapshot = WorkflowPluginCapabilityBridge::snapshot(
        &registry,
        [
            requirement(
                "workflow.tool",
                workflow_version(1, 4, 0),
                CapabilityKind::Tool,
            ),
            requirement(
                "workflow.tool",
                workflow_version(1, 2, 0),
                CapabilityKind::Tool,
            ),
        ],
    )
    .expect("compatible duplicates consolidate deterministically");

    assert_eq!(
        snapshot.version(&capability_id),
        Some(workflow_version(1, 5, 3))
    );
}

#[test]
fn bridge_rejects_conflicting_duplicate_kinds_before_registry_resolution() {
    let registry = CapabilityRegistry::new();

    let error = WorkflowPluginCapabilityBridge::snapshot(
        &registry,
        [
            requirement(
                "workflow.shared",
                workflow_version(1, 0, 0),
                CapabilityKind::Resource,
            ),
            requirement(
                "workflow.shared",
                workflow_version(1, 0, 0),
                CapabilityKind::Tool,
            ),
        ],
    )
    .expect_err("one stable ID cannot require two kinds");

    assert_eq!(
        error,
        WorkflowPluginCapabilityBridgeError::ConflictingCapabilityKind {
            capability: WorkflowCapabilityId::new("workflow.shared").expect("valid capability ID"),
            first: CapabilityKind::Tool,
            second: CapabilityKind::Resource,
        }
    );
}

#[test]
fn bridge_rejects_conflicting_duplicate_major_versions_deterministically() {
    let registry = CapabilityRegistry::new();

    let error = WorkflowPluginCapabilityBridge::snapshot(
        &registry,
        [
            requirement(
                "workflow.shared",
                workflow_version(2, 0, 0),
                CapabilityKind::Tool,
            ),
            requirement(
                "workflow.shared",
                workflow_version(1, 9, 0),
                CapabilityKind::Tool,
            ),
        ],
    )
    .expect_err("one stable ID cannot require incompatible major versions");

    assert_eq!(
        error,
        WorkflowPluginCapabilityBridgeError::ConflictingCapabilityVersion {
            capability: WorkflowCapabilityId::new("workflow.shared").expect("valid capability ID"),
            first: workflow_version(1, 9, 0),
            second: workflow_version(2, 0, 0),
        }
    );
}
