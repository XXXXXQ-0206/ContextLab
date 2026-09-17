import type { BilingualCapabilityText, FrozenCapabilityStateDto } from "./capability-state-data";
import { presentCapabilityState, type CapabilityStateScreenModel } from "./capability-state-presenter";
import {
  adaptLocalPluginCapabilityAvailabilityV1,
  type LocalPluginCapabilityAvailabilityResource
} from "./local-plugin-capability-availability-data";

export type LocalPluginCapabilityEntryViewModel = Readonly<{
  id: string;
  status: CapabilityStateScreenModel;
  facts: ReadonlyArray<Readonly<{ id: string; label: string; value: string }>>;
}>;

export type LocalPluginCapabilityAvailabilityViewModel = Readonly<{
  title: string;
  description: string;
  status: CapabilityStateScreenModel;
  scope: ReadonlyArray<Readonly<{ id: string; label: string; value: string }>>;
  entries: ReadonlyArray<LocalPluginCapabilityEntryViewModel>;
}>;

export function presentLocalPluginCapabilityAvailability(
  resource: LocalPluginCapabilityAvailabilityResource
): LocalPluginCapabilityAvailabilityViewModel {
  const dto = adaptLocalPluginCapabilityAvailabilityV1(resource);
  const status = presentCapabilityState(Object.freeze({
    id: dto.id,
    capability: dto.capability,
    state: dto.state,
    summary: stateSummary(dto.state),
    detail: dto.summary ? bilingual(
      "Server-owned capability metadata only; plugin source and execution payloads are excluded.",
      "仅展示服务端控制的能力元数据；不包含 plugin 源码与执行 payload。"
    ) : undefined
  }) satisfies FrozenCapabilityStateDto);

  const entries = dto.summary?.entries.map((entry) => {
    const capability = bilingual(entry.capability_id, entry.capability_id);
    const entryState = entry.availability === "available" ? "available" : "unavailable";
    return Object.freeze({
      id: `${entry.plugin_id}:${entry.capability_id}`,
      status: presentCapabilityState(Object.freeze({
        id: `${entry.plugin_id}:${entry.capability_id}`,
        capability,
        state: entryState,
        summary: bilingual(
          `${entry.plugin_id} / ${entry.capability_version}`,
          `${entry.plugin_id} / ${entry.capability_version}`
        ),
        detail: entry.diagnostic_code ? bilingual(
          `Diagnostic: ${entry.diagnostic_code}`,
          `诊断：${entry.diagnostic_code}`
        ) : bilingual("Registered and compatible.", "已注册且兼容。")
      }) satisfies FrozenCapabilityStateDto),
      facts: Object.freeze([
        { id: "plugin", label: "Plugin / 插件", value: entry.plugin_id },
        { id: "version", label: "Capability version / 能力版本", value: entry.capability_version },
        { id: "compatibility", label: "Compatibility / 兼容性", value: entry.compatibility },
        ...(entry.diagnostic_code ? [{ id: "diagnostic", label: "Diagnostic / 诊断", value: entry.diagnostic_code }] : [])
      ])
    });
  }) ?? [];

  return deepFreeze({
    title: "Plugin and MCP capabilities / Plugin 与 MCP 能力",
    description: "Private, read-only server-owned availability metadata. / 私有、只读、由服务端控制的可用性元数据。",
    status,
    scope: [{ id: "context", label: "Context / 上下文", value: dto.context_id }],
    entries
  });
}

function stateSummary(state: FrozenCapabilityStateDto["state"]): BilingualCapabilityText {
  switch (state) {
    case "loading": return bilingual("Reading capability availability.", "正在读取能力可用性。");
    case "error": return bilingual("The protected capability read failed.", "受保护能力读取失败。");
    case "empty": return bilingual("No registered plugin capabilities are available.", "暂无已注册的 plugin 能力。");
    case "unavailable": return bilingual("Plugin capabilities are unavailable.", "Plugin 能力不可用。");
    case "available": return bilingual("Registered plugin capabilities are available.", "已注册的 plugin 能力可用。");
  }
}

function bilingual(en: string, zh: string): BilingualCapabilityText {
  return Object.freeze({ en, zh });
}

function deepFreeze<T>(value: T): T {
  if (value !== null && typeof value === "object") {
    for (const child of Object.values(value)) deepFreeze(child);
    Object.freeze(value);
  }
  return value;
}
