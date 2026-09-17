import assert from "node:assert/strict";
import test from "node:test";
import {
  buildLocalLifecycleCommitRequest,
  canSubmitLocalLifecycle,
  deriveLocalGraphRelationshipFacts,
  deriveLocalUsesAddCandidates,
  deriveLocalUsesEdges,
  deriveLocalUsesRemoveCandidates,
  lifecycleErrorMessage,
  lifecycleNoticeRole,
  presentContextLifecycleReadInspector,
  presentLocalLifecycleMetadataDraft
} from "./context-lifecycle-presenter";

function lifecycleState(overrides: Record<string, unknown> = {}) {
  return {
    context_id: "context-a",
    commit_id: "commit-a",
    components: [
      { component_id: "component-a", component_kind: "prompt", name: "A", metadata: {}, content: "A", content_hash: "hash-a", creation_commit_id: "commit-a", content_commit_id: "commit-a" },
      { component_id: "component-b", component_kind: "knowledge", name: "B", metadata: {}, content: "B", content_hash: "hash-b", creation_commit_id: "commit-a", content_commit_id: "commit-a" },
      { component_id: "component-c", component_kind: "tool", name: "C", metadata: {}, content: "C", content_hash: "hash-c", creation_commit_id: "commit-a", content_commit_id: "commit-a" }
    ],
    graph_snapshot: {
      context_id: "context-a",
      commit_id: "commit-a",
      captured_at: "2026-07-30T00:00:00Z",
      schema_version: 1,
      graph: {
        nodes: {
          "context:context-a": { id: "context:context-a", kind: "context", label: "Context" },
          "component:component-a": { id: "component:component-a", kind: "prompt", label: "A" },
          "component:component-b": { id: "component:component-b", kind: "knowledge", label: "B" },
          "component:component-c": { id: "component:component-c", kind: "tool", label: "C" }
        },
        edges: [
          { source: "component:component-b", target: "component:component-c", kind: "uses" },
          { source: "context:context-a", target: "component:component-b", kind: "contains" }
        ]
      }
    },
    ...overrides
  };
}

test("lifecycle presenter builds a create command with JSON metadata", () => {
  const result = buildLocalLifecycleCommitRequest({
    branchName: "main",
    expectedHeadCommitId: "commit-a",
    message: "Create instruction",
    operation: "create",
    componentKind: "prompt",
    name: "Instruction",
    metadataText: '{"locale":"en-US"}',
    content: "Use the checked Context."
  });

  assert.deepEqual(result, {
    ok: true,
    value: {
      branch_name: "main",
      expected_head_commit_id: "commit-a",
      message: "Create instruction",
      operation: {
        kind: "create",
        component_kind: "prompt",
        name: "Instruction",
        metadata: { locale: "en-US" },
        content: "Use the checked Context."
      }
    }
  });
});

test("lifecycle presenter builds a descriptor-only command without a body rewrite", () => {
  const result = buildLocalLifecycleCommitRequest({
    branchName: "main",
    expectedHeadCommitId: "commit-a",
    message: "Correct bilingual instruction descriptor",
    operation: "update_descriptor",
    componentKind: "prompt",
    componentId: "component-a",
    name: "Instruction / 操作说明",
    metadataText: '{"locale":"zh-CN","audience":"operator"}',
    content: "This immutable body must not be sent."
  });

  assert.deepEqual(result, {
    ok: true,
    value: {
      branch_name: "main",
      expected_head_commit_id: "commit-a",
      message: "Correct bilingual instruction descriptor",
      operation: {
        kind: "update_descriptor",
        component_id: "component-a",
        name: "Instruction / 操作说明",
        metadata: { locale: "zh-CN", audience: "operator" }
      }
    }
  });
});

test("lifecycle presenter builds a typed Context metadata-only commit", () => {
  const result = buildLocalLifecycleCommitRequest({
    branchName: "main",
    expectedHeadCommitId: "commit-a",
    message: "Update Context metadata",
    operation: "update_metadata",
    componentKind: "prompt",
    name: "",
    metadataText: JSON.stringify({
      created_at: "2026-07-30T00:00:00Z",
      updated_at: "2026-07-30T01:00:00Z",
      labels: { owner: "luna" }
    }),
    content: ""
  });

  assert.deepEqual(result, {
    ok: true,
    value: {
      branch_name: "main",
      expected_head_commit_id: "commit-a",
      message: "Update Context metadata",
      operation: {
        kind: "update_metadata",
        metadata: {
          created_at: "2026-07-30T00:00:00Z",
          updated_at: "2026-07-30T01:00:00Z",
          labels: { owner: "luna" }
        }
      }
    }
  });
});

test("lifecycle presenter formats loaded metadata for a bilingual descriptor draft", () => {
  assert.deepEqual(
    presentLocalLifecycleMetadataDraft({
      component_id: "component-a",
      name: "Instruction",
      metadata: { locale: "zh-CN", audience: "operator" }
    }),
    {
      name: "Instruction",
      metadataText: '{\n  "locale": "zh-CN",\n  "audience": "operator"\n}'
    }
  );
});

test("lifecycle read presenter preserves the exact Context and commit scope", () => {
  const view = presentContextLifecycleReadInspector({
    kind: "available",
    target: { contextId: "context-a", commitId: "commit-a" },
    state: lifecycleState({
      metadata: {
        created_at: "2026-07-30T00:00:00Z",
        updated_at: "2026-07-30T01:00:00Z",
        labels: { owner: "luna" }
      }
    }) as never
  });

  assert.equal(view.status.state, "available");
  assert.deepEqual(view.scope, [
    { id: "context-id", label: "Context / 上下文", value: "context-a" },
    { id: "commit-id", label: "Commit / 提交", value: "commit-a" }
  ]);
  assert.equal(view.metadataText, '{\n  "created_at": "2026-07-30T00:00:00Z",\n  "updated_at": "2026-07-30T01:00:00Z",\n  "labels": {\n    "owner": "luna"\n  }\n}');
  assert.equal(view.components[0]?.content, "A");
  assert.equal(view.relationships?.commitId, "commit-a");
  assert.match(view.status.description, /selected commit/i);
  assert.match(view.status.description, /精确提交/);
});

test("lifecycle read presenter exposes loading, error, empty, and available states without leaking upstream text", () => {
  const resources = [
    { kind: "loading", target: { contextId: "context-a", commitId: "commit-a" } },
    { kind: "error", target: { contextId: "context-a", commitId: "commit-a" }, message: "private upstream detail" },
    { kind: "empty", target: { contextId: "context-a", commitId: "commit-a" } },
    { kind: "available", target: { contextId: "context-a", commitId: "commit-a" }, state: lifecycleState() as never }
  ] as const;

  for (const resource of resources) {
    const view = presentContextLifecycleReadInspector(resource);
    assert.equal(view.status.state, resource.kind);
    assert.match(view.status.capabilityLabel, /Context lifecycle state/);
    assert.match(view.status.capabilityLabel, /Context 生命周期状态/);
    if (resource.kind === "error") {
      assert.doesNotMatch(view.status.description, /private upstream detail/);
    }
  }
});

test("lifecycle presenter builds an unborn Context initialization without a selected head", () => {
  const result = buildLocalLifecycleCommitRequest({
    branchName: "main",
    expectedHeadCommitId: "",
    message: "Initialize Context lifecycle",
    operation: "initialize",
    componentKind: "prompt",
    name: "",
    metadataText: "{}",
    content: ""
  });

  assert.deepEqual(result, {
    ok: true,
    value: {
      branch_name: "main",
      expected_head_commit_id: null,
      message: "Initialize Context lifecycle",
      operation: { kind: "initialize" }
    }
  });
  assert.equal(
    canSubmitLocalLifecycle({
      bearerToken: "request-token",
      selectedCommitId: "",
      allowsUnbornHead: true,
      isCommandValid: true,
      isLoading: false,
      isSubmitting: false
    }),
    true
  );
});

test("lifecycle presenter builds a typed Uses relationship command", () => {
  const result = buildLocalLifecycleCommitRequest({
    branchName: "main",
    expectedHeadCommitId: "commit-a",
    message: "Add Uses relationship",
    operation: "add_uses_relationship",
    componentKind: "prompt",
    componentId: "source-component",
    targetComponentId: "target-component",
    name: "",
    metadataText: "{}",
    content: ""
  });

  assert.deepEqual(result, {
    ok: true,
    value: {
      branch_name: "main",
      expected_head_commit_id: "commit-a",
      message: "Add Uses relationship",
      operation: {
        kind: "add_uses_relationship",
        source_component_id: "source-component",
        target_component_id: "target-component"
      }
    }
  });
  assert.deepEqual(
    buildLocalLifecycleCommitRequest({
      branchName: "main",
      expectedHeadCommitId: "commit-a",
      message: "Reject self Uses relationship",
      operation: "remove_uses_relationship",
      componentKind: "prompt",
      componentId: "same-component",
      targetComponentId: "same-component",
      name: "",
      metadataText: "{}",
      content: ""
    }),
    {
      ok: false,
      message: "Uses endpoints must differ / Uses 关系端点必须不同。"
    }
  );
});

test("lifecycle presenter rejects unsafe incomplete update input", () => {
  const result = buildLocalLifecycleCommitRequest({
    branchName: "main",
    expectedHeadCommitId: "commit-a",
    message: "Update instruction",
    operation: "update",
    componentKind: "prompt",
    name: "",
    metadataText: "{}",
    content: ""
  });

  assert.deepEqual(result, {
    ok: false,
    message: "Component ID and content are required for an update / 更新需要组件 ID 与正文。"
  });
});

test("lifecycle presenter gives bilingual remediation for protected failures", () => {
  assert.match(lifecycleErrorMessage(401), /Bearer/);
  assert.match(lifecycleErrorMessage(403), /permission/);
  assert.match(lifecycleErrorMessage(409), /head/);
  assert.match(lifecycleErrorMessage(429), /rate limit/);
});

test("lifecycle presenter distinguishes a disabled local-development gate from authorization denial", () => {
  const message = lifecycleErrorMessage(403, undefined, "local_lifecycle_disabled");

  assert.match(message, /development environment/);
  assert.match(message, /开发环境/);
  assert.doesNotMatch(message, /permission|authorization/i);
});

test("lifecycle presenter blocks duplicate or incomplete local submissions", () => {
  assert.equal(
    canSubmitLocalLifecycle({ bearerToken: "", selectedCommitId: "commit-a", isCommandValid: true, isLoading: false, isSubmitting: false }),
    false
  );
  assert.equal(
    canSubmitLocalLifecycle({ bearerToken: "request-token", selectedCommitId: "", isCommandValid: true, isLoading: false, isSubmitting: false }),
    false
  );
  assert.equal(
    canSubmitLocalLifecycle({ bearerToken: "request-token", selectedCommitId: "commit-a", isCommandValid: true, isLoading: true, isSubmitting: false }),
    false
  );
  assert.equal(
    canSubmitLocalLifecycle({ bearerToken: "request-token", selectedCommitId: "commit-a", isCommandValid: true, isLoading: false, isSubmitting: true }),
    false
  );
  assert.equal(
    canSubmitLocalLifecycle({ bearerToken: "request-token", selectedCommitId: "commit-a", isCommandValid: false, isLoading: false, isSubmitting: false }),
    false
  );
  assert.equal(
    canSubmitLocalLifecycle({ bearerToken: "request-token", selectedCommitId: "commit-a", isCommandValid: true, isLoading: false, isSubmitting: false }),
    true
  );
});

test("lifecycle presenter requires explicit removal confirmation", () => {
  const result = buildLocalLifecycleCommitRequest({
    branchName: "main",
    expectedHeadCommitId: "commit-a",
    message: "Remove instruction",
    operation: "remove",
    componentKind: "prompt",
    componentId: "component-a",
    name: "",
    metadataText: "{}",
    content: "",
    removeConfirmed: false
  });

  assert.deepEqual(result, {
    ok: false,
    message: "Confirm component removal before committing / 提交前请确认移除组件。"
  });
});

test("lifecycle presenter maps failure notices to assertive announcements", () => {
  assert.equal(lifecycleNoticeRole("error"), "alert");
  assert.equal(lifecycleNoticeRole("success"), "status");
});

test("lifecycle presenter derives directed Uses edges in deterministic order", () => {
  const state = lifecycleState();

  assert.deepEqual(deriveLocalUsesEdges(state), [
    { sourceComponentId: "component-b", targetComponentId: "component-c" }
  ]);
  assert.deepEqual(deriveLocalUsesRemoveCandidates(state), [
    { sourceComponentId: "component-b", targetComponentId: "component-c" }
  ]);
});

test("lifecycle presenter adapts exact-commit graph facts across every Rust edge kind", () => {
  const state = lifecycleState({
    graph_snapshot: {
      ...lifecycleState().graph_snapshot,
      graph: {
        ...lifecycleState().graph_snapshot.graph,
        edges: [
          { source: "component:component-c", target: "component:component-b", kind: "tracks" },
          { source: "component:component-b", target: "component:component-c", kind: "uses" },
          { source: "context:context-a", target: "component:component-b", kind: "contains" },
          { source: "component:component-a", target: "component:component-b", kind: "contains" },
          { source: "component:component-c", target: "component:component-a", kind: "evaluates" },
          { source: "component:component-b", target: "component:component-a", kind: "retrieves" },
          { source: "component:component-a", target: "component:component-b", kind: "owns" },
          { source: "component:component-b", target: "component:component-c", kind: "uses" },
          { source: "component:component-b", target: "component:component-a", kind: "configures" },
          { source: "component:component-c", target: "component:component-a", kind: "produces" }
        ]
      }
    }
  });

  assert.deepEqual(deriveLocalGraphRelationshipFacts(state), [
    { source: "component:component-a", target: "component:component-b", kind: "owns" },
    { source: "component:component-a", target: "component:component-b", kind: "contains" },
    { source: "component:component-b", target: "component:component-a", kind: "configures" },
    { source: "component:component-b", target: "component:component-a", kind: "retrieves" },
    { source: "component:component-b", target: "component:component-c", kind: "uses" },
    { source: "component:component-b", target: "component:component-c", kind: "uses" },
    { source: "component:component-c", target: "component:component-a", kind: "evaluates" },
    { source: "component:component-c", target: "component:component-a", kind: "produces" },
    { source: "component:component-c", target: "component:component-b", kind: "tracks" },
    { source: "context:context-a", target: "component:component-b", kind: "contains" }
  ]);
});

test("lifecycle presenter rejects exact-commit graph facts with malformed endpoints or unknown kinds", () => {
  const state = lifecycleState({
    graph_snapshot: {
      ...lifecycleState().graph_snapshot,
      graph: {
        ...lifecycleState().graph_snapshot.graph,
        edges: [{ source: "component:component-a", target: "component:missing", kind: "uses" }]
      }
    }
  });

  assert.deepEqual(deriveLocalGraphRelationshipFacts(state), []);

  const unknownKind = lifecycleState({
    graph_snapshot: {
      ...lifecycleState().graph_snapshot,
      graph: {
        ...lifecycleState().graph_snapshot.graph,
        edges: [{ source: "component:component-a", target: "component:component-b", kind: "depends_on" }]
      }
    }
  });

  assert.deepEqual(deriveLocalGraphRelationshipFacts(unknownKind), []);

  const scopeDrift = lifecycleState({
    graph_snapshot: {
      ...lifecycleState().graph_snapshot,
      commit_id: "commit-b"
    }
  });

  assert.deepEqual(deriveLocalGraphRelationshipFacts(scopeDrift), []);
});

test("lifecycle presenter excludes self and existing edges from Add candidates", () => {
  const state = lifecycleState({
    graph_snapshot: {
      ...lifecycleState().graph_snapshot,
      graph: {
        ...lifecycleState().graph_snapshot.graph,
        edges: [
          { source: "component:component-a", target: "component:component-c", kind: "uses" },
          { source: "component:component-a", target: "component:component-b", kind: "uses" }
        ]
      }
    }
  });

  assert.deepEqual(deriveLocalUsesAddCandidates(state, "component-a"), []);
  assert.deepEqual(deriveLocalUsesAddCandidates(state, "component-b"), ["component-a", "component-c"]);
});

test("lifecycle presenter fails closed for malformed or unknown graph facts", () => {
  const malformed = lifecycleState({
    graph_snapshot: {
      ...lifecycleState().graph_snapshot,
      graph: {
        ...lifecycleState().graph_snapshot.graph,
        edges: [{ source: "component:component-a", target: "component:missing", kind: "uses" }]
      }
    }
  });
  const unknownKind = lifecycleState({
    graph_snapshot: {
      ...lifecycleState().graph_snapshot,
      graph: {
        ...lifecycleState().graph_snapshot.graph,
        edges: [{ source: "component:component-a", target: "component:component-b", kind: "depends_on" }]
      }
    }
  });

  assert.deepEqual(deriveLocalUsesEdges(malformed), []);
  assert.deepEqual(deriveLocalUsesRemoveCandidates(unknownKind), []);
  assert.deepEqual(deriveLocalUsesAddCandidates(malformed, "component-a"), []);
});
