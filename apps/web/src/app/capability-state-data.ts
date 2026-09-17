export type CapabilityStateKind = "loading" | "error" | "empty" | "available" | "unavailable";

export type BilingualCapabilityText = Readonly<{
  en: string;
  zh: string;
}>;

export type FrozenCapabilityStateDto = Readonly<{
  id: string;
  capability: BilingualCapabilityText;
  state: CapabilityStateKind;
  summary?: BilingualCapabilityText;
  detail?: BilingualCapabilityText;
}>;
