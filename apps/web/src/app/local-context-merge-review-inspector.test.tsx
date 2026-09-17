import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { LocalContextMergeReviewInspector } from "./local-context-merge-review-inspector";

const target = {
  projectId: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
  contextId: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
  candidates: [
    { id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc", label: "left" },
    { id: "dddddddd-dddd-4ddd-8ddd-dddddddddddd", label: "right" }
  ]
} as const;

test("keeps private merge review absent until its exact local gate is enabled", () => {
  const disabled = renderToStaticMarkup(createElement(LocalContextMergeReviewInspector, {
    ...target,
    enabled: false
  }));
  assert.equal(disabled, "");

  const enabled = renderToStaticMarkup(createElement(LocalContextMergeReviewInspector, {
    ...target,
    enabled: true
  }));
  assert.match(enabled, /Private Context merge review/);
  assert.match(enabled, /Bearer token/);
  assert.match(enabled, /read-only/);
  assert.match(enabled, /local-context-merge-review-heading/);
});

test("mounts the merge review inspector through the workspace's private read region", () => {
  const source = readFileSync(new URL("./context-workspace-screen.tsx", import.meta.url), "utf8");
  assert.match(source, /import \{ LocalContextMergeReviewInspector \} from "\.\/local-context-merge-review-inspector"/);
  assert.match(source, /<LocalContextMergeReviewInspector/);
  assert.match(source, /enabled=\{mergeReviewEnabled\}/);
  assert.match(source, /defaultLeftCommitId=\{props\.operations\.graphReview\.defaultOriginalCommitId\}/);
  assert.match(source, /defaultRightCommitId=\{props\.operations\.graphReview\.defaultRevisedCommitId\}/);
});
