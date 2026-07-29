import assert from "node:assert/strict";
import test from "node:test";
import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import {
  LOCAL_WORKFLOW_CONTEXT_BINDINGS_SCHEMA_V1,
  adaptLocalWorkflowContextBindingsV1,
  parseLocalWorkflowContextBindingsV1,
  type LocalWorkflowContextBindingsResource
} from "./local-workflow-context-bindings-data";
import { presentLocalWorkflowContextBindings } from "./local-workflow-context-bindings-presenter";
import { LocalWorkflowContextBindingsInspector } from "./local-workflow-context-bindings-inspector";
import { LocalWorkflowContextBindingsScreen } from "./local-workflow-context-bindings-screen";
import { LocalWorkflowExecutionStatusScreen } from "./local-workflow-execution-status-screen";
import type { LocalWorkflowExecutionStatusResource } from "./local-workflow-execution-status-data";

Object.assign(globalThis, { React });

const target = Object.freeze({
  context_id: "context-001",
  commit_id: "commit-042",
  capability: Object.freeze({
    en: "Workflow context bindings",
    zh: "Workflow 上下文绑定"
  })
});

const binding = Object.freeze({
  binding_id: "binding-001",
  workflow_id: "workflow-001",
  workflow_revision: 7,
  node_count: 3,
  edge_count: 2
});

const executionContextId = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
const executionCommitId = "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb";
const executionBinding = Object.freeze({
  binding_id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
  workflow_id: "dddddddd-dddd-4ddd-8ddd-dddddddddddd",
  workflow_revision: 7,
  node_count: 3,
  edge_count: 2
});
const executionRunId = "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee";

test("local Workflow binding V1 parser accepts the redacted summary and preserves exact scope/order", () => {
  const dto = parseLocalWorkflowContextBindingsV1({
    schema_version: LOCAL_WORKFLOW_CONTEXT_BINDINGS_SCHEMA_V1,
    context_id: target.context_id,
    commit_id: target.commit_id,
    bindings: [
      binding,
      {
        binding_id: "binding-002",
        workflow_id: "workflow-002",
        workflow_revision: 2,
        node_count: 1,
        edge_count: 0
      }
    ]
  });

  assert.equal(Object.isFrozen(dto), true);
  assert.equal(Object.isFrozen(dto.bindings), true);
  assert.deepEqual(dto.bindings.map((item) => item.binding_id), ["binding-001", "binding-002"]);
  assert.deepEqual(Object.keys(dto), ["schema_version", "context_id", "commit_id", "bindings"]);
});

test("local Workflow binding V1 parser rejects transport drift and raw Workflow fields", () => {
  assert.throws(
    () =>
      parseLocalWorkflowContextBindingsV1({
        schema_version: LOCAL_WORKFLOW_CONTEXT_BINDINGS_SCHEMA_V1,
        context_id: target.context_id,
        commit_id: target.commit_id,
        bindings: [{ ...binding, nodes: [], edges: [] }]
      }),
    /unexpected shape/
  );

  assert.throws(
    () =>
      parseLocalWorkflowContextBindingsV1({
        schema_version: "contextlab.local-workflow-context-bindings.v2",
        context_id: target.context_id,
        commit_id: target.commit_id,
        bindings: []
      }),
    /schema_version/
  );
});

test("local Workflow binding adapter rejects a summary outside the requested Context+commit scope", () => {
  const dto = parseLocalWorkflowContextBindingsV1({
    schema_version: LOCAL_WORKFLOW_CONTEXT_BINDINGS_SCHEMA_V1,
    context_id: "other-context",
    commit_id: target.commit_id,
    bindings: [binding]
  });

  assert.throws(
    () =>
      adaptLocalWorkflowContextBindingsV1({
        kind: "ready",
        target,
        summary: dto
      }),
    /Context and commit scope/
  );
});

test("binding inspector is anchored to the selected exact commit and never substitutes preview data", () => {
  const selectedCommit = Object.freeze({
    context_id: target.context_id,
    id: target.commit_id
  });
  const resource: LocalWorkflowContextBindingsResource = {
    kind: "ready",
    target: {
      context_id: selectedCommit.context_id,
      commit_id: selectedCommit.id,
      capability: target.capability
    },
    summary: parseLocalWorkflowContextBindingsV1({
      schema_version: LOCAL_WORKFLOW_CONTEXT_BINDINGS_SCHEMA_V1,
      context_id: selectedCommit.context_id,
      commit_id: selectedCommit.id,
      bindings: [binding]
    })
  };

  const markup = renderToStaticMarkup(<LocalWorkflowContextBindingsScreen resource={resource} />);

  assert.equal(resource.target.commit_id, selectedCommit.id);
  assert.equal(resource.summary.commit_id, selectedCommit.id);
  assert.match(markup, new RegExp(selectedCommit.id));
  assert.doesNotMatch(markup, /preview-commit|current-head|preview binding/i);

  const substitutedCommit = {
    ...resource,
    target: {
      ...resource.target,
      commit_id: "preview-commit"
    }
  };
  assert.throws(
    () => presentLocalWorkflowContextBindings(substitutedCommit),
    /Context and commit scope/
  );
});

test("binding inspector contract rejects preview, current-head, and raw workflow fields", () => {
  const payload = {
    schema_version: LOCAL_WORKFLOW_CONTEXT_BINDINGS_SCHEMA_V1,
    context_id: target.context_id,
    commit_id: target.commit_id,
    bindings: [binding]
  };

  for (const [name, mutate] of [
    ["preview field", (value: Record<string, unknown>) => { value.preview = {}; }],
    ["current head field", (value: Record<string, unknown>) => { value.current_head = {}; }],
    ["raw workflow field", (value: Record<string, unknown>) => {
      const firstBinding = value.bindings as Array<Record<string, unknown>>;
      firstBinding[0]!.raw_workflow = {};
    }]
  ] as const) {
    assert.throws(
      () => parseLocalWorkflowContextBindingsV1(mutatePayload(payload, mutate)),
      /unexpected shape/,
      name
    );
  }
});

test("local Workflow binding data adapts loading, error, empty, available, and unavailable states", () => {
  const available = parseLocalWorkflowContextBindingsV1({
    schema_version: LOCAL_WORKFLOW_CONTEXT_BINDINGS_SCHEMA_V1,
    context_id: target.context_id,
    commit_id: target.commit_id,
    bindings: [binding]
  });
  const empty = parseLocalWorkflowContextBindingsV1({
    schema_version: LOCAL_WORKFLOW_CONTEXT_BINDINGS_SCHEMA_V1,
    context_id: target.context_id,
    commit_id: target.commit_id,
    bindings: []
  });
  const cases: ReadonlyArray<Readonly<{ expectedState: "loading" | "error" | "empty" | "available" | "unavailable"; resource: LocalWorkflowContextBindingsResource }>> = [
    { expectedState: "loading", resource: { kind: "loading", target } },
    { expectedState: "error", resource: { kind: "error", target } },
    { expectedState: "empty", resource: { kind: "empty", target } },
    { expectedState: "empty", resource: { kind: "ready", target, summary: empty } },
    { expectedState: "available", resource: { kind: "ready", target, summary: available } },
    { expectedState: "unavailable", resource: { kind: "unavailable", target } }
  ];

  for (const { expectedState, resource } of cases) {
    const dto = adaptLocalWorkflowContextBindingsV1(resource);
    const view = presentLocalWorkflowContextBindings(resource);
    const markup = renderToStaticMarkup(<LocalWorkflowContextBindingsScreen resource={resource} />);

    assert.equal(dto.state, expectedState);
    assert.equal(view.status.state, expectedState);
    assert.match(markup, new RegExp(`data-state="${expectedState}"`));
    assert.match(markup, /Workflow context bindings \/ Workflow 上下文绑定/);
    assert.match(markup, /Context \/ 上下文/);
    assert.match(markup, /Commit \/ 提交/);

    if (expectedState === "error") {
      assert.match(markup, /role="alert"/);
      assert.match(markup, /aria-live="assertive"/);
    } else {
      assert.match(markup, /role="status"/);
      assert.match(markup, /aria-live="polite"/);
    }

    if (expectedState === "loading") {
      assert.match(markup, /aria-busy="true"/);
    } else {
      assert.doesNotMatch(markup, /aria-busy="true"/);
    }

    if (expectedState === "available") {
      assert.match(markup, /binding-001/);
      assert.match(markup, /workflow-001/);
      assert.match(markup, />7</);
      assert.match(markup, />3</);
      assert.match(markup, />2</);
    } else {
      assert.doesNotMatch(markup, /binding-001/);
      assert.doesNotMatch(markup, /workflow-001/);
    }

    assert.doesNotMatch(markup, /nodes|edges|raw workflow|provider invocation/i);
  }
});

test("binding inspector starts as an accessible empty read for the exact selected Context commit", () => {
  const markup = renderToStaticMarkup(
    <LocalWorkflowContextBindingsInspector contextId="context/id" commitId="commit/042" />
  );

  assert.match(markup, /aria-labelledby="local-workflow-context-bindings-inspector-heading"/);
  assert.match(markup, /Selected commit bindings \/ 选定提交绑定/);
  assert.match(markup, /Context \/ 上下文: <code>context\/id<\/code>/);
  assert.match(markup, /Commit \/ 提交: <code>commit\/042<\/code>/);
  assert.match(markup, /data-state="empty"/);
  assert.match(markup, /role="status"/);
  assert.match(markup, /aria-live="polite"/);
  assert.match(markup, /name="workflow-context-bindings-bearer-token"/);
  assert.match(markup, /Inspect bindings \/ 审阅绑定/);
  assert.match(markup, /disabled=""/);
  assert.match(markup, /<select[^>]*disabled=""[^>]*name="workflow-context-bindings-selected-binding"/);
  assert.match(markup, /<input[^>]*disabled=""[^>]*name="workflow-context-bindings-workflow-run-id"/);
  assert.match(markup, /Inspect run status \/ 审阅运行状态/);
  assert.doesNotMatch(markup, /preview-commit|current-head|raw workflow/i);
});

test("binding inspector performs an exact BFF read and projects the redacted ready state", async () => {
  const requests: Array<{ input: string; init?: RequestInit }> = [];
  const originalFetch = globalThis.fetch;
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    requests.push({ input: String(input), init });
    return new Response(JSON.stringify({
      schema_version: LOCAL_WORKFLOW_CONTEXT_BINDINGS_SCHEMA_V1,
      context_id: "context/id",
      commit_id: "commit/042",
      bindings: [binding]
    }), {
      headers: { "content-type": "application/json" }
    });
  }) as typeof fetch;

  try {
    const harness = createWorkflowBindingsInspectorHarness();
    changeWorkflowInspector(harness, "request-token");
    const button = findElement(harness.tree, (element) =>
      typeof element.props.onClick === "function"
      && textContent(element.props.children as React.ReactNode).includes("Inspect bindings")
    );
    assert.ok(button);
    assert.notEqual(button.props.disabled, true);

    await (button.props.onClick as () => Promise<void>)();
    harness.render();

    assert.deepEqual(requests.map(({ input }) => input), [
      "/api/local/contexts/context%2Fid/commits/commit%2F042/workflow-bindings"
    ]);
    assert.deepEqual(requests[0]?.init, {
      headers: {
        accept: "application/json",
        authorization: "Bearer request-token"
      },
      credentials: "omit",
      cache: "no-store"
    });

    const screen = findElement(harness.tree, (element) =>
      element.type === LocalWorkflowContextBindingsScreen
    );
    assert.ok(screen);
    const resource = screen.props.resource as LocalWorkflowContextBindingsResource;
    assert.equal(resource.kind, "ready");
    if (resource.kind === "ready") {
      assert.equal(resource.summary.context_id, "context/id");
      assert.equal(resource.summary.commit_id, "commit/042");
      assert.deepEqual(resource.summary.bindings, [binding]);
      const markup = renderToStaticMarkup(<LocalWorkflowContextBindingsScreen resource={resource} />);
      assert.match(markup, /binding-001/);
      assert.match(markup, /workflow-001/);
    }
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("binding inspector mounts status for the exact selected row and canonical run URL", async () => {
  const requests: Array<{ input: string; init?: RequestInit }> = [];
  const originalFetch = globalThis.fetch;
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    const request = { input: String(input), init };
    requests.push(request);
    if (request.input.endsWith("/workflow-bindings")) {
      return new Response(JSON.stringify({
        schema_version: LOCAL_WORKFLOW_CONTEXT_BINDINGS_SCHEMA_V1,
        context_id: executionContextId,
        commit_id: executionCommitId,
        bindings: [executionBinding]
      }), { headers: { "content-type": "application/json" } });
    }
    return new Response(JSON.stringify(executionStatusPayload()), {
      headers: { "content-type": "application/json" }
    });
  }) as typeof fetch;

  try {
    const harness = createWorkflowBindingsInspectorHarness(executionCommitId, executionContextId);
    changeWorkflowInspector(harness, "request-token");
    const bindingsButton = findElement(harness.tree, (element) =>
      typeof element.props.onClick === "function"
      && textContent(element.props.children as React.ReactNode).includes("Inspect bindings")
    );
    assert.ok(bindingsButton);
    await (bindingsButton.props.onClick as () => Promise<void>)();
    harness.render();

    const bindingSelect = findElement(harness.tree, (element) =>
      element.props.name === "workflow-context-bindings-selected-binding"
    );
    assert.ok(bindingSelect);
    (bindingSelect.props.onChange as (event: { target: { value: string } }) => void)({
      target: { value: executionBinding.binding_id }
    });
    const runInput = findElement(harness.tree, (element) =>
      element.props.name === "workflow-context-bindings-workflow-run-id"
    );
    assert.ok(runInput);
    (runInput.props.onChange as (event: { target: { value: string } }) => void)({
      target: { value: executionRunId }
    });
    harness.render();

    const statusButton = findElement(harness.tree, (element) =>
      typeof element.props.onClick === "function"
      && textContent(element.props.children as React.ReactNode).includes("Inspect run status")
    );
    assert.ok(statusButton);
    assert.notEqual(statusButton.props.disabled, true);
    await (statusButton.props.onClick as () => Promise<void>)();
    harness.render();

    assert.deepEqual(requests.map(({ input }) => input), [
      `/api/local/contexts/${executionContextId}/commits/${executionCommitId}/workflow-bindings`,
      `/api/local/contexts/${executionContextId}/workflow/runs/${executionRunId}/status`
    ]);
    assert.deepEqual(requests[1]?.init, {
      method: "GET",
      credentials: "omit",
      cache: "no-store",
      headers: {
        accept: "application/json",
        authorization: "Bearer request-token"
      }
    });

    const screen = findElement(harness.tree, (element) =>
      element.type === LocalWorkflowExecutionStatusScreen
    );
    assert.ok(screen);
    const resource = screen.props.resource as { kind: string; target: Record<string, unknown>; status?: Record<string, unknown> };
    assert.equal(resource.kind, "ready");
    assert.equal(resource.target.binding_id, executionBinding.binding_id);
    assert.equal(resource.target.workflow_id, executionBinding.workflow_id);
    assert.equal(resource.target.workflow_revision, executionBinding.workflow_revision);
    assert.equal(resource.target.run_id, executionRunId);
    const markup = renderToStaticMarkup(
      <LocalWorkflowExecutionStatusScreen resource={resource as LocalWorkflowExecutionStatusResource} />
    );
    assert.match(markup, /data-state="available"/);
    assert.match(markup, /Failed \/ 已失败/);
    assert.match(markup, /Event count \/ 事件数/);
    assert.match(markup, /sha256:status-digest/);
    assert.doesNotMatch(markup, /raw_workflow|provider invocation|secret/i);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("binding inspector rejects an invalid run UUID before any status request", async () => {
  const requests: string[] = [];
  const originalFetch = globalThis.fetch;
  globalThis.fetch = (async (input: string | URL | Request) => {
    requests.push(String(input));
    return new Response(JSON.stringify({
      schema_version: LOCAL_WORKFLOW_CONTEXT_BINDINGS_SCHEMA_V1,
      context_id: executionContextId,
      commit_id: executionCommitId,
      bindings: [executionBinding]
    }), { headers: { "content-type": "application/json" } });
  }) as typeof fetch;

  try {
    const harness = createWorkflowBindingsInspectorHarness(executionCommitId, executionContextId);
    changeWorkflowInspector(harness, "request-token");
    const bindingsButton = findElement(harness.tree, (element) =>
      typeof element.props.onClick === "function"
      && textContent(element.props.children as React.ReactNode).includes("Inspect bindings")
    );
    assert.ok(bindingsButton);
    await (bindingsButton.props.onClick as () => Promise<void>)();
    harness.render();

    const bindingSelect = findElement(harness.tree, (element) =>
      element.props.name === "workflow-context-bindings-selected-binding"
    );
    assert.ok(bindingSelect);
    (bindingSelect.props.onChange as (event: { target: { value: string } }) => void)({
      target: { value: executionBinding.binding_id }
    });
    const runInput = findElement(harness.tree, (element) =>
      element.props.name === "workflow-context-bindings-workflow-run-id"
    );
    assert.ok(runInput);
    (runInput.props.onChange as (event: { target: { value: string } }) => void)({
      target: { value: "not-a-canonical-run-id" }
    });
    harness.render();

    const statusButton = findElement(harness.tree, (element) =>
      typeof element.props.onClick === "function"
      && textContent(element.props.children as React.ReactNode).includes("Inspect run status")
    );
    assert.ok(statusButton);
    assert.equal(statusButton.props.disabled, true);
    assert.equal(requests.length, 1);
    const notice = findElement(harness.tree, (element) => element.props.role === "alert");
    assert.ok(notice);
    assert.match(textContent(notice), /canonical workflow run UUID/);
    assert.doesNotMatch(textContent(notice), /not-a-canonical-run-id/);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("binding inspector fails closed when status response scope drifts", async () => {
  const originalFetch = globalThis.fetch;
  globalThis.fetch = (async (input: string | URL | Request) => {
    if (String(input).endsWith("/workflow-bindings")) {
      return new Response(JSON.stringify({
        schema_version: LOCAL_WORKFLOW_CONTEXT_BINDINGS_SCHEMA_V1,
        context_id: executionContextId,
        commit_id: executionCommitId,
        bindings: [executionBinding]
      }), { headers: { "content-type": "application/json" } });
    }
    return new Response(JSON.stringify({
      ...executionStatusPayload(),
      context_commit_id: "ffffffff-ffff-4fff-8fff-ffffffffffff"
    }), { headers: { "content-type": "application/json" } });
  }) as typeof fetch;

  try {
    const harness = createWorkflowBindingsInspectorHarness(executionCommitId, executionContextId);
    changeWorkflowInspector(harness, "request-token");
    const bindingsButton = findElement(harness.tree, (element) =>
      typeof element.props.onClick === "function"
      && textContent(element.props.children as React.ReactNode).includes("Inspect bindings")
    );
    assert.ok(bindingsButton);
    await (bindingsButton.props.onClick as () => Promise<void>)();
    harness.render();

    const bindingSelect = findElement(harness.tree, (element) =>
      element.props.name === "workflow-context-bindings-selected-binding"
    );
    assert.ok(bindingSelect);
    (bindingSelect.props.onChange as (event: { target: { value: string } }) => void)({
      target: { value: executionBinding.binding_id }
    });
    const runInput = findElement(harness.tree, (element) =>
      element.props.name === "workflow-context-bindings-workflow-run-id"
    );
    assert.ok(runInput);
    (runInput.props.onChange as (event: { target: { value: string } }) => void)({
      target: { value: executionRunId }
    });
    harness.render();
    const statusButton = findElement(harness.tree, (element) =>
      typeof element.props.onClick === "function"
      && textContent(element.props.children as React.ReactNode).includes("Inspect run status")
    );
    assert.ok(statusButton);
    await (statusButton.props.onClick as () => Promise<void>)();
    harness.render();

    const screen = findElement(harness.tree, (element) =>
      element.type === LocalWorkflowExecutionStatusScreen
    );
    assert.ok(screen);
    const resource = screen.props.resource as { kind: string; target: Record<string, unknown> };
    assert.equal(resource.kind, "error");
    assert.equal(resource.target.context_commit_id, executionCommitId);
    const markup = renderToStaticMarkup(
      <LocalWorkflowExecutionStatusScreen resource={resource as LocalWorkflowExecutionStatusResource} />
    );
    assert.doesNotMatch(markup, /ffffffff-ffff-4fff-8fff-ffffffffffff/);
    const notice = findElement(harness.tree, (element) => element.props.role === "alert");
    assert.ok(notice);
    assert.match(textContent(notice), /response was rejected/);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("binding inspector preserves the selected commit and exposes a typed failure state", async () => {
  const originalFetch = globalThis.fetch;
  globalThis.fetch = (async () => new Response(
    JSON.stringify({ error: "context_read_forbidden", message: "Context read denied" }),
    { status: 403, headers: { "content-type": "application/json" } }
  )) as typeof fetch;

  try {
    const harness = createWorkflowBindingsInspectorHarness();
    changeWorkflowInspector(harness, "request-token");
    const button = findElement(harness.tree, (element) =>
      typeof element.props.onClick === "function"
      && textContent(element.props.children as React.ReactNode).includes("Inspect bindings")
    );
    assert.ok(button);

    await (button.props.onClick as () => Promise<void>)();
    harness.render();

    const screen = findElement(harness.tree, (element) =>
      element.type === LocalWorkflowContextBindingsScreen
    );
    assert.ok(screen);
    const resource = screen.props.resource as LocalWorkflowContextBindingsResource;
    assert.equal(resource.kind, "error");
    assert.equal(resource.target.context_id, "context/id");
    assert.equal(resource.target.commit_id, "commit/042");

    const notice = findElement(harness.tree, (element) => element.props.role === "alert");
    assert.ok(notice);
    assert.match(textContent(notice), /You do not have permission for this Context commit/);
    assert.doesNotMatch(textContent(notice), /Context read denied/);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("binding inspector maps service unavailability to the shared unavailable state", async () => {
  const originalFetch = globalThis.fetch;
  globalThis.fetch = (async () => new Response(
    JSON.stringify({ error: "contextlab_workflow_context_bindings_unavailable", message: "upstream unavailable" }),
    { status: 503, headers: { "content-type": "application/json" } }
  )) as typeof fetch;

  try {
    const harness = createWorkflowBindingsInspectorHarness();
    changeWorkflowInspector(harness, "request-token");
    const button = findElement(harness.tree, (element) =>
      typeof element.props.onClick === "function"
      && textContent(element.props.children as React.ReactNode).includes("Inspect bindings")
    );
    assert.ok(button);

    await (button.props.onClick as () => Promise<void>)();
    harness.render();

    const screen = findElement(harness.tree, (element) =>
      element.type === LocalWorkflowContextBindingsScreen
    );
    assert.ok(screen);
    const resource = screen.props.resource as LocalWorkflowContextBindingsResource;
    assert.equal(resource.kind, "unavailable");
    assert.equal(resource.target.context_id, "context/id");
    assert.equal(resource.target.commit_id, "commit/042");

    const notice = findElement(harness.tree, (element) => element.props.role === "alert");
    assert.ok(notice);
    assert.match(textContent(notice), /Workflow context bindings are unavailable/);
    assert.doesNotMatch(textContent(notice), /upstream unavailable/);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("binding inspector ignores a late response from the previously selected commit", async () => {
  const originalFetch = globalThis.fetch;
  let resolveOldResponse!: (response: Response) => void;
  globalThis.fetch = (async () => new Promise<Response>((resolve) => {
    resolveOldResponse = resolve;
  })) as typeof fetch;

  try {
    const harness = createWorkflowBindingsInspectorHarness("commit-old");
    changeWorkflowInspector(harness, "request-token", "commit-old");
    const oldButton = findElement(harness.tree, (element) =>
      typeof element.props.onClick === "function"
      && textContent(element.props.children as React.ReactNode).includes("Inspect bindings")
    );
    assert.ok(oldButton);

    const oldInspection = (oldButton.props.onClick as () => Promise<void>)();
    harness.render("commit-new");
    harness.render("commit-new");

    resolveOldResponse(new Response(JSON.stringify({
      schema_version: LOCAL_WORKFLOW_CONTEXT_BINDINGS_SCHEMA_V1,
      context_id: "context/id",
      commit_id: "commit-old",
      bindings: [binding]
    }), {
      headers: { "content-type": "application/json" }
    }));
    await oldInspection;
    harness.render("commit-new");

    const screen = findElement(harness.tree, (element) =>
      element.type === LocalWorkflowContextBindingsScreen
    );
    assert.ok(screen);
    const resource = screen.props.resource as LocalWorkflowContextBindingsResource;
    assert.equal(resource.kind, "empty");
    assert.equal(resource.target.commit_id, "commit-new");
    assert.doesNotMatch(textContent(screen), /commit-old|binding-001/);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

type WorkflowInspectorTree = React.ReactElement<Record<string, unknown>>;
type WorkflowHookSlot =
  | { kind: "state"; value: unknown }
  | { kind: "ref"; value: { current: unknown } };

function createWorkflowBindingsInspectorHarness(
  initialCommitId = "commit/042",
  initialContextId = "context/id"
) {
  const slots: WorkflowHookSlot[] = [];
  let cursor = 0;
  let tree: WorkflowInspectorTree;
  let memoizedValue: unknown;
  let memoizedDependencies: React.DependencyList | undefined;
  let hasMemoizedValue = false;
  let effectDependencies: React.DependencyList | undefined;
  let hasMountedEffect = false;
  const internals = React as unknown as {
    __CLIENT_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE: { H: unknown };
  };
  const dispatcher = {
    useState<T>(initialValue: T | (() => T)): [T, React.Dispatch<React.SetStateAction<T>>] {
      const index = cursor;
      cursor += 1;
      if (!slots[index]) {
        slots[index] = {
          kind: "state",
          value: typeof initialValue === "function" ? (initialValue as () => T)() : initialValue
        };
      }
      const slot = slots[index];
      assert.equal(slot?.kind, "state");
      const setValue: React.Dispatch<React.SetStateAction<T>> = (nextValue) => {
        assert.equal(slot?.kind, "state");
        slot.value = typeof nextValue === "function"
          ? (nextValue as (previous: T) => T)(slot.value as T)
          : nextValue;
      };
      return [slot.value as T, setValue];
    },
    useRef<T>(initialValue: T): { current: T } {
      const index = cursor;
      cursor += 1;
      if (!slots[index]) {
        slots[index] = { kind: "ref", value: { current: initialValue } };
      }
      const slot = slots[index];
      assert.equal(slot?.kind, "ref");
      return slot.value as { current: T };
    },
    useMemo<T>(factory: () => T, dependencies?: React.DependencyList): T {
      cursor += 1;
      if (!hasMemoizedValue || dependenciesChanged(memoizedDependencies, dependencies)) {
        memoizedValue = factory();
        memoizedDependencies = dependencies;
        hasMemoizedValue = true;
      }
      return memoizedValue as T;
    },
    useEffect(effect: React.EffectCallback, dependencies?: React.DependencyList) {
      cursor += 1;
      if (!hasMountedEffect || dependenciesChanged(effectDependencies, dependencies)) {
        effectDependencies = dependencies;
        hasMountedEffect = true;
        effect();
      }
    }
  };

  const harness = {
    get tree() {
      return tree;
    },
    render(commitId = initialCommitId) {
      cursor = 0;
      const previousDispatcher =
        internals.__CLIENT_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE.H;
      internals.__CLIENT_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE.H = dispatcher;
      try {
        tree = LocalWorkflowContextBindingsInspector({
          contextId: initialContextId,
          commitId
        }) as WorkflowInspectorTree;
      } finally {
        internals.__CLIENT_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE.H = previousDispatcher;
      }
      return tree;
    }
  };
  harness.render(initialCommitId);
  return harness;
}

function dependenciesChanged(
  previous: React.DependencyList | undefined,
  next: React.DependencyList | undefined
): boolean {
  if (!previous || !next || previous.length !== next.length) {
    return true;
  }
  return next.some((value, index) => !Object.is(value, previous[index]));
}

function changeWorkflowInspector(
  harness: ReturnType<typeof createWorkflowBindingsInspectorHarness>,
  value: string,
  commitId?: string
) {
  const input = findElement(harness.tree, (element) =>
    element.props.name === "workflow-context-bindings-bearer-token"
  );
  assert.ok(input);
  (input.props.onChange as (event: { target: { value: string } }) => void)({
    target: { value }
  });
  harness.render(commitId);
}

function findElement(
  node: React.ReactNode,
  predicate: (element: WorkflowInspectorTree) => boolean
): WorkflowInspectorTree | undefined {
  if (!React.isValidElement(node)) {
    return undefined;
  }
  const element = node as WorkflowInspectorTree;
  if (predicate(element)) {
    return element;
  }
  for (const child of React.Children.toArray(element.props.children as React.ReactNode)) {
    const match = findElement(child, predicate);
    if (match) {
      return match;
    }
  }
  return undefined;
}

function textContent(node: React.ReactNode): string {
  if (typeof node === "string" || typeof node === "number") {
    return String(node);
  }
  if (!React.isValidElement(node)) {
    return React.Children.toArray(node).map(textContent).join(" ");
  }
  return textContent((node as WorkflowInspectorTree).props.children as React.ReactNode);
}

function mutatePayload<T>(payload: T, mutate: (value: T) => void): T {
  const copy = structuredClone(payload);
  mutate(copy);
  return copy;
}

function executionStatusPayload() {
  return {
    schema_version: "contextlab.local-workflow-execution-status.v1",
    context_id: executionContextId,
    context_commit_id: executionCommitId,
    binding_id: executionBinding.binding_id,
    workflow_id: executionBinding.workflow_id,
    workflow_revision: executionBinding.workflow_revision,
    run_id: executionRunId,
    replay_of: null,
    run_state: "failed",
    event_count: 6,
    last_event_sequence: 6,
    capability_snapshot_digest: "sha256:status-digest",
    node_status_counts: { pending: 0, running: 0, succeeded: 2, failed: 1, blocked: 3 }
  } as const;
}
