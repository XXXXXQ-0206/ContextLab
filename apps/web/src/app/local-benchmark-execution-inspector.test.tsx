import assert from "node:assert/strict";
import test from "node:test";
import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { LocalBenchmarkExecutionInspector } from "./local-benchmark-execution-inspector";

test("execution inspector is default-off, memory-token based, exact-scope, and accessible", () => {
  const markup = renderToStaticMarkup(
    <LocalBenchmarkExecutionInspector
      projectId="aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa"
      contextId="bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb"
      commitId="cccccccc-cccc-4ccc-8ccc-cccccccccccc"
    />
  );
  assert.match(markup, /Private benchmark execution is off by default/);
  assert.match(markup, /仅内存/);
  assert.match(markup, /aria-label="Execute exact private benchmark/);
  assert.match(markup, /disabled=""/);
  assert.doesNotMatch(markup, /production-ready|production safe/i);
});
