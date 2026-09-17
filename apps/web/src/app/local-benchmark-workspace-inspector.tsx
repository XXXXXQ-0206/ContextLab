"use client";

import { Button, Input, Select } from "@contextlab/ui";
import { RefreshCw } from "lucide-react";
import React, { useRef, useState } from "react";
import { CapabilityStateScreen } from "./capability-state-screen";
import {
  LocalBenchmarkDecisionDiscoveryProxyError
} from "./context-benchmark-decision-discovery-data";
import { benchmarkDecisionDiscoveryErrorMessage } from "./context-benchmark-decision-discovery-presenter";
import { ContextBenchmarkDecisionDiffInspector } from "./context-benchmark-decision-diff-inspector";
import { ContextBenchmarkEvidenceInspector } from "./context-benchmark-evidence-inspector";
import {
  createLocalBenchmarkWorkspaceResource,
  loadLocalBenchmarkWorkspaceDecisionDiscoveries,
  loadLocalBenchmarkWorkspace,
  LocalBenchmarkWorkspaceProxyError,
  type LocalBenchmarkWorkspaceResource,
  type LocalBenchmarkWorkspaceResourceState,
  type LocalBenchmarkWorkspaceTarget
} from "./local-benchmark-workspace-data";
import {
  presentLocalBenchmarkWorkspace,
  presentLocalBenchmarkWorkspaceDecisionDiscoveries,
  presentLocalBenchmarkWorkspaceDecisionDiscovery,
  type LocalBenchmarkWorkspaceDecisionDiscoveryViewModel
} from "./local-benchmark-workspace-presenter";
import { LocalBenchmarkWorkspaceScreen } from "./local-benchmark-workspace-screen";
import type { CommitGraphReviewOption } from "./context-workspace-presenter";

export type LocalBenchmarkWorkspaceInspectorProps = Readonly<{
  projectId: string;
  contextId: string;
  candidates: ReadonlyArray<CommitGraphReviewOption>;
}>;

type LocalBenchmarkWorkspaceControlInput = Readonly<{
  bearerToken: string;
  projectId: string;
  contextId: string;
  revisedCommitId: string;
  revisedDecisionId: string;
  baselineCommitId: string;
  baselineDecisionId: string;
  isLoading: boolean;
  isDecisionDiscoveryLoading?: boolean;
  revisedDecisionOptions?: ReadonlyArray<string>;
  baselineDecisionOptions?: ReadonlyArray<string>;
}>;

export type LocalBenchmarkWorkspaceControlState = Readonly<{
  controlsDisabled: boolean;
  baselineDecisionDisabled: boolean;
  canLoad: boolean;
}>;

export function getLocalBenchmarkWorkspaceControlState(
  input: LocalBenchmarkWorkspaceControlInput
): LocalBenchmarkWorkspaceControlState {
  const hasBaselineCommit = input.baselineCommitId.trim().length > 0;
  const hasBaselineDecision = input.baselineDecisionId.trim().length > 0;
  const hasCompleteBaseline = hasBaselineCommit === hasBaselineDecision;
  const revisedDecisionMatches = input.revisedDecisionOptions === undefined
    || input.revisedDecisionOptions.includes(input.revisedDecisionId);
  const baselineDecisionMatches = !hasBaselineCommit
    || input.baselineDecisionOptions === undefined
    || input.baselineDecisionOptions.includes(input.baselineDecisionId);
  const controlsDisabled = input.isLoading || input.isDecisionDiscoveryLoading === true;
  return Object.freeze({
    controlsDisabled,
    baselineDecisionDisabled: controlsDisabled || !hasBaselineCommit,
    canLoad: Boolean(
      !controlsDisabled
      && input.bearerToken.trim()
      && input.projectId.trim()
      && input.contextId.trim()
      && input.revisedCommitId.trim()
      && input.revisedDecisionId.trim()
      && revisedDecisionMatches
      && hasCompleteBaseline
      && baselineDecisionMatches
    )
  });
}

export function LocalBenchmarkWorkspaceInspector({
  candidates,
  contextId,
  projectId
}: LocalBenchmarkWorkspaceInspectorProps) {
  const firstCommitId = candidates[0]?.id ?? "";
  const [bearerToken, setBearerToken] = useState("");
  const [revisedCommitId, setRevisedCommitId] = useState(firstCommitId);
  const [revisedDecisionId, setRevisedDecisionId] = useState("");
  const [baselineCommitId, setBaselineCommitId] = useState("");
  const [baselineDecisionId, setBaselineDecisionId] = useState("");
  const [decisionDiscoveries, setDecisionDiscoveries] = useState(() =>
    initialDecisionDiscoveries(projectId, contextId, firstCommitId, "")
  );
  const [resource, setResource] = useState<LocalBenchmarkWorkspaceResource>(() =>
    initialResource(projectId, contextId, firstCommitId)
  );
  const requestId = useRef(0);
  const decisionDiscoveryRequestId = useRef(0);
  const isLoading = resource.state === "loading";
  const isDecisionDiscoveryLoading = decisionDiscoveries.revised.status.state === "loading"
    || decisionDiscoveries.baseline.status.state === "loading";
  const controls = getLocalBenchmarkWorkspaceControlState({
    bearerToken,
    projectId,
    contextId,
    revisedCommitId,
    revisedDecisionId,
    baselineCommitId,
    baselineDecisionId,
    isLoading,
    isDecisionDiscoveryLoading,
    revisedDecisionOptions: decisionDiscoveries.revised.options.map((option) => option.value),
    baselineDecisionOptions: decisionDiscoveries.baseline.options.map((option) => option.value)
  });

  function resetProjection(next: {
    revisedCommitId?: string;
    revisedDecisionId?: string;
    baselineCommitId?: string;
    baselineDecisionId?: string;
  } = {}) {
    requestId.current += 1;
    const target = createTarget(
      projectId,
      contextId,
      next.revisedCommitId ?? revisedCommitId,
      next.revisedDecisionId ?? revisedDecisionId,
      next.baselineCommitId ?? baselineCommitId,
      next.baselineDecisionId ?? baselineDecisionId
    );
    setResource(
      createLocalBenchmarkWorkspaceResource(
        target.commitId
          ? {
              state: "empty",
              target,
              message: "No server projection loaded. / 尚未加载服务端投影。"
            }
          : {
              state: "unavailable",
              target,
              message: "No materialized commits are available. / 没有可用的已物化提交。"
            }
      )
    );
  }

  function resetDecisionDiscoveries(
    nextRevisedCommitId = revisedCommitId,
    nextBaselineCommitId = baselineCommitId
  ) {
    decisionDiscoveryRequestId.current += 1;
    setDecisionDiscoveries(
      initialDecisionDiscoveries(projectId, contextId, nextRevisedCommitId, nextBaselineCommitId)
    );
    setRevisedDecisionId("");
    setBaselineDecisionId("");
  }

  async function discoverDecisions(
    nextRevisedCommitId = revisedCommitId,
    nextBaselineCommitId = baselineCommitId,
    nextBearerToken = bearerToken
  ) {
    const resolvedRevisedCommitId = nextRevisedCommitId.trim();
    const resolvedBaselineCommitId = nextBaselineCommitId.trim();
    const resolvedBearerToken = nextBearerToken.trim();
    if (!resolvedRevisedCommitId || !resolvedBearerToken || isLoading) {
      return;
    }
    const target = {
      projectId,
      contextId,
      revisedCommitId: resolvedRevisedCommitId,
      ...(resolvedBaselineCommitId ? { baselineCommitId: resolvedBaselineCommitId } : {})
    };
    const currentRequestId = ++decisionDiscoveryRequestId.current;
    setDecisionDiscoveries(loadingDecisionDiscoveries(target));
    setRevisedDecisionId("");
    setBaselineDecisionId("");
    try {
      const discoveries = await loadLocalBenchmarkWorkspaceDecisionDiscoveries(target, resolvedBearerToken);
      if (currentRequestId !== decisionDiscoveryRequestId.current) return;
      const view = presentLocalBenchmarkWorkspaceDecisionDiscoveries(target, discoveries);
      setDecisionDiscoveries(view);
      setRevisedDecisionId(view.revised.firstDecisionId ?? "");
      setBaselineDecisionId(view.baseline.firstDecisionId ?? "");
    } catch (error) {
      if (currentRequestId !== decisionDiscoveryRequestId.current) return;
      const message = presentDecisionDiscoveryError(error);
      setDecisionDiscoveries({
        revised: decisionDiscoveryErrorView(
          projectId,
          contextId,
          resolvedRevisedCommitId,
          error,
          message
        ),
        baseline: resolvedBaselineCommitId
          ? decisionDiscoveryErrorView(
              projectId,
              contextId,
              resolvedBaselineCommitId,
              error,
              message
            )
          : presentLocalBenchmarkWorkspaceDecisionDiscovery({
              kind: "empty",
              target: { projectId, contextId, commitId: "" }
            })
      });
      setRevisedDecisionId("");
      setBaselineDecisionId("");
    }
  }

  async function loadWorkspace() {
    if (!controls.canLoad) {
      return;
    }

    const target = createTarget(
      projectId,
      contextId,
      revisedCommitId,
      revisedDecisionId,
      baselineCommitId,
      baselineDecisionId
    );
    const currentRequestId = ++requestId.current;
    setResource(createLocalBenchmarkWorkspaceResource({ state: "loading", target }));

    try {
      const workspace = await loadLocalBenchmarkWorkspace(target, bearerToken);
      if (currentRequestId === requestId.current) {
        setResource(
          createLocalBenchmarkWorkspaceResource({ state: "available", target, workspace })
        );
      }
    } catch (error) {
      if (currentRequestId === requestId.current) {
        setResource(errorResource(target, error));
      }
    }
  }

  const scopeControls = (
    <>
      <DecisionDiscoveryState label="Revised decision discovery / 修订 decision 发现" view={decisionDiscoveries.revised} />
      {baselineCommitId ? (
        <DecisionDiscoveryState label="Baseline decision discovery / 基线 decision 发现" view={decisionDiscoveries.baseline} />
      ) : null}
    <form
      aria-busy={isLoading || undefined}
      aria-label="Benchmark workspace exact scope / Benchmark 工作台精确范围"
      className="local-benchmark-workspace__controls"
      onSubmit={(event) => {
        event.preventDefault();
        void loadWorkspace();
      }}
    >
      <Input
        autoComplete="off"
        description="Memory only / 仅内存"
        disabled={controls.controlsDisabled || candidates.length === 0}
        label="Bearer token / 访问令牌"
        name="benchmark-workspace-bearer-token"
        onChange={(event) => {
          setBearerToken(event.target.value);
          resetDecisionDiscoveries();
          resetProjection({ revisedDecisionId: "", baselineDecisionId: "" });
        }}
        onBlur={() => void discoverDecisions()}
        type="password"
        value={bearerToken}
      />
      <Select
        disabled={controls.controlsDisabled || candidates.length === 0}
        label="Revised commit / 修订提交"
        name="benchmark-workspace-revised-commit"
        onChange={(event) => {
          const value = event.target.value;
          setRevisedCommitId(value);
          resetDecisionDiscoveries(value);
          resetProjection({ revisedCommitId: value, revisedDecisionId: "", baselineDecisionId: "" });
          if (bearerToken.trim()) void discoverDecisions(value, baselineCommitId, bearerToken);
        }}
        value={revisedCommitId}
      >
        {candidates.length === 0 ? (
          <option value="">No materialized commits / 暂无已物化提交</option>
        ) : (
          candidates.map((candidate) => (
            <option key={candidate.id} value={candidate.id}>{candidate.label}</option>
          ))
        )}
      </Select>
      <Select
        disabled={controls.controlsDisabled || decisionDiscoveries.revised.options.length === 0}
        label="Revised sealed decision / 修订 sealed decision"
        name="benchmark-workspace-revised-decision"
        value={revisedDecisionId}
        onChange={(event) => {
          const value = event.target.value;
          setRevisedDecisionId(value);
          resetProjection({ revisedDecisionId: value });
        }}
      >
        {decisionDiscoveries.revised.options.length === 0
          ? <option value="">No sealed decisions / 暂无 sealed decision</option>
          : decisionDiscoveries.revised.options.map((option) => (
              <option key={option.id} value={option.value}>{option.label}</option>
            ))}
      </Select>
      <Select
        disabled={controls.controlsDisabled || candidates.length === 0}
        label="Baseline commit / 基线提交"
        name="benchmark-workspace-baseline-commit"
        onChange={(event) => {
          const value = event.target.value;
          setBaselineCommitId(value);
          resetDecisionDiscoveries(revisedCommitId, value);
          resetProjection({ baselineCommitId: value, revisedDecisionId: "", baselineDecisionId: "" });
          if (bearerToken.trim()) void discoverDecisions(revisedCommitId, value, bearerToken);
        }}
        value={baselineCommitId}
      >
        <option value="">No baseline / 无基线</option>
        {candidates.map((candidate) => (
          <option key={candidate.id} value={candidate.id}>{candidate.label}</option>
        ))}
      </Select>
      <Select
        disabled={controls.baselineDecisionDisabled || decisionDiscoveries.baseline.options.length === 0}
        label="Baseline sealed decision / 基线 sealed decision"
        name="benchmark-workspace-baseline-decision"
        value={baselineDecisionId}
        onChange={(event) => {
          const value = event.target.value;
          setBaselineDecisionId(value);
          resetProjection({ baselineDecisionId: value });
        }}
      >
        {decisionDiscoveries.baseline.options.length === 0
          ? <option value="">No sealed decisions / 暂无 sealed decision</option>
          : decisionDiscoveries.baseline.options.map((option) => (
              <option key={option.id} value={option.value}>{option.label}</option>
            ))}
      </Select>
      <Button
        disabled={!controls.canLoad}
        icon={<RefreshCw aria-hidden="true" />}
        tone="muted"
        type="submit"
      >
        {isLoading
          ? "Loading exact workspace... / 正在加载精确工作台..."
          : "Load exact workspace / 加载精确工作台"}
      </Button>
    </form>
    </>
  );

  return (
    <section
      aria-label="Protected local benchmark workspace / 受保护的本地 Benchmark 工作台"
      className="local-benchmark-workspace-inspector"
    >
      <LocalBenchmarkWorkspaceScreen
        controls={scopeControls}
        view={presentLocalBenchmarkWorkspace(resource)}
      />

      <section
        aria-label="Benchmark decision inspectors / Benchmark 决策检查器"
        className="local-benchmark-workspace__decision-inspectors"
      >
        <ContextBenchmarkEvidenceInspector
          candidates={[...candidates]}
          contextId={contextId}
          projectId={projectId}
        />
        <ContextBenchmarkDecisionDiffInspector
          candidates={[...candidates]}
          contextId={contextId}
          projectId={projectId}
        />
      </section>
    </section>
  );
}

function initialDecisionDiscoveries(
  projectId: string,
  contextId: string,
  revisedCommitId: string,
  baselineCommitId: string
) {
  return {
    revised: presentLocalBenchmarkWorkspaceDecisionDiscovery({
      kind: "empty",
      target: { projectId, contextId, commitId: revisedCommitId }
    }),
    baseline: presentLocalBenchmarkWorkspaceDecisionDiscovery({
      kind: "empty",
      target: { projectId, contextId, commitId: baselineCommitId }
    })
  };
}

function loadingDecisionDiscoveries(target: {
  projectId: string;
  contextId: string;
  revisedCommitId: string;
  baselineCommitId?: string;
}) {
  return {
    revised: presentLocalBenchmarkWorkspaceDecisionDiscovery({
      kind: "loading",
      target: { projectId: target.projectId, contextId: target.contextId, commitId: target.revisedCommitId }
    }),
    baseline: target.baselineCommitId
      ? presentLocalBenchmarkWorkspaceDecisionDiscovery({
          kind: "loading",
          target: { projectId: target.projectId, contextId: target.contextId, commitId: target.baselineCommitId }
        })
      : presentLocalBenchmarkWorkspaceDecisionDiscovery({
          kind: "empty",
          target: { projectId: target.projectId, contextId: target.contextId, commitId: "" }
        })
  };
}

function DecisionDiscoveryState({
  label,
  view
}: Readonly<{ label: string; view: LocalBenchmarkWorkspaceDecisionDiscoveryViewModel }>) {
  return <CapabilityStateScreen view={{ ...view.status, id: `local-benchmark-workspace-${label}` }} />;
}

function presentDecisionDiscoveryError(error: unknown): string {
  if (error instanceof LocalBenchmarkDecisionDiscoveryProxyError) {
    return `${benchmarkDecisionDiscoveryErrorMessage(error.status, error.retryAfterMs)} ${error.body.message}`;
  }
  return "Unable to reach local benchmark decision discovery / 无法连接本地 benchmark decision 发现。";
}

function isUnavailableDecisionDiscoveryError(error: unknown): boolean {
  return error instanceof LocalBenchmarkDecisionDiscoveryProxyError && error.status === 503;
}

function decisionDiscoveryErrorView(
  projectId: string,
  contextId: string,
  commitId: string,
  error: unknown,
  message: string
): LocalBenchmarkWorkspaceDecisionDiscoveryViewModel {
  return isUnavailableDecisionDiscoveryError(error)
    ? presentLocalBenchmarkWorkspaceDecisionDiscovery({
        kind: "unavailable",
        target: { projectId, contextId, commitId }
      })
    : presentLocalBenchmarkWorkspaceDecisionDiscovery({
        kind: "error",
        target: { projectId, contextId, commitId },
        message
      });
}

function initialResource(
  projectId: string,
  contextId: string,
  commitId: string
): LocalBenchmarkWorkspaceResource {
  const target = createTarget(projectId, contextId, commitId, "", "", "");
  return createLocalBenchmarkWorkspaceResource(
    commitId
      ? {
          state: "empty",
          target,
          message: "No server projection loaded. / 尚未加载服务端投影。"
        }
      : {
          state: "unavailable",
          target,
          message: "No materialized commits are available. / 没有可用的已物化提交。"
        }
  );
}

function createTarget(
  projectId: string,
  contextId: string,
  commitId: string,
  decisionId: string,
  baselineCommitId: string,
  baselineDecisionId: string
): LocalBenchmarkWorkspaceTarget {
  const hasCompleteBaseline = baselineCommitId.trim() && baselineDecisionId.trim();
  return Object.freeze({
    projectId,
    contextId,
    commitId,
    decisionId,
    ...(hasCompleteBaseline
      ? {
          baseline: Object.freeze({
            commitId: baselineCommitId,
            decisionId: baselineDecisionId
          })
        }
      : {})
  });
}

function errorResource(
  target: LocalBenchmarkWorkspaceTarget,
  error: unknown
): LocalBenchmarkWorkspaceResource {
  const state = errorState(error);
  return createLocalBenchmarkWorkspaceResource({
    state,
    target,
    message: errorMessage(error)
  });
}

function errorState(error: unknown): Exclude<LocalBenchmarkWorkspaceResourceState, "available" | "loading"> {
  if (error instanceof LocalBenchmarkWorkspaceProxyError) {
    if (error.status === 404) {
      return "empty";
    }
    if (error.status === 503) {
      return "unavailable";
    }
  }
  return "error";
}

function errorMessage(error: unknown): string {
  if (error instanceof LocalBenchmarkWorkspaceProxyError) {
    const suffix = ` ${error.body.message}`;
    if (error.status === 401) {
      return `Bearer authentication is required / 需要 Bearer 身份验证。${suffix}`;
    }
    if (error.status === 403) {
      return `This exact Context scope is forbidden / 无权访问此精确 Context 范围。${suffix}`;
    }
    if (error.status === 404) {
      return `No exact server projection was found / 未找到精确的服务端投影。${suffix}`;
    }
    if (error.status === 429) {
      const retry = error.retryAfterMs === undefined
        ? ""
        : ` Retry in ${Math.ceil(error.retryAfterMs / 1_000)} seconds / ${Math.ceil(error.retryAfterMs / 1_000)} 秒后重试。`;
      return `The local rate limit is active / 本地速率限制已生效。${retry}${suffix}`;
    }
    if (error.status === 503) {
      return `Benchmark workspace is unavailable / Benchmark 工作台暂不可用。${suffix}`;
    }
    return `The protected benchmark workspace read failed / 受保护的 Benchmark 工作台读取失败。${suffix}`;
  }

  return "Unable to reach the protected benchmark workspace / 无法连接受保护的 Benchmark 工作台。";
}
