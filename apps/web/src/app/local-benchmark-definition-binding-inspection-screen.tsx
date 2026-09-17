"use client";

import {
  Button,
  CapabilityState,
  CodeChip,
  DefinitionGrid,
  Input,
  Select,
  StackTable,
  StatusPill
} from "@contextlab/ui";
import { ListChecks } from "lucide-react";
import React, { useRef, useState } from "react";
import {
  createLocalBenchmarkDefinitionBindingInspectionResource,
  loadLocalBenchmarkDefinitionBindings,
  LocalBenchmarkDefinitionBindingInspectionProxyError,
  type LocalBenchmarkDefinitionBindingInspectionResource,
  type LocalBenchmarkDefinitionBindingInspectionTarget
} from "./local-benchmark-definition-binding-inspection-data";
import {
  bindingInspectionErrorMessage,
  createInitialBindingInspectionResource,
  presentLocalBenchmarkDefinitionBindingInspection,
  type LocalBenchmarkDefinitionBindingInspectionViewModel
} from "./local-benchmark-definition-binding-inspection-presenter";

export type LocalBenchmarkDefinitionBindingInspectionEditorProps = Readonly<{
  projectId: string;
  contextId: string;
  commitId: string;
  enabled: boolean;
}>;

export type LocalBenchmarkDefinitionBindingInspectionScreenProps = Readonly<{
  view: LocalBenchmarkDefinitionBindingInspectionViewModel;
  controls?: React.ReactNode;
  onSelect: (bindingId: string) => void;
}>;

export function LocalBenchmarkDefinitionBindingInspectionEditor({
  projectId,
  contextId,
  commitId,
  enabled
}: LocalBenchmarkDefinitionBindingInspectionEditorProps) {
  const target = Object.freeze({ projectId, contextId, commitId });
  const [bearerToken, setBearerToken] = useState("");
  const [selectedBindingId, setSelectedBindingId] = useState("");
  const [resource, setResource] = useState<LocalBenchmarkDefinitionBindingInspectionResource>(() =>
    createInitialBindingInspectionResource(target, enabled)
  );
  const requestId = useRef(0);
  const isLoading = resource.state === "loading";
  const canLoad = enabled && bearerToken.trim().length > 0 && commitId.trim().length > 0 && !isLoading;

  async function loadBindings() {
    if (!canLoad) return;
    const currentRequestId = ++requestId.current;
    setResource(createLocalBenchmarkDefinitionBindingInspectionResource({ state: "loading", target }));
    try {
      const result = await loadLocalBenchmarkDefinitionBindings(target, bearerToken);
      if (currentRequestId === requestId.current) {
        setResource(createLocalBenchmarkDefinitionBindingInspectionResource({ state: "available", target, result }));
      }
    } catch (error) {
      if (currentRequestId === requestId.current) {
        setResource(createLocalBenchmarkDefinitionBindingInspectionResource({
          state: error instanceof LocalBenchmarkDefinitionBindingInspectionProxyError && error.status === 404
            ? "empty"
            : "error",
          target,
          message: error instanceof LocalBenchmarkDefinitionBindingInspectionProxyError
            ? bindingInspectionErrorMessage(error.status, error.body.error)
            : "Unable to reach the private binding inspection adapter / 无法连接私有绑定检查适配器。"
        }));
      }
    }
  }

  const view = presentLocalBenchmarkDefinitionBindingInspection(resource, enabled, selectedBindingId);
  const controls = (
    <form
      aria-busy={isLoading || undefined}
      aria-label="Exact benchmark binding inspection scope / 精确 Benchmark 绑定检查范围"
      className="local-benchmark-definition-binding-inspection__controls"
      onSubmit={(event) => {
        event.preventDefault();
        void loadBindings();
      }}
    >
      <Input
        autoComplete="off"
        description="Memory only / 仅内存"
        disabled={!enabled || isLoading}
        label="Bearer token / 访问令牌"
        name="benchmark-binding-inspection-bearer-token"
        onChange={(event) => setBearerToken(event.target.value)}
        type="password"
        value={bearerToken}
      />
      <Button disabled={!canLoad} icon={<ListChecks aria-hidden="true" />} type="submit">
        {isLoading ? "Inspecting bindings... / 正在检查绑定..." : "Inspect exact bindings / 检查精确绑定"}
      </Button>
    </form>
  );

  return (
    <LocalBenchmarkDefinitionBindingInspectionScreen
      controls={controls}
      onSelect={setSelectedBindingId}
      view={view}
    />
  );
}

export function LocalBenchmarkDefinitionBindingInspectionScreen({
  controls,
  onSelect,
  view
}: LocalBenchmarkDefinitionBindingInspectionScreenProps) {
  return (
    <section
      aria-labelledby="local-benchmark-definition-binding-inspection-heading"
      className="operation-block local-benchmark-definition-binding-inspection"
      id="local-benchmark-definition-binding-inspection"
    >
      <div className="context-benchmark-evidence__heading">
        <div>
          <span className="eyebrow">Benchmark definitions / Benchmark 定义</span>
          <h3 id="local-benchmark-definition-binding-inspection-heading">{view.title}</h3>
          <p>{view.description}</p>
        </div>
        <StatusPill tone="info">protected local read / 受保护本地读取</StatusPill>
      </div>
      {controls}
      <CapabilityState
        ariaLabel={view.status.ariaLabel}
        description={view.status.description}
        detail={view.status.detail}
        label={view.status.label}
        state={view.status.state}
        stateLabel={view.status.stateLabel}
      />
      <DefinitionGrid
        aria-label="Exact binding inspection scope / 精确绑定检查范围"
        columns={3}
        compact
        items={view.scope.map((fact) => ({ ...fact, value: <CodeChip>{fact.value}</CodeChip> }))}
        surface="raised"
        valueTone="info"
      />
      {view.bindings.length > 0 ? (
        <>
          <Select
            label="Selected binding / 选中绑定"
            name="benchmark-binding-inspection-selected-binding"
            onChange={(event) => onSelect(event.target.value)}
            value={view.selectedBindingId}
          >
            {view.bindings.map((binding) => (
              <option key={binding.id} value={binding.id}>{binding.label}</option>
            ))}
          </Select>
          <StackTable
            aria-label="Redacted benchmark definition bindings / 脱敏 Benchmark 定义绑定"
            headers={["Binding ID / 绑定 ID", "Suite / 套件", "Branch / 分支", "Datasets / 数据集", "Captured / 捕获时间"]}
            rows={view.bindings.map((binding) => ({
              id: binding.id,
              cells: [
                <CodeChip key="id">{binding.id}</CodeChip>,
                binding.suite,
                <CodeChip key="branch">{binding.branch}</CodeChip>,
                binding.datasetCount,
                binding.capturedAt
              ]
            }))}
          />
        </>
      ) : null}
    </section>
  );
}
