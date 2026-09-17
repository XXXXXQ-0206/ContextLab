import assert from "node:assert/strict";
import test from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { CommitGraphReview, canCompareCommitGraphReview } from "./context-graph-review";
import type { CommitGraphReviewViewModel } from "./context-workspace-presenter";

test("CommitGraphReview renders persisted snapshot facts, bilingual counts, and read-only rows", () => {
  const markup = renderToStaticMarkup(createElement(CommitGraphReview, { review: reviewFixture() }));

  assert.match(markup, /Commit Graph Review \/ 提交图谱审查/);
  assert.match(markup, /Base \/ 基线提交/);
  assert.match(markup, /Compare \/ 对比提交/);
  assert.match(markup, /Added nodes \/ 新增节点/);
  assert.match(markup, /memory:timeline/);
  assert.match(markup, /Preview fixture \/ 预览示例/);
});

test("CommitGraphReview disables comparisons with identical commits and renders unavailable state", () => {
  assert.equal(canCompareCommitGraphReview("commit-a", "commit-a"), false);
  assert.equal(canCompareCommitGraphReview("commit-a", "commit-b"), true);
  assert.equal(canCompareCommitGraphReview(null, "commit-b"), false);

  const review = reviewFixture();
  review.initialResult = null;
  review.unavailableMessage = "Snapshot not materialized / 快照尚未物化。";
  const markup = renderToStaticMarkup(createElement(CommitGraphReview, { review }));

  assert.match(markup, /Snapshot not materialized \/ 快照尚未物化。/);
  assert.match(markup, /disabled=""/);
});

test("CommitGraphReview renders request-memory Bearer controls and accessible unavailable state", () => {
  const review = reviewFixture();
  review.initialResult = null;
  review.unavailableMessage = "A request-memory Bearer token is required / 本地图谱审阅需要请求内存中的 Bearer 令牌。";
  review.isInteractive = true;
  const markup = renderToStaticMarkup(createElement(CommitGraphReview, { review }));

  assert.match(markup, /Bearer token \/ 访问令牌/);
  assert.match(markup, /Memory only \/ 仅内存/);
  assert.match(markup, /aria-live="polite"/);
  assert.match(markup, /aria-busy="false"/);
  assert.match(markup, /disabled=""/);
});

test("CommitGraphReview keeps empty graph-diff sections distinct and accessible", () => {
  const review = reviewFixture();
  for (const section of review.initialResult?.sections ?? []) {
    section.rows = [];
  }

  const markup = renderToStaticMarkup(createElement(CommitGraphReview, { review }));

  assert.match(markup, /No changes \/ 无变更/);
  assert.match(markup, /aria-busy="false"/);
  assert.match(markup, /commit-graph-review__section/);
});

function reviewFixture(): CommitGraphReviewViewModel {
  return {
    title: "Commit Graph Review / 提交图谱审查",
    contextId: "context-a",
    candidates: [
      { id: "commit-b", label: "Compare commit" },
      { id: "commit-a", label: "Base commit" }
    ],
    defaultOriginalCommitId: "commit-a",
    defaultRevisedCommitId: "commit-b",
    isInteractive: false,
    initialResult: {
      original: {
        title: "Base / 基线",
        facts: [
          { id: "commit-id", label: "Commit ID", value: "commit-a" },
          { id: "captured-at", label: "Captured", value: "2026-07-08 00:00:00 UTC" },
          { id: "schema", label: "Schema", value: "1" }
        ]
      },
      revised: {
        title: "Compare / 对比",
        facts: [
          { id: "commit-id", label: "Commit ID", value: "commit-b" },
          { id: "captured-at", label: "Captured", value: "2026-07-09 00:00:00 UTC" },
          { id: "schema", label: "Schema", value: "1" }
        ]
      },
      stats: [
        { label: "Added nodes / 新增节点", value: "1", detail: "Nodes" },
        { label: "Removed nodes / 移除节点", value: "0", detail: "Nodes" },
        { label: "Modified nodes / 修改节点", value: "0", detail: "Nodes" },
        { label: "Added edges / 新增关系", value: "0", detail: "Edges" },
        { label: "Removed edges / 移除关系", value: "0", detail: "Edges" }
      ],
      sections: [
        {
          id: "added-nodes",
          title: "Added Nodes / 新增节点",
          headers: ["Node ID / 节点 ID", "Kind / 类型", "Label / 标签"],
          rows: [{ id: "memory:timeline", cells: ["memory:timeline", "memory", "Memory Timeline"] }]
        }
      ]
    },
    unavailableMessage: null
  };
}
