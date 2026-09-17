"use client";

import {
  Button,
  CodeChip,
  DefinitionGrid,
  Input,
  Select,
  StackTable,
  StatGrid,
  StatusPill
} from "@contextlab/ui";
import { GitCompareArrows } from "lucide-react";
import React, { useState } from "react";
import {
  loadLocalCommitGraphDiff,
  LocalCommitGraphDiffProxyError
} from "./local-commit-graph-diff-data";
import {
  presentCommitGraphDiff,
  type CommitGraphReviewResult,
  type CommitGraphReviewViewModel
} from "./context-workspace-presenter";

export type CommitGraphReviewProps = {
  review: CommitGraphReviewViewModel;
};

export function CommitGraphReview({ review }: CommitGraphReviewProps) {
  const [originalCommitId, setOriginalCommitId] = useState(review.defaultOriginalCommitId);
  const [revisedCommitId, setRevisedCommitId] = useState(review.defaultRevisedCommitId);
  const [result, setResult] = useState(review.initialResult);
  const [message, setMessage] = useState(review.unavailableMessage);
  const [isLoading, setIsLoading] = useState(false);
  const [bearerToken, setBearerToken] = useState("");
  const canCompare = review.isInteractive
    && bearerToken.trim().length > 0
    && canCompareCommitGraphReview(originalCommitId, revisedCommitId);

  async function compareCommits() {
    if (!canCompare || !originalCommitId || !revisedCommitId) {
      return;
    }

    setIsLoading(true);
    setMessage(null);

    try {
      const payload = await loadLocalCommitGraphDiff(
        review.contextId,
        originalCommitId,
        revisedCommitId,
        bearerToken
      );
      setResult(presentCommitGraphDiff(payload));
    } catch (error) {
      setResult(null);
      setMessage(presentCommitGraphDiffError(error));
    } finally {
      setIsLoading(false);
    }
  }

  return (
    <section aria-busy={isLoading} className="commit-graph-review" aria-label={review.title} id="graph-diff-review">
      <div className="block-heading">
        <GitCompareArrows aria-hidden="true" />
        <span>{review.title}</span>
        <StatusPill tone={review.isInteractive ? "info" : "neutral"}>
          {review.isInteractive ? "Read-only / 只读" : "Preview fixture / 预览示例"}
        </StatusPill>
      </div>

      <div className="commit-graph-review__controls">
        <Input
          autoComplete="off"
          description="Memory only / 仅内存"
          disabled={!review.isInteractive || isLoading}
          label="Bearer token / 访问令牌"
          name="commit-graph-diff-bearer-token"
          onChange={(event) => setBearerToken(event.target.value)}
          type="password"
          value={bearerToken}
        />
        <Select
          aria-label="Base commit / 基线提交"
          disabled={!review.isInteractive || isLoading}
          label="Base / 基线提交"
          onChange={(event) => setOriginalCommitId(event.target.value)}
          value={originalCommitId ?? ""}
        >
          {review.candidates.map((commit) => (
            <option key={commit.id} value={commit.id}>
              {commit.label}
            </option>
          ))}
        </Select>
        <Select
          aria-label="Compare commit / 对比提交"
          disabled={!review.isInteractive || isLoading}
          label="Compare / 对比提交"
          onChange={(event) => setRevisedCommitId(event.target.value)}
          value={revisedCommitId ?? ""}
        >
          {review.candidates.map((commit) => (
            <option key={commit.id} value={commit.id}>
              {commit.label}
            </option>
          ))}
        </Select>
        <Button disabled={!canCompare || isLoading} icon={<GitCompareArrows aria-hidden="true" />} onClick={compareCommits}>
          {isLoading ? "Loading review / 正在加载" : "Compare commits / 比较提交"}
        </Button>
      </div>

      {result ? <CommitGraphReviewResultView result={result} /> : <CommitGraphReviewUnavailable message={message} />}
    </section>
  );
}

export function canCompareCommitGraphReview(
  originalCommitId: string | null,
  revisedCommitId: string | null
) {
  return Boolean(originalCommitId && revisedCommitId && originalCommitId !== revisedCommitId);
}

function CommitGraphReviewResultView({ result }: { result: CommitGraphReviewResult }) {
  return (
    <div className="commit-graph-review__result">
      <div className="commit-graph-review__snapshots">
        {[result.original, result.revised].map((snapshot) => (
          <section className="commit-graph-review__snapshot" key={snapshot.title}>
            <div className="commit-graph-review__snapshot-heading">
              <strong>{snapshot.title}</strong>
              <CodeChip>{snapshot.facts[0]?.value}</CodeChip>
            </div>
            <DefinitionGrid columns={1} compact items={snapshot.facts.slice(1)} surface="raised" />
          </section>
        ))}
      </div>
      <StatGrid columns={3} items={result.stats} />
      {result.sections.map((section) => (
        <section className="commit-graph-review__section" key={section.id}>
          <h3>{section.title}</h3>
          {section.rows.length > 0 ? (
            <StackTable headers={section.headers} rows={section.rows} />
          ) : (
            <p>No changes / 无变更</p>
          )}
        </section>
      ))}
    </div>
  );
}

function CommitGraphReviewUnavailable({ message }: { message: string | null }) {
  return (
    <div aria-live="polite" className="commit-graph-review__unavailable" role="status">
      <StatusPill tone="warning">Review unavailable / 审阅不可用</StatusPill>
      <p>{message ?? "Graph review unavailable / 图谱审阅暂不可用。"}</p>
    </div>
  );
}

function presentCommitGraphDiffError(error: unknown): string {
  if (error instanceof LocalCommitGraphDiffProxyError) {
    if (error.status === 401) return "Bearer authentication is required / 需要 Bearer 身份验证。";
    if (error.status === 403) return "You do not have permission for this Context / 你没有此 Context 的权限。";
    if (error.status === 409) return "Snapshot not materialized / 快照尚未物化。";
    if (error.status === 429) return "The local rate limit is active / 本地速率限制已生效。";
    if (error.status === 503) return "Local graph review is unavailable / 本地图谱审阅暂不可用。";
  }

  return "Unable to load local graph review / 无法加载本地图谱审阅。";
}
