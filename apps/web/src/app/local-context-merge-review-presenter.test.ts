import assert from "node:assert/strict";
import test from "node:test";
import { createLocalContextMergeReviewResource } from "./local-context-merge-review-data";
import { presentLocalContextMergeReview } from "./local-context-merge-review-presenter";

const target = {
  project_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
  context_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
  left_commit_id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
  right_commit_id: "dddddddd-dddd-4ddd-8ddd-dddddddddddd"
} as const;

test("presenter exposes exact base, left, right scopes and server-owned classification", () => {
  const view = presentLocalContextMergeReview(createLocalContextMergeReviewResource({
    kind: "ready",
    target,
    review: {
      schema_version: "v1",
      plan: { ThreeWay: { base: "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee", left: target.left_commit_id, right: target.right_commit_id } },
      base_scope: { project_id: target.project_id, context_id: target.context_id, commit_id: "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee" },
      left_scope: { project_id: target.project_id, context_id: target.context_id, commit_id: target.left_commit_id },
      right_scope: { project_id: target.project_id, context_id: target.context_id, commit_id: target.right_commit_id },
      classification: { Conflict: { conflicts: [{ Node: { node_id: "prompt:system" } }] } }
    }
  }));

  assert.equal(view.state, "available");
  assert.match(view.title, /Context Merge/);
  assert.deepEqual(view.scopes.map((scope) => [scope.id, scope.commitId]), [
    ["base", "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee"],
    ["left", target.left_commit_id],
    ["right", target.right_commit_id]
  ]);
  assert.equal(view.classification?.kind, "conflict");
  assert.equal(view.classification?.items[0]?.value, "prompt:system");
  assert.match(view.plan!, /Three-way/);
});

test("presenter maps all resource states to bilingual safe messages", () => {
  for (const kind of ["loading", "error", "empty", "unavailable"] as const) {
    const view = presentLocalContextMergeReview({ kind, target, message: "private raw upstream prompt" });
    assert.equal(view.state, kind);
    assert.doesNotMatch(view.message, /private raw upstream prompt/);
    assert.match(view.message, /\//);
    assert.deepEqual(view.scopes, []);
  }
});
