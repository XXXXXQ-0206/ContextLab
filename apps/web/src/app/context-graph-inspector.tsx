"use client";

import {
  DefinitionGrid,
  Panel,
  PanelHeader,
  StackTable,
  StatGrid,
  StatusPill
} from "@contextlab/ui";
import { Network } from "lucide-react";
import { useState, type CSSProperties } from "react";
import {
  presentGraphNodeInspector,
  type ContextGraphViewModel,
  type GraphRelationshipItem
} from "./context-workspace-presenter";

type GraphRelationshipTableProps = {
  ariaLabel: string;
  emptyLabel: string;
  relationships: GraphRelationshipItem[];
};

function GraphRelationshipTable({ ariaLabel, emptyLabel, relationships }: GraphRelationshipTableProps) {
  const rows =
    relationships.length > 0
      ? relationships.map((relationship) => ({
          id: relationship.id,
          cells: [
            <div className="graph-relationship__entity" key={`${relationship.id}-source`}>
              <strong>{relationship.sourceLabel}</strong>
              <span>{relationship.sourceKind}</span>
            </div>,
            <StatusPill key={`${relationship.id}-relationship`} tone="info">
              {relationship.relationshipLabel}
            </StatusPill>,
            <div className="graph-relationship__entity" key={`${relationship.id}-target`}>
              <strong>{relationship.targetLabel}</strong>
              <span>{relationship.targetKind}</span>
            </div>
          ]
        }))
      : [
          {
            id: `${ariaLabel}-empty`,
            cells: [
              <span className="graph-relationship__empty" key={`${ariaLabel}-empty-label`}>
                {emptyLabel}
              </span>,
              "-",
              "-"
            ]
          }
        ];

  return (
    <StackTable
      aria-label={ariaLabel}
      columnTemplate="minmax(0, 1fr) minmax(7.5rem, 0.55fr) minmax(0, 1fr)"
      headers={["Source / 来源", "Relationship / 关系", "Target / 目标"]}
      rows={rows}
    />
  );
}

export function ContextGraphInspector({ graph }: { graph: ContextGraphViewModel }) {
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(graph.inspectorNodes[0]?.id ?? null);
  const inspector = presentGraphNodeInspector(graph, selectedNodeId);

  return (
    <Panel as="section" className="graph-panel" id="graph" aria-labelledby="graph-heading">
      <PanelHeader actions={<StatusPill tone="info">read-only / 只读</StatusPill>}>
        <h2 className="panel-title" id="graph-heading">
          Context Graph
        </h2>
        <p className="panel-caption">Explicit relationships across workspace resources</p>
      </PanelHeader>

      <div className="graph-stage" role="group" aria-label="Context graph node selection">
        {graph.edges.map((edge) => (
          <span
            className="graph-line"
            key={edge.id}
            style={
              {
                "--angle": edge.angle,
                "--length": edge.length,
                "--x1": edge.x1,
                "--y1": edge.y1
              } as CSSProperties
            }
          />
        ))}
        {graph.nodes.length > 0 ? (
          graph.nodes.map((node) => (
            <button
              aria-label={`Select ${node.detail} (${node.kindLabel})`}
              aria-pressed={node.id === selectedNodeId}
              className="graph-node"
              data-root={node.root}
              data-selected={node.id === selectedNodeId}
              key={node.id}
              onClick={() => setSelectedNodeId(node.id)}
              style={
                {
                  "--x": node.x,
                  "--y": node.y
                } as CSSProperties
              }
              type="button"
            >
              <strong>{node.kindLabel}</strong>
              <span>{node.detail}</span>
            </button>
          ))
        ) : (
          <p className="graph-empty">No graph nodes / 暂无图节点</p>
        )}
      </div>

      <StatGrid aria-label="Context score facts" className="graph-score-grid" items={graph.scoreFacts} />

      <label className="graph-node-selector">
        <span>Inspect graph node / 检查图节点</span>
        <select
          aria-label="Select context graph node"
          disabled={graph.inspectorNodes.length === 0}
          onChange={(event) => setSelectedNodeId(event.target.value || null)}
          value={selectedNodeId ?? ""}
        >
          {graph.inspectorNodes.length === 0 ? <option value="">No graph nodes / 暂无图节点</option> : null}
          {graph.inspectorNodes.map((node) => (
            <option key={node.id} value={node.id}>
              {node.kindLabel}: {node.detail}
            </option>
          ))}
        </select>
      </label>

      {inspector.selectedNode ? (
        <div className="graph-inspector" aria-labelledby="graph-inspector-heading">
          <div className="block-heading">
            <Network aria-hidden="true" />
            <span id="graph-inspector-heading">Selected Node / 已选节点</span>
            <StatusPill tone="info">{inspector.selectedNode.kindLabel}</StatusPill>
          </div>
          <DefinitionGrid
            aria-label="Selected context graph node detail"
            columns={1}
            compact
            items={[
              { id: "node-id", label: "Node ID / 节点标识", value: inspector.selectedNode.id },
              { id: "node-type", label: "Type / 类型", value: inspector.selectedNode.kindLabel },
              { id: "node-label", label: "Name / 名称", value: inspector.selectedNode.detail }
            ]}
            surface="raised"
            valueTone="info"
          />
          <div className="graph-inspector__relationships">
            <div className="graph-inspector__relationship-block">
              <h3>Incoming Relationships / 入边</h3>
              <GraphRelationshipTable
                ariaLabel="Selected node incoming relationships"
                emptyLabel="No incoming relationships / 暂无入边"
                relationships={inspector.incomingRelationships}
              />
            </div>
            <div className="graph-inspector__relationship-block">
              <h3>Outgoing Relationships / 出边</h3>
              <GraphRelationshipTable
                ariaLabel="Selected node outgoing relationships"
                emptyLabel="No outgoing relationships / 暂无出边"
                relationships={inspector.outgoingRelationships}
              />
            </div>
          </div>
        </div>
      ) : null}

      <div className="graph-relationships" aria-labelledby="graph-relationships-heading">
        <div className="block-heading">
          <Network aria-hidden="true" />
          <span id="graph-relationships-heading">Graph Relationships / 图谱关系</span>
        </div>
        <GraphRelationshipTable
          ariaLabel="Context graph relationships"
          emptyLabel="No relationships / 暂无关系"
          relationships={graph.relationships}
        />
      </div>
    </Panel>
  );
}
