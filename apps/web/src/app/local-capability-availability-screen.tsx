import React from "react";
import { CapabilityStateScreen } from "./capability-state-screen";
import type {
  LocalCapabilityAvailabilityDto,
  LocalCapabilityAvailabilityV1
} from "./local-capability-availability-data";
import { presentLocalCapabilityAvailability } from "./local-capability-availability-presenter";

export type LocalCapabilityAvailabilityAdapterProps = {
  dto: LocalCapabilityAvailabilityDto | LocalCapabilityAvailabilityV1;
};

export function LocalCapabilityAvailabilityAdapter({ dto }: LocalCapabilityAvailabilityAdapterProps) {
  return <CapabilityStateScreen view={presentLocalCapabilityAvailability(dto)} />;
}
