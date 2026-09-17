"use client";

import { Button, Input } from "@contextlab/ui";
import { Play } from "lucide-react";
import React, { useState } from "react";
import type { LocalBenchmarkExecutionRequestV1 } from "@contextlab/local-sdk";
import {
  createLocalBenchmarkExecutionResource,
  executeLocalBenchmarkExecution,
  LocalBenchmarkExecutionProxyError,
  type LocalBenchmarkExecutionResource,
  type LocalBenchmarkExecutionTarget
} from "./local-benchmark-execution-data";
import { presentLocalBenchmarkExecution } from "./local-benchmark-execution-presenter";
import { LocalBenchmarkExecutionScreen } from "./local-benchmark-execution-screen";

export type LocalBenchmarkExecutionInspectorProps = Readonly<{
  projectId: string;
  contextId: string;
  commitId: string;
  enabled?: boolean;
}>;

type Draft = Readonly<{
  bearerToken: string;
  bindingId: string;
  decisionId: string;
  modelVersion: string;
  temperature: string;
  evaluatorKey: string;
  evaluatorVersion: string;
}>;

const emptyDraft: Draft = Object.freeze({
  bearerToken: "",
  bindingId: "",
  decisionId: "",
  modelVersion: "deterministic-v1",
  temperature: "0",
  evaluatorKey: "local-deterministic",
  evaluatorVersion: "v1"
});

export function LocalBenchmarkExecutionInspector({
  projectId,
  contextId,
  commitId,
  enabled = false
}: LocalBenchmarkExecutionInspectorProps) {
  const target = Object.freeze({ projectId, contextId, commitId });
  const [draft, setDraft] = useState<Draft>(emptyDraft);
  const [resource, setResource] = useState<LocalBenchmarkExecutionResource>(() => initialResource(target, enabled));
  const isPending = resource.state === "loading";
  const temperature = Number(draft.temperature);
  const canSubmit = Boolean(
    enabled
    && !isPending
    && commitId.trim()
    && draft.bearerToken.trim()
    && draft.bindingId.trim()
    && draft.decisionId.trim()
    && draft.modelVersion.trim()
    && draft.evaluatorKey.trim()
    && draft.evaluatorVersion.trim()
    && Number.isFinite(temperature)
    && temperature >= 0
    && temperature <= 2
  );

  function update(field: keyof Draft, value: string) {
    setDraft((current) => Object.freeze({ ...current, [field]: value }));
    if (!isPending) setResource(initialResource(target, enabled));
  }

  async function submit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!canSubmit) return;
    const request: LocalBenchmarkExecutionRequestV1 = {
      schema_version: 1,
      binding_id: draft.bindingId,
      decision_id: draft.decisionId,
      model_version: draft.modelVersion,
      temperature,
      evaluator_key: draft.evaluatorKey,
      evaluator_version: draft.evaluatorVersion
    };
    setResource(createLocalBenchmarkExecutionResource({ state: "loading", target }));
    try {
      const receipt = await executeLocalBenchmarkExecution(
        target,
        request,
        draft.bearerToken,
        createIdempotencyKey()
      );
      setResource(createLocalBenchmarkExecutionResource({
        state: receipt.disposition,
        target,
        receipt
      }));
    } catch (error) {
      setResource(createLocalBenchmarkExecutionResource({
        state: errorState(error),
        target,
        message: executionErrorMessage(error)
      }));
    }
  }

  const controls = (
    <form
      aria-busy={isPending || undefined}
      aria-label="Execute exact private benchmark / 执行精确私有 Benchmark"
      className="local-benchmark-execution__controls"
      onSubmit={submit}
    >
      <Input autoComplete="off" description="Memory only / 仅内存" disabled={isPending} label="Bearer token / 访问令牌" name="benchmark-execution-bearer-token" onChange={(event) => update("bearerToken", event.target.value)} type="password" value={draft.bearerToken} />
      <Input disabled={isPending} label="Binding ID / 绑定 ID" name="benchmark-execution-binding-id" onChange={(event) => update("bindingId", event.target.value)} value={draft.bindingId} />
      <Input disabled={isPending} label="Decision ID / 决策 ID" name="benchmark-execution-decision-id" onChange={(event) => update("decisionId", event.target.value)} value={draft.decisionId} />
      <Input disabled={isPending} label="Model version / 模型版本" name="benchmark-execution-model-version" onChange={(event) => update("modelVersion", event.target.value)} value={draft.modelVersion} />
      <Input disabled={isPending} inputMode="decimal" label="Temperature / 温度（0-2）" name="benchmark-execution-temperature" onChange={(event) => update("temperature", event.target.value)} value={draft.temperature} />
      <Input disabled={isPending} label="Evaluator / 评测器" name="benchmark-execution-evaluator-key" onChange={(event) => update("evaluatorKey", event.target.value)} value={draft.evaluatorKey} />
      <Input disabled={isPending} label="Evaluator version / 评测器版本" name="benchmark-execution-evaluator-version" onChange={(event) => update("evaluatorVersion", event.target.value)} value={draft.evaluatorVersion} />
      <div className="local-benchmark-execution__submit">
        <Button disabled={!canSubmit} icon={<Play aria-hidden="true" />} type="submit">
          {isPending ? "Executing exact benchmark... / 正在执行精确 Benchmark..." : "Execute privately / 私有执行"}
        </Button>
        <span>Server-owned binding, policy, evidence, and replay / 绑定、策略、证据与回放均由服务端控制。</span>
      </div>
    </form>
  );

  return <LocalBenchmarkExecutionScreen controls={controls} view={presentLocalBenchmarkExecution(resource)} />;
}

function initialResource(target: LocalBenchmarkExecutionTarget, enabled: boolean): LocalBenchmarkExecutionResource {
  if (!enabled) {
    return createLocalBenchmarkExecutionResource({
      state: "unavailable",
      target,
      message: "Private benchmark execution is off by default / 私有 Benchmark 执行默认关闭。"
    });
  }
  if (!target.commitId.trim()) {
    return createLocalBenchmarkExecutionResource({
      state: "unavailable",
      target,
      message: "No exact materialized commit is available / 没有可用的精确已物化提交。"
    });
  }
  return createLocalBenchmarkExecutionResource({
    state: "empty",
    target,
    message: "Ready for one server-owned execution / 已准备一次服务端控制的执行。"
  });
}

function errorState(error: unknown): "error" | "empty" | "conflict" | "unavailable" {
  if (error instanceof LocalBenchmarkExecutionProxyError) {
    if (error.status === 404) return "empty";
    if (error.status === 409) return "conflict";
    if (error.status === 503) return "unavailable";
  }
  return "error";
}

function executionErrorMessage(error: unknown): string {
  if (error instanceof LocalBenchmarkExecutionProxyError) {
    if (error.status === 401) return "Bearer authentication is required / 需要 Bearer 身份验证。";
    if (error.status === 403) return "This exact Context scope is forbidden / 无权访问此精确 Context 范围。";
    if (error.status === 404) return "The exact benchmark definition is unavailable / 精确 Benchmark 定义不可用。";
    if (error.status === 409) return "The idempotency key conflicts with immutable execution state / 幂等键与不可变执行状态冲突。";
    if (error.status === 429) return "The local benchmark execution rate limit is active / 本地 Benchmark 执行速率限制已生效。";
    if (error.status === 503) return "The local evaluator or execution adapter is unavailable / 本地评测器或执行适配器不可用。";
  }
  return "The private benchmark execution request failed / 私有 Benchmark 执行请求失败。";
}

function createIdempotencyKey(): string {
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") return crypto.randomUUID();
  throw new Error("Secure idempotency-key generation is unavailable");
}
