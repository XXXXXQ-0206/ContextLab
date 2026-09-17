import { CapabilityState } from "@contextlab/ui";
import React from "react";
import type { FrozenCapabilityStateDto } from "./capability-state-data";
import { presentCapabilityState, type CapabilityStateScreenModel } from "./capability-state-presenter";

export type CapabilityStateAdapterProps = {
  dto: FrozenCapabilityStateDto;
};

export type CapabilityStateScreenProps = {
  view: CapabilityStateScreenModel;
};

export function CapabilityStateAdapter({ dto }: CapabilityStateAdapterProps) {
  return <CapabilityStateScreen view={presentCapabilityState(dto)} />;
}

export function CapabilityStateScreen({ view }: CapabilityStateScreenProps) {
  return (
    <CapabilityState
      ariaLabel={view.ariaLabel}
      description={view.description}
      detail={view.detail}
      id={view.id}
      label={view.capabilityLabel}
      state={view.state}
      stateLabel={view.stateLabel}
    />
  );
}
