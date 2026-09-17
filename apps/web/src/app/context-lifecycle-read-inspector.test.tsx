import assert from "node:assert/strict";
import test from "node:test";
import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import type { LocalContextLifecycleState } from "@contextlab/local-sdk";
import {
  ContextLifecycleReadInspector,
  ContextLifecycleReadInspectorScreen
} from "./context-lifecycle-read-inspector";
import type { ContextLifecycleReadResource } from "./context-lifecycle-presenter";

const contextId = "11111111-1111-4111-8111-111111111111";
const commitId = "22222222-2222-4222-8222-222222222222";

test("lifecycle read screen renders bilingual exact-scope read-only available state", () => {
  const markup = renderToStaticMarkup(
    <ContextLifecycleReadInspectorScreen resource={availableResource(contextId, commitId)} />
  );

  assert.match(markup, /Context lifecycle state/);
  assert.match(markup, /Context 生命周期状态/);
  assert.match(markup, /read-only inspection \/ 受保护的本地只读检查/);
  assert.match(markup, new RegExp(contextId));
  assert.match(markup, new RegExp(commitId));
  assert.match(markup, /data-state="available"/);
  assert.match(markup, /role="status"/);
  assert.match(markup, /Instruction/);
  assert.match(markup, /private body/);
  assert.match(markup, /Relationships \/ 关系/);
  assert.doesNotMatch(markup, /GraphDiff|graph diff|semantic diff/i);
});

test("lifecycle read inspector starts empty and exposes only a memory-only credential control", () => {
  const markup = renderToStaticMarkup(
    <ContextLifecycleReadInspector contextId={contextId} commitId={commitId} />
  );

  assert.match(markup, /data-state="empty"/);
  assert.match(markup, /Bearer token \/ 访问令牌/);
  assert.match(markup, /type="password"/);
  assert.match(markup, /Memory only \/ 仅内存/);
  assert.match(markup, /Inspect lifecycle state \/ 审阅生命周期状态/);
  assert.match(markup, /No credential is persisted \/ 不会持久化凭据/);
  assert.match(markup, new RegExp(commitId));
  assert.doesNotMatch(markup, /textarea|submit|write|mutation/i);
});

test("lifecycle read screen keeps error and loading status semantics bilingual", () => {
  for (const kind of ["loading", "error", "empty"] as const) {
    const markup = renderToStaticMarkup(
      <ContextLifecycleReadInspectorScreen
        resource={{ kind, target: { contextId, commitId }, ...(kind === "error" ? { message: "private upstream detail" } : {}) }}
      />
    );

    assert.match(markup, new RegExp(`data-state="${kind}"`));
    assert.match(markup, kind === "error" ? /role="alert"/ : /role="status"/);
    assert.match(markup, /Context lifecycle state/);
    assert.match(markup, /Context 生命周期状态/);
    assert.doesNotMatch(markup, /private upstream detail/);
  }
});

test("lifecycle read inspector maps a 503 response to an unavailable polite status", async () => {
  const originalFetch = globalThis.fetch;
  globalThis.fetch = (async () => new Response(JSON.stringify({
    error: "contextlab_web_api_unavailable",
    message: "private upstream detail"
  }), {
    status: 503,
    headers: { "content-type": "application/json", "cache-control": "private, no-store" }
  })) as typeof fetch;

  try {
    const harness = createInspectorHarness(commitId);
    const tokenInput = findElement(harness.tree, (element) => element.props.name === "context-lifecycle-read-bearer-token");
    assert.ok(tokenInput);
    (tokenInput.props.onChange as (event: { target: { value: string } }) => void)({
      target: { value: "request-token" }
    });
    harness.render(commitId);

    const inspectButton = findElement(harness.tree, (element) =>
      typeof element.props.onClick === "function"
      && textContent(element.props.children as React.ReactNode).includes("Inspect lifecycle state")
    );
    assert.ok(inspectButton);
    await (inspectButton.props.onClick as () => Promise<void>)();
    harness.render(commitId);

    const screen = findElement(harness.tree, (element) => element.type === ContextLifecycleReadInspectorScreen);
    assert.ok(screen);
    const resource = screen.props.resource as ContextLifecycleReadResource;
    assert.equal(resource.kind, "unavailable");

    const markup = renderToStaticMarkup(
      <ContextLifecycleReadInspectorScreen resource={resource} />
    );
    assert.match(markup, /data-state="unavailable"/);
    assert.match(markup, /role="status"/);
    assert.match(markup, /aria-live="polite"/);
    assert.match(markup, /aria-atomic="true"/);
    assert.match(markup, /Unavailable \/ 不可用/);
    assert.doesNotMatch(markup, /private upstream detail/);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("lifecycle read inspector resets exact state on commit change and ignores a late prior response", async () => {
  const originalFetch = globalThis.fetch;
  let resolveOldResponse!: (response: Response) => void;
  globalThis.fetch = (async () => new Promise<Response>((resolve) => {
    resolveOldResponse = resolve;
  })) as typeof fetch;

  try {
    const harness = createInspectorHarness(commitId);
    const tokenInput = findElement(harness.tree, (element) => element.props.name === "context-lifecycle-read-bearer-token");
    assert.ok(tokenInput);
    (tokenInput.props.onChange as (event: { target: { value: string } }) => void)({
      target: { value: "request-token" }
    });
    harness.render(commitId);

    const inspectButton = findElement(harness.tree, (element) =>
      typeof element.props.onClick === "function"
      && textContent(element.props.children as React.ReactNode).includes("Inspect lifecycle state")
    );
    assert.ok(inspectButton);
    const oldInspection = (inspectButton.props.onClick as () => Promise<void>)();

    const nextCommitId = "44444444-4444-4444-8444-444444444444";
    harness.render(nextCommitId);
    harness.render(nextCommitId);
    let screen = findElement(harness.tree, (element) => element.type === ContextLifecycleReadInspectorScreen);
    assert.ok(screen);
    let resource = screen.props.resource as ContextLifecycleReadResource;
    assert.equal(resource.kind, "empty");
    assert.equal(resource.target.commitId, nextCommitId);

    resolveOldResponse(new Response(JSON.stringify(availableResource(contextId, commitId).state), {
      headers: { "content-type": "application/json" }
    }));
    await oldInspection;
    harness.render(nextCommitId);

    screen = findElement(harness.tree, (element) => element.type === ContextLifecycleReadInspectorScreen);
    assert.ok(screen);
    resource = screen.props.resource as ContextLifecycleReadResource;
    assert.equal(resource.kind, "empty");
    assert.equal(resource.target.commitId, nextCommitId);
    assert.doesNotMatch(textContent(screen), /private body|22222222/);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

function availableResource(
  nextContextId: string,
  nextCommitId: string
): Extract<ContextLifecycleReadResource, { kind: "available" }> {
  return {
    kind: "available",
    target: { contextId: nextContextId, commitId: nextCommitId },
    state: {
      schema_version: "contextlab.local-context-lifecycle-state.v1",
      context_id: nextContextId,
      commit_id: nextCommitId,
      metadata: null,
      components: [
        {
          component_id: "component-a",
          component_kind: "prompt",
          name: "Instruction",
          metadata: { locale: "en-US" },
          content: "private body",
          content_hash: "hash-a",
          creation_commit_id: nextCommitId,
          content_commit_id: nextCommitId
        }
      ],
      graph_snapshot: {
        project_id: "33333333-3333-4333-8333-333333333333",
        context_id: nextContextId,
        commit_id: nextCommitId,
        graph: {
          nodes: {
            "component:component-a": { id: "component:component-a", kind: "prompt", label: "Instruction" }
          },
          edges: []
        },
        captured_at: "2026-08-02T00:00:00Z",
        schema_version: 1
      }
    } as LocalContextLifecycleState
  };
}

type InspectorTree = React.ReactElement<Record<string, unknown>>;
type InspectorHookSlot =
  | { kind: "state"; value: unknown }
  | { kind: "ref"; value: { current: unknown } };

function createInspectorHarness(initialCommitId: string) {
  const slots: InspectorHookSlot[] = [];
  let cursor = 0;
  let tree: InspectorTree;
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
      const index = cursor++;
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
      const index = cursor++;
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
    render(nextCommitId = initialCommitId) {
      cursor = 0;
      const previousDispatcher =
        internals.__CLIENT_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE.H;
      internals.__CLIENT_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE.H = dispatcher;
      try {
        tree = ContextLifecycleReadInspector({ contextId, commitId: nextCommitId }) as InspectorTree;
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

function findElement(
  root: React.ReactNode,
  predicate: (element: React.ReactElement<Record<string, unknown>>) => boolean
): React.ReactElement<Record<string, unknown>> | undefined {
  if (!React.isValidElement(root)) {
    return undefined;
  }
  const element = root as React.ReactElement<Record<string, unknown>>;
  if (predicate(element)) {
    return element;
  }
  const children = element.props.children;
  if (Array.isArray(children)) {
    for (const child of children) {
      const found = findElement(child, predicate);
      if (found) {
        return found;
      }
    }
  } else {
    return findElement(children as React.ReactNode, predicate);
  }
  return undefined;
}

function textContent(node: React.ReactNode): string {
  if (node === null || node === undefined || typeof node === "boolean") {
    return "";
  }
  if (typeof node === "string" || typeof node === "number") {
    return String(node);
  }
  if (Array.isArray(node)) {
    return node.map(textContent).join("");
  }
  if (React.isValidElement(node)) {
    return textContent((node as React.ReactElement<{ children?: React.ReactNode }>).props.children);
  }
  return "";
}
