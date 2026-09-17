import assert from "node:assert/strict";
import test from "node:test";
import {
  ContextLabLocalClient,
  type FetchLike
} from "./client";
import {
  ContextLabLocalCommitGraphDiffError,
  parseLocalCommitGraphDiffResponseV1
} from "./commit-graph-diff";

function jsonResponse(body: unknown, init: ResponseInit = {}) {
  return new Response(JSON.stringify(body), {
    headers: {
      "content-type": "application/json"
    },
    ...init
  });
}

function graphDiffPayload() {
  return {
    context_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
    pair_witness: {
      schema_version: 1,
      project_id: "dddddddd-dddd-4ddd-8ddd-dddddddddddd",
      context_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
      baseline_commit_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
      revised_commit_id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc"
    },
    original: {
      commit_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
      captured_at: "2026-07-27T00:00:00Z",
      schema_version: 1
    },
    revised: {
      commit_id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
      captured_at: "2026-07-27T00:01:00Z",
      schema_version: 1
    },
    diff: {
      added_nodes: [
        { id: "knowledge:policy", kind: "knowledge", label: "Policy" }
      ],
      removed_nodes: [
        { id: "prompt:legacy", kind: "prompt", label: "Legacy policy" }
      ],
      modified_nodes: [
        {
          node_id: "context:agent",
          original_kind: "context",
          revised_kind: "workflow",
          original_label: "Support Agent",
          revised_label: "Support Agent v2"
        }
      ],
      added_edges: [
        { source: "context:agent", target: "knowledge:policy", kind: "retrieves" }
      ],
      removed_edges: [
        { source: "context:agent", target: "prompt:legacy", kind: "contains" }
      ]
    }
  };
}

test("loads an exact protected local commit graph diff with Bearer-only credentials", async () => {
  const requests: Array<{ url: string; init: RequestInit | undefined }> = [];
  const fetchImpl: FetchLike = async (input, init) => {
    requests.push({ url: String(input), init });
    return jsonResponse(graphDiffPayload());
  };
  const client = new ContextLabLocalClient({
    baseUrl: "http://127.0.0.1:3100/",
    fetch: fetchImpl
  });

  const result = await client.getCommitGraphDiff(
    "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
    "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
    "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
    { bearerToken: "local-read-token" }
  );

  assert.equal(result.diff.modified_nodes[0]?.revised_kind, "workflow");
  assert.equal(Object.isFrozen(result), true);
  assert.equal(Object.isFrozen(result.diff.added_nodes), true);
  assert.equal(
    requests[0]?.url,
    "http://127.0.0.1:3100/api/v1/local/contexts/aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa/graph-diff?original_commit_id=bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb&revised_commit_id=cccccccc-cccc-4ccc-8ccc-cccccccccccc"
  );
  assert.equal(requests[0]?.init?.method, "GET");
  assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer local-read-token");
  assert.equal(new Headers(requests[0]?.init?.headers).get("cookie"), null);
  assert.equal(requests[0]?.init?.credentials, "omit");
  assert.equal(requests[0]?.init?.cache, "no-store");
  assert.equal(requests[0]?.init?.body, undefined);
});

test("accepts RFC3339 UTC timestamps with a +00:00 offset", () => {
  const payload = graphDiffPayload();
  const result = parseLocalCommitGraphDiffResponseV1({
    ...payload,
    original: { ...payload.original, captured_at: "2026-07-27T00:00:00+00:00" },
    revised: { ...payload.revised, captured_at: "2026-07-27T00:01:00+00:00" }
  });

  assert.equal(result.original.captured_at, "2026-07-27T00:00:00+00:00");
  assert.equal(result.revised.captured_at, "2026-07-27T00:01:00+00:00");
});

test("rejects non-UTC offsets and malformed UTC timestamp shapes", () => {
  const payload = graphDiffPayload();
  for (const capturedAt of [
    "2026-07-27T00:00:00+08:00",
    "2026-07-27T00:00:00+0000",
    "2026-02-30T00:00:00+00:00"
  ]) {
    assert.throws(
      () =>
        parseLocalCommitGraphDiffResponseV1({
          ...payload,
          original: { ...payload.original, captured_at: capturedAt }
        }),
      TypeError
    );
  }
});

test("rejects malformed graph diff payloads and non-deterministic projections", () => {
  const payload = graphDiffPayload();

  assert.deepEqual(parseLocalCommitGraphDiffResponseV1(payload), payload);
  assert.throws(
    () =>
      parseLocalCommitGraphDiffResponseV1({
        ...payload,
        diff: {
          ...payload.diff,
          added_nodes: [
            { id: "workflow:z", kind: "workflow", label: "Z" },
            ...payload.diff.added_nodes
          ]
        }
      }),
    TypeError
  );
  assert.throws(
    () =>
      parseLocalCommitGraphDiffResponseV1({
        ...payload,
        unexpected: true
      }),
    TypeError
  );
  assert.throws(
    () =>
      parseLocalCommitGraphDiffResponseV1({
        ...payload,
        original: { ...payload.original, schema_version: 2 }
      }),
    TypeError
  );
  assert.throws(
    () =>
      parseLocalCommitGraphDiffResponseV1({
        ...payload,
        original: { ...payload.original, captured_at: "2026-02-30T00:00:00Z" }
      }),
    TypeError
  );
  assert.throws(
    () =>
      parseLocalCommitGraphDiffResponseV1({
        ...payload,
        diff: {
          ...payload.diff,
          added_nodes: [
            { id: "prompt:legacy", kind: "prompt", label: "Duplicate category" }
          ]
        }
      }),
    /node categories must be disjoint/
  );
});

test("rejects missing or drifting pair witness fields", () => {
  const payload = graphDiffPayload();
  const { pair_witness: _pairWitness, ...withoutPairWitness } = payload;

  assert.throws(
    () => parseLocalCommitGraphDiffResponseV1(withoutPairWitness),
    TypeError
  );
  assert.throws(
    () =>
      parseLocalCommitGraphDiffResponseV1({
        ...payload,
        pair_witness: { ...payload.pair_witness, baseline_commit: payload.pair_witness.baseline_commit_id }
      }),
    TypeError
  );
  assert.throws(
    () =>
      parseLocalCommitGraphDiffResponseV1({
        ...payload,
        pair_witness: { ...payload.pair_witness, schema_version: 2 }
      }),
    TypeError
  );
});

test("rejects mixed-scope and self-pair witnesses", () => {
  const payload = graphDiffPayload();

  assert.throws(
    () =>
      parseLocalCommitGraphDiffResponseV1({
        ...payload,
        pair_witness: {
          ...payload.pair_witness,
          context_id: "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee"
        }
      }),
    TypeError
  );
  assert.throws(
    () =>
      parseLocalCommitGraphDiffResponseV1({
        ...payload,
        pair_witness: {
          ...payload.pair_witness,
          baseline_commit_id: "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee"
        }
      }),
    TypeError
  );
  assert.throws(
    () =>
      parseLocalCommitGraphDiffResponseV1({
        ...payload,
        revised: payload.original,
        pair_witness: {
          ...payload.pair_witness,
          revised_commit_id: payload.original.commit_id
        }
      }),
    TypeError
  );
});

test("fails closed when the protected graph diff response scope is not exact", async () => {
  const client = new ContextLabLocalClient({
    fetch: async () =>
      jsonResponse({
        ...graphDiffPayload(),
        revised: { ...graphDiffPayload().revised, commit_id: "dddddddd-dddd-4ddd-8ddd-dddddddddddd" }
      })
  });

  await assert.rejects(
    () =>
      client.getCommitGraphDiff(
        "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
        "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
        "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
        {
        bearerToken: "local-read-token"
        }
      ),
    /scope/
  );
});

test("fails closed for same commit pairs and structured or malformed error responses", async (t) => {
  await t.test("same commit pair does not issue a read", async () => {
    let calls = 0;
    const client = new ContextLabLocalClient({
      fetch: async () => {
        calls += 1;
        return jsonResponse(graphDiffPayload());
      }
    });

    await assert.rejects(
      () =>
        client.getCommitGraphDiff(
          "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
          "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
          "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
          { bearerToken: "local-read-token" }
        ),
      RangeError
    );
    assert.equal(calls, 0);
  });

  await t.test("structured API error remains typed", async () => {
    const client = new ContextLabLocalClient({
      fetch: async () =>
        jsonResponse(
          { error: "context_read_forbidden", message: "Context read is forbidden." },
          { status: 403 }
        )
    });

    await assert.rejects(
      () =>
        client.getCommitGraphDiff(
          "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
          "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
          "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
          { bearerToken: "local-read-token" }
        ),
      (error: unknown) =>
        error instanceof ContextLabLocalCommitGraphDiffError
        && error.status === 403
        && error.code === "context_read_forbidden"
    );
  });

  for (const code of ["commit_graph_diff_unavailable", "storage_scope_unavailable"]) {
    await t.test(`preserves safe server error code ${code}`, async () => {
      const client = new ContextLabLocalClient({
        fetch: async () => jsonResponse({ error: code, message: "must be redacted" }, { status: 503 })
      });

      await assert.rejects(
        () =>
          client.getCommitGraphDiff(
            "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
            "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
            "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
            { bearerToken: "local-read-token" }
          ),
        (error: unknown) =>
          error instanceof ContextLabLocalCommitGraphDiffError
          && error.status === 503
          && error.code === code
          && !error.message.includes("must be redacted")
      );
    });
  }

  await t.test("malformed API error is redacted to the local error contract", async () => {
    const client = new ContextLabLocalClient({
      fetch: async () => new Response("upstream diagnostic must not escape", { status: 502 })
    });

    await assert.rejects(
      () =>
        client.getCommitGraphDiff(
          "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
          "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
          "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
          { bearerToken: "local-read-token" }
        ),
      (error: unknown) =>
        error instanceof ContextLabLocalCommitGraphDiffError
        && error.code === "contextlab_local_api_error"
        && !error.message.includes("upstream diagnostic")
    );
  });
});
