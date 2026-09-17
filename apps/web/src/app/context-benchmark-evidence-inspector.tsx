"use client";

import { Button, CodeChip, DefinitionGrid, Input, StackTable, StatusPill } from "@contextlab/ui";
import type { LocalBenchmarkDecisionRunDetails } from "@contextlab/local-sdk";
import React, { useRef, useState } from "react";
import { CapabilityStateScreen } from "./capability-state-screen";
import {
  LocalBenchmarkDecisionDiscoveryProxyError,
  loadLocalBenchmarkDecisionDiscovery,
} from "./context-benchmark-decision-discovery-data";
import {
  benchmarkDecisionDiscoveryErrorMessage,
  presentLocalBenchmarkDecisionDiscovery,
  type BenchmarkDecisionDiscoveryTarget,
  type BenchmarkDecisionDiscoveryViewModel,
  type BenchmarkDecisionDiscoveryResource
} from "./context-benchmark-decision-discovery-presenter";
import {
  LocalBenchmarkEvidenceProxyError,
  loadLocalBenchmarkDecision,
  loadLocalBenchmarkDecisionRunDetails
} from "./context-benchmark-evidence-data";
import {
  benchmarkEvidenceErrorMessage,
  canInspectBenchmarkEvidence,
  presentBenchmarkDecision,
  presentBenchmarkDecisionRunDetails,
  type BenchmarkDefinitionViewModel,
  type BenchmarkDecisionViewModel
} from "./context-benchmark-evidence-presenter";

export type ContextBenchmarkEvidenceCandidate = {
  id: string;
  label: string;
};

export type ContextBenchmarkEvidenceInspectorProps = {
  projectId: string;
  contextId: string;
  candidates: ContextBenchmarkEvidenceCandidate[];
};

type EvidenceNotice = {
  message: string;
  tone: "error";
};

type BenchmarkDecisionRunDetailsStatus = "idle" | "loading" | "error" | "ready";

export type BenchmarkDecisionRunDetailsProps = {
  details: LocalBenchmarkDecisionRunDetails | null;
  errorMessage?: string | null;
  status: BenchmarkDecisionRunDetailsStatus;
};

export type BenchmarkDecisionDiscoveryStateProps = {
  view: BenchmarkDecisionDiscoveryViewModel;
};

export function ContextBenchmarkEvidenceInspector(props: ContextBenchmarkEvidenceInspectorProps) {
  return <ContextBenchmarkEvidenceInspectorControl {...props} />;
}

export function canEditBenchmarkEvidenceScope(isLoading: boolean): boolean {
  return !isLoading;
}

export function ContextBenchmarkEvidenceInspectorControl({
  candidates,
  contextId,
  projectId
}: ContextBenchmarkEvidenceInspectorProps) {
  const [bearerToken, setBearerToken] = useState("");
  const [selectedCommitId, setSelectedCommitId] = useState(candidates[0]?.id ?? "");
  const [decisionId, setDecisionId] = useState("");
  const [model, setModel] = useState<BenchmarkDecisionViewModel | null>(null);
  const [runDetails, setRunDetails] = useState<LocalBenchmarkDecisionRunDetails | null>(null);
  const [runDetailsStatus, setRunDetailsStatus] = useState<BenchmarkDecisionRunDetailsStatus>("idle");
  const [runDetailsError, setRunDetailsError] = useState<string | null>(null);
  const [notice, setNotice] = useState<EvidenceNotice | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [discovery, setDiscovery] = useState<BenchmarkDecisionDiscoveryViewModel>(() =>
    presentLocalBenchmarkDecisionDiscovery({
      kind: "empty",
      target: discoveryTarget(projectId, contextId, candidates[0]?.id ?? "")
    })
  );
  const discoveryRequestId = useRef(0);
  const canInspect = canInspectBenchmarkEvidence({
    bearerToken,
    decisionId,
    commitId: selectedCommitId,
    isLoading: isLoading || discovery.status.state === "loading"
  });
  const canEditScope = canEditBenchmarkEvidenceScope(isLoading || discovery.status.state === "loading");

  async function inspectDecision(
    decisionIdOverride?: string,
    commitIdOverride = selectedCommitId,
    bearerTokenOverride = bearerToken
  ) {
    const resolvedDecisionId = (decisionIdOverride ?? decisionId).trim();
    const resolvedCommitId = commitIdOverride.trim();
    const resolvedBearerToken = bearerTokenOverride.trim();
    if (!resolvedBearerToken || !resolvedDecisionId || !resolvedCommitId || isLoading) {
      return;
    }

    setIsLoading(true);
    setNotice(null);
    setModel(null);
    setRunDetails(null);
    setRunDetailsStatus("idle");
    setRunDetailsError(null);
    try {
      const evidence = await loadLocalBenchmarkDecision(
        projectId,
        contextId,
        resolvedCommitId,
        resolvedDecisionId,
        resolvedBearerToken
      );
      assertExactEvidenceScope(evidence, contextId, resolvedCommitId, resolvedDecisionId);
      setModel(presentBenchmarkDecision(evidence));
      setRunDetailsStatus("loading");
      try {
        const details = await loadLocalBenchmarkDecisionRunDetails(
          projectId,
          contextId,
          resolvedCommitId,
          resolvedDecisionId,
          resolvedBearerToken
        );
        assertExactEvidenceScope(details, contextId, resolvedCommitId, resolvedDecisionId);
        setRunDetails(details);
        setRunDetailsStatus("ready");
      } catch (error) {
        setRunDetailsError(presentBenchmarkEvidenceError(error));
        setRunDetailsStatus("error");
      }
    } catch (error) {
      setNotice({ message: presentBenchmarkEvidenceError(error), tone: "error" });
    } finally {
      setIsLoading(false);
    }
  }

  async function discoverFirstDecision(commitId = selectedCommitId, token = bearerToken) {
    const resolvedCommitId = commitId.trim();
    const resolvedBearerToken = token.trim();
    if (!resolvedCommitId || !resolvedBearerToken || isLoading) {
      return;
    }

    const requestId = ++discoveryRequestId.current;
    const target = discoveryTarget(projectId, contextId, resolvedCommitId);
    setDiscovery(presentLocalBenchmarkDecisionDiscovery({ kind: "loading", target }));
    setDecisionId("");
    setModel(null);
    setRunDetails(null);
    setRunDetailsStatus("idle");
    setRunDetailsError(null);
    setNotice(null);

    try {
      const summary = await loadLocalBenchmarkDecisionDiscovery(
        projectId,
        contextId,
        resolvedCommitId,
        resolvedBearerToken
      );
      if (requestId !== discoveryRequestId.current) {
        return;
      }

      const view = presentLocalBenchmarkDecisionDiscovery({ kind: "ready", target, summary });
      setDiscovery(view);
      const firstDecisionId = view.firstDecisionId;
      if (firstDecisionId) {
        setDecisionId(firstDecisionId);
        await inspectDecision(firstDecisionId, resolvedCommitId, resolvedBearerToken);
      }
    } catch (error) {
      if (requestId !== discoveryRequestId.current) {
        return;
      }

      const unavailable = isUnavailableDiscoveryError(error);
      const resource: BenchmarkDecisionDiscoveryResource = unavailable
        ? { kind: "unavailable", target }
        : { kind: "error", target, message: presentBenchmarkDiscoveryError(error) };
      setDiscovery(
        presentLocalBenchmarkDecisionDiscovery(resource)
      );
      setDecisionId("");
    }
  }

  const metricRows = model?.metrics.map((metric) => ({
    id: metric.id,
    cells: [
      metric.metric,
      metric.threshold,
      metric.observed,
      metric.coverage,
      <StatusPill key={`${metric.id}-outcome`} tone={metric.outcome.tone}>
        {metric.outcome.label}
      </StatusPill>
    ]
  }));

  return (
    <section className="context-benchmark-evidence" aria-labelledby="context-benchmark-evidence-heading">
      <div className="context-benchmark-evidence__heading">
        <div>
          <span className="eyebrow">Local Benchmark Evidence / 本地 Benchmark 证据</span>
          <h3 id="context-benchmark-evidence-heading">Benchmark Evidence Inspection</h3>
          <p>
            Protected local read-only review / 只读本地受保护审阅。No credential is persisted / 不会持久化凭据。
          </p>
        </div>
        <StatusPill tone="info">read only / 只读</StatusPill>
      </div>

      <div className="context-benchmark-evidence__controls">
        <Input
          autoComplete="off"
          disabled={!canEditScope}
          description="Memory only / 仅内存"
          label="Bearer token / 访问令牌"
          name="benchmark-evidence-bearer-token"
          onBlur={() => void discoverFirstDecision()}
          onChange={(event) => {
            const nextToken = event.target.value;
            discoveryRequestId.current += 1;
            setBearerToken(nextToken);
            setDecisionId("");
            setDiscovery(
              presentLocalBenchmarkDecisionDiscovery({
                kind: "empty",
                target: discoveryTarget(projectId, contextId, selectedCommitId)
              })
            );
          }}
          type="password"
          value={bearerToken}
        />
        <label className="cl-field">
          <span className="cl-field__label">Materialized commit / 已物化提交</span>
          <select
            className="cl-select"
            disabled={!canEditScope}
            name="benchmark-evidence-commit"
            onChange={(event) => {
              const nextCommitId = event.target.value;
              discoveryRequestId.current += 1;
              setSelectedCommitId(nextCommitId);
              setDecisionId("");
              setDiscovery(
                presentLocalBenchmarkDecisionDiscovery({
                  kind: "empty",
                  target: discoveryTarget(projectId, contextId, nextCommitId)
                })
              );
              if (bearerToken.trim()) {
                void discoverFirstDecision(nextCommitId, bearerToken);
              }
            }}
            value={selectedCommitId}
          >
            {candidates.length === 0 ? <option value="">No materialized commits / 暂无已物化提交</option> : null}
            {candidates.map((candidate) => (
              <option key={candidate.id} value={candidate.id}>
                {candidate.label}
              </option>
            ))}
          </select>
        </label>
        <Input
          autoComplete="off"
          disabled={!canEditScope}
          label="Decision ID / 决策 ID"
          name="benchmark-evidence-decision-id"
          description="Auto-selected from the exact commit / 从精确提交自动选择"
          onChange={(event) => setDecisionId(event.target.value)}
          value={decisionId}
        />
        <Button disabled={!canInspect} tone="muted" type="button" onClick={() => void inspectDecision()}>
          {isLoading ? "Inspecting... / 正在审阅..." : "Inspect decision / 审阅 decision"}
        </Button>
      </div>

      <BenchmarkDecisionDiscoveryState view={discovery} />

      {notice ? (
        <p className="context-benchmark-evidence__notice" data-tone={notice.tone} role="alert">
          {notice.message}
        </p>
      ) : null}

      {model ? (
        <div className="context-benchmark-evidence__result" aria-live="polite">
          <div className="context-benchmark-evidence__summary">
            <StatusPill tone={model.status.tone}>{model.status.label}</StatusPill>
            <CodeChip>{model.facts[0]?.value ?? ""}</CodeChip>
          </div>
          <DefinitionGrid columns={2} compact items={model.facts} surface="raised" valueTone="info" />
          <DefinitionGrid columns={2} compact items={model.comparability} surface="raised" valueTone="info" />
          <BenchmarkDefinitionMetadata definition={model.definition} />
          <div className="context-benchmark-evidence__identities">
            <div>
              <strong>Datasets / 数据集</strong>
              <p>{model.datasets.join(", ") || "None / 无"}</p>
            </div>
            <div>
              <strong>Runs / 运行</strong>
              <p>{model.runs.join(", ") || "None / 无"}</p>
            </div>
          </div>
          <StackTable
            aria-label="Benchmark decision metric evidence"
            columnTemplate="minmax(7rem, 1fr) minmax(5rem, 0.7fr) minmax(7rem, 0.9fr) minmax(10rem, 1.1fr) minmax(8rem, 0.9fr)"
            headers={["Metric / 指标", "Threshold / 阈值", "Observed / 观测值", "Coverage / 覆盖", "Outcome / 结果"]}
            rows={metricRows ?? []}
          />
          <BenchmarkDecisionRunDetails
            details={runDetails}
            errorMessage={runDetailsError}
            status={runDetailsStatus}
          />
        </div>
      ) : isLoading ? (
        <p aria-live="polite" className="context-benchmark-evidence__empty" role="status">
          Loading benchmark decision and sealed run details / 正在加载 benchmark decision 与已封存运行详情。
        </p>
      ) : notice ? null : (
        <p className="context-benchmark-evidence__empty">No decision selected / 未选择 decision。</p>
      )}
    </section>
  );
}

export function BenchmarkDecisionDiscoveryState({ view }: BenchmarkDecisionDiscoveryStateProps) {
  return (
    <section
      aria-labelledby="context-benchmark-decision-discovery-heading"
      className="context-benchmark-evidence__result"
    >
      <div className="context-benchmark-evidence__summary">
        <div>
          <h4 id="context-benchmark-decision-discovery-heading">{view.title}</h4>
          <p>{view.description}</p>
        </div>
        <StatusPill tone="info">read only / 只读</StatusPill>
      </div>
      <CapabilityStateScreen view={view.status} />
      <DefinitionGrid
        aria-label="Benchmark decision discovery scope / Benchmark decision 发现范围"
        columns={3}
        compact
        items={view.scope.map((item) => ({
          id: item.id,
          label: item.label,
          value: <CodeChip>{item.value || "-"}</CodeChip>
        }))}
        surface="raised"
        valueTone="info"
      />
      {view.decisions.length > 0 ? (
        <StackTable
          aria-label="Benchmark decision summaries / Benchmark decision 摘要"
          columnTemplate="minmax(8rem, 1fr) minmax(8rem, 1fr) minmax(9rem, 1fr) minmax(12rem, 1.2fr)"
          headers={["Decision / 决策", "Suite / 套件", "Status / 状态", "Recorded / 记录时间"]}
          rows={view.decisions.map((decision) => ({
            id: decision.id,
            cells: [
              <CodeChip key={`${decision.id}-id`}>{decision.decisionId}</CodeChip>,
              <CodeChip key={`${decision.id}-suite`}>{decision.suiteId}</CodeChip>,
              <StatusPill key={`${decision.id}-status`} tone={decision.status.tone}>{decision.status.label}</StatusPill>,
              decision.recordedAt
            ]
          }))}
        />
      ) : null}
    </section>
  );
}

export function BenchmarkDecisionRunDetails({
  details,
  errorMessage,
  status
}: BenchmarkDecisionRunDetailsProps) {
  const model = status === "ready" && details !== null
    ? presentBenchmarkDecisionRunDetails(details)
    : null;

  if (status === "loading") {
    return (
      <section aria-labelledby="context-benchmark-run-details-heading" className="context-benchmark-evidence__result">
        <RunDetailsHeading />
        <p aria-live="polite" className="context-benchmark-evidence__empty" role="status">
          Loading sealed run details / 正在加载已封存运行详情。
        </p>
      </section>
    );
  }

  if (status === "error") {
    return (
      <section aria-labelledby="context-benchmark-run-details-heading" className="context-benchmark-evidence__result">
        <RunDetailsHeading />
        <p className="context-benchmark-evidence__notice" data-tone="error" role="alert">
          Run details unavailable / 运行详情不可用。{errorMessage ?? "Unable to load sealed run details / 无法加载已封存运行详情。"}
        </p>
      </section>
    );
  }

  if (status === "ready" && model?.runs.length === 0) {
    return (
      <section aria-labelledby="context-benchmark-run-details-heading" className="context-benchmark-evidence__result">
        <RunDetailsHeading />
        <p aria-live="polite" className="context-benchmark-evidence__empty">
          No sealed runs / 暂无已封存运行。
        </p>
      </section>
    );
  }

  if (status !== "ready" || model === null) {
    return null;
  }

  return (
    <section aria-labelledby="context-benchmark-run-details-heading" className="context-benchmark-evidence__result">
      <RunDetailsHeading />
      {model.runs.map((run) => (
        <div className="context-benchmark-evidence__identities" key={run.id}>
          <div className="context-benchmark-evidence__summary">
            <CodeChip>{run.id}</CodeChip>
            <StatusPill tone="info">sealed / 已封存</StatusPill>
          </div>
          <DefinitionGrid columns={2} compact items={run.facts} surface="raised" valueTone="info" />
          <StackTable
            aria-label={`Metrics for ${run.id} / ${run.id} 的指标`}
            columnTemplate="minmax(10rem, 1fr) minmax(8rem, 0.8fr)"
            headers={["Metric / 指标", "Value / 数值"]}
            rows={run.metrics.map((metric) => ({
              id: metric.id,
              cells: [metric.label, metric.value]
            }))}
          />
        </div>
      ))}
    </section>
  );
}

function RunDetailsHeading() {
  return (
    <div className="context-benchmark-evidence__summary">
      <div>
        <h4 id="context-benchmark-run-details-heading">Sealed run details / 已封存运行详情</h4>
        <p>Safe numeric evidence only / 仅安全数值证据</p>
      </div>
      <StatusPill tone="info">read only / 只读</StatusPill>
    </div>
  );
}

export function BenchmarkDefinitionMetadata({
  definition
}: {
  definition: BenchmarkDefinitionViewModel;
}) {
  const datasetRows = definition.datasets.map((dataset) => ({
    id: dataset.id,
    cells: [dataset.id, dataset.name, dataset.caseCount]
  }));

  return (
    <section aria-labelledby="context-benchmark-definition-heading" className="context-benchmark-evidence__result">
      <div className="context-benchmark-evidence__summary">
        <div>
          <h4 id="context-benchmark-definition-heading">Sealed definition / 已封存定义</h4>
          <p>Metadata only / 仅元数据</p>
        </div>
        <StatusPill tone="info">sealed / 已封存</StatusPill>
      </div>
      <DefinitionGrid
        columns={2}
        compact
        items={[
          { id: "suite-id", label: "Suite ID / 套件 ID", value: definition.suite.id },
          { id: "suite-name", label: "Suite name / 套件名称", value: definition.suite.name }
        ]}
        surface="raised"
        valueFont="sans"
        valueTone="info"
      />
      <div>
        <h5>Thresholds / 阈值</h5>
        <DefinitionGrid
          columns={2}
          compact
          items={definition.suite.thresholds}
          surface="raised"
          valueFont="sans"
          valueTone="info"
        />
      </div>
      <StackTable
        aria-label="Benchmark definition datasets"
        columnTemplate="minmax(8rem, 1fr) minmax(10rem, 1.5fr) minmax(8rem, 0.8fr)"
        headers={["Dataset ID / 数据集 ID", "Dataset name / 数据集名称", "Case count / 用例数"]}
        rows={datasetRows}
      />
    </section>
  );
}

export function presentBenchmarkEvidenceError(error: unknown): string {
  if (error instanceof LocalBenchmarkEvidenceProxyError) {
    return benchmarkEvidenceErrorMessage(error.status, error.retryAfterMs);
  }

  return "Unable to reach local benchmark evidence inspection / 无法连接本地 benchmark 证据审阅。";
}

function presentBenchmarkDiscoveryError(error: unknown): string {
  if (error instanceof LocalBenchmarkDecisionDiscoveryProxyError) {
    return benchmarkDecisionDiscoveryErrorMessage(error.status, error.retryAfterMs);
  }

  return "Unable to reach local benchmark decision discovery / 无法连接本地 benchmark decision 发现。";
}

function isUnavailableDiscoveryError(error: unknown): boolean {
  return error instanceof LocalBenchmarkDecisionDiscoveryProxyError && error.status === 503;
}

function discoveryTarget(projectId: string, contextId: string, commitId: string): BenchmarkDecisionDiscoveryTarget {
  return { projectId, contextId, commitId };
}

function assertExactEvidenceScope(
  value: { project_id: string; context_id: string; commit_id: string; decision_id: string },
  contextId: string,
  commitId: string,
  decisionId: string
): void {
  if (value.context_id !== contextId || value.commit_id !== commitId || value.decision_id !== decisionId) {
    throw new TypeError("benchmark evidence scope does not match the selected exact commit");
  }
}
