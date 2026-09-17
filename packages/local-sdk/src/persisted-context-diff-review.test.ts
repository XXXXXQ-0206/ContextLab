import assert from "node:assert/strict";
import test from "node:test";
import {
  ContextLabLocalClient,
  type FetchLike
} from "./client";
import {
  ContextLabLocalPersistedContextDiffReviewError,
  parsePersistedContextDiffReviewV1
} from "./persisted-context-diff-review";

const PROJECT_ID = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
const CONTEXT_ID = "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb";
const SOURCE_COMMIT_ID = "cccccccc-cccc-4ccc-8ccc-cccccccccccc";
const TARGET_COMMIT_ID = "dddddddd-dddd-4ddd-8ddd-dddddddddddd";

function jsonResponse(body: unknown, init: ResponseInit = {}): Response {
  return new Response(JSON.stringify(body), {
    headers: { "content-type": "application/json" },
    ...init
  });
}

function reviewPayload() {
  return {
    schema_version: "v1",
    source_scope: {
      project_id: PROJECT_ID,
      context_id: CONTEXT_ID,
      commit_id: SOURCE_COMMIT_ID
    },
    target_scope: {
      project_id: PROJECT_ID,
      context_id: CONTEXT_ID,
      commit_id: TARGET_COMMIT_ID
    },
    diff: {
      contract_version: "v1",
      semantic: {
        graph_diff: {
          added_nodes: [{ id: "prompt:z", kind: "prompt", label: "Added" }],
          removed_nodes: [],
          modified_nodes: [],
          added_edges: [],
          removed_edges: []
        },
        document_changes: [
          {
            kind: "modified",
            document_id: "prompt:system",
            text_diff: {
              lines: [
                { kind: "removed", text: "old" },
                { kind: "added", text: "new" }
              ]
            }
          }
        ]
      },
      behavior: {
        case_changes: [
          {
            kind: "modified",
            original: {
              case_id: "case:answer",
              input_fingerprint: "sha256:input",
              outcome: { kind: "succeeded", output: "old" }
            },
            revised: {
              case_id: "case:answer",
              input_fingerprint: "sha256:input",
              outcome: { kind: "failed", error_code: "timeout" }
            }
          }
        ]
      },
      evaluation: {
        comparability_fingerprint: "suite:context:v1",
        metric_changes: [
          {
            kind: "modified",
            original: { metric_id: "accuracy", value: 0.8, sample_count: 10 },
            revised: { metric_id: "accuracy", value: 0.9, sample_count: 10 }
          }
        ]
      }
    }
  };
}

test("parses and freezes the complete Rust persisted diff review projection", () => {
  const result = parsePersistedContextDiffReviewV1(reviewPayload());

  assert.equal(result.schema_version, "v1");
  assert.equal(result.source_scope.commit_id, SOURCE_COMMIT_ID);
  assert.equal(result.diff.semantic.document_changes[0]?.kind, "modified");
  assert.equal(result.diff.behavior.case_changes[0]?.kind, "modified");
  assert.equal(result.diff.semantic.document_changes.length > 0, true);
  assert.equal(result.diff.behavior.case_changes.length > 0, true);
  assert.equal(result.diff.evaluation.metric_changes.length > 0, true);
  assert.equal(Object.isFrozen(result), true);
  assert.equal(Object.isFrozen(result.diff.semantic.graph_diff), true);
  assert.equal(Object.isFrozen(result.diff.behavior.case_changes), true);
  assert.equal(result.diff.semantic.metadata_change, undefined);
});

test("accepts added and removed document, behavior, and evaluation variants with exact shapes", () => {
  const payload = reviewPayload();
  const document = { id: "prompt:matrix", content: "matrix content" };
  const observation = {
    case_id: "case:matrix",
    input_fingerprint: "sha256:matrix-input",
    outcome: { kind: "succeeded", output: "matrix output" }
  };
  const metric = { metric_id: "matrix_accuracy", value: 0.75, sample_count: 8 };

  const cases = [
    {
      name: "semantic document added",
      section: "semantic" as const,
      change: { kind: "added", document },
      expected: { kind: "added", document },
      invalid: { kind: "added", document, original: document }
    },
    {
      name: "semantic document removed",
      section: "semantic" as const,
      change: { kind: "removed", document },
      expected: { kind: "removed", document },
      invalid: { kind: "removed", document, revised: document }
    },
    {
      name: "behavior case added",
      section: "behavior" as const,
      change: { kind: "added", revised: observation },
      expected: { kind: "added", revised: observation },
      invalid: { kind: "added", revised: observation, original: observation }
    },
    {
      name: "behavior case removed",
      section: "behavior" as const,
      change: { kind: "removed", original: observation },
      expected: { kind: "removed", original: observation },
      invalid: { kind: "removed", original: observation, revised: observation }
    },
    {
      name: "evaluation metric added",
      section: "evaluation" as const,
      change: { kind: "added", revised: metric },
      expected: { kind: "added", revised: metric },
      invalid: { kind: "added", revised: metric, original: metric }
    },
    {
      name: "evaluation metric removed",
      section: "evaluation" as const,
      change: { kind: "removed", original: metric },
      expected: { kind: "removed", original: metric },
      invalid: { kind: "removed", original: metric, revised: metric }
    }
  ] as const;

  for (const entry of cases) {
    const result = parsePersistedContextDiffReviewV1({
      ...payload,
      diff: {
        ...payload.diff,
        semantic: {
          ...payload.diff.semantic,
          document_changes: entry.section === "semantic"
            ? [entry.change]
            : payload.diff.semantic.document_changes
        },
        behavior: {
          ...payload.diff.behavior,
          case_changes: entry.section === "behavior"
            ? [entry.change]
            : payload.diff.behavior.case_changes
        },
        evaluation: {
          ...payload.diff.evaluation,
          metric_changes: entry.section === "evaluation"
            ? [entry.change]
            : payload.diff.evaluation.metric_changes
        }
      }
    });
    const parsed = entry.section === "semantic"
      ? result.diff.semantic.document_changes[0]
      : entry.section === "behavior"
        ? result.diff.behavior.case_changes[0]
        : result.diff.evaluation.metric_changes[0];
    assert.deepEqual(parsed, entry.expected, entry.name);

    assert.throws(
      () => parsePersistedContextDiffReviewV1({
        ...payload,
        diff: {
          ...payload.diff,
          semantic: {
            ...payload.diff.semantic,
            document_changes: entry.section === "semantic"
              ? [entry.invalid]
              : payload.diff.semantic.document_changes
          },
          behavior: {
            ...payload.diff.behavior,
            case_changes: entry.section === "behavior"
              ? [entry.invalid]
              : payload.diff.behavior.case_changes
          },
          evaluation: {
            ...payload.diff.evaluation,
            metric_changes: entry.section === "evaluation"
              ? [entry.invalid]
              : payload.diff.evaluation.metric_changes
          }
        }
      }),
      TypeError,
      entry.name
    );
  }
});

test("parses and freezes a valid modified Context metadata transition", () => {
  const payload = reviewPayload();
  const result = parsePersistedContextDiffReviewV1({
    ...payload,
    diff: {
      ...payload.diff,
      semantic: {
        ...payload.diff.semantic,
        metadata_change: {
          kind: "modified",
          original: {
            created_at: "2026-07-30T00:00:00Z",
            updated_at: "2026-07-30T00:00:00Z",
            labels: { owner: "support" }
          },
          revised: {
            created_at: "2026-07-30T00:00:00Z",
            updated_at: "2026-07-30T01:00:00Z",
            labels: { owner: "platform" }
          }
        }
      }
    }
  });

  assert.deepEqual(result.diff.semantic.metadata_change, {
    kind: "modified",
    original: {
      created_at: "2026-07-30T00:00:00Z",
      updated_at: "2026-07-30T00:00:00Z",
      labels: { owner: "support" }
    },
    revised: {
      created_at: "2026-07-30T00:00:00Z",
      updated_at: "2026-07-30T01:00:00Z",
      labels: { owner: "platform" }
    }
  });
  assert.equal(Object.isFrozen(result.diff.semantic.metadata_change), true);
  assert.equal(Object.isFrozen(result.diff.semantic.metadata_change?.original.labels), true);
});

test("parses and freezes nullable-prior added and removed metadata transitions", () => {
  const payload = reviewPayload();
  const addedMetadata = {
    created_at: "2026-07-30T00:00:00Z",
    updated_at: "2026-07-30T01:00:00Z",
    labels: { owner: "platform" }
  };
  const removedMetadata = {
    created_at: "2026-07-29T00:00:00Z",
    updated_at: "2026-07-29T01:00:00Z",
    labels: { owner: "support" }
  };

  const added = parsePersistedContextDiffReviewV1({
    ...payload,
    diff: {
      ...payload.diff,
      semantic: {
        ...payload.diff.semantic,
        metadata_change: { kind: "added", revised: addedMetadata }
      }
    }
  }).diff.semantic.metadata_change;
  const removed = parsePersistedContextDiffReviewV1({
    ...payload,
    diff: {
      ...payload.diff,
      semantic: {
        ...payload.diff.semantic,
        metadata_change: { kind: "removed", original: removedMetadata }
      }
    }
  }).diff.semantic.metadata_change;

  assert.deepEqual(added, { kind: "added", revised: addedMetadata });
  assert.deepEqual(removed, { kind: "removed", original: removedMetadata });
  assert.equal(Object.isFrozen(added), true);
  assert.equal(Object.isFrozen(removed), true);
  assert.equal(Object.isFrozen(added && "revised" in added ? added.revised.labels : null), true);
  assert.equal(Object.isFrozen(removed && "original" in removed ? removed.original.labels : null), true);
  assert.throws(
    () => parsePersistedContextDiffReviewV1({
      ...payload,
      diff: {
        ...payload.diff,
        semantic: {
          ...payload.diff.semantic,
          metadata_change: {
            kind: "added",
            revised: addedMetadata,
            original: removedMetadata
          }
        }
      }
    }),
    TypeError
  );
  assert.throws(
    () => parsePersistedContextDiffReviewV1({
      ...payload,
      diff: {
        ...payload.diff,
        semantic: {
          ...payload.diff.semantic,
          metadata_change: {
            kind: "removed",
            original: removedMetadata,
            revised: addedMetadata
          }
        }
      }
    }),
    TypeError
  );
});

test("fails closed for malformed or unknown metadata transition fields", () => {
  const payload = reviewPayload();
  const metadataChange = {
    kind: "modified",
    original: {
      created_at: "2026-07-30T00:00:00Z",
      updated_at: "2026-07-30T00:00:00Z",
      labels: { owner: "support" }
    },
    revised: {
      created_at: "2026-07-30T00:00:00Z",
      updated_at: "2026-07-30T01:00:00Z",
      labels: { owner: "platform" }
    }
  };

  assert.throws(
    () => parsePersistedContextDiffReviewV1({
      ...payload,
      diff: {
        ...payload.diff,
        semantic: { ...payload.diff.semantic, metadata_change: { ...metadataChange, unexpected: true } }
      }
    }),
    TypeError
  );
  assert.throws(
    () => parsePersistedContextDiffReviewV1({
      ...payload,
      diff: {
        ...payload.diff,
        semantic: {
          ...payload.diff.semantic,
          metadata_change: {
            ...metadataChange,
            revised: { ...metadataChange.revised, unexpected: true }
          }
        }
      }
    }),
    TypeError
  );
  assert.throws(
    () => parsePersistedContextDiffReviewV1({
      ...payload,
      diff: {
        ...payload.diff,
        semantic: {
          ...payload.diff.semantic,
          metadata_change: { ...metadataChange, kind: "added" }
        }
      }
    }),
    TypeError
  );
  assert.throws(
    () => parsePersistedContextDiffReviewV1({
      ...payload,
      diff: {
        ...payload.diff,
        semantic: {
          ...payload.diff.semantic,
          metadata_change: {
            ...metadataChange,
            revised: metadataChange.original
          }
        }
      }
    }),
    TypeError
  );
});

test("fails closed for unknown fields, scope drift, and unstable ordering", () => {
  const payload = reviewPayload();

  assert.throws(
    () => parsePersistedContextDiffReviewV1({ ...payload, unexpected: true }),
    TypeError
  );
  assert.throws(
    () => parsePersistedContextDiffReviewV1({
      ...payload,
      target_scope: { ...payload.target_scope, project_id: "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee" }
    }),
    TypeError
  );
  assert.throws(
    () => parsePersistedContextDiffReviewV1({
      ...payload,
      diff: {
        ...payload.diff,
        semantic: {
          ...payload.diff.semantic,
          document_changes: [
            payload.diff.semantic.document_changes[0],
            { kind: "added", document: { id: "prompt:a", content: "a" } }
          ]
        }
      }
    }),
    TypeError
  );
  assert.throws(
    () => parsePersistedContextDiffReviewV1({
      ...payload,
      diff: {
        ...payload.diff,
        evaluation: {
          ...payload.diff.evaluation,
          metric_changes: [{
            kind: "modified",
            original: { metric_id: "accuracy", value: Number.NaN, sample_count: 1 },
            revised: { metric_id: "accuracy", value: 0.9, sample_count: 1 }
          }]
        }
      }
    }),
    TypeError
  );
  assert.throws(
    () => parsePersistedContextDiffReviewV1({
      ...payload,
      diff: {
        ...payload.diff,
        semantic: {
          ...payload.diff.semantic,
          document_changes: [{
            kind: "unknown",
            document_id: "prompt:system",
            text_diff: payload.diff.semantic.document_changes[0].text_diff
          }]
        }
      }
    }),
    TypeError
  );
});

test("reads the exact private route with request-scoped bearer and no cookies", async () => {
  const requests: Array<{ url: string; init: RequestInit | undefined }> = [];
  const fetchImpl: FetchLike = async (input, init) => {
    requests.push({ url: String(input), init });
    return jsonResponse(reviewPayload());
  };
  const client = new ContextLabLocalClient({
    baseUrl: "http://contextlab.test/",
    fetch: fetchImpl
  });

  const result = await client.getPersistedContextDiffReview(
    PROJECT_ID,
    CONTEXT_ID,
    SOURCE_COMMIT_ID,
    TARGET_COMMIT_ID,
    { bearerToken: "  local-read-token  " }
  );

  assert.equal(result.diff.evaluation.metric_changes[0]?.kind, "modified");
  assert.equal(
    requests[0]?.url,
    `http://contextlab.test/api/v1/local/projects/${PROJECT_ID}/contexts/${CONTEXT_ID}/diff-review?source_commit_id=${SOURCE_COMMIT_ID}&target_commit_id=${TARGET_COMMIT_ID}`
  );
  assert.equal(requests[0]?.init?.method, "GET");
  assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer local-read-token");
  assert.equal(new Headers(requests[0]?.init?.headers).get("cookie"), null);
  assert.equal(requests[0]?.init?.credentials, "omit");
  assert.equal(requests[0]?.init?.cache, "no-store");
});

test("rejects same commits before transport and redacts upstream failures", async (t) => {
  await t.test("same commit pair does not issue a request", async () => {
    let calls = 0;
    const client = new ContextLabLocalClient({
      fetch: async () => {
        calls += 1;
        return jsonResponse(reviewPayload());
      }
    });

    await assert.rejects(
      () => client.getPersistedContextDiffReview(
        PROJECT_ID,
        CONTEXT_ID,
        SOURCE_COMMIT_ID,
        SOURCE_COMMIT_ID,
        { bearerToken: "local-read-token" }
      ),
      RangeError
    );
    assert.equal(calls, 0);
  });

  await t.test("malformed API failure is redacted", async () => {
    const client = new ContextLabLocalClient({
      fetch: async () => new Response("private upstream diagnostic", { status: 502 })
    });

    await assert.rejects(
      () => client.getPersistedContextDiffReview(
        PROJECT_ID,
        CONTEXT_ID,
        SOURCE_COMMIT_ID,
        TARGET_COMMIT_ID,
        { bearerToken: "local-read-token" }
      ),
      (error: unknown) =>
        error instanceof ContextLabLocalPersistedContextDiffReviewError
        && error.code === "contextlab_local_api_error"
        && !error.message.includes("private upstream diagnostic")
    );
  });
});
