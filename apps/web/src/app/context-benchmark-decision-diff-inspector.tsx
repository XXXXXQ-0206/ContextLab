"use client";

import { Button, CodeChip, DefinitionGrid, Input, StackTable, StatusPill } from "@contextlab/ui";
import React, { useRef, useState } from "react";
import {
  LocalBenchmarkDecisionDiffProxyError,
  loadLocalBenchmarkDecisionDiff
} from "./context-benchmark-decision-diff-data";
import {
  LocalBenchmarkDecisionDiscoveryProxyError
} from "./context-benchmark-decision-discovery-data";
import { benchmarkDecisionDiscoveryErrorMessage } from "./context-benchmark-decision-discovery-presenter";
import { loadLocalBenchmarkDecisionDiffSelections } from "./context-benchmark-decision-diff-selection-data";
import {
  benchmarkDecisionDiffErrorMessage,
  canCompareBenchmarkDecisions,
  presentBenchmarkDecisionDiff,
  type BenchmarkDecisionDiffViewModel
} from "./context-benchmark-decision-diff-presenter";
import {
  presentBenchmarkDecisionDiffSelection,
  selectionStatusTone,
  type BenchmarkDecisionDiffSelectionViewModel
} from "./context-benchmark-decision-diff-selection-presenter";

export type ContextBenchmarkDecisionDiffCandidate = {
  id: string;
  label: string;
};

export type ContextBenchmarkDecisionDiffInspectorProps = {
  projectId: string;
  contextId: string;
  candidates: ContextBenchmarkDecisionDiffCandidate[];
};

type DiffNotice = { message: string; tone: "error" };

export function ContextBenchmarkDecisionDiffInspector(props: ContextBenchmarkDecisionDiffInspectorProps) {
  return <ContextBenchmarkDecisionDiffInspectorControl {...props} />;
}

export function canEditBenchmarkDecisionDiffScope(isLoading: boolean): boolean {
  return !isLoading;
}

export function ContextBenchmarkDecisionDiffInspectorControl({
  candidates,
  contextId,
  projectId
}: ContextBenchmarkDecisionDiffInspectorProps) {
  const [bearerToken, setBearerToken] = useState("");
  const [baselineCommitId, setBaselineCommitId] = useState(candidates[0]?.id ?? "");
  const [baselineDecisionId, setBaselineDecisionId] = useState("");
  const [revisedCommitId, setRevisedCommitId] = useState(candidates[0]?.id ?? "");
  const [revisedDecisionId, setRevisedDecisionId] = useState("");
  const [model, setModel] = useState<BenchmarkDecisionDiffViewModel | null>(null);
  const [notice, setNotice] = useState<DiffNotice | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [selection, setSelection] = useState<BenchmarkDecisionDiffSelectionViewModel>(() =>
    presentBenchmarkDecisionDiffSelection({
      kind: "empty",
      target: {
        baselineCommitId: candidates[0]?.id ?? "",
        revisedCommitId: candidates[0]?.id ?? ""
      }
    })
  );
  const selectionRequestId = useRef(0);
  const canCompare = canCompareBenchmarkDecisions({
    bearerToken,
    baselineCommitId,
    baselineDecisionId,
    revisedCommitId,
    revisedDecisionId,
    isLoading: isLoading || selection.state === "loading"
  });
  const canEditScope = canEditBenchmarkDecisionDiffScope(
    isLoading || selection.state === "loading"
  );

  async function discoverDecisionSelections(
    nextBaselineCommitId = baselineCommitId,
    nextRevisedCommitId = revisedCommitId,
    nextBearerToken = bearerToken
  ) {
    const resolvedBaselineCommitId = nextBaselineCommitId.trim();
    const resolvedRevisedCommitId = nextRevisedCommitId.trim();
    const resolvedBearerToken = nextBearerToken.trim();
    if (
      !resolvedBaselineCommitId
      || !resolvedRevisedCommitId
      || !resolvedBearerToken
      || isLoading
    ) {
      return;
    }

    const requestId = ++selectionRequestId.current;
    setSelection(
      presentBenchmarkDecisionDiffSelection({
        kind: "loading",
        target: {
          baselineCommitId: resolvedBaselineCommitId,
          revisedCommitId: resolvedRevisedCommitId
        }
      })
    );
    setBaselineDecisionId("");
    setRevisedDecisionId("");
    setModel(null);
    setNotice(null);

    try {
      const selections = await loadLocalBenchmarkDecisionDiffSelections(
        projectId,
        contextId,
        resolvedBaselineCommitId,
        resolvedRevisedCommitId,
        resolvedBearerToken
      );
      if (requestId !== selectionRequestId.current) {
        return;
      }

      const view = presentBenchmarkDecisionDiffSelection({
        kind: "ready",
        target: {
          baselineCommitId: resolvedBaselineCommitId,
          revisedCommitId: resolvedRevisedCommitId
        },
        selections
      });
      setSelection(view);
      setBaselineDecisionId(view.baseline.firstDecisionId ?? "");
      setRevisedDecisionId(view.revised.firstDecisionId ?? "");
    } catch (error) {
      if (requestId !== selectionRequestId.current) {
        return;
      }
      setSelection(
        presentBenchmarkDecisionDiffSelection({
          kind: "error",
          target: {
            baselineCommitId: resolvedBaselineCommitId,
            revisedCommitId: resolvedRevisedCommitId
          },
          message: presentBenchmarkDecisionSelectionError(error)
        })
      );
      setBaselineDecisionId("");
      setRevisedDecisionId("");
    }
  }

  async function compareDecisions() {
    if (!canCompare) {
      return;
    }

    setIsLoading(true);
    setNotice(null);
    setModel(null);
    try {
      const diff = await loadLocalBenchmarkDecisionDiff(
        projectId,
        contextId,
        { commit_id: baselineCommitId, decision_id: baselineDecisionId },
        { commit_id: revisedCommitId, decision_id: revisedDecisionId },
        bearerToken
      );
      setModel(presentBenchmarkDecisionDiff(diff));
    } catch (error) {
      setNotice({ message: presentBenchmarkDecisionDiffError(error), tone: "error" });
    } finally {
      setIsLoading(false);
    }
  }

  const metricRows = model?.metrics.map((metric) => ({
    id: metric.id,
    cells: [
      metric.change,
      metric.metric,
      metric.baseline,
      metric.revised,
      metric.coverage,
      <StatusPill key={`${metric.id}-outcome`} tone={metric.outcome.tone}>
        {metric.outcome.label}
      </StatusPill>
    ]
  }));

  return (
    <section className="context-benchmark-evidence" aria-labelledby="context-benchmark-decision-diff-heading">
      <div className="context-benchmark-evidence__heading">
        <div>
          <span className="eyebrow">Local Benchmark Decision Diff / 本地 Benchmark Decision Diff</span>
          <h3 id="context-benchmark-decision-diff-heading">Benchmark Decision Diff</h3>
          <p>Protected local read-only comparison / 只读本地受保护比较。No credential is persisted / 不会持久化凭据。</p>
        </div>
        <StatusPill tone="info">read only / 只读</StatusPill>
      </div>

      <div className="context-benchmark-evidence__controls">
        <Input
          autoComplete="off"
          disabled={!canEditScope}
          description="Memory only / 仅内存"
          label="Bearer token / 访问令牌"
          name="benchmark-diff-bearer-token"
          onBlur={() => void discoverDecisionSelections()}
          onChange={(event) => {
            const nextToken = event.target.value;
            selectionRequestId.current += 1;
            setBearerToken(nextToken);
            setBaselineDecisionId("");
            setRevisedDecisionId("");
            setSelection(
              presentBenchmarkDecisionDiffSelection({
                kind: "empty",
                target: { baselineCommitId, revisedCommitId }
              })
            );
          }}
          type="password"
          value={bearerToken}
        />
        <label className="cl-field">
          <span className="cl-field__label">Baseline commit / 基线提交</span>
          <select
            className="cl-select"
            disabled={!canEditScope}
            name="benchmark-diff-baseline-commit"
            onChange={(event) => {
              const nextCommitId = event.target.value;
              selectionRequestId.current += 1;
              setBaselineCommitId(nextCommitId);
              setBaselineDecisionId("");
              setModel(null);
              setSelection(
                presentBenchmarkDecisionDiffSelection({
                  kind: "empty",
                  target: { baselineCommitId: nextCommitId, revisedCommitId }
                })
              );
              if (bearerToken.trim()) {
                void discoverDecisionSelections(nextCommitId, revisedCommitId, bearerToken);
              }
            }}
            value={baselineCommitId}
          >
            {candidates.map((candidate) => <option key={candidate.id} value={candidate.id}>{candidate.label}</option>)}
          </select>
        </label>
        <label className="cl-field">
          <span className="cl-field__label">Baseline decision / 基线决策</span>
          <select
            className="cl-select"
            disabled={!canEditScope || selection.baseline.options.length === 0}
            name="benchmark-diff-baseline-decision"
            onChange={(event) => setBaselineDecisionId(event.target.value)}
            value={baselineDecisionId}
          >
            {selection.baseline.options.length === 0
              ? <option value="">No sealed decisions / 暂无 sealed decision</option>
              : selection.baseline.options.map((option) => (
                <option key={option.id} value={option.value}>{option.label}</option>
              ))}
          </select>
        </label>
        <label className="cl-field">
          <span className="cl-field__label">Revised commit / 修订提交</span>
          <select
            className="cl-select"
            disabled={!canEditScope}
            name="benchmark-diff-revised-commit"
            onChange={(event) => {
              const nextCommitId = event.target.value;
              selectionRequestId.current += 1;
              setRevisedCommitId(nextCommitId);
              setRevisedDecisionId("");
              setModel(null);
              setSelection(
                presentBenchmarkDecisionDiffSelection({
                  kind: "empty",
                  target: { baselineCommitId, revisedCommitId: nextCommitId }
                })
              );
              if (bearerToken.trim()) {
                void discoverDecisionSelections(baselineCommitId, nextCommitId, bearerToken);
              }
            }}
            value={revisedCommitId}
          >
            {candidates.map((candidate) => <option key={candidate.id} value={candidate.id}>{candidate.label}</option>)}
          </select>
        </label>
        <label className="cl-field">
          <span className="cl-field__label">Revised decision / 修订决策</span>
          <select
            className="cl-select"
            disabled={!canEditScope || selection.revised.options.length === 0}
            name="benchmark-diff-revised-decision"
            onChange={(event) => setRevisedDecisionId(event.target.value)}
            value={revisedDecisionId}
          >
            {selection.revised.options.length === 0
              ? <option value="">No sealed decisions / 暂无 sealed decision</option>
              : selection.revised.options.map((option) => (
                <option key={option.id} value={option.value}>{option.label}</option>
              ))}
          </select>
        </label>
        <Button disabled={!canCompare} tone="muted" type="button" onClick={compareDecisions}>{isLoading ? "Comparing... / 正在比较..." : "Compare decisions / 比较 decision"}</Button>
      </div>

      <div className="context-benchmark-evidence__selection" aria-live="polite">
        <StatusPill tone={selectionStatusTone(selection.state)}>
          {selectionStateLabel(selection.state)}
        </StatusPill>
        <p>{selection.message}</p>
        <Button
          disabled={!bearerToken.trim() || !canEditScope}
          tone="muted"
          type="button"
          onClick={() => void discoverDecisionSelections()}
        >
          Discover exact decisions / 发现精确 decision
        </Button>
      </div>

      {notice ? <p className="context-benchmark-evidence__notice" data-tone={notice.tone} role="alert">{notice.message}</p> : null}

      {model ? (
        <div className="context-benchmark-evidence__result" aria-live="polite">
          <div className="context-benchmark-evidence__summary"><StatusPill tone={model.status.tone}>{model.status.label}</StatusPill><CodeChip>{model.scopes[0]?.value ?? ""}</CodeChip></div>
          <DefinitionGrid columns={2} compact items={model.scopes} surface="raised" valueTone="info" />
          <StackTable aria-label="Benchmark decision metric diff" columnTemplate="minmax(8rem, 0.9fr) minmax(7rem, 0.8fr) minmax(7rem, 0.8fr) minmax(7rem, 0.8fr) minmax(10rem, 1fr) minmax(8rem, 0.9fr)" headers={["Change / 变更", "Metric / 指标", "Baseline / 基线", "Revised / 修订", "Coverage / 覆盖", "Outcome / 结果"]} rows={metricRows ?? []} />
        </div>
      ) : <p className="context-benchmark-evidence__empty">No comparison selected / 未选择比较。</p>}
    </section>
  );
}

function selectionStateLabel(state: BenchmarkDecisionDiffSelectionViewModel["state"]): string {
  if (state === "loading") {
    return "Loading / 加载中";
  }
  if (state === "ready") {
    return "Discovered / 已发现";
  }
  if (state === "error") {
    return "Unavailable / 不可用";
  }
  return "Awaiting token / 等待 token";
}

function presentBenchmarkDecisionDiffError(error: unknown): string {
  if (error instanceof LocalBenchmarkDecisionDiffProxyError) {
    return `${benchmarkDecisionDiffErrorMessage(error.status, error.retryAfterMs)} ${error.body.message}`;
  }

  return "Unable to reach local benchmark decision comparison / 无法连接本地 benchmark decision 比较。";
}

function presentBenchmarkDecisionSelectionError(error: unknown): string {
  if (error instanceof LocalBenchmarkDecisionDiscoveryProxyError) {
    return `${benchmarkDecisionDiscoveryErrorMessage(error.status, error.retryAfterMs)} ${error.body.message}`;
  }

  if (error instanceof LocalBenchmarkDecisionDiffProxyError) {
    return `${benchmarkDecisionDiffErrorMessage(error.status, error.retryAfterMs)} ${error.body.message}`;
  }

  if (error instanceof Error) {
    return error.message;
  }

  return "Unable to discover local benchmark decisions / 无法发现本地 benchmark decision。";
}
