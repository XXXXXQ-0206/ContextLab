import assert from "node:assert/strict";
import test from "node:test";
import {
  ContextLabLocalApiError,
  ContextLabLocalClient,
  type FetchLike
} from "./client";
import {
  LOCAL_COMPONENT_LIFECYCLE_COMMIT_RESPONSE_SCHEMA_V1,
  LOCAL_CONTEXT_LIFECYCLE_STATE_SCHEMA_V1,
  parseLocalBenchmarkDecision,
  parseLocalBenchmarkDecisionRunDetails,
  parseLocalComponentLifecycleCommitRequest,
  parseLocalComponentLifecycleCommitResponse,
  parseLocalContextLifecycleState
} from "./types";

function jsonResponse(body: unknown, init: ResponseInit = {}) {
  return new Response(JSON.stringify(body), {
    headers: {
      "content-type": "application/json"
    },
    ...init
  });
}

test("parses a versioned lifecycle state into a deeply frozen exact DTO", () => {
  const payload = lifecycleStatePayload();
  const parsed = parseLocalContextLifecycleState(payload);

  assert.equal(parsed.schema_version, LOCAL_CONTEXT_LIFECYCLE_STATE_SCHEMA_V1);
  assert.equal(Object.isFrozen(parsed), true);
  assert.equal(Object.isFrozen(parsed.metadata), true);
  assert.equal(Object.isFrozen(parsed.components), true);
  assert.equal(Object.isFrozen(parsed.components[0]), true);
  assert.equal(Object.isFrozen(parsed.components[0]?.metadata), true);
  assert.equal(Object.isFrozen(parsed.graph_snapshot), true);
  assert.equal(Object.isFrozen(parsed.graph_snapshot.graph), true);
  assert.equal(Object.isFrozen(parsed.graph_snapshot.graph.nodes), true);
  assert.equal(Object.isFrozen(parsed.graph_snapshot.graph.edges), true);
  assert.equal(parsed.graph_snapshot.project_id, "55555555-5555-4555-8555-555555555555");
  assert.throws(
    () => (parsed.components as Array<unknown>).push({}),
    TypeError
  );
  assert.deepEqual(parsed, payload);
});

test("fails closed for lifecycle response schema, UUID, graph, metadata, and component drift", () => {
  const cases: Array<[string, (payload: Record<string, any>) => void]> = [
    ["unknown state field", (payload) => { payload.unexpected = true; }],
    ["state schema", (payload) => { payload.schema_version = "other.v1"; }],
    ["context UUID", (payload) => { payload.context_id = "context/id"; }],
    ["metadata field", (payload) => { payload.metadata.extra = true; }],
    ["metadata label value", (payload) => { payload.metadata.labels.owner = 1; }],
    ["component order", (payload) => { payload.components.reverse(); }],
    ["component field", (payload) => { payload.components[0].extra = true; }],
    ["snapshot project UUID", (payload) => { payload.graph_snapshot.project_id = "project/id"; }],
    ["snapshot scope", (payload) => { payload.graph_snapshot.commit_id = "55555555-5555-4555-8555-555555555555"; }],
    ["snapshot unknown field", (payload) => { payload.graph_snapshot.extra = true; }],
    ["graph node field", (payload) => { payload.graph_snapshot.graph.nodes["component:33333333-3333-4333-8333-333333333333"].extra = true; }]
  ];

  for (const [label, mutate] of cases) {
    const payload = lifecycleStatePayload();
    mutate(payload);
    assert.throws(() => parseLocalContextLifecycleState(payload), TypeError, label);
  }
});

test("parses a versioned lifecycle commit response and binds its write snapshot", () => {
  const payload = lifecycleCommitResponsePayload();
  const parsed = parseLocalComponentLifecycleCommitResponse(payload);

  assert.equal(parsed.schema_version, LOCAL_COMPONENT_LIFECYCLE_COMMIT_RESPONSE_SCHEMA_V1);
  assert.equal(parsed.commit_id, parsed.snapshot.commit_id);
  assert.equal(Object.isFrozen(parsed), true);
  assert.equal(Object.isFrozen(parsed.snapshot.graph.nodes), true);
  assert.throws(
    () => parseLocalComponentLifecycleCommitResponse({
      ...payload,
      snapshot: {
        ...payload.snapshot,
        commit_id: "55555555-5555-4555-8555-555555555555"
      }
    }),
    TypeError
  );
});

test("rejects lifecycle client responses with read or write scope drift", async () => {
  const contextId = "11111111-1111-4111-8111-111111111111";
  const commitId = "22222222-2222-4222-8222-222222222222";
  const client = new ContextLabLocalClient({
    fetch: async (input) => {
      const url = String(input);
      if (url.includes("lifecycle-state")) {
        return jsonResponse({
          ...lifecycleStatePayload(),
          context_id: "33333333-3333-4333-8333-333333333333"
        });
      }
      return jsonResponse({
        ...lifecycleCommitResponsePayload(),
        snapshot: {
          ...lifecycleCommitResponsePayload().snapshot,
          context_id: "33333333-3333-4333-8333-333333333333"
        }
      });
    }
  });

  await assert.rejects(
    () => client.getContextLifecycleState(contextId, commitId, { bearerToken: "token" }),
    TypeError
  );
  await assert.rejects(
    () => client.commitComponentLifecycle(
      contextId,
      {
        branch_name: "main",
        expected_head_commit_id: commitId,
        message: "write",
        operation: { kind: "initialize" }
      },
      { bearerToken: "token", idempotencyKey: "key" }
    ),
    TypeError
  );
});

test("uses only protected local lifecycle endpoints with request-scoped credentials", async () => {
  const requests: Array<{ url: string; init: RequestInit | undefined }> = [];
  const fetchImpl: FetchLike = async (input, init) => {
    requests.push({ url: String(input), init });
    return jsonResponse(String(input).includes("lifecycle-state")
      ? lifecycleStatePayload()
      : lifecycleCommitResponsePayload());
  };
  const client = new ContextLabLocalClient({
    baseUrl: "http://127.0.0.1:3100/",
    fetch: fetchImpl
  });

  await client.getContextLifecycleState(
    "11111111-1111-4111-8111-111111111111",
    "22222222-2222-4222-8222-222222222222",
    {
    bearerToken: "first-token"
    }
  );
  await client.commitComponentLifecycle(
    "11111111-1111-4111-8111-111111111111",
    {
      branch_name: "main",
      expected_head_commit_id: "commit/id",
      message: "Create local prompt",
      operation: {
        kind: "create",
        component_kind: "prompt",
        name: "Instruction",
        metadata: null,
        content: "Use the committed Context."
      }
    },
    {
      bearerToken: "second-token",
      idempotencyKey: "local-request-001"
    }
  );

  assert.equal(
    requests[0]?.url,
    "http://127.0.0.1:3100/api/v1/local/contexts/11111111-1111-4111-8111-111111111111/commits/22222222-2222-4222-8222-222222222222/lifecycle-state"
  );
  assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer first-token");
  assert.equal(new Headers(requests[0]?.init?.headers).get("idempotency-key"), null);
  assert.equal(requests[0]?.init?.credentials, "omit");
  assert.equal(
    requests[1]?.url,
    "http://127.0.0.1:3100/api/v1/local/contexts/11111111-1111-4111-8111-111111111111/component-lifecycle-commits"
  );
  assert.equal(requests[1]?.init?.method, "POST");
  assert.equal(new Headers(requests[1]?.init?.headers).get("authorization"), "Bearer second-token");
  assert.equal(
    new Headers(requests[1]?.init?.headers).get("idempotency-key"),
    "local-request-001"
  );
  assert.equal(requests[1]?.init?.credentials, "omit");
  assert.equal(
    requests[1]?.init?.body,
    JSON.stringify({
      branch_name: "main",
      expected_head_commit_id: "commit/id",
      message: "Create local prompt",
      operation: {
        kind: "create",
        component_kind: "prompt",
        name: "Instruction",
        metadata: null,
        content: "Use the committed Context."
      }
    })
  );
  assert.equal("bearerToken" in (client as unknown as Record<string, unknown>), false);
});

test("parses and transports an exact descriptor lifecycle update through the protected local contract", async () => {
  const requests: Array<{ url: string; init: RequestInit | undefined }> = [];
  const command = {
    branch_name: "main",
    expected_head_commit_id: "commit/id",
    message: "Correct local descriptor metadata",
    operation: {
      kind: "update_descriptor",
      component_id: "component/id",
      name: "Localized policy",
      metadata: {
        locale: "zh-CN",
        audience: "internal"
      }
    }
  };
  const fetchImpl: FetchLike = async (input, init) => {
    requests.push({ url: String(input), init });
    return jsonResponse(lifecycleCommitResponsePayload());
  };
  const client = new ContextLabLocalClient({
    baseUrl: "http://127.0.0.1:3100/",
    fetch: fetchImpl
  });

  const parsedCommand = parseLocalComponentLifecycleCommitRequest(command);
  assert.deepEqual(parsedCommand, command);
  await client.commitComponentLifecycle("11111111-1111-4111-8111-111111111111", parsedCommand, {
    bearerToken: "descriptor-token",
    idempotencyKey: "descriptor-request-001"
  });

  assert.equal(
    requests[0]?.url,
    "http://127.0.0.1:3100/api/v1/local/contexts/11111111-1111-4111-8111-111111111111/component-lifecycle-commits"
  );
  assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer descriptor-token");
  assert.equal(
    new Headers(requests[0]?.init?.headers).get("idempotency-key"),
    "descriptor-request-001"
  );
  assert.equal(new Headers(requests[0]?.init?.headers).get("cookie"), null);
  assert.equal(requests[0]?.init?.credentials, "omit");
  assert.equal(requests[0]?.init?.body, JSON.stringify(command));
  assert.throws(
    () =>
      parseLocalComponentLifecycleCommitRequest({
        ...command,
        operation: { ...command.operation, content: "must not be accepted" }
      }),
    TypeError
  );
});

test("parses and transports an unborn Context initialization through the protected local contract", async () => {
  const requests: Array<{ url: string; init: RequestInit | undefined }> = [];
  const command = {
    branch_name: "main",
    expected_head_commit_id: null,
    message: "Initialize Context lifecycle",
    operation: { kind: "initialize" }
  };
  const fetchImpl: FetchLike = async (input, init) => {
    requests.push({ url: String(input), init });
    return jsonResponse(lifecycleCommitResponsePayload());
  };
  const client = new ContextLabLocalClient({
    baseUrl: "http://127.0.0.1:3100/",
    fetch: fetchImpl
  });

  const parsedCommand = parseLocalComponentLifecycleCommitRequest(command);
  assert.deepEqual(parsedCommand, command);
  await client.commitComponentLifecycle("11111111-1111-4111-8111-111111111111", parsedCommand, {
    bearerToken: "initialize-token",
    idempotencyKey: "initialize-request-001"
  });

  assert.equal(
    requests[0]?.url,
    "http://127.0.0.1:3100/api/v1/local/contexts/11111111-1111-4111-8111-111111111111/component-lifecycle-commits"
  );
  assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer initialize-token");
  assert.equal(new Headers(requests[0]?.init?.headers).get("idempotency-key"), "initialize-request-001");
  assert.equal(requests[0]?.init?.credentials, "omit");
  assert.equal(requests[0]?.init?.body, JSON.stringify(command));
  assert.throws(
    () =>
      parseLocalComponentLifecycleCommitRequest({
        ...command,
        expected_head_commit_id: "commit/id"
      }),
    TypeError
  );
});

test("parses and transports a typed Uses relationship through the protected local contract", async () => {
  const requests: Array<{ url: string; init: RequestInit | undefined }> = [];
  const command = {
    branch_name: "main",
    expected_head_commit_id: "commit/id",
    message: "Add Uses relationship",
    operation: {
      kind: "add_uses_relationship" as const,
      source_component_id: "source/component",
      target_component_id: "target/component"
    }
  };
  const fetchImpl: FetchLike = async (input, init) => {
    requests.push({ url: String(input), init });
    return jsonResponse(lifecycleCommitResponsePayload());
  };
  const client = new ContextLabLocalClient({
    baseUrl: "http://127.0.0.1:3100/",
    fetch: fetchImpl
  });

  const parsedCommand = parseLocalComponentLifecycleCommitRequest(command);
  assert.deepEqual(parsedCommand, command);
  await client.commitComponentLifecycle("11111111-1111-4111-8111-111111111111", parsedCommand, {
    bearerToken: "uses-token",
    idempotencyKey: "uses-request-001"
  });

  assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer uses-token");
  assert.equal(requests[0]?.init?.credentials, "omit");
  assert.equal(requests[0]?.init?.body, JSON.stringify(command));
  assert.throws(
    () =>
      parseLocalComponentLifecycleCommitRequest({
        ...command,
        expected_head_commit_id: null
      }),
    TypeError
  );
  assert.throws(
    () =>
      parseLocalComponentLifecycleCommitRequest({
        ...command,
        operation: { ...command.operation, component_id: "must not be accepted" }
      }),
    TypeError
  );
});

test("parses and serializes a strict Context metadata lifecycle operation", async () => {
  const requests: Array<{ url: string; init: RequestInit | undefined }> = [];
  const command = {
    branch_name: "main",
    expected_head_commit_id: "commit/id",
    message: "Update Context metadata",
    operation: {
      kind: "update_metadata" as const,
      metadata: {
        created_at: "2026-07-30T00:00:00Z",
        updated_at: "2026-07-30T01:00:00Z",
        labels: { owner: "luna" }
      }
    }
  };
  const fetchImpl: FetchLike = async (input, init) => {
    requests.push({ url: String(input), init });
    return jsonResponse(lifecycleCommitResponsePayload());
  };
  const client = new ContextLabLocalClient({
    baseUrl: "http://127.0.0.1:3100/",
    fetch: fetchImpl
  });

  await client.commitComponentLifecycle("11111111-1111-4111-8111-111111111111", command, {
    bearerToken: "metadata-token",
    idempotencyKey: "metadata-request-001"
  });

  assert.equal(
    requests[0]?.init?.body,
    JSON.stringify(command)
  );
  assert.throws(
    () =>
      parseLocalComponentLifecycleCommitRequest({
        ...command,
        operation: {
          ...command.operation,
          unexpected: true
        }
      }),
    TypeError
  );
  assert.throws(
    () =>
      parseLocalComponentLifecycleCommitRequest({
        ...command,
        operation: {
          ...command.operation,
          metadata: {
            ...command.operation.metadata,
            extra: true
          }
        }
      }),
    TypeError
  );
});

test("loads an exact redacted benchmark decision through the protected local contract", async () => {
  const requests: Array<{ url: string; init: RequestInit | undefined }> = [];
  const fetchImpl: FetchLike = async (input, init) => {
    requests.push({ url: String(input), init });
    return jsonResponse({
      project_id: "project/id",
      context_id: "context/id",
      commit_id: "commit/id",
      decision_id: "decision/id",
      suite_id: "suite/id",
      dataset_ids: ["dataset/id"],
      definition: {
        suite: {
          id: "suite/id",
          name: "Release gate",
          thresholds: []
        },
        datasets: [
          { id: "dataset/id", name: "Release dataset", case_count: 1 }
        ]
      },
      run_ids: ["run/id"],
      comparability: {
        evaluator_key: "quality",
        evaluator_version: "1.0.0",
        fingerprint: "f".repeat(64)
      },
      evidence_digest: "d".repeat(64),
      status: "passed",
      recorded_at: "2026-07-18T00:00:00Z",
      metrics: []
    });
  };
  const client = new ContextLabLocalClient({
    baseUrl: "http://127.0.0.1:3100/",
    fetch: fetchImpl
  });

  const decision = await client.getBenchmarkDecision(
    "project/id",
    "context/id",
    "commit/id",
    "decision/id",
    { bearerToken: "decision-token" }
  );

  assert.equal(decision.decision_id, "decision/id");
  assert.equal("cases" in decision, false);
  assert.equal("input" in decision, false);
  assert.equal("expected_output" in decision, false);
  assert.equal(
    requests[0]?.url,
    "http://127.0.0.1:3100/api/v1/local/projects/project%2Fid/contexts/context%2Fid/commits/commit%2Fid/benchmark-decisions/decision%2Fid"
  );
  assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer decision-token");
  assert.equal(new Headers(requests[0]?.init?.headers).get("cookie"), null);
  assert.equal(requests[0]?.init?.credentials, "omit");
});

test("lists exact sealed benchmark decisions through the protected local contract", async () => {
  const requests: Array<{ url: string; init: RequestInit | undefined }> = [];
  const payload = {
    schema_version: "contextlab.local-benchmark-decision-list.v1",
    project_id: "project/id",
    context_id: "context/id",
    commit_id: "commit/id",
    decisions: [
      {
        decision_id: "decision/new",
        suite: { id: "suite/id", name: "Release gate" },
        datasets: [{ id: "dataset/a", name: "Release dataset", case_count: 2 }],
        status: "passed",
        recorded_at: "2026-07-18T02:00:00Z",
        run_count: 3
      },
      {
        decision_id: "decision/old",
        suite: { id: "suite/id", name: "Release gate" },
        datasets: [],
        status: "insufficient_data",
        recorded_at: "2026-07-18T01:00:00Z",
        run_count: 0
      }
    ]
  };
  const fetchImpl: FetchLike = async (input, init) => {
    requests.push({ url: String(input), init });
    return jsonResponse(payload);
  };
  const client = new ContextLabLocalClient({
    baseUrl: "http://127.0.0.1:3100/",
    fetch: fetchImpl
  });

  const list = await client.listBenchmarkDecisions(
    "project/id",
    "context/id",
    "commit/id",
    { bearerToken: "decision-list-token" }
  );

  assert.deepEqual(list, payload);
  assert.equal(
    requests[0]?.url,
    "http://127.0.0.1:3100/api/v1/local/projects/project%2Fid/contexts/context%2Fid/commits/commit%2Fid/benchmark-decisions"
  );
  assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer decision-list-token");
  assert.equal(new Headers(requests[0]?.init?.headers).get("cookie"), null);
  assert.equal(requests[0]?.init?.credentials, "omit");
});

test("rejects out-of-scope, unstable, and raw benchmark decision lists", async (t) => {
  const response = {
    schema_version: "contextlab.local-benchmark-decision-list.v1",
    project_id: "project",
    context_id: "context",
    commit_id: "commit",
    decisions: [
      {
        decision_id: "decision/a",
        suite: { id: "suite", name: "Release gate" },
        datasets: [
          { id: "dataset/a", name: "First dataset", case_count: 1 },
          { id: "dataset/b", name: "Second dataset", case_count: 2 }
        ],
        status: "regressed",
        recorded_at: "2026-07-18T01:00:00Z",
        run_count: 1
      },
      {
        decision_id: "decision/b",
        suite: { id: "suite", name: "Release gate" },
        datasets: [],
        status: "passed",
        recorded_at: "2026-07-18T01:00:00Z",
        run_count: 1
      }
    ]
  };

  for (const [name, mutate] of [
    ["project scope", (value: typeof response) => { value.project_id = "other-project"; }],
    ["Context scope", (value: typeof response) => { value.context_id = "other-context"; }],
    ["commit scope", (value: typeof response) => { value.commit_id = "other-commit"; }],
    ["schema version", (value: typeof response) => { value.schema_version = "other.v1"; }],
    ["decision ordering", (value: typeof response) => { value.decisions.reverse(); }],
    ["duplicate decision identity", (value: typeof response) => {
      value.decisions[1]!.decision_id = "decision/a";
      value.decisions[1]!.recorded_at = "2026-07-18T00:00:00Z";
    }],
    ["dataset ordering", (value: typeof response) => { value.decisions[0]!.datasets.reverse(); }],
    ["raw cases", (value: typeof response) => {
      (value.decisions[0]!.datasets[0] as Record<string, unknown>).cases = [];
    }],
    ["raw measurements", (value: typeof response) => {
      (value.decisions[0] as Record<string, unknown>).measurements = [];
    }]
  ] as const) {
    await t.test(name, async () => {
      const payload = structuredClone(response);
      mutate(payload);
      const client = new ContextLabLocalClient({
        fetch: async () => jsonResponse(payload)
      });

      await assert.rejects(
        () => client.listBenchmarkDecisions("project", "context", "commit", {
          bearerToken: "local-token"
        }),
        TypeError
      );
    });
  }
});

test("loads exact redacted benchmark decision run details through the protected local contract", async () => {
  const requests: Array<{ url: string; init: RequestInit | undefined }> = [];
  const payload = {
    project_id: "project/id",
    context_id: "context/id",
    commit_id: "commit/id",
    decision_id: "decision/id",
    runs: [
      {
        run_id: "run/id",
        model_version: "model-a",
        temperature: 0.2,
        metrics: [
          { metric: "accuracy", value: 0.95 },
          { metric: "latency_ms", value: 700 }
        ],
        executed_at: "2026-07-18T00:00:00Z"
      }
    ]
  };
  const fetchImpl: FetchLike = async (input, init) => {
    requests.push({ url: String(input), init });
    return jsonResponse(payload);
  };
  const client = new ContextLabLocalClient({
    baseUrl: "http://127.0.0.1:3100/",
    fetch: fetchImpl
  });

  const details = await client.getBenchmarkDecisionRunDetails(
    "project/id",
    "context/id",
    "commit/id",
    "decision/id",
    { bearerToken: "run-details-token" }
  );

  assert.deepEqual(details, payload);
  assert.equal(
    requests[0]?.url,
    "http://127.0.0.1:3100/api/v1/local/projects/project%2Fid/contexts/context%2Fid/commits/commit%2Fid/benchmark-decisions/decision%2Fid/run-details"
  );
  assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer run-details-token");
  assert.equal(new Headers(requests[0]?.init?.headers).get("cookie"), null);
  assert.equal(requests[0]?.init?.credentials, "omit");
  assert.throws(
    () =>
      parseLocalBenchmarkDecisionRunDetails({
        ...payload,
        runs: [{ ...payload.runs[0], measurements: [] }]
      }),
    TypeError
  );
  assert.throws(
    () =>
      parseLocalBenchmarkDecisionRunDetails({
        ...payload,
        cases: []
      }),
    TypeError
  );
});

test("parses sealed benchmark definition metadata only when it matches the decision membership", async () => {
  const fetchImpl: FetchLike = async () =>
    jsonResponse({
      project_id: "project",
      context_id: "context",
      commit_id: "commit",
      decision_id: "decision",
      suite_id: "suite",
      dataset_ids: ["dataset-a", "dataset-b"],
      run_ids: ["run"],
      comparability: {
        evaluator_key: "quality",
        evaluator_version: "1.0.0",
        fingerprint: "f".repeat(64)
      },
      evidence_digest: "d".repeat(64),
      status: "passed",
      recorded_at: "2026-07-18T00:00:00Z",
      metrics: [],
      definition: {
        suite: {
          id: "suite",
          name: "Release gate",
          thresholds: [
            {
              metric: "accuracy",
              direction: "minimum",
              value: 0.9
            }
          ]
        },
        datasets: [
          { id: "dataset-a", name: "First dataset", case_count: 2 },
          { id: "dataset-b", name: "Second dataset", case_count: 1 }
        ]
      }
    });
  const client = new ContextLabLocalClient({ fetch: fetchImpl });

  const decision = await client.getBenchmarkDecision("project", "context", "commit", "decision", {
    bearerToken: "local-token"
  });

  assert.deepEqual(decision.definition, {
    suite: {
      id: "suite",
      name: "Release gate",
      thresholds: [
        {
          metric: "accuracy",
          direction: "minimum",
          value: 0.9
        }
      ]
    },
    datasets: [
      { id: "dataset-a", name: "First dataset", case_count: 2 },
      { id: "dataset-b", name: "Second dataset", case_count: 1 }
    ]
  });
});

test("rejects mismatched, duplicate, unsorted, or raw sealed benchmark definition metadata", async (t) => {
  const response = {
    project_id: "project",
    context_id: "context",
    commit_id: "commit",
    decision_id: "decision",
    suite_id: "suite",
    dataset_ids: ["dataset-a", "dataset-b"],
    run_ids: ["run"],
    comparability: {
      evaluator_key: "quality",
      evaluator_version: "1.0.0",
      fingerprint: "f".repeat(64)
    },
    evidence_digest: "d".repeat(64),
    status: "passed",
    recorded_at: "2026-07-18T00:00:00Z",
    metrics: [],
    definition: {
      suite: {
        id: "suite",
        name: "Release gate",
        thresholds: [
          {
            metric: "latency_ms",
            direction: "maximum",
            value: 800
          },
          {
            metric: "accuracy",
            direction: "minimum",
            value: 0.9
          }
        ]
      },
      datasets: [
        { id: "dataset-a", name: "First dataset", case_count: 2 },
        { id: "dataset-b", name: "Second dataset", case_count: 1 }
      ]
    }
  };

  for (const [name, mutate] of [
    ["suite identity", (value: typeof response) => { value.definition.suite.id = "other-suite"; }],
    ["blank suite name", (value: typeof response) => { value.definition.suite.name = "   "; }],
    ["dataset membership", (value: typeof response) => { value.definition.datasets[1]!.id = "other-dataset"; }],
    ["duplicate dataset identity", (value: typeof response) => { value.definition.datasets[1]!.id = "dataset-a"; }],
    ["blank dataset name", (value: typeof response) => { value.definition.datasets[0]!.name = ""; }],
    ["unstable root dataset ordering", (value: typeof response) => { value.dataset_ids.reverse(); }],
    ["unstable dataset ordering", (value: typeof response) => { value.definition.datasets.reverse(); }],
    ["unstable threshold ordering", (value: typeof response) => { value.definition.suite.thresholds.reverse(); }],
    ["raw run payload", (value: typeof response) => { (value.definition as Record<string, unknown>).runs = []; }],
    ["raw measurement payload", (value: typeof response) => { (value.definition.datasets[0] as Record<string, unknown>).measurements = []; }],
    ["raw output payload", (value: typeof response) => { (value.definition.suite as Record<string, unknown>).output = "private"; }],
    ["raw model output payload", (value: typeof response) => { (value.definition.suite as Record<string, unknown>).model_output = "private"; }]
  ] as const) {
    await t.test(name, async () => {
      const payload = structuredClone(response);
      mutate(payload);
      const client = new ContextLabLocalClient({ fetch: async () => jsonResponse(payload) });

      await assert.rejects(
        () => client.getBenchmarkDecision("project", "context", "commit", "decision", {
          bearerToken: "local-token"
        }),
        TypeError
      );
    });
  }
});

test("loads a redacted benchmark decision diff through the protected local contract", async () => {
  const requests: Array<{ url: string; init: RequestInit | undefined }> = [];
  const metric = {
    metric: "accuracy",
    threshold_direction: "minimum",
    threshold_value: 0.9,
    observed: 0.94,
    sample_count: 1,
    required_sample_count: 1,
    has_complete_coverage: true,
    outcome: "passed"
  };
  const fetchImpl: FetchLike = async (input, init) => {
    requests.push({ url: String(input), init });
    return jsonResponse({
      project_id: "project/id",
      context_id: "context/id",
      baseline: { commit_id: "baseline/commit", decision_id: "baseline/decision" },
      revised: { commit_id: "revised/commit", decision_id: "revised/decision" },
      status_change: { baseline: "passed", revised: "regressed" },
      metric_changes: [
        {
          kind: "modified",
          metric: "accuracy",
          baseline: metric,
          revised: { ...metric, observed: 0.84, outcome: "regressed" }
        }
      ]
    });
  };
  const client = new ContextLabLocalClient({
    baseUrl: "http://127.0.0.1:3100/",
    fetch: fetchImpl
  });

  const diff = await client.getBenchmarkDecisionDiff(
    "project/id",
    "context/id",
    { commit_id: "baseline/commit", decision_id: "baseline/decision" },
    { commit_id: "revised/commit", decision_id: "revised/decision" },
    { bearerToken: "diff-token" }
  );

  assert.equal(diff.status_change?.revised, "regressed");
  assert.equal(diff.metric_changes[0]?.kind, "modified");
  assert.equal(
    requests[0]?.url,
    "http://127.0.0.1:3100/api/v1/local/projects/project%2Fid/contexts/context%2Fid/benchmark-decision-diffs?baseline_commit_id=baseline%2Fcommit&baseline_decision_id=baseline%2Fdecision&revised_commit_id=revised%2Fcommit&revised_decision_id=revised%2Fdecision"
  );
  assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer diff-token");
  assert.equal(new Headers(requests[0]?.init?.headers).get("cookie"), null);
  assert.equal(requests[0]?.init?.credentials, "omit");
});

test("rejects raw benchmark payloads nested in benchmark decision diffs", async () => {
  const fetchImpl: FetchLike = async () =>
    jsonResponse({
      project_id: "project",
      context_id: "context",
      baseline: { commit_id: "baseline", decision_id: "baseline-decision" },
      revised: { commit_id: "revised", decision_id: "revised-decision" },
      status_change: null,
      metric_changes: [
        {
          kind: "added",
          metric: "accuracy",
          revised: {
            metric: "accuracy",
            threshold_direction: "minimum",
            threshold_value: 0.9,
            observed: 0.94,
            sample_count: 1,
            required_sample_count: 1,
            has_complete_coverage: true,
            outcome: "passed",
            input: "must not cross the client boundary"
          }
        }
      ]
    });
  const client = new ContextLabLocalClient({ fetch: fetchImpl });

  await assert.rejects(
    () =>
      client.getBenchmarkDecisionDiff(
        "project",
        "context",
        { commit_id: "baseline", decision_id: "baseline-decision" },
        { commit_id: "revised", decision_id: "revised-decision" },
        { bearerToken: "local-token" }
      ),
    TypeError
  );
});

test("rejects benchmark detail, run-detail, and diff responses outside the requested scopes", async (t) => {
  await t.test("decision detail", async () => {
    const client = new ContextLabLocalClient({
      fetch: async () => jsonResponse(decisionPayload({ project_id: "other-project" }))
    });
    await assert.rejects(
      () => client.getBenchmarkDecision("project", "context", "commit", "decision", {
        bearerToken: "local-token"
      }),
      /requested scope/
    );
  });

  await t.test("run details", async () => {
    const client = new ContextLabLocalClient({
      fetch: async () => jsonResponse(runDetailsPayload({ decision_id: "other-decision" }))
    });
    await assert.rejects(
      () => client.getBenchmarkDecisionRunDetails("project", "context", "commit", "decision", {
        bearerToken: "local-token"
      }),
      /requested scope/
    );
  });

  await t.test("decision diff", async () => {
    const client = new ContextLabLocalClient({
      fetch: async () => jsonResponse(diffPayload({ context_id: "other-context" }))
    });
    await assert.rejects(
      () => client.getBenchmarkDecisionDiff(
        "project",
        "context",
        { commit_id: "baseline", decision_id: "baseline-decision" },
        { commit_id: "revised", decision_id: "revised-decision" },
        { bearerToken: "local-token" }
      ),
      /requested scopes/
    );
  });
});

test("rejects impossible benchmark timestamps", async () => {
  const client = new ContextLabLocalClient({
    fetch: async () => jsonResponse(decisionPayload({ recorded_at: "2026-02-30T00:00:00Z" }))
  });
  await assert.rejects(
    () => client.getBenchmarkDecision("project", "context", "commit", "decision", {
      bearerToken: "local-token"
    }),
    /valid timestamp/
  );
  assert.throws(
    () => parseLocalBenchmarkDecisionRunDetails(
      runDetailsPayload({
        runs: [{
          ...runDetailsPayload().runs[0],
          executed_at: "2026-02-30T00:00:00Z"
        }]
      })
    ),
    /valid timestamp/
  );
});

test("rejects invalid or raw benchmark payloads before exposing them to local callers", async () => {
  const fetchImpl: FetchLike = async () =>
    jsonResponse({
      project_id: "project",
      context_id: "context",
      commit_id: "commit",
      decision_id: "decision",
      suite_id: "suite",
      dataset_ids: [],
      definition: {
        suite: {
          id: "suite",
          name: "Release gate",
          thresholds: []
        },
        datasets: []
      },
      run_ids: [],
      comparability: {
        evaluator_key: "quality",
        evaluator_version: "1.0.0",
        fingerprint: "f".repeat(64)
      },
      evidence_digest: "d".repeat(64),
      status: "accepted",
      recorded_at: "2026-07-18T00:00:00Z",
      metrics: [
        {
          metric: "accuracy",
          threshold_direction: "minimum",
          threshold_value: 0.9,
          observed: 0.94,
          sample_count: 1,
          required_sample_count: 1,
          has_complete_coverage: true,
          outcome: "passed",
          input: "must not cross the client boundary"
        }
      ]
    });
  const client = new ContextLabLocalClient({ fetch: fetchImpl });

  await assert.rejects(
    () =>
      client.getBenchmarkDecision("project", "context", "commit", "decision", {
        bearerToken: "local-token"
      }),
    TypeError
  );
});

test("preserves structured errors from the protected lifecycle service", async () => {
  const fetchImpl: FetchLike = async () =>
    jsonResponse(
      {
        error: "context_lifecycle_state_conflict",
        message: "component is unavailable at the materialized branch head"
      },
      { status: 409 }
    );
  const client = new ContextLabLocalClient({ fetch: fetchImpl });

  await assert.rejects(
    () =>
      client.getContextLifecycleState("context", "commit", {
        bearerToken: "local-token"
      }),
    (error: unknown) => {
      assert.ok(error instanceof ContextLabLocalApiError);
      assert.equal(error.status, 409);
      assert.equal(error.code, "context_lifecycle_state_conflict");
      return true;
    }
  );
});

test("preserves retry timing from rate-limited protected lifecycle requests", async () => {
  const fetchImpl: FetchLike = async () =>
    jsonResponse(
      {
        error: "rate_limit_exceeded",
        message: "too many lifecycle requests"
      },
      {
        status: 429,
        headers: { "retry-after": "30" }
      }
    );
  const client = new ContextLabLocalClient({ fetch: fetchImpl });

  await assert.rejects(
    () =>
      client.getContextLifecycleState("context", "commit", {
        bearerToken: "local-token"
      }),
    (error: unknown) => {
      assert.ok(error instanceof ContextLabLocalApiError);
      assert.equal(error.status, 429);
      assert.equal(error.retryAfterMs, 30_000);
      return true;
    }
  );
});

test("rejects empty request-scoped credentials before transport", async () => {
  let calls = 0;
  const fetchImpl: FetchLike = async () => {
    calls += 1;
    return jsonResponse({});
  };
  const client = new ContextLabLocalClient({ fetch: fetchImpl });

  await assert.rejects(
    () =>
      client.getContextLifecycleState("context", "commit", {
        bearerToken: ""
      }),
    RangeError
  );
  await assert.rejects(
    () =>
      client.commitComponentLifecycle(
        "context",
        {
          branch_name: "main",
          expected_head_commit_id: "commit",
          message: "Remove component",
          operation: {
            kind: "remove",
            component_id: "component"
          }
        },
        {
          bearerToken: "local-token",
          idempotencyKey: ""
        }
      ),
    RangeError
  );
  assert.equal(calls, 0);
});

function decisionPayload(overrides: Record<string, unknown> = {}) {
  return {
    project_id: "project",
    context_id: "context",
    commit_id: "commit",
    decision_id: "decision",
    suite_id: "suite",
    dataset_ids: [],
    definition: {
      suite: { id: "suite", name: "Release gate", thresholds: [] },
      datasets: []
    },
    run_ids: [],
    comparability: {
      evaluator_key: "quality",
      evaluator_version: "1.0.0",
      fingerprint: "f".repeat(64)
    },
    evidence_digest: "d".repeat(64),
    status: "passed",
    recorded_at: "2026-07-18T00:00:00Z",
    metrics: [],
    ...overrides
  };
}

function runDetailsPayload(overrides: Record<string, unknown> = {}) {
  return {
    project_id: "project",
    context_id: "context",
    commit_id: "commit",
    decision_id: "decision",
    runs: [{
      run_id: "run",
      model_version: "model",
      temperature: 0.2,
      metrics: [],
      executed_at: "2026-07-18T00:00:00Z"
    }],
    ...overrides
  };
}

function diffPayload(overrides: Record<string, unknown> = {}) {
  return {
    project_id: "project",
    context_id: "context",
    baseline: { commit_id: "baseline", decision_id: "baseline-decision" },
    revised: { commit_id: "revised", decision_id: "revised-decision" },
    status_change: null,
    metric_changes: [],
    ...overrides
  };
}

function lifecycleStatePayload() {
  return {
    schema_version: LOCAL_CONTEXT_LIFECYCLE_STATE_SCHEMA_V1,
    context_id: "11111111-1111-4111-8111-111111111111",
    commit_id: "22222222-2222-4222-8222-222222222222",
    metadata: {
      created_at: "2026-07-30T00:00:00Z",
      updated_at: "2026-07-30T01:00:00.123Z",
      labels: { owner: "luna" }
    },
    components: [
      lifecycleComponent("33333333-3333-4333-8333-333333333333", "Prompt"),
      lifecycleComponent("44444444-4444-4444-8444-444444444444", "Memory")
    ],
    graph_snapshot: lifecycleSnapshot()
  };
}

function lifecycleComponent(componentId: string, name: string) {
  return {
    component_id: componentId,
    component_kind: "prompt" as const,
    name,
    metadata: { locale: "en-US" },
    content: `content for ${name}`,
    content_hash: `sha256:${componentId}`,
    creation_commit_id: "22222222-2222-4222-8222-222222222222",
    content_commit_id: "22222222-2222-4222-8222-222222222222"
  };
}

function lifecycleSnapshot() {
  const contextNodeId = "context:11111111-1111-4111-8111-111111111111";
  const firstComponentId = "33333333-3333-4333-8333-333333333333";
  const secondComponentId = "44444444-4444-4444-8444-444444444444";
  const firstComponentNodeId = `component:${firstComponentId}`;
  const secondComponentNodeId = `component:${secondComponentId}`;
  return {
    project_id: "55555555-5555-4555-8555-555555555555",
    context_id: "11111111-1111-4111-8111-111111111111",
    commit_id: "22222222-2222-4222-8222-222222222222",
    graph: {
      nodes: {
        [contextNodeId]: { id: contextNodeId, kind: "context", label: "Context" },
        [firstComponentNodeId]: { id: firstComponentNodeId, kind: "prompt", label: "Prompt" },
        [secondComponentNodeId]: { id: secondComponentNodeId, kind: "prompt", label: "Memory" }
      },
      edges: [
        { source: contextNodeId, target: firstComponentNodeId, kind: "contains" },
        { source: contextNodeId, target: secondComponentNodeId, kind: "contains" }
      ]
    },
    captured_at: "2026-07-30T01:00:00Z",
    schema_version: 1
  };
}

function lifecycleCommitResponsePayload() {
  return {
    schema_version: LOCAL_COMPONENT_LIFECYCLE_COMMIT_RESPONSE_SCHEMA_V1,
    disposition: "created" as const,
    commit_id: "22222222-2222-4222-8222-222222222222",
    snapshot: lifecycleSnapshot()
  };
}
