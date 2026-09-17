import React from "react";
import {
  LocalPluginCapabilityAvailabilityScreen
} from "./local-plugin-capability-availability-screen";
import {
  presentLocalPluginCapabilityAvailability
} from "./local-plugin-capability-availability-presenter";
import type { LocalPluginCapabilityAvailabilityResource } from "./local-plugin-capability-availability-data";

export type LocalPluginCapabilityAvailabilityInspectorProps = Readonly<{
  resource: LocalPluginCapabilityAvailabilityResource;
}>;

export function LocalPluginCapabilityAvailabilityInspector({
  resource
}: LocalPluginCapabilityAvailabilityInspectorProps) {
  return <LocalPluginCapabilityAvailabilityScreen view={presentLocalPluginCapabilityAvailability(resource)} />;
}
