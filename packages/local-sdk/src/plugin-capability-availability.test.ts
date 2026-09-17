import assert from "node:assert/strict";
import test from "node:test";
import {
  ContextLabLocalPluginCapabilityAvailabilityClient,
  LOCAL_PLUGIN_CAPABILITY_AVAILABILITY_SCHEMA_V1,
  parseLocalPluginCapabilityAvailability,
  type LocalPluginCapabilityAvailabilityResourceV1
} from "./index";
import type { FetchLike } from "./client";

const contextId = "11111111-1111-4111-8111-111111111111";

test("parses a deterministic redacted plugin capability projection", () => {
  const payload = resourcePayload();
  const parsed = parseLocalPluginCapabilityAvailability(payload);

  assert.deepEqual(parsed, payload);
  assert.equal(Object.isFrozen(parsed), true);
  assert.equal(Object.isFrozen(parsed.entries), true);
});

test("rejects unknown fields, inconsistent outcomes, and unstable ordering", () => {
  const payload = resourcePayload();
  assert.throws(() => parseLocalPluginCapabilityAvailability({ ...payload, raw_manifest: {} }), TypeError);
  assert.throws(() => parseLocalPluginCapabilityAvailability({
    ...payload,
    entries: [{ ...payload.entries[0], availability: "available", diagnostic_code: "factory_rejected" }]
  }), TypeError);
  assert.throws(() => parseLocalPluginCapabilityAvailability({
    ...payload,
    entries: [...payload.entries].reverse()
  }), TypeError);
});

test("reads the exact private scope with request-scoped bearer and no cookies", async () => {
  const requests: Array<{ input: string; init: RequestInit | undefined }> = [];
  const fetchImpl: FetchLike = async (input, init) => {
    requests.push({ input: String(input), init });
    return new Response(JSON.stringify(resourcePayload()), {
      headers: { "content-type": "application/json" }
    });
  };
  const client = new ContextLabLocalPluginCapabilityAvailabilityClient({
    baseUrl: "http://127.0.0.1:3100/",
    fetch: fetchImpl
  });
  const result = await client.getPluginCapabilityAvailability(contextId, { bearerToken: "request-token" });

  assert.equal(result.context_id, contextId);
  assert.equal(
    requests[0]?.input,
    `http://127.0.0.1:3100/api/v1/local/contexts/${contextId}/plugins/capabilities`
  );
  assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer request-token");
  assert.equal(new Headers(requests[0]?.init?.headers).get("cookie"), null);
  assert.equal(requests[0]?.init?.credentials, "omit");
  assert.equal(requests[0]?.init?.cache, "no-store");
});

function resourcePayload(): LocalPluginCapabilityAvailabilityResourceV1 {
  return {
    schema_version: LOCAL_PLUGIN_CAPABILITY_AVAILABILITY_SCHEMA_V1,
    context_id: contextId,
    entries: [
      {
        plugin_id: "contextlab.renderer",
        capability_id: "renderer.preview",
        capability_version: "1.0.0",
        availability: "available",
        compatibility: "compatible",
        diagnostic_code: null
      },
      {
        plugin_id: "contextlab.tooling",
        capability_id: "tool.mcp",
        capability_version: "1.1.0",
        availability: "unavailable",
        compatibility: "compatible",
        diagnostic_code: "activation_rejected"
      }
    ]
  };
}
