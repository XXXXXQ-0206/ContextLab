import assert from "node:assert/strict";
import test from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { createLocalContextMergeReviewResource } from "./local-context-merge-review-data";
import { presentLocalContextMergeReview } from "./local-context-merge-review-presenter";
import { LocalContextMergeReviewScreen } from "./local-context-merge-review-screen";

const target = {
  project_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
  context_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
  left_commit_id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
  right_commit_id: "dddddddd-dddd-4ddd-8ddd-dddddddddddd"
} as const;

test("screen keeps every state accessible and bilingual", () => {
  for (const kind of ["loading", "error", "empty", "unavailable"] as const) {
    const markup = renderToStaticMarkup(createElement(LocalContextMergeReviewScreen, {
      view: presentLocalContextMergeReview({ kind, target })
    }));
    assert.match(markup, new RegExp(`data-state="${kind}"`));
    assert.match(markup, /Context Merge Review/);
    assert.match(markup, /Context Merge 审阅/);
    assert.match(markup, /aria-live="(polite|assertive)"/);
    assert.match(markup, kind === "error" ? /role="alert"/ : /role="status"/);
    if (kind === "loading") assert.match(markup, /aria-busy="true"/);
  }
});

test("ready screen renders exact three scopes, classification, and read-only design-system primitives", () => {
  const view = presentLocalContextMergeReview(createLocalContextMergeReviewResource({
    kind: "ready",
    target,
    review: {
      schema_version: "v1",
      plan: { ThreeWay: { base: "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee", left: target.left_commit_id, right: target.right_commit_id } },
      base_scope: { project_id: target.project_id, context_id: target.context_id, commit_id: "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee" },
      left_scope: { project_id: target.project_id, context_id: target.context_id, commit_id: target.left_commit_id },
      right_scope: { project_id: target.project_id, context_id: target.context_id, commit_id: target.right_commit_id },
      classification: { Equivalent: { changes: [] } }
    }
  }));
  const markup = renderToStaticMarkup(createElement(LocalContextMergeReviewScreen, { view }));

  assert.match(markup, /Base \/ 基线/);
  assert.match(markup, /Left \/ 左支线/);
  assert.match(markup, /Right \/ 右支线/);
  assert.match(markup, /Equivalent \/ 等价/);
  assert.match(markup, /read-only \/ 只读/);
  assert.match(markup, /Exact three-way Context scope/);
  assert.doesNotMatch(markup, /private raw upstream prompt|&quot;graph&quot;|&quot;nodes&quot;|&quot;edges&quot;/);
});
