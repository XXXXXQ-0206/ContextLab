# Context Graph Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the first tested Context Graph foundation so ContextLab can represent workspaces, projects, experiments, contexts, components, evaluations, and their relationships as explicit graph data.

**Architecture:** Graph behavior belongs in a framework-independent Rust crate named `contextlab-graph`. The API layer exposes a preview graph route by composing graph crate types; UI and API must not invent separate graph schemas.

**Tech Stack:** Rust stable, Cargo workspace, Serde, Axum, existing `context-core` identities.

---

## File Structure

- Modify: `Cargo.toml` to add `crates/graph` as a workspace member.
- Create: `crates/graph/Cargo.toml` for graph crate metadata and dependencies.
- Create: `crates/graph/src/lib.rs` for graph node, edge, error, and aggregate types.
- Modify: `server/api/Cargo.toml` to depend on `contextlab-graph`.
- Modify: `server/api/src/routes.rs` to expose `GET /api/v1/context-graph/preview`.
- Modify: `server/api/src/lib.rs` to register and test the route.
- Modify: `README.md` and `docs/roadmap/long-term-roadmap.md` to document the graph foundation.

## Task 1: Graph Domain Crate

- [x] **Step 1: Add workspace member**

```toml
"crates/graph",
```

- [x] **Step 2: Create `contextlab-graph` manifest**

```toml
[package]
name = "contextlab-graph"
version = "0.1.0"
edition.workspace = true
```

- [x] **Step 3: Implement graph aggregate**

```rust
pub struct ContextGraph {
    nodes: BTreeMap<GraphNodeId, GraphNode>,
    edges: Vec<GraphEdge>,
}
```

- [x] **Step 4: Verify graph tests**

Run: `cargo test -p contextlab-graph`

Expected: graph accepts valid edges, rejects dangling edges, and rejects duplicate nodes.

## Task 2: API Preview Route

- [x] **Step 1: Register route**

```rust
.route("/api/v1/context-graph/preview", axum::routing::get(routes::context_graph_preview))
```

- [x] **Step 2: Return a graph preview built from crate types**

```rust
Json(ContextGraphPreviewResponse {
    graph: ContextGraph::context_engineering_preview().expect("valid static graph"),
})
```

- [x] **Step 3: Verify API route test**

Run: `cargo test -p contextlab-api context_graph_preview_route_returns_graph`

Expected: response contains nodes, edges, and a `context` node.

## Task 3: Full Verification

- [x] **Step 1: Run complete check**

Run: `pnpm check`

Expected: Rust tests, TypeScript typecheck, and Next production build pass.

- [ ] **Step 2: Smoke test route**

Run: `curl http://127.0.0.1:3100/api/v1/context-graph/preview`

Expected: JSON response contains `graph.nodes` and `graph.edges`.
