import assert from "node:assert/strict";
import test from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import {
  createLocalPersistedContextDiffReviewResource,
  loadLocalPersistedContextDiffReview
} from "./local-persisted-context-diff-review-data";
import { presentLocalPersistedContextDiffReview } from "./local-persisted-context-diff-review-presenter";
import { LocalPersistedContextDiffReviewScreen } from "./local-persisted-context-diff-review-screen";

const originalFetch = globalThis.fetch;
const target = {
  project_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
  context_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
  source_commit_id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
  target_commit_id: "dddddddd-dddd-4ddd-8ddd-dddddddddddd"
} as const;

test("screen exposes bilingual state, exact commits, and accessible loading state", () => {
  const markup = renderToStaticMarkup(createElement(LocalPersistedContextDiffReviewScreen, {
    view: {
      state: "loading",
      title: "Persisted Context diff review / 持久化 Context Diff 审阅",
      message: "Loading persisted review / 正在加载持久化审阅。",
      sourceCommitId: "source",
      targetCommitId: "target",
      stats: [],
      sections: []
    }
  }));
  assert.match(markup, /持久化 Context Diff 审阅/);
  assert.match(markup, /source/);
  assert.match(markup, /target/);
  assert.match(markup, /aria-busy="true"/);
  assert.match(markup, /aria-live="polite"/);
});

test("screen renders the complete private data-to-presenter diff review chain", async () => {
  const requests: Array<{ input: string; init: RequestInit | undefined }> = [];
  globalThis.fetch = (async (input, init) => {
    requests.push({ input: String(input), init });
    return jsonResponse(completeReview());
  }) as typeof fetch;

  try {
    const review = await loadLocalPersistedContextDiffReview(target, " request-token ");
    const resource = createLocalPersistedContextDiffReviewResource({ kind: "ready", target, review });
    const view = presentLocalPersistedContextDiffReview(resource);
    const markup = renderToStaticMarkup(createElement(LocalPersistedContextDiffReviewScreen, { view }));

    assert.equal(review.diff.semantic.document_changes.length > 0, true);
    assert.equal(review.diff.behavior.case_changes.length > 0, true);
    assert.equal(review.diff.evaluation.metric_changes.length > 0, true);
    assert.equal(view.state, "available");
    assert.equal(view.sections.find((section) => section.id === "semantic-documents")?.rows.length, 1);
    assert.equal(view.sections.find((section) => section.id === "behavior-cases")?.rows.length, 1);
    assert.equal(view.sections.find((section) => section.id === "evaluation-metrics")?.rows.length, 1);
    assert.match(markup, /Persisted Context diff review \/ 持久化 Context Diff 审阅/);
    assert.match(markup, /Rust-owned semantic, behavior, and evaluation projection \/ Rust 所有的 semantic、behavior 与 evaluation 投影/);
    assert.match(markup, /Semantic documents \/ Semantic 文档/);
    assert.match(markup, /Behavior cases \/ Behavior 案例/);
    assert.match(markup, /Evaluation metrics \/ Evaluation 指标/);
    assert.match(markup, /prompt:system/);
    assert.match(markup, /case:answer/);
    assert.match(markup, /accuracy/);
    assert.match(markup, new RegExp(target.source_commit_id));
    assert.match(markup, new RegExp(target.target_commit_id));
    assert.doesNotMatch(markup, /request-token/);
    assert.equal(
      requests[0]?.input,
      `/api/local/projects/${target.project_id}/contexts/${target.context_id}/diff-review?source_commit_id=${target.source_commit_id}&target_commit_id=${target.target_commit_id}`
    );
    assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer request-token");
    assert.equal(new Headers(requests[0]?.init?.headers).get("cookie"), null);
    assert.equal(requests[0]?.init?.credentials, "omit");
    assert.equal(requests[0]?.init?.cache, "no-store");
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("screen keeps bilingual error and empty status contracts", () => {
  for (const view of [
    {
      state: "error" as const,
      message: "Unable to load persisted review / 无法加载持久化审阅。"
    },
    {
      state: "empty" as const,
      message: "No persisted review is available for this commit pair / 此提交对暂无持久化审阅。"
    }
  ]) {
    const markup = renderToStaticMarkup(createElement(LocalPersistedContextDiffReviewScreen, {
      view: {
        state: view.state,
        title: "Persisted Context diff review / 持久化 Context Diff 审阅",
        message: view.message,
        sourceCommitId: "source",
        targetCommitId: "target",
        stats: [],
        sections: []
      }
    }));
    assert.match(markup, new RegExp(view.state));
    assert.match(markup, /aria-live="polite"/);
    assert.match(markup, /role="status"/);
    assert.match(markup, /Unable|No persisted/);
    assert.match(markup, /无法|暂无/);
  }
});

test("screen renders the presenter's bilingual metadata section with shared table primitives", () => {
  const markup = renderToStaticMarkup(createElement(LocalPersistedContextDiffReviewScreen, {
    view: {
      state: "available",
      title: "Persisted Context diff review / 持久化 Context Diff 审阅",
      message: "Rust-owned semantic projection / Rust 所有的 semantic 投影",
      sourceCommitId: "source",
      targetCommitId: "target",
      stats: [{ label: "Metadata changes / Metadata 变更", value: "1", detail: "changes" }],
      sections: [{
        id: "semantic-metadata",
        title: "Context metadata / Context 元数据",
        rows: [{ id: "metadata-modified", cells: ["modified", "labels: owner=support -> owner=platform"] }]
      }]
    }
  }));
  assert.match(markup, /Context metadata \/ Context 元数据/);
  assert.match(markup, /owner=support -&gt; owner=platform/);
  assert.match(markup, /Change \/ 变更/);
  assert.match(markup, /Identity \/ 标识/);
});

function completeReview() {
  return {
    schema_version: "v1",
    source_scope: {
      project_id: target.project_id,
      context_id: target.context_id,
      commit_id: target.source_commit_id
    },
    target_scope: {
      project_id: target.project_id,
      context_id: target.context_id,
      commit_id: target.target_commit_id
    },
    diff: {
      contract_version: "v1",
      semantic: {
        graph_diff: {
          added_nodes: [{ id: "prompt:system", kind: "prompt", label: "System prompt" }],
          removed_nodes: [],
          modified_nodes: [],
          added_edges: [],
          removed_edges: []
        },
        document_changes: [{
          kind: "modified",
          document_id: "prompt:system",
          text_diff: {
            lines: [
              { kind: "removed", text: "old system instruction" },
              { kind: "added", text: "new system instruction" }
            ]
          }
        }]
      },
      behavior: {
        case_changes: [{
          kind: "modified",
          original: {
            case_id: "case:answer",
            input_fingerprint: "sha256:input",
            outcome: { kind: "succeeded", output: "old answer" }
          },
          revised: {
            case_id: "case:answer",
            input_fingerprint: "sha256:input",
            outcome: { kind: "failed", error_code: "timeout" }
          }
        }]
      },
      evaluation: {
        comparability_fingerprint: "suite:context:v1",
        metric_changes: [{
          kind: "modified",
          original: { metric_id: "accuracy", value: 0.8, sample_count: 10 },
          revised: { metric_id: "accuracy", value: 0.9, sample_count: 10 }
        }]
      }
    }
  };
}

function jsonResponse(body: unknown): Response {
  return new Response(JSON.stringify(body), {
    headers: { "content-type": "application/json" }
  });
}
