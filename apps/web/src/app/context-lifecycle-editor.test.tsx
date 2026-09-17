import assert from "node:assert/strict";
import test from "node:test";
import { AppRouterContext } from "next/dist/shared/lib/app-router-context.shared-runtime";
import type { LocalContextLifecycleState } from "@contextlab/local-sdk";
import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { ContextWorkspaceScreen } from "./context-workspace-screen";
import type { ContextWorkspaceData } from "./context-workspace-data";
import {
  commitPreview,
  componentPreview,
  contextPreview,
  evaluationScorecardPreview,
  evaluationRunPreview,
  projectPreview,
  selectedComponentDetail,
  selectedCommitDetail,
  selectedEvaluationRunDetail,
  workspaceContextGraphPreview,
  workspacePreview
} from "./context-workspace-preview";
import { presentContextWorkspaceScreen } from "./context-workspace-presenter";
import { LocalLifecycleProxyError } from "./context-lifecycle-data";
import {
  deriveLocalUsesAddCandidates,
  deriveLocalUsesRemoveCandidates,
  presentContextLifecycleRelationships
} from "./context-lifecycle-presenter";
import * as lifecycleEditor from "./context-lifecycle-editor";
import type { LocalBranchHeadTarget } from "./local-branch-heads-data";

const {
  ContextLifecycleEditorControl,
  createLifecycleCommitReviewPair,
  refreshAfterLifecycleCommit,
  resolveLifecycleIdempotencyKey
} = lifecycleEditor;

(globalThis as typeof globalThis & { React: typeof React }).React = React;

const testRouter = {
  back: () => undefined,
  forward: () => undefined,
  refresh: () => undefined,
  hmrRefresh: () => undefined,
  push: () => undefined,
  replace: () => undefined,
  prefetch: () => undefined
};

function createRelationshipState(edges: Array<{ source: string; target: string; kind: string }>) {
  const componentIds = ["component-source", "component-target", "component-other"];
  return {
    components: componentIds.map((component_id) => ({ component_id })),
    graph_snapshot: {
      schema_version: 1,
      graph: {
        nodes: Object.fromEntries(componentIds.map((id) => [`component:${id}`, { id: `component:${id}`, kind: "component", label: id }])),
        edges
      }
    }
  };
}

function createPreviewWorkspaceData(): ContextWorkspaceData {
  return {
    source: "preview",
    workspacePreview,
    workspaceContextGraph: workspaceContextGraphPreview,
    projectPreview,
    contextPreview,
    commitPreview,
    selectedCommitDetail,
    componentPreview,
    selectedComponentDetail,
    evaluationRunPreview,
    selectedEvaluationRunDetail,
    evaluationScorecard: evaluationScorecardPreview,
    commitGraphDiff: null,
    commitGraphDiffUnavailableReason: null,
    selectedWorkspace: workspacePreview.items[0]!,
    selectedProject: projectPreview.items[0]!,
    selectedContext: contextPreview.items[0]!,
    latestCommit: commitPreview.items[0]!,
    latestEvaluationRun: evaluationRunPreview.items[0]!,
    componentCount: componentPreview.pagination.total
  };
}

function renderLifecycleEditorWithRelationshipOperation(
  operation = "add_uses_relationship",
  edges = [{ source: "component:component-target", target: "component:component-source", kind: "uses" }]
): string {
  const internals = (
    React as typeof React & {
      __CLIENT_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE: { H: unknown };
    }
  ).__CLIENT_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE;
  const previousDispatcher = internals.H;
  const stateValues = [
    "memory-only-token",
    "commit-a",
    "main",
    "Add a Uses relationship",
    operation,
    "prompt",
    "component-source",
    "component-target",
    "Instruction",
    "{}",
    "",
    false,
    {
      commit_id: "commit-a",
      components: [
        {
          component_id: "component-source",
          component_kind: "prompt",
          name: "Source prompt",
          content_hash: "source-hash"
        },
        {
          component_id: "component-target",
          component_kind: "knowledge",
          name: "Target knowledge",
          content_hash: "target-hash"
        }
      ],
      graph_snapshot: {
        schema_version: 1,
        graph: {
          nodes: {
            "component:component-source": { id: "component:component-source", kind: "prompt", label: "Source prompt" },
            "component:component-target": { id: "component:component-target", kind: "knowledge", label: "Target knowledge" }
          },
          edges
        }
      }
    },
    null,
    false,
    false
  ];
  let stateIndex = 0;

  internals.H = {
    useState<T>(initialValue: T) {
      const value = stateValues[stateIndex++] ?? initialValue;
      return [value as T, () => undefined];
    },
    useMemo<T>(create: () => T) {
      return create();
    },
    useRef<T>(initialValue: T) {
      return { current: initialValue };
    }
  };

  try {
    return renderToStaticMarkup(
      ContextLifecycleEditorControl({
        candidates: [{ id: "commit-a", label: "Initialize Context" }],
        contextId: "context-a",
        refreshWorkspace: () => undefined
      })
    );
  } finally {
    internals.H = previousDispatcher;
  }
}

test("workspace defaults the local lifecycle editor off and renders it only with explicit enablement", () => {
  const workspace = presentContextWorkspaceScreen(createPreviewWorkspaceData());
  const defaultMarkup = renderToStaticMarkup(
    <AppRouterContext.Provider value={testRouter}>
      <ContextWorkspaceScreen {...workspace} />
    </AppRouterContext.Provider>
  );
  const enabledMarkup = renderToStaticMarkup(
    <AppRouterContext.Provider value={testRouter}>
      <ContextWorkspaceScreen {...{ ...workspace, localLifecycleEnabled: true }} />
    </AppRouterContext.Provider>
  );

  assert.doesNotMatch(defaultMarkup, /context-lifecycle-editor-heading/);
  assert.match(enabledMarkup, /context-lifecycle-editor-heading/);
  assert.match(defaultMarkup, /local-workflow-capability-heading/);
  assert.equal(defaultMarkup.match(/id="local-benchmark-workspace-heading"/g)?.length, 1);
  assert.equal(defaultMarkup.match(/id="context-benchmark-evidence-heading"/g)?.length, 1);
  assert.equal(defaultMarkup.match(/id="context-benchmark-decision-diff-heading"/g)?.length, 1);
});

test("lifecycle editor renders a bilingual local-only form with a disabled initial submit", () => {
  const markup = renderToStaticMarkup(
    <ContextLifecycleEditorControl
      candidates={[
        { id: "commit-b", label: "Create instruction" },
        { id: "commit-a", label: "Initialize Context" }
      ]}
      contextId="context-a"
      refreshWorkspace={() => undefined}
    />
  );

  assert.match(markup, /Local Context Lifecycle/);
  assert.match(markup, /仅本地受保护工作流/);
  assert.match(markup, /Bearer token/);
  assert.match(markup, /type="password"/);
  assert.match(markup, /Create component/);
  assert.match(markup, /Initialize Context/);
  assert.match(markup, /Update descriptor \/ 更新描述符/);
  assert.match(markup, /disabled=""/);
  assert.match(markup, /No credential is persisted/);
});

test("lifecycle editor accepts the selected server-owned branch-head target", () => {
  const selectedTarget: LocalBranchHeadTarget = Object.freeze({
    branch_name: "feature/read-only",
    head_commit_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb"
  });
  const markup = renderToStaticMarkup(
    <ContextLifecycleEditorControl
      candidates={[{ id: "unrelated-commit", label: "Unrelated materialized commit" }]}
      contextId="context-a"
      refreshWorkspace={() => undefined}
      selectedBranchHeadTarget={selectedTarget}
    />
  );

  assert.match(markup, /name="lifecycle-branch"[^>]*value="feature\/read-only"/);
  assert.match(markup, /name="lifecycle-head"/);
  assert.match(markup, /bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb/);
  assert.doesNotMatch(markup, /value="main"/);
});

test("lifecycle editor exposes an explicit bilingual read control for descriptor metadata", () => {
  const markup = renderLifecycleEditorWithRelationshipOperation("update_descriptor");

  assert.match(markup, /Read current metadata \/ 读取当前元数据/);
  assert.match(markup, /name="lifecycle-descriptor-metadata"/);
});

test("lifecycle editor renders typed Uses mode with bilingual source and target Select options", () => {
  const markup = renderLifecycleEditorWithRelationshipOperation();

  assert.match(markup, /Add Uses relationship \/ 添加 Uses 关系/);
  assert.match(markup, /name="lifecycle-uses-source-component-id"/);
  assert.match(markup, /name="lifecycle-uses-target-component-id"/);
  assert.match(markup, /Source component \/ 源组件/);
  assert.match(markup, /Target component \/ 目标组件/);
  assert.match(markup, /Select source component \/ 选择源组件/);
  assert.match(markup, /Select target component \/ 选择目标组件/);
  assert.match(markup, /Source prompt \(prompt\)/);
  assert.match(markup, /Target knowledge \(knowledge\)/);
});

test("lifecycle relationship presenter includes every edge kind in deterministic exact-commit order", () => {
  const state = {
    ...createRelationshipState([
      { source: "component:component-target", target: "component:component-source", kind: "uses" },
      { source: "component:component-source", target: "component:component-other", kind: "owns" },
      { source: "component:component-other", target: "component:component-source", kind: "tracks" },
      { source: "component:component-source", target: "component:component-target", kind: "contains" },
      { source: "component:component-other", target: "component:component-target", kind: "configures" },
      { source: "component:component-target", target: "component:component-other", kind: "retrieves" },
      { source: "component:component-source", target: "component:component-other", kind: "evaluates" },
      { source: "component:component-target", target: "component:component-source", kind: "produces" }
    ]),
    commit_id: "exact-commit"
  } as unknown as LocalContextLifecycleState;

  const model = presentContextLifecycleRelationships(state);

  assert.ok(model);
  assert.equal(model.commitId, "exact-commit");
  assert.deepEqual(model.rows.map((row) => row.kindLabel), [
    "Tracks / 跟踪",
    "Configures / 配置",
    "Owns / 拥有",
    "Evaluates / 评估",
    "Contains / 包含",
    "Retrieves / 检索",
    "Uses / 使用",
    "Produces / 产出"
  ]);
  assert.deepEqual(model.rows.map((row) => [row.sourceId, row.targetId]), [
    ["component:component-other", "component:component-source"],
    ["component:component-other", "component:component-target"],
    ["component:component-source", "component:component-other"],
    ["component:component-source", "component:component-other"],
    ["component:component-source", "component:component-target"],
    ["component:component-target", "component:component-other"],
    ["component:component-target", "component:component-source"],
    ["component:component-target", "component:component-source"]
  ]);
});

test("lifecycle relationship section exposes exact-commit table semantics and empty state", () => {
  const markup = renderLifecycleEditorWithRelationshipOperation();
  const emptyMarkup = renderLifecycleEditorWithRelationshipOperation("add_uses_relationship", []);

  assert.match(markup, /Relationships \/ 关系/);
  assert.match(markup, /Exact commit \/ 精确提交/);
  assert.match(markup, /commit-a/);
  assert.match(markup, /role="table"/);
  assert.match(markup, /Relationships in exact commit \/ 精确提交中的关系/);
  assert.match(markup, /Uses \/ 使用/);
  assert.match(emptyMarkup, /No relationships in this exact commit \/ 此精确提交没有关系。/);
  assert.match(emptyMarkup, /role="status"/);
});

test("lifecycle graph helpers exclude self and existing Add Uses edges while preserving exact Remove edges", () => {
  const state = createRelationshipState([
    { source: "component:component-source", target: "component:component-target", kind: "uses" },
    { source: "component:component-target", target: "component:component-other", kind: "uses" }
  ]);

  assert.deepEqual(deriveLocalUsesAddCandidates(state, "component-source"), ["component-other"]);
  assert.deepEqual(deriveLocalUsesRemoveCandidates(state), [
    { sourceComponentId: "component-source", targetComponentId: "component-target" },
    { sourceComponentId: "component-target", targetComponentId: "component-other" }
  ]);
});

test("lifecycle graph refresh reads the revised commit edge set", async () => {
  const refreshedState = createRelationshipState([
    { source: "component:component-source", target: "component:component-other", kind: "uses" }
  ]);
  let refreshCount = 0;

  const result = await refreshAfterLifecycleCommit(
    async () => refreshedState,
    () => refreshCount++
  );

  assert.equal(result.ok, true);
  if (result.ok) {
    assert.deepEqual(deriveLocalUsesRemoveCandidates(result.value), [
      { sourceComponentId: "component-source", targetComponentId: "component-other" }
    ]);
  }
  assert.equal(refreshCount, 1);
});

test("lifecycle editor preserves a committed-state reload failure while refreshing the workspace", async () => {
  let refreshCount = 0;

  const result = await refreshAfterLifecycleCommit(
    async () => Promise.reject(new Error("state read failed")),
    () => refreshCount++
  );

  assert.equal(result.ok, false);
  assert.match(result.error instanceof Error ? result.error.message : "", /state read failed/);
  assert.equal(refreshCount, 1);
});

test("lifecycle commit review pairs preserve exact ancestry and reject replay duplicates", () => {
  for (const operation of ["create", "update", "remove", "relationship"]) {
    const pair = createLifecycleCommitReviewPair(`head-${operation}`, `commit-${operation}`);
    assert.deepEqual(pair, {
      originalCommitId: `head-${operation}`,
      revisedCommitId: `commit-${operation}`
    });
  }

  assert.equal(createLifecycleCommitReviewPair("", "new-commit"), null);
  assert.equal(createLifecycleCommitReviewPair("same-commit", "same-commit"), null);
});

test("lifecycle retries reuse one key only for the unchanged draft", () => {
  let generated = 0;
  const createKey = () => `key-${++generated}`;

  const first = resolveLifecycleIdempotencyKey(null, 0, createKey);
  const retry = resolveLifecycleIdempotencyKey(first, 0, createKey);
  const changedDraft = resolveLifecycleIdempotencyKey(retry, 1, createKey);

  assert.equal(first.key, "key-1");
  assert.strictEqual(retry, first);
  assert.equal(changedDraft.key, "key-2");
  assert.notEqual(changedDraft.key, first.key);
});

test("lifecycle editor renders disabled local-development remediation without raw BFF text", () => {
  const rawProxyMessage = "Local Context lifecycle mutations are disabled";
  const presentLifecycleError = (
    lifecycleEditor as typeof lifecycleEditor & { presentLifecycleError?: (error: unknown) => string }
  ).presentLifecycleError;

  assert.equal(typeof presentLifecycleError, "function");
  assert.ok(presentLifecycleError);

  const message = presentLifecycleError(
    new LocalLifecycleProxyError(403, {
      error: "local_lifecycle_disabled",
      message: rawProxyMessage
    })
  );

  assert.match(message, /development environment/);
  assert.match(message, /开发环境/);
  assert.doesNotMatch(message, /permission|authorization/i);
  assert.doesNotMatch(message, new RegExp(rawProxyMessage));
});

test("lifecycle editor redacts upstream diagnostics for every structured failure", () => {
  const presentLifecycleError = (
    lifecycleEditor as typeof lifecycleEditor & { presentLifecycleError?: (error: unknown) => string }
  ).presentLifecycleError;

  assert.equal(typeof presentLifecycleError, "function");
  assert.ok(presentLifecycleError);

  const failures = [
    {
      status: 409,
      error: "context_lifecycle_state_conflict",
      message: "private stale-head diagnostics"
    },
    {
      status: 429,
      error: "context_lifecycle_rate_limited",
      message: "private quota diagnostics"
    },
    {
      status: 599,
      error: "context_lifecycle_unknown",
      message: "private upstream stack trace"
    }
  ] as const;

  for (const failure of failures) {
    const message = presentLifecycleError(
      new LocalLifecycleProxyError(failure.status, failure)
    );

    assert.ok(message.length > 0);
    assert.doesNotMatch(message, new RegExp(failure.message));
  }
});
