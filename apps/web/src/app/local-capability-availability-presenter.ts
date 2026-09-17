import {
  adaptLocalCapabilityAvailabilityV1,
  toFrozenCapabilityStateDto,
  type LocalCapabilityAvailabilityDto,
  type LocalCapabilityAvailabilityV1
} from "./local-capability-availability-data";
import {
  presentCapabilityState,
  type CapabilityStateScreenModel
} from "./capability-state-presenter";

export function presentLocalCapabilityAvailability(
  dto: LocalCapabilityAvailabilityDto | LocalCapabilityAvailabilityV1
): CapabilityStateScreenModel {
  if ("operation_id" in dto) {
    return presentCapabilityState(toFrozenCapabilityStateDto(dto));
  }

  return presentCapabilityState(
    adaptLocalCapabilityAvailabilityV1({ kind: "ready", availability: dto })
  );
}
