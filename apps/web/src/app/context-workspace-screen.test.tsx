import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { fileURLToPath } from "node:url";

test("mounts the protected local benchmark workspace inspector below the three-column grid", () => {
  const screenSource = readFileSync(
    fileURLToPath(new URL("./context-workspace-screen.tsx", import.meta.url)),
    "utf8"
  );

  assert.match(screenSource, /className="benchmark-workspace-region"/);
  assert.match(
    screenSource,
    /import \{ LocalBenchmarkWorkspaceInspector \} from "\.\/local-benchmark-workspace-inspector"/
  );
  assert.match(
    screenSource,
    /import \{ LocalBenchmarkExecutionInspector \} from "\.\/local-benchmark-execution-inspector"/
  );
  assert.match(screenSource, /<LocalBenchmarkWorkspaceInspector/);
  assert.match(screenSource, /<LocalBenchmarkExecutionInspector/);
  assert.match(screenSource, /import \{ LocalKnowledgeMemoryProjectionInspector \} from "\.\/local-knowledge-memory-projection-inspector"/);
  assert.match(screenSource, /<LocalKnowledgeMemoryProjectionInspector/);
  assert.match(screenSource, /<LocalBenchmarkDefinitionAuthoringEditor/);
  assert.match(screenSource, /enabled=\{localLifecycleEnabled\}/);
  assert.match(screenSource, /key=\{benchmarkWorkspaceScopeKey\}/);
  assert.match(
    screenSource,
    /aria-label="Local benchmark workspace \/ 本地 Benchmark 工作台"/
  );
  assert.match(
    screenSource,
    /<\/div>\s*<section[\s\S]{0,160}?className="benchmark-workspace-region"/,
    "the full benchmark workflow must not be constrained by the Operations sidebar"
  );
  assert.doesNotMatch(
    screenSource,
    /<LocalBenchmarkWorkspaceInspector[\s\S]*?evaluationScorecard=/,
    "the browser workspace must render the server-owned projection instead of page scorecard data"
  );
});

test("keeps the mounted Context workspace connected to the Workflow binding status entry path", () => {
  const screenSource = readFileSync(
    fileURLToPath(new URL("./context-workspace-screen.tsx", import.meta.url)),
    "utf8"
  );
  const bindingsInspectorSource = readFileSync(
    fileURLToPath(new URL("./local-workflow-context-bindings-inspector.tsx", import.meta.url)),
    "utf8"
  );

  assert.match(
    screenSource,
    /import \{ LocalWorkflowContextBindingsInspector \} from "\.\/local-workflow-context-bindings-inspector"/
  );
  assert.match(
    screenSource,
    /<LocalWorkflowContextBindingsInspector\s+commitId=\{operations\.commitDetail\.commitId\}\s+contextId=\{operations\.graphReview\.contextId\}\s+key=\{`\$\{operations\.graphReview\.contextId\}:\$\{operations\.commitDetail\.commitId\}`\}\s*\/>/
  );
  assert.match(
    bindingsInspectorSource,
    /import \{ LocalWorkflowExecutionStatusScreen \} from "\.\/local-workflow-execution-status-screen"/
  );
  assert.match(bindingsInspectorSource, /<LocalWorkflowExecutionStatusScreen[\s\S]{0,320}?resource=/);
});
