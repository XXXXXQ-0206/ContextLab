"use client";

import { Button, Input, Select, StatusPill } from "@contextlab/ui";
import type { LocalApiErrorBody } from "@contextlab/local-sdk";
import React, { useEffect, useMemo, useRef, useState } from "react";
import {
  parseLocalWorkflowContextBindingsV1,
  type LocalWorkflowContextBindingV1,
  type LocalWorkflowContextBindingsResource,
  type LocalWorkflowContextBindingsTarget
} from "./local-workflow-context-bindings-data";
import { LocalWorkflowContextBindingsScreen } from "./local-workflow-context-bindings-screen";
import {
  loadLocalWorkflowExecutionStatus,
  LocalWorkflowExecutionStatusProxyError,
  type LocalWorkflowExecutionStatusResource,
  type LocalWorkflowExecutionStatusTarget
} from "./local-workflow-execution-status-data";
import { LocalWorkflowExecutionStatusScreen } from "./local-workflow-execution-status-screen";

export type LocalWorkflowContextBindingsInspectorProps = {
  contextId: string;
  commitId: string;
};

export function LocalWorkflowContextBindingsInspector({
  contextId,
  commitId
}: LocalWorkflowContextBindingsInspectorProps) {
  const [bearerToken, setBearerToken] = useState("");
  const [resource, setResource] = useState<LocalWorkflowContextBindingsResource>(() => ({
    kind: "empty",
    target: createTarget(contextId, commitId)
  }));
  const [selectedBindingId, setSelectedBindingId] = useState("");
  const [workflowRunId, setWorkflowRunId] = useState("");
  const [statusResource, setStatusResource] = useState<LocalWorkflowExecutionStatusResource | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [statusNotice, setStatusNotice] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isStatusLoading, setIsStatusLoading] = useState(false);
  const requestGeneration = useRef(0);
  const statusRequestGeneration = useRef(0);
  const target = useMemo(() => createTarget(contextId, commitId), [contextId, commitId]);
  const canInspect = bearerToken.trim().length > 0 && !isLoading && !isStatusLoading;
  const bindings = resource.kind === "ready" ? resource.summary.bindings : [];
  const selectedBinding = bindings.find((binding) => binding.binding_id === selectedBindingId);
  const canInspectStatus = resource.kind === "ready"
    && selectedBinding !== undefined
    && isCanonicalUuid(workflowRunId.trim())
    && bearerToken.trim().length > 0
    && !isLoading
    && !isStatusLoading;

  useEffect(() => {
    requestGeneration.current += 1;
    statusRequestGeneration.current += 1;
    setResource({ kind: "empty", target });
    setSelectedBindingId("");
    setWorkflowRunId("");
    setStatusResource(null);
    setNotice(null);
    setStatusNotice(null);
    setIsLoading(false);
    setIsStatusLoading(false);
  }, [target]);

  async function inspectBindings() {
    if (!canInspect) {
      return;
    }

    statusRequestGeneration.current += 1;
    const generation = requestGeneration.current + 1;
    requestGeneration.current = generation;
    setIsLoading(true);
    setNotice(null);
    setSelectedBindingId("");
    setWorkflowRunId("");
    setStatusResource(null);
    setStatusNotice(null);
    setResource({ kind: "loading", target });

    try {
      const response = await fetch(
        `/api/local/contexts/${encodeURIComponent(contextId)}/commits/${encodeURIComponent(commitId)}/workflow-bindings`,
        {
          headers: {
            accept: "application/json",
            authorization: `Bearer ${bearerToken.trim()}`
          },
          credentials: "omit",
          cache: "no-store"
        }
      );

      if (!response.ok) {
        throw new LocalWorkflowContextBindingsInspectorError(
          response.status,
          await parseProxyErrorBody(response)
        );
      }

      const summary = parseLocalWorkflowContextBindingsV1(await response.json());
      if (summary.context_id !== contextId || summary.commit_id !== commitId) {
        throw new TypeError("workflow Context bindings response does not match the selected commit scope");
      }
      if (requestGeneration.current !== generation) {
        return;
      }
      setResource({ kind: "ready", target, summary });
    } catch (error) {
      if (requestGeneration.current !== generation) {
        return;
      }
      const kind = error instanceof LocalWorkflowContextBindingsInspectorError && error.status === 503
        ? "unavailable"
        : "error";
      setResource({ kind, target });
      setNotice(presentBindingsError(error));
    } finally {
      if (requestGeneration.current === generation) {
        setIsLoading(false);
      }
    }
  }

  function selectBinding(bindingId: string) {
    statusRequestGeneration.current += 1;
    setSelectedBindingId(bindingId);
    setStatusResource(null);
    setStatusNotice(null);
    setIsStatusLoading(false);
  }

  function changeWorkflowRunId(runId: string) {
    statusRequestGeneration.current += 1;
    setWorkflowRunId(runId);
    setStatusResource(null);
    setStatusNotice(
      runId.trim().length > 0 && !isCanonicalUuid(runId.trim())
        ? "Enter a lowercase canonical workflow run UUID / 请输入小写规范 Workflow run UUID。"
        : null
    );
    setIsStatusLoading(false);
  }

  async function inspectStatus() {
    if (!canInspectStatus || selectedBinding === undefined) {
      return;
    }

    const exactTarget = createExecutionStatusTarget(target, selectedBinding, workflowRunId.trim());
    if (!isCanonicalExecutionStatusTarget(exactTarget)) {
      setStatusResource(null);
      setStatusNotice("The selected binding scope is not canonical / 选定绑定范围不是规范 UUID。读取已拒绝。/ Read rejected.");
      return;
    }

    const generation = statusRequestGeneration.current + 1;
    statusRequestGeneration.current = generation;
    setIsStatusLoading(true);
    setStatusNotice(null);
    setStatusResource({ kind: "loading", target: exactTarget });

    try {
      const status = await loadLocalWorkflowExecutionStatus(exactTarget, bearerToken.trim());
      if (statusRequestGeneration.current !== generation) {
        return;
      }
      setStatusResource({ kind: "ready", target: exactTarget, status });
    } catch (error) {
      if (statusRequestGeneration.current !== generation) {
        return;
      }
      const kind = error instanceof LocalWorkflowExecutionStatusProxyError && error.status === 503
        ? "unavailable"
        : "error";
      setStatusResource({ kind, target: exactTarget });
      setStatusNotice(presentStatusError(error));
    } finally {
      if (statusRequestGeneration.current === generation) {
        setIsStatusLoading(false);
      }
    }
  }

  return (
    <section aria-labelledby="local-workflow-context-bindings-inspector-heading" className="context-benchmark-evidence">
      <div className="context-benchmark-evidence__heading">
        <div>
          <span className="eyebrow">Workflow Context Bindings / Workflow 上下文绑定</span>
          <h3 id="local-workflow-context-bindings-inspector-heading">
            Selected commit bindings / 选定提交绑定
          </h3>
          <p>
            Protected local read-only inspection / 受保护的本地只读检查。No credential is persisted / 不会持久化凭据。
          </p>
        </div>
        <StatusPill tone="info">read only / 只读</StatusPill>
      </div>
      <div className="context-benchmark-evidence__controls">
        <Input
          autoComplete="off"
          description="Memory only / 仅内存"
          label="Bearer token / 访问令牌"
          name="workflow-context-bindings-bearer-token"
          onChange={(event) => setBearerToken(event.target.value)}
          type="password"
          value={bearerToken}
        />
        <Button disabled={!canInspect} tone="muted" type="button" onClick={inspectBindings}>
          {isLoading ? "Inspecting... / 正在审阅..." : "Inspect bindings / 审阅绑定"}
        </Button>
      </div>
      <p className="context-benchmark-evidence__scope">
        Context / 上下文: <code>{contextId}</code> · Commit / 提交: <code>{commitId}</code>
      </p>
      {notice ? (
        <p className="context-benchmark-evidence__notice" role="alert">
          {notice}
        </p>
      ) : null}
      <LocalWorkflowContextBindingsScreen resource={resource} />
      <div className="context-benchmark-evidence__controls">
        <Select
          disabled={resource.kind !== "ready" || isLoading || isStatusLoading}
          label="Exact binding row / 精确绑定行"
          name="workflow-context-bindings-selected-binding"
          onChange={(event) => selectBinding(event.target.value)}
          value={selectedBindingId}
        >
          <option value="">Select a binding row / 选择绑定行</option>
          {bindings.map((binding) => (
            <option key={binding.binding_id} value={binding.binding_id}>
              {binding.binding_id} · {binding.workflow_id} · r{binding.workflow_revision}
            </option>
          ))}
        </Select>
        <Input
          autoComplete="off"
          description="Explicit canonical UUID; memory only / 显式规范 UUID；仅内存"
          disabled={resource.kind !== "ready" || isLoading || isStatusLoading}
          label="Workflow run_id / Workflow run_id"
          name="workflow-context-bindings-workflow-run-id"
          onChange={(event) => changeWorkflowRunId(event.target.value)}
          pattern="[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}"
          spellCheck={false}
          value={workflowRunId}
        />
        <Button disabled={!canInspectStatus} tone="muted" type="button" onClick={inspectStatus}>
          {isStatusLoading ? "Inspecting status... / 正在审阅状态..." : "Inspect run status / 审阅运行状态"}
        </Button>
      </div>
      {statusNotice ? (
        <p className="context-benchmark-evidence__notice" role="alert">
          {statusNotice}
        </p>
      ) : null}
      {statusResource ? <LocalWorkflowExecutionStatusScreen resource={statusResource} /> : null}
    </section>
  );
}

class LocalWorkflowContextBindingsInspectorError extends Error {
  readonly status: number;
  readonly body: LocalApiErrorBody;

  constructor(status: number, body: LocalApiErrorBody) {
    super(body.message);
    this.name = "LocalWorkflowContextBindingsInspectorError";
    this.status = status;
    this.body = body;
  }
}

function createTarget(contextId: string, commitId: string): LocalWorkflowContextBindingsTarget {
  return Object.freeze({
    context_id: contextId,
    commit_id: commitId,
    capability: Object.freeze({
      en: "Workflow context bindings",
      zh: "Workflow 上下文绑定"
    })
  });
}

async function parseProxyErrorBody(response: Response): Promise<LocalApiErrorBody> {
  try {
    const body = (await response.json()) as Partial<LocalApiErrorBody>;
    if (typeof body.error === "string" && typeof body.message === "string") {
      return { error: body.error, message: body.message };
    }
  } catch {
  }

  return {
    error: "contextlab_workflow_context_bindings_proxy_error",
    message: `ContextLab workflow context bindings request failed with status ${response.status}`
  };
}

function presentBindingsError(error: unknown): string {
  if (error instanceof LocalWorkflowContextBindingsInspectorError) {
    if (error.status === 401) {
      return "Bearer authentication is required / 需要 Bearer 身份验证。";
    }
    if (error.status === 403) {
      return "You do not have permission for this Context commit / 你没有此 Context 提交的权限。";
    }
    if (error.status === 429) {
      return "The local rate limit is active / 本地速率限制已生效。";
    }
    if (error.status === 503) {
      return "Workflow context bindings are unavailable / Workflow 上下文绑定暂不可用。";
    }
    return `Unable to inspect workflow context bindings / 无法检查 Workflow 上下文绑定。${error.body.message}`;
  }

  if (error instanceof TypeError) {
    return "Workflow context bindings response was rejected / Workflow 上下文绑定响应已被拒绝。";
  }

  return "Unable to reach workflow context bindings / 无法连接 Workflow 上下文绑定。";
}

function createExecutionStatusTarget(
  bindingsTarget: LocalWorkflowContextBindingsTarget,
  binding: LocalWorkflowContextBindingV1,
  runId: string
): LocalWorkflowExecutionStatusTarget {
  return {
    context_id: bindingsTarget.context_id,
    context_commit_id: bindingsTarget.commit_id,
    binding_id: binding.binding_id,
    workflow_id: binding.workflow_id,
    workflow_revision: binding.workflow_revision,
    run_id: runId,
    capability: {
      en: "Workflow execution status",
      zh: "工作流执行状态"
    }
  };
}

function isCanonicalExecutionStatusTarget(target: LocalWorkflowExecutionStatusTarget): boolean {
  return [
    target.context_id,
    target.context_commit_id,
    target.binding_id,
    target.workflow_id,
    target.run_id
  ].every(isCanonicalUuid);
}

function isCanonicalUuid(value: string): boolean {
  return /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(value);
}

function presentStatusError(error: unknown): string {
  if (error instanceof LocalWorkflowExecutionStatusProxyError) {
    if (error.status === 401) {
      return "Bearer authentication is required / 需要 Bearer 身份验证。";
    }
    if (error.status === 403) {
      return "You do not have permission for this workflow run / 你没有此 Workflow run 的权限。";
    }
    if (error.status === 429) {
      return "The local rate limit is active / 本地速率限制已生效。";
    }
    if (error.status === 503) {
      return "Workflow execution status is unavailable / Workflow 执行状态暂不可用。";
    }
    return "Unable to inspect workflow execution status / 无法检查 Workflow 执行状态。";
  }

  if (error instanceof TypeError) {
    return "Workflow execution status response was rejected / Workflow 执行状态响应已被拒绝。";
  }

  return "Unable to reach workflow execution status / 无法连接 Workflow 执行状态。";
}
