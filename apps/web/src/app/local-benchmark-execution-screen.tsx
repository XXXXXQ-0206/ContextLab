import { CapabilityState, CodeChip, DefinitionGrid, StatusPill } from "@contextlab/ui";
import React from "react";
import type { LocalBenchmarkExecutionViewModel } from "./local-benchmark-execution-presenter";

export type LocalBenchmarkExecutionScreenProps = Readonly<{
  view: LocalBenchmarkExecutionViewModel;
  controls?: React.ReactNode;
}>;

export function LocalBenchmarkExecutionScreen({ controls, view }: LocalBenchmarkExecutionScreenProps) {
  const receipt = view.receipt;
  return (
    <section
      aria-labelledby="local-benchmark-execution-heading"
      className="operation-block local-benchmark-execution"
      data-state={view.capabilityState}
      id="local-benchmark-execution"
    >
      <div className="context-benchmark-evidence__heading">
        <div>
          <span className="eyebrow">Benchmark operations / Benchmark 操作</span>
          <h3 id="local-benchmark-execution-heading">{view.title}</h3>
          <p>{view.description}</p>
        </div>
        <StatusPill tone={view.state === "created" || view.state === "replayed" ? "success" : "info"}>
          private local / 私有本地
        </StatusPill>
      </div>
      {controls}
      <div aria-live={view.capabilityState === "error" ? "assertive" : "polite"} role={view.capabilityState === "error" ? "alert" : "status"}>
        <CapabilityState
          ariaLabel={view.status.ariaLabel}
          description={view.status.description}
          detail={view.status.detail}
          label={view.status.label}
          state={view.capabilityState}
          stateLabel={view.status.stateLabel}
        />
      </div>
      <DefinitionGrid
        aria-label="Exact benchmark execution scope / 精确 Benchmark 执行范围"
        columns={3}
        compact
        items={view.scope.map((fact) => ({ ...fact, value: <CodeChip>{fact.value}</CodeChip> }))}
        surface="raised"
        valueTone="info"
      />
      {receipt ? (
        <DefinitionGrid
          aria-label="Redacted benchmark execution receipt / 脱敏 Benchmark 执行回执"
          columns={3}
          compact
          items={[
            { id: "disposition", label: "Disposition / 处置", value: receipt.disposition },
            { id: "projection", label: "Projection / 投影", value: receipt.projectionDisposition },
            { id: "binding", label: "Binding / 绑定", value: <CodeChip>{receipt.bindingId}</CodeChip> },
            { id: "decision", label: "Decision / 决策", value: <CodeChip>{receipt.decisionId}</CodeChip> },
            { id: "suite", label: "Suite / 套件", value: <CodeChip>{receipt.suiteId}</CodeChip> },
            { id: "datasets", label: "Datasets / 数据集", value: receipt.datasetIds },
            { id: "cohort", label: "Cohort / 队列", value: <CodeChip>{receipt.cohortId}</CodeChip> }
          ]}
          surface="raised"
        />
      ) : null}
    </section>
  );
}
