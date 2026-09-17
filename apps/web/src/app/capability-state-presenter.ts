import type { BilingualCapabilityText, CapabilityStateKind, FrozenCapabilityStateDto } from "./capability-state-data";

export type CapabilityStateScreenModel = Readonly<{
  id: string;
  state: CapabilityStateKind;
  ariaLabel: string;
  capabilityLabel: string;
  stateLabel: string;
  description: string;
  detail?: string;
}>;

type CapabilityStateCopy = Readonly<{
  label: BilingualCapabilityText;
  fallbackDescription: BilingualCapabilityText;
}>;

const stateCopy: Record<CapabilityStateKind, CapabilityStateCopy> = {
  loading: {
    label: { en: "Loading", zh: "加载中" },
    fallbackDescription: { en: "The capability status is being retrieved.", zh: "正在读取能力状态。" }
  },
  error: {
    label: { en: "Unable to load", zh: "加载失败" },
    fallbackDescription: { en: "The capability status could not be loaded.", zh: "无法加载能力状态。" }
  },
  empty: {
    label: { en: "No data", zh: "暂无数据" },
    fallbackDescription: { en: "No records are available yet.", zh: "暂时没有可用记录。" }
  },
  available: {
    label: { en: "Available", zh: "可用" },
    fallbackDescription: { en: "The capability is ready to use.", zh: "此能力已准备就绪。" }
  },
  unavailable: {
    label: { en: "Unavailable", zh: "不可用" },
    fallbackDescription: { en: "The capability is not available in this workspace.", zh: "此能力在当前工作区不可用。" }
  }
};

export function presentCapabilityState(dto: FrozenCapabilityStateDto): CapabilityStateScreenModel {
  const copy = stateCopy[dto.state];
  const capabilityLabel = formatBilingual(dto.capability);

  return {
    id: dto.id,
    state: dto.state,
    ariaLabel: `Capability status / 能力状态: ${capabilityLabel}`,
    capabilityLabel,
    stateLabel: formatBilingual(copy.label),
    description: formatBilingual(dto.summary ?? copy.fallbackDescription),
    detail: dto.detail ? formatBilingual(dto.detail) : undefined
  };
}

function formatBilingual(value: BilingualCapabilityText): string {
  return `${value.en} / ${value.zh}`;
}
