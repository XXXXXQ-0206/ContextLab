//! Contract tests for versioned MCP server capability descriptors.

use contextlab_mcp::{
    CapabilityKind, MCP_SERVER_CAPABILITY_DESCRIPTOR_V1, McpServerCapabilityDescriptor,
    McpServerCapabilityDescriptorParseError, Version,
};

#[test]
fn descriptor_parser_requires_v1_and_canonicalizes_stable_capability_ids() {
    let descriptor = McpServerCapabilityDescriptor::parse_json(
        r#"{
            "schema_version": "1.0.0",
            "plugin_id": "example.plugin",
            "capabilities": [
                {"id": "zeta.tool", "kind": "tool", "version": "2.4.6"},
                {"id": "alpha.resource", "kind": "resource", "version": "1.2.3"}
            ]
        }"#,
    )
    .expect("valid descriptor");

    assert_eq!(
        descriptor.schema_version(),
        MCP_SERVER_CAPABILITY_DESCRIPTOR_V1
    );
    assert_eq!(descriptor.plugin_id().as_str(), "example.plugin");
    assert_eq!(
        descriptor
            .capabilities()
            .iter()
            .map(|capability| (capability.id(), capability.kind(), capability.version()))
            .collect::<Vec<_>>(),
        vec![
            (
                "alpha.resource",
                CapabilityKind::Resource,
                Version::new(1, 2, 3),
            ),
            ("zeta.tool", CapabilityKind::Tool, Version::new(2, 4, 6)),
        ]
    );
}

#[test]
fn descriptor_parser_rejects_malformed_stable_ids_and_unknown_fields() {
    for raw in [
        r#"{
            "schema_version": "1.0.0",
            "plugin_id": "bad plugin",
            "capabilities": []
        }"#,
        r#"{
            "schema_version": "1.0.0",
            "plugin_id": "example.plugin",
            "capabilities": [],
            "transport": "stdio"
        }"#,
    ] {
        assert!(matches!(
            McpServerCapabilityDescriptor::parse_json(raw),
            Err(McpServerCapabilityDescriptorParseError::Malformed { .. })
        ));
    }
}

#[test]
fn descriptor_parser_rejects_unsupported_schema_and_duplicate_capabilities() {
    let unsupported = McpServerCapabilityDescriptor::parse_json(
        r#"{
            "schema_version": "1.1.0",
            "plugin_id": "example.plugin",
            "capabilities": []
        }"#,
    )
    .expect_err("schema versions require exact compatibility");
    assert_eq!(
        unsupported,
        McpServerCapabilityDescriptorParseError::UnsupportedSchemaVersion {
            found: Version::new(1, 1, 0),
            supported: MCP_SERVER_CAPABILITY_DESCRIPTOR_V1,
        }
    );

    let duplicate = McpServerCapabilityDescriptor::parse_json(
        r#"{
            "schema_version": "1.0.0",
            "plugin_id": "example.plugin",
            "capabilities": [
                {"id": "same.tool", "kind": "tool", "version": "1.0.0"},
                {"id": "same.tool", "kind": "resource", "version": "1.0.0"}
            ]
        }"#,
    )
    .expect_err("duplicate stable IDs are ambiguous");
    assert!(matches!(
        duplicate,
        McpServerCapabilityDescriptorParseError::DuplicateCapability { capability }
            if capability.as_str() == "same.tool"
    ));
}
