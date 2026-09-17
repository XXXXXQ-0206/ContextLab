import assert from "node:assert/strict";
import test from "node:test";
import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import {
  createLocalPluginCapabilityAvailabilityResource,
  LOCAL_PLUGIN_CAPABILITY_AVAILABILITY_SCHEMA_V1,
  type LocalPluginCapabilityAvailabilityResource
} from "./local-plugin-capability-availability-data";
import { presentLocalPluginCapabilityAvailability } from "./local-plugin-capability-availability-presenter";
import { LocalPluginCapabilityAvailabilityInspector } from "./local-plugin-capability-availability-inspector";

const contextId = "11111111-1111-4111-8111-111111111111";
const target = { context_id: contextId, capability: { en: "Plugin capabilities", zh: "Plugin 能力" } };

test("plugin capability resource maps every shared capability state", () => {
  const states = ["loading", "error", "empty", "unavailable"] as const;
  for (const kind of states) {
    const view = presentLocalPluginCapabilityAvailability(createLocalPluginCapabilityAvailabilityResource({ kind, target }));
    assert.equal(view.status.state, kind);
    assert.equal(view.entries.length, 0);
  }

  const available = presentLocalPluginCapabilityAvailability(createLocalPluginCapabilityAvailabilityResource({
    kind: "ready",
    target,
    summary: {
      schema_version: LOCAL_PLUGIN_CAPABILITY_AVAILABILITY_SCHEMA_V1,
      context_id: contextId,
      entries: [{
        plugin_id: "contextlab.renderer",
        capability_id: "renderer.preview",
        capability_version: "1.0.0",
        availability: "available",
        compatibility: "compatible",
        diagnostic_code: null
      }]
    }
  }));
  assert.equal(available.status.state, "available");
  assert.equal(available.entries[0]?.status.state, "available");
});

test("plugin capability inspector uses shared accessible status semantics and redacts execution facts", () => {
  const resource: LocalPluginCapabilityAvailabilityResource = createLocalPluginCapabilityAvailabilityResource({
    kind: "ready",
    target,
    summary: {
      schema_version: LOCAL_PLUGIN_CAPABILITY_AVAILABILITY_SCHEMA_V1,
      context_id: contextId,
      entries: [{
        plugin_id: "contextlab.tooling",
        capability_id: "tool.mcp",
        capability_version: "1.1.0",
        availability: "unavailable",
        compatibility: "compatible",
        diagnostic_code: "activation_rejected"
      }]
    }
  });
  const markup = renderToStaticMarkup(<LocalPluginCapabilityAvailabilityInspector resource={resource} />);

  assert.match(markup, /Plugin and MCP capabilities \/ Plugin 与 MCP 能力/);
  assert.match(markup, /data-state="unavailable"/);
  assert.match(markup, /aria-live="polite"/);
  assert.match(markup, /tool\.mcp/);
  assert.match(markup, /activation_rejected/);
  assert.doesNotMatch(markup, /raw_manifest|execution_payload|credentials=/i);
});
