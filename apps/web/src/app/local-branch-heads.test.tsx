import assert from "node:assert/strict";
import test from "node:test";
import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import {
  LOCAL_CONTEXT_BRANCH_HEADS_SCHEMA_V1,
  createInitialLocalBranchHeadsResource,
  loadLocalBranchHeads,
  LocalBranchHeadsProxyError,
  parseLocalBranchHeadsV1,
  selectedLocalBranchHeadTarget,
  selectedLocalBranchHeadCommit,
  type LocalBranchHeadTarget,
  type LocalBranchHeadsResource
} from "./local-branch-heads-data";
import { presentLocalBranchHeads } from "./local-branch-heads-presenter";
import { LocalBranchHeadsInspector } from "./local-branch-heads-inspector";
import { LocalBranchHeadsScreen } from "./local-branch-heads-screen";

Object.assign(globalThis, { React });

const contextId = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
const target = Object.freeze({
  contextId,
  capability: Object.freeze({ en: "Local branch heads", zh: "本地分支 head" })
});

function payload(branches: unknown[] = [
  { branch_name: "feature/read-only", head_commit_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb", revision: 4 },
  { branch_name: "main", head_commit_id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc", revision: 9 }
]) {
  return {
    schema_version: LOCAL_CONTEXT_BRANCH_HEADS_SCHEMA_V1,
    context_id: contextId,
    branches
  };
}

function jsonResponse(body: unknown, init: ResponseInit = {}) {
  return new Response(JSON.stringify(body), {
    headers: { "content-type": "application/json" },
    ...init
  });
}

test("branch-head data loader sends only the request-memory credential and preserves server order", async () => {
  const calls: Array<{ input: string; init?: RequestInit }> = [];
  const originalFetch = globalThis.fetch;
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    calls.push({ input: String(input), init });
    return jsonResponse(payload());
  }) as typeof fetch;

  try {
    const result = await loadLocalBranchHeads(target, " request-token ");
    assert.deepEqual(result.branches.map((branch) => branch.branch_name), ["feature/read-only", "main"]);
    assert.equal(Object.isFrozen(result), true);
    assert.equal(calls[0]?.input, `/api/local/contexts/${contextId}/branches`);
    assert.equal(new Headers(calls[0]?.init?.headers).get("authorization"), "Bearer request-token");
    assert.equal(new Headers(calls[0]?.init?.headers).get("cookie"), null);
    assert.equal(calls[0]?.init?.credentials, "omit");
    assert.equal(calls[0]?.init?.cache, "no-store");
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("malformed branch-head payloads fail closed and unavailable starts without a credential or capability", async () => {
  assert.throws(
    () => parseLocalBranchHeadsV1({ ...payload(), unexpected: true }),
    /unexpected shape/
  );
  assert.throws(
    () => parseLocalBranchHeadsV1(payload([
      { branch_name: "main", head_commit_id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc", revision: -1 }
    ])),
    /revision/
  );
  assert.throws(
    () => parseLocalBranchHeadsV1(payload([
      { branch_name: "main", head_commit_id: null, revision: 1 },
      { branch_name: "feature/read-only", head_commit_id: null, revision: 2 }
    ])),
    /ordered/
  );
  assert.throws(
    () => parseLocalBranchHeadsV1({ ...payload(), context_id: "not-a-uuid" }),
    /UUID/
  );
  assert.throws(
    () => parseLocalBranchHeadsV1(payload([
      { branch_name: "feature read-only", head_commit_id: null, revision: 1 }
    ])),
    /characters/
  );
  assert.equal(
    createInitialLocalBranchHeadsResource(contextId).kind,
    "unavailable"
  );

  const originalFetch = globalThis.fetch;
  globalThis.fetch = (async () => jsonResponse(
    { error: "branch_heads_unavailable", message: "Branch heads are unavailable" },
    { status: 503 }
  )) as typeof fetch;
  try {
    await assert.rejects(
      () => loadLocalBranchHeads(contextId, "request-token"),
      (error: unknown) => error instanceof LocalBranchHeadsProxyError
        && error.status === 503
        && error.body.error === "branch_heads_unavailable"
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("branch-head data adapter redacts unknown-status upstream messages", async () => {
  const originalFetch = globalThis.fetch;
  globalThis.fetch = (async () => jsonResponse(
    { error: "upstream_private_error", message: "private upstream details" },
    { status: 599 }
  )) as typeof fetch;

  try {
    await assert.rejects(
      () => loadLocalBranchHeads(contextId, "request-token"),
      (error: unknown) => error instanceof LocalBranchHeadsProxyError
        && error.status === 599
        && error.body.error === "upstream_private_error"
        && error.body.message === "Unable to load local branch heads / 无法加载本地分支 head。"
        && error.message === error.body.message
        && !error.body.message.includes("private upstream details")
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("branch-head data adapter redacts unknown-status upstream diagnostics", async () => {
  const originalFetch = globalThis.fetch;
  globalThis.fetch = (async () => jsonResponse(
    { error: "branch_heads_unknown", message: "upstream diagnostic leak" },
    { status: 418 }
  )) as typeof fetch;

  try {
    await assert.rejects(
      () => loadLocalBranchHeads(contextId, "request-token"),
      (error: unknown) => error instanceof LocalBranchHeadsProxyError
        && error.status === 418
        && error.body.error === "branch_heads_unknown"
        && error.body.message === "Unable to load local branch heads / 无法加载本地分支 head。"
        && !error.body.message.includes("upstream diagnostic leak")
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("presenter exposes bilingual loading, error, empty, ready, and unavailable states", () => {
  const summary = parseLocalBranchHeadsV1(payload());
  const resources: ReadonlyArray<LocalBranchHeadsResource> = [
    { kind: "loading", target },
    { kind: "error", target, message: "Read failed / 读取失败" },
    { kind: "empty", target },
    { kind: "ready", target, summary },
    { kind: "unavailable", target }
  ];

  for (const resource of resources) {
    const view = presentLocalBranchHeads(resource);
    assert.equal(Object.isFrozen(view), true);
    assert.match(view.status.stateLabel, /\/ /);
    assert.match(view.status.description, /\/ /);
    assert.equal(view.scope[0]?.value, contextId);
  }

  assert.equal(presentLocalBranchHeads(resources[3]!).state, "ready");
  assert.deepEqual(presentLocalBranchHeads(resources[3]!, "main").rows.map((row) => row.branch), [
    "feature/read-only",
    "main"
  ]);
});

test("presenter preserves a valid branch selection and falls back only when it is absent", () => {
  const resource: LocalBranchHeadsResource = {
    kind: "ready",
    target,
    summary: parseLocalBranchHeadsV1(payload())
  };
  assert.equal(presentLocalBranchHeads(resource, "main").selectedBranch, "main");
  assert.equal(presentLocalBranchHeads(resource, "removed").selectedBranch, "feature/read-only");
});

test("branch-head selection resolves only the server-owned exact head or null", () => {
  const ready: LocalBranchHeadsResource = {
    kind: "ready",
    target,
    summary: parseLocalBranchHeadsV1(payload())
  };
  assert.equal(
    selectedLocalBranchHeadCommit(ready, "main"),
    "cccccccc-cccc-4ccc-8ccc-cccccccccccc"
  );
  assert.equal(selectedLocalBranchHeadCommit(ready, "missing"), null);
  assert.equal(selectedLocalBranchHeadCommit({ kind: "loading", target }, "main"), null);
});

test("branch-head selection returns the server-owned branch and exact head together", () => {
  const ready: LocalBranchHeadsResource = {
    kind: "ready",
    target,
    summary: parseLocalBranchHeadsV1(payload())
  };

  const selected = selectedLocalBranchHeadTarget(ready, "feature/read-only");
  assert.deepEqual(selected, {
    branch_name: "feature/read-only",
    head_commit_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb"
  });
  assert.equal(Object.isFrozen(selected), true);
  assert.equal(selectedLocalBranchHeadTarget(ready, "missing"), null);
  assert.equal(selectedLocalBranchHeadTarget({ kind: "loading", target }, "main"), null);
  assert.equal(
    selectedLocalBranchHeadTarget({
      kind: "ready",
      target,
      summary: parseLocalBranchHeadsV1(payload([
        { branch_name: "main", head_commit_id: null, revision: 1 }
      ]))
    }, "main"),
    null
  );
});

test("screen renders accessible state announcements and preserves branch selection in the control", () => {
  const ready = presentLocalBranchHeads({
    kind: "ready",
    target,
    summary: parseLocalBranchHeadsV1(payload())
  }, "main");
  const loading = presentLocalBranchHeads({ kind: "loading", target });
  const readyMarkup = renderToStaticMarkup(
    <LocalBranchHeadsScreen onSelectBranch={() => undefined} view={ready} />
  );
  const loadingMarkup = renderToStaticMarkup(<LocalBranchHeadsScreen view={loading} />);

  assert.match(readyMarkup, /aria-live="polite"/);
  assert.match(readyMarkup, /<span class="cl-select-label">Selected branch \/ 选中分支<\/span>/);
  assert.match(readyMarkup, /value="main"/);
  assert.match(readyMarkup, /feature\/read-only/);
  assert.match(readyMarkup, /Durable Context branch heads \/ 持久 Context 分支 head/);
  assert.match(loadingMarkup, /aria-busy="true"/);
  assert.match(loadingMarkup, /Loading branch heads \/ 正在加载分支 head/);
  assert.doesNotMatch(readyMarkup, /button|input type="password"|write|commit changes/i);
});

test("inspector keeps a selected target when only the callback identity changes", () => {
  const selectedTarget: LocalBranchHeadTarget = Object.freeze({
    branch_name: "feature/read-only",
    head_commit_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb"
  });
  const internals = (
    React as typeof React & {
      __CLIENT_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE: { H: unknown };
    }
  ).__CLIENT_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE;
  const previousDispatcher = internals.H;
  const slots: Array<{ kind: "state" | "ref"; value: unknown }> = [];
  let cursor = 0;
  let effectDependencies: React.DependencyList | undefined;
  let memoizedTarget: unknown;
  let memoizedTargetDependencies: React.DependencyList | undefined;
  let mounted = false;
  let observedTarget: LocalBranchHeadTarget | null = selectedTarget;

  const dispatcher = {
    useState<T>(initialValue: T | (() => T)): [T, React.Dispatch<React.SetStateAction<T>>] {
      const index = cursor++;
      if (!slots[index]) {
        slots[index] = { kind: "state", value: typeof initialValue === "function" ? (initialValue as () => T)() : initialValue };
      }
      const slot = slots[index]!;
      const setValue: React.Dispatch<React.SetStateAction<T>> = (nextValue) => {
        slot.value = typeof nextValue === "function"
          ? (nextValue as (previous: T) => T)(slot.value as T)
          : nextValue;
      };
      return [slot.value as T, setValue];
    },
    useMemo<T>(factory: () => T, dependencies?: React.DependencyList): T {
      cursor++;
      if (!mounted || dependenciesChanged(memoizedTargetDependencies, dependencies)) {
        memoizedTarget = factory();
        memoizedTargetDependencies = dependencies;
      }
      return memoizedTarget as T;
    },
    useRef<T>(initialValue: T): { current: T } {
      const index = cursor++;
      if (!slots[index]) {
        slots[index] = { kind: "ref", value: { current: initialValue } };
      }
      return slots[index]!.value as { current: T };
    },
    useEffect(effect: React.EffectCallback, dependencies?: React.DependencyList) {
      cursor++;
      if (!mounted || dependenciesChanged(effectDependencies, dependencies)) {
        effectDependencies = dependencies;
        effect();
      }
    }
  };

  try {
    const render = (onSelectHeadTarget: (target: LocalBranchHeadTarget | null) => void) => {
      cursor = 0;
      internals.H = dispatcher;
      try {
        LocalBranchHeadsInspector({
          contextId,
          onSelectHeadTarget
        });
      } finally {
        internals.H = previousDispatcher;
        mounted = true;
      }
    };

    render((target) => { observedTarget = target; });
    observedTarget = selectedTarget;
    render((target) => { observedTarget = target; });
    assert.strictEqual(observedTarget, selectedTarget);
  } finally {
    internals.H = previousDispatcher;
  }
});

function dependenciesChanged(previous: React.DependencyList | undefined, next: React.DependencyList | undefined): boolean {
  if (previous === undefined || next === undefined || previous.length !== next.length) return true;
  return previous.some((value, index) => !Object.is(value, next[index]));
}
