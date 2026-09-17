"use client";

import {
  Button,
  CapabilityState,
  CodeChip,
  DefinitionGrid,
  Input,
  Select,
  StatusPill,
  Textarea
} from "@contextlab/ui";
import { DatabaseZap } from "lucide-react";
import React, { useState } from "react";
import {
  authorLocalBenchmarkDefinition,
  createLocalBenchmarkDefinitionAuthoringResource,
  LocalBenchmarkDefinitionAuthoringProxyError,
  type LocalBenchmarkDefinitionAuthoringResource,
  type LocalBenchmarkDefinitionAuthoringTarget
} from "./local-benchmark-definition-authoring-data";
import {
  benchmarkDefinitionAuthoringErrorMessage,
  buildLocalBenchmarkDefinitionAuthoringCommand,
  canSubmitLocalBenchmarkDefinition,
  createLocalBenchmarkDefinitionAuthoringDraft,
  createLocalBenchmarkDefinitionAuthoringFormView,
  localBenchmarkDefinitionMetricOptions,
  presentLocalBenchmarkDefinitionAuthoring,
  type LocalBenchmarkDefinitionAuthoringDraft,
  type LocalBenchmarkDefinitionAuthoringDraftField,
  type LocalBenchmarkDefinitionAuthoringFormView,
  type LocalBenchmarkDefinitionAuthoringViewModel
} from "./local-benchmark-definition-authoring-presenter";

export type LocalBenchmarkDefinitionAuthoringScreenProps = Readonly<{
  view: LocalBenchmarkDefinitionAuthoringViewModel;
  form: LocalBenchmarkDefinitionAuthoringFormView;
  onChange: (field: LocalBenchmarkDefinitionAuthoringDraftField, value: string) => void;
  onSubmit: (event: React.FormEvent<HTMLFormElement>) => void;
}>;

export type LocalBenchmarkDefinitionAuthoringEditorProps = Readonly<{
  projectId: string;
  contextId: string;
  commitId: string;
  enabled?: boolean;
  onAuthored?: () => void;
}>;

export function LocalBenchmarkDefinitionAuthoringEditor({
  projectId,
  contextId,
  commitId,
  enabled = false,
  onAuthored
}: LocalBenchmarkDefinitionAuthoringEditorProps) {
  const target = Object.freeze({ projectId, contextId, commitId });
  const [draft, setDraft] = useState<LocalBenchmarkDefinitionAuthoringDraft>(
    createLocalBenchmarkDefinitionAuthoringDraft
  );
  const [resource, setResource] = useState<LocalBenchmarkDefinitionAuthoringResource>(() =>
    initialResource(target, enabled)
  );
  const isPending = resource.state === "loading";
  const validation = buildLocalBenchmarkDefinitionAuthoringCommand(
    target,
    withoutBearerToken(draft),
    () => "validation-only-id"
  );
  const canSubmit = canSubmitLocalBenchmarkDefinition({
    enabled,
    bearerToken: draft.bearerToken,
    hasCommit: commitId.trim().length > 0,
    isValid: validation.ok,
    isPending
  });

  function updateDraft(field: LocalBenchmarkDefinitionAuthoringDraftField, value: string) {
    setDraft((current) => Object.freeze({ ...current, [field]: value }));
    if (resource.state !== "loading") {
      setResource(initialResource(target, enabled));
    }
  }

  async function submit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!canSubmit) return;
    const command = buildLocalBenchmarkDefinitionAuthoringCommand(target, withoutBearerToken(draft));
    if (!command.ok) {
      setResource(createLocalBenchmarkDefinitionAuthoringResource({
        state: "error",
        target,
        message: command.message
      }));
      return;
    }

    setResource(createLocalBenchmarkDefinitionAuthoringResource({ state: "loading", target }));
    try {
      const result = await authorLocalBenchmarkDefinition(
        target,
        draft.bearerToken,
        createIdempotencyKey(),
        command.value
      );
      setResource(createLocalBenchmarkDefinitionAuthoringResource({ state: "available", target, result }));
      onAuthored?.();
    } catch (error) {
      setResource(createLocalBenchmarkDefinitionAuthoringResource({
        state: "error",
        target,
        message: presentAuthoringError(error)
      }));
    }
  }

  return (
    <LocalBenchmarkDefinitionAuthoringScreen
      form={createLocalBenchmarkDefinitionAuthoringFormView({ enabled, isPending, canSubmit, values: draft })}
      onChange={updateDraft}
      onSubmit={submit}
      view={presentLocalBenchmarkDefinitionAuthoring(resource, enabled)}
    />
  );
}

export function LocalBenchmarkDefinitionAuthoringScreen({
  form,
  onChange,
  onSubmit,
  view
}: LocalBenchmarkDefinitionAuthoringScreenProps) {
  const disabled = form.disabled;
  return (
    <section
      aria-labelledby="local-benchmark-definition-authoring-heading"
      className="operation-block local-benchmark-definition-authoring"
      id="local-benchmark-definition-authoring"
    >
      <div className="context-benchmark-evidence__heading">
        <div>
          <span className="eyebrow">Benchmark definitions / Benchmark 定义</span>
          <h3 id="local-benchmark-definition-authoring-heading">{view.title}</h3>
          <p>{view.description}</p>
        </div>
        <StatusPill tone="warning">Local development scope / 本地开发范围</StatusPill>
      </div>

      <DefinitionGrid
        aria-label="Exact immutable target / 精确不可变目标"
        columns={3}
        compact
        items={view.scope.map((fact) => ({ ...fact, value: <CodeChip>{fact.value}</CodeChip> }))}
        surface="raised"
        valueTone="info"
      />

      <CapabilityState
        ariaLabel={view.status.ariaLabel}
        description={view.status.description}
        detail={view.status.detail}
        label={view.status.label}
        state={view.status.state}
        stateLabel={view.status.stateLabel}
      />

      <form
        aria-busy={view.state === "loading" || undefined}
        aria-describedby="local-benchmark-definition-authoring-scope-note"
        aria-label="Create private benchmark dataset and suite / 创建私有 Benchmark 数据集与套件"
        className="context-lifecycle-editor__form"
        onSubmit={onSubmit}
      >
        <p id="local-benchmark-definition-authoring-scope-note">{form.scopeNotice}</p>
        <div className="context-lifecycle-editor__controls">
          <Input autoComplete="off" description="Memory only / 仅内存" disabled={disabled} label="Bearer token / 访问令牌" name="benchmark-authoring-bearer-token" onChange={(event) => onChange("bearerToken", event.target.value)} type="password" value={form.values.bearerToken} />
          <Input disabled={disabled} label="Branch / 分支" name="benchmark-authoring-branch" onChange={(event) => onChange("branchName", event.target.value)} value={form.values.branchName} />
          <Input disabled={disabled} label="Dataset name / 数据集名称" name="benchmark-authoring-dataset-name" onChange={(event) => onChange("datasetName", event.target.value)} value={form.values.datasetName} />
          <Input disabled={disabled} label="Case name / 用例名称" name="benchmark-authoring-case-name" onChange={(event) => onChange("caseName", event.target.value)} value={form.values.caseName} />
        </div>
        <Textarea description="Structured JSON only / 仅结构化 JSON" disabled={disabled} label="Case input JSON / 用例输入 JSON" name="benchmark-authoring-case-input" onChange={(event) => onChange("caseInputText", event.target.value)} rows={4} value={form.values.caseInputText} />
        <Textarea description="Leave blank for unspecified / 留空表示未指定" disabled={disabled} label="Expected output JSON / 预期输出 JSON" name="benchmark-authoring-expected-output" onChange={(event) => onChange("expectedOutputText", event.target.value)} rows={3} value={form.values.expectedOutputText} />
        <div className="context-lifecycle-editor__controls">
          <Input disabled={disabled} label="Suite name / 套件名称" name="benchmark-authoring-suite-name" onChange={(event) => onChange("suiteName", event.target.value)} value={form.values.suiteName} />
          <Select disabled={disabled} label="Metric / 指标" name="benchmark-authoring-metric" onChange={(event) => onChange("metric", event.target.value)} value={form.values.metric}>
            {localBenchmarkDefinitionMetricOptions.map((option) => <option key={option.value} value={option.value}>{option.label}</option>)}
          </Select>
          <Select disabled={disabled} label="Threshold direction / 阈值方向" name="benchmark-authoring-direction" onChange={(event) => onChange("direction", event.target.value)} value={form.values.direction}>
            <option value="minimum">Minimum / 最小值</option>
            <option value="maximum">Maximum / 最大值</option>
          </Select>
          <Input disabled={disabled} inputMode="decimal" label="Threshold / 阈值" name="benchmark-authoring-threshold" onChange={(event) => onChange("thresholdValue", event.target.value)} value={form.values.thresholdValue} />
        </div>
        <div className="context-lifecycle-editor__submit">
          <Button disabled={!form.canSubmit} icon={<DatabaseZap aria-hidden="true" />} type="submit">{form.submitLabel}</Button>
          <span>Server-owned authorization, sealing, and conflict checks / 授权、封存与冲突检查由服务端负责。</span>
        </div>
      </form>

      {view.result ? (
        <DefinitionGrid
          aria-label="Authored benchmark definition receipt / 已创作 Benchmark 定义回执"
          columns={3}
          compact
          items={[
            { id: "disposition", label: "Disposition / 处置", value: view.result.disposition },
            { id: "binding", label: "Binding ID / 绑定 ID", value: <CodeChip>{view.result.bindingId}</CodeChip> },
            { id: "branch", label: "Branch / 分支", value: <CodeChip>{view.result.branch}</CodeChip> },
            { id: "suite", label: "Suite ID / 套件 ID", value: <CodeChip>{view.result.suiteId}</CodeChip> },
            { id: "datasets", label: "Datasets / 数据集", value: view.result.datasetCount },
            { id: "captured", label: "Captured at / 捕获时间", value: view.result.capturedAt },
            { id: "message", label: "Server message / 服务端消息", value: view.result.message }
          ]}
          surface="raised"
        />
      ) : null}
    </section>
  );
}

function initialResource(
  target: LocalBenchmarkDefinitionAuthoringTarget,
  enabled: boolean
): LocalBenchmarkDefinitionAuthoringResource {
  if (!enabled || !target.commitId.trim()) {
    return createLocalBenchmarkDefinitionAuthoringResource({
      state: "unavailable",
      target,
      message: enabled
        ? "No exact materialized commit is available / 没有可用的精确已物化提交。"
        : "Private mutation is off by default / 私有变更默认关闭。"
    });
  }
  return createLocalBenchmarkDefinitionAuthoringResource({
    state: "empty",
    target,
    message: "Ready for one private dataset and suite / 已准备创作一个私有数据集与套件。"
  });
}

function withoutBearerToken(
  draft: LocalBenchmarkDefinitionAuthoringDraft
): Omit<LocalBenchmarkDefinitionAuthoringDraft, "bearerToken"> {
  const { bearerToken: _bearerToken, ...commandDraft } = draft;
  return commandDraft;
}

function presentAuthoringError(error: unknown): string {
  if (error instanceof LocalBenchmarkDefinitionAuthoringProxyError) {
    return benchmarkDefinitionAuthoringErrorMessage(
      error.status,
      error.body.error,
      error.retryAfterMs
    );
  }
  return "Unable to reach the private local authoring adapter / 无法连接私有本地创作适配器。";
}

function createIdempotencyKey(): string {
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
    return crypto.randomUUID();
  }
  throw new Error("Secure idempotency-key generation is unavailable");
}
