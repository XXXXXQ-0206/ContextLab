//! Pure merge-base and ancestry-conflict contracts for Context history.

use crate::CommitId;
use contextlab_context_core::ContextId;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

/// One validated commit node used by the private ancestry contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitGraphNode {
    id: CommitId,
    context_id: ContextId,
    parent_ids: Vec<CommitId>,
}

impl CommitGraphNode {
    /// Creates a commit node. Graph-wide parent validation happens in [`CommitGraph::try_from_nodes`].
    #[must_use]
    pub fn new(id: CommitId, context_id: ContextId, parent_ids: Vec<CommitId>) -> Self {
        Self {
            id,
            context_id,
            parent_ids,
        }
    }

    /// Returns this commit identifier.
    #[must_use]
    pub const fn id(&self) -> CommitId {
        self.id
    }

    /// Returns the Context owning this commit.
    #[must_use]
    pub const fn context_id(&self) -> ContextId {
        self.context_id
    }

    /// Returns parents in their persisted order.
    #[must_use]
    pub fn parent_ids(&self) -> &[CommitId] {
        &self.parent_ids
    }
}

/// A finite, validated commit DAG.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitGraph {
    nodes: BTreeMap<String, CommitGraphNode>,
}

impl CommitGraph {
    /// Validates and constructs a commit graph from its complete node set.
    pub fn try_from_nodes(
        nodes: impl IntoIterator<Item = CommitGraphNode>,
    ) -> Result<Self, CommitGraphValidationError> {
        let mut indexed = BTreeMap::new();
        for node in nodes {
            let key = commit_key(node.id());
            if indexed.insert(key.clone(), node).is_some() {
                return Err(CommitGraphValidationError::DuplicateCommit { commit_id: key });
            }
        }

        for node in indexed.values() {
            let mut parents = BTreeSet::new();
            for parent_id in node.parent_ids() {
                let parent_key = commit_key(*parent_id);
                if !parents.insert(parent_key.clone()) {
                    return Err(CommitGraphValidationError::DuplicateParent {
                        child: commit_key(node.id()),
                        parent: parent_key,
                    });
                }
                let parent = indexed.get(&parent_key).ok_or_else(|| {
                    CommitGraphValidationError::UnknownParent {
                        child: commit_key(node.id()),
                        parent: parent_key.clone(),
                    }
                })?;
                if parent.context_id() != node.context_id() {
                    return Err(CommitGraphValidationError::CrossContextParent {
                        child: commit_key(node.id()),
                        parent: parent_key,
                        child_context: node.context_id(),
                        parent_context: parent.context_id(),
                    });
                }
            }
        }

        if let Some(expected_context) = indexed.values().next().map(CommitGraphNode::context_id) {
            for node in indexed.values() {
                if node.context_id() != expected_context {
                    return Err(CommitGraphValidationError::CrossContextNode {
                        commit_id: commit_key(node.id()),
                        expected_context,
                        actual_context: node.context_id(),
                    });
                }
            }
        }

        let mut visiting = BTreeSet::new();
        let mut visited = BTreeSet::new();
        for key in indexed.keys() {
            validate_acyclic(&indexed, key, &mut visiting, &mut visited)?;
        }

        Ok(Self { nodes: indexed })
    }

    /// Returns a validated node by exact identifier.
    #[must_use]
    pub fn node(&self, id: CommitId) -> Option<&CommitGraphNode> {
        self.nodes.get(&commit_key(id))
    }
}

/// Ancestry validation failures that must stop merge planning.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CommitGraphValidationError {
    /// Two records declared the same commit identity.
    #[error("duplicate commit in graph: {commit_id}")]
    DuplicateCommit {
        /// Duplicate commit identity.
        commit_id: String,
    },
    /// A commit listed a parent more than once.
    #[error("duplicate parent {parent} for commit {child}")]
    DuplicateParent {
        /// Child commit identity.
        child: String,
        /// Repeated parent identity.
        parent: String,
    },
    /// A parent record was not supplied in the graph.
    #[error("commit {child} refers to missing parent {parent}")]
    UnknownParent {
        /// Child commit identity.
        child: String,
        /// Missing parent identity.
        parent: String,
    },
    /// A parent belongs to another Context.
    #[error("commit {child} crosses Context scope to parent {parent}")]
    CrossContextParent {
        /// Child commit identity.
        child: String,
        /// Parent commit identity.
        parent: String,
        /// Context owning the child.
        child_context: ContextId,
        /// Context owning the parent.
        parent_context: ContextId,
    },
    /// A disconnected node belongs to another Context than the graph scope.
    #[error("commit {commit_id} belongs to a different Context than the graph")]
    CrossContextNode {
        /// Commit identity outside the graph Context.
        commit_id: String,
        /// Context selected by the first graph node.
        expected_context: ContextId,
        /// Context declared by the out-of-scope node.
        actual_context: ContextId,
    },
    /// Parent traversal found a cycle.
    #[error("commit ancestry contains a cycle at {commit_id}")]
    Cycle {
        /// Commit where the cycle was detected.
        commit_id: String,
    },
}

/// The ancestry-only result a future merge writer may consume.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergePlan {
    /// Both branch tips already identify the same immutable commit.
    NoOp {
        /// Shared tip identity.
        commit_id: CommitId,
    },
    /// One tip is an ancestor of the other and can be fast-forwarded.
    FastForward {
        /// Existing ancestor head.
        base: CommitId,
        /// Descendant target head.
        target: CommitId,
    },
    /// The tips have one deterministic common base and need a content-level merge decision.
    ThreeWay {
        /// Left branch tip.
        left: CommitId,
        /// Right branch tip.
        right: CommitId,
        /// Unique common ancestor.
        base: CommitId,
    },
    /// Ancestry is insufficient or ambiguous for a safe merge decision.
    Conflict {
        /// Left branch tip.
        left: CommitId,
        /// Right branch tip.
        right: CommitId,
        /// Ancestry-only conflict classification.
        reason: MergeBaseConflict,
    },
}

/// Ancestry conflicts that do not attempt to calculate content differences.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeBaseConflict {
    /// The two tips have no shared ancestor.
    NoCommonAncestor,
    /// Multiple incomparable common ancestors require an explicit policy.
    AmbiguousBases {
        /// Maximal common ancestors in stable commit-key order.
        bases: Vec<CommitId>,
    },
}

/// Errors resolving exact merge tips.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MergePlanError {
    /// A requested tip was not present in the validated graph.
    #[error("unknown merge tip: {commit_id}")]
    UnknownCommit {
        /// Missing tip identity.
        commit_id: String,
    },
    /// Merge tips belong to different Contexts.
    #[error("merge tips belong to different Contexts: {left} and {right}")]
    CrossContextTips {
        /// Context owning the left tip.
        left: ContextId,
        /// Context owning the right tip.
        right: ContextId,
    },
}

impl MergePlan {
    /// Resolves ancestry without calculating graph or content diffs.
    pub fn resolve(
        graph: &CommitGraph,
        left: CommitId,
        right: CommitId,
    ) -> Result<Self, MergePlanError> {
        let left_node = graph
            .node(left)
            .ok_or_else(|| MergePlanError::UnknownCommit {
                commit_id: commit_key(left),
            })?;
        let right_node = graph
            .node(right)
            .ok_or_else(|| MergePlanError::UnknownCommit {
                commit_id: commit_key(right),
            })?;
        if left_node.context_id() != right_node.context_id() {
            return Err(MergePlanError::CrossContextTips {
                left: left_node.context_id(),
                right: right_node.context_id(),
            });
        }
        if left == right {
            return Ok(Self::NoOp { commit_id: left });
        }

        let left_ancestors = ancestors_inclusive(graph, left);
        let right_ancestors = ancestors_inclusive(graph, right);
        if left_ancestors.contains(&commit_key(right)) {
            return Ok(Self::FastForward {
                base: right,
                target: left,
            });
        }
        if right_ancestors.contains(&commit_key(left)) {
            return Ok(Self::FastForward {
                base: left,
                target: right,
            });
        }

        let common = left_ancestors
            .intersection(&right_ancestors)
            .cloned()
            .collect::<BTreeSet<_>>();
        let bases = common
            .iter()
            .filter(|candidate| {
                !common
                    .iter()
                    .any(|other| candidate != &other && is_ancestor(graph, candidate, other))
            })
            .filter_map(|key| graph.nodes.get(key).map(CommitGraphNode::id))
            .collect::<Vec<_>>();

        match bases.as_slice() {
            [] => Ok(Self::Conflict {
                left,
                right,
                reason: MergeBaseConflict::NoCommonAncestor,
            }),
            [base] => Ok(Self::ThreeWay {
                left,
                right,
                base: *base,
            }),
            _ => Ok(Self::Conflict {
                left,
                right,
                reason: MergeBaseConflict::AmbiguousBases { bases },
            }),
        }
    }
}

fn validate_acyclic(
    nodes: &BTreeMap<String, CommitGraphNode>,
    key: &str,
    visiting: &mut BTreeSet<String>,
    visited: &mut BTreeSet<String>,
) -> Result<(), CommitGraphValidationError> {
    if visited.contains(key) {
        return Ok(());
    }
    if !visiting.insert(key.to_owned()) {
        return Err(CommitGraphValidationError::Cycle {
            commit_id: key.to_owned(),
        });
    }
    let node = &nodes[key];
    for parent_id in node.parent_ids() {
        validate_acyclic(nodes, &commit_key(*parent_id), visiting, visited)?;
    }
    visiting.remove(key);
    visited.insert(key.to_owned());
    Ok(())
}

fn ancestors_inclusive(graph: &CommitGraph, start: CommitId) -> BTreeSet<String> {
    let mut ancestors = BTreeSet::new();
    let mut pending = vec![start];
    while let Some(commit_id) = pending.pop() {
        let key = commit_key(commit_id);
        if !ancestors.insert(key) {
            continue;
        }
        pending.extend(
            graph.nodes[&commit_key(commit_id)]
                .parent_ids()
                .iter()
                .copied(),
        );
    }
    ancestors
}

fn is_ancestor(graph: &CommitGraph, ancestor: &str, descendant: &str) -> bool {
    ancestors_inclusive(graph, graph.nodes[descendant].id()).contains(ancestor)
}

fn commit_key(id: CommitId) -> String {
    id.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(context_id: ContextId, parent_ids: Vec<CommitId>) -> CommitGraphNode {
        CommitGraphNode::new(CommitId::new(), context_id, parent_ids)
    }

    fn graph(nodes: impl IntoIterator<Item = CommitGraphNode>) -> CommitGraph {
        CommitGraph::try_from_nodes(nodes).expect("valid commit graph")
    }

    #[test]
    fn resolves_no_op_and_fast_forward_plans() {
        let context_id = ContextId::new();
        let root = node(context_id, Vec::new());
        let root_id = root.id();
        let child = node(context_id, vec![root_id]);
        let child_id = child.id();
        let graph = graph([root, child]);

        assert_eq!(
            MergePlan::resolve(&graph, root_id, root_id).expect("no-op"),
            MergePlan::NoOp { commit_id: root_id }
        );
        assert_eq!(
            MergePlan::resolve(&graph, root_id, child_id).expect("fast-forward"),
            MergePlan::FastForward {
                base: root_id,
                target: child_id,
            }
        );
    }

    #[test]
    fn resolves_a_single_common_base_as_three_way() {
        let context_id = ContextId::new();
        let root = node(context_id, Vec::new());
        let root_id = root.id();
        let left = node(context_id, vec![root_id]);
        let left_id = left.id();
        let right = node(context_id, vec![root_id]);
        let right_id = right.id();
        let plan = MergePlan::resolve(&graph([root, left, right]), left_id, right_id)
            .expect("three-way plan");

        assert_eq!(
            plan,
            MergePlan::ThreeWay {
                left: left_id,
                right: right_id,
                base: root_id,
            }
        );
    }

    #[test]
    fn classifies_no_common_ancestor_as_conflict() {
        let context_id = ContextId::new();
        let left = node(context_id, Vec::new());
        let left_id = left.id();
        let right = node(context_id, Vec::new());
        let right_id = right.id();
        let plan =
            MergePlan::resolve(&graph([left, right]), left_id, right_id).expect("conflict plan");

        assert_eq!(
            plan,
            MergePlan::Conflict {
                left: left_id,
                right: right_id,
                reason: MergeBaseConflict::NoCommonAncestor,
            }
        );
    }

    #[test]
    fn classifies_ambiguous_common_bases_in_deterministic_order() {
        let context_id = ContextId::new();
        let root = node(context_id, Vec::new());
        let root_id = root.id();
        let first_base = node(context_id, vec![root_id]);
        let first_base_id = first_base.id();
        let second_base = node(context_id, vec![root_id]);
        let second_base_id = second_base.id();
        let left = node(context_id, vec![first_base_id, second_base_id]);
        let left_id = left.id();
        let right = node(context_id, vec![second_base_id, first_base_id]);
        let right_id = right.id();
        let plan = MergePlan::resolve(
            &graph([root, first_base, second_base, left, right]),
            left_id,
            right_id,
        )
        .expect("ambiguous conflict plan");

        let MergePlan::Conflict {
            reason: MergeBaseConflict::AmbiguousBases { bases },
            ..
        } = plan
        else {
            panic!("expected ambiguous bases");
        };
        assert_eq!(bases.len(), 2);
        assert!(bases[0].to_string() < bases[1].to_string());
    }

    #[test]
    fn rejects_missing_and_cross_context_parents() {
        let context_id = ContextId::new();
        let missing_parent = CommitId::new();
        let child = node(context_id, vec![missing_parent]);
        assert!(matches!(
            CommitGraph::try_from_nodes([child]),
            Err(CommitGraphValidationError::UnknownParent { .. })
        ));

        let foreign_context = ContextId::new();
        let foreign_parent = node(foreign_context, Vec::new());
        let child = node(context_id, vec![foreign_parent.id()]);
        assert!(matches!(
            CommitGraph::try_from_nodes([foreign_parent, child]),
            Err(CommitGraphValidationError::CrossContextParent { .. })
        ));
    }

    #[test]
    fn rejects_disconnected_nodes_from_another_context() {
        let graph_context = ContextId::new();
        let other_context = ContextId::new();
        let first = node(graph_context, Vec::new());
        let disconnected = node(other_context, Vec::new());

        assert!(matches!(
            CommitGraph::try_from_nodes([first, disconnected]),
            Err(CommitGraphValidationError::CrossContextNode { .. })
        ));
    }

    #[test]
    fn rejects_duplicate_parents_and_cycles() {
        let context_id = ContextId::new();
        let parent = node(context_id, Vec::new());
        let parent_id = parent.id();
        let duplicate = node(context_id, vec![parent_id, parent_id]);
        assert!(matches!(
            CommitGraph::try_from_nodes([parent.clone(), duplicate]),
            Err(CommitGraphValidationError::DuplicateParent { .. })
        ));

        let first = node(context_id, Vec::new());
        let first_id = first.id();
        let second = node(context_id, vec![first_id]);
        let second_id = second.id();
        let cyclic_first = CommitGraphNode::new(first_id, context_id, vec![second_id]);
        assert!(matches!(
            CommitGraph::try_from_nodes([cyclic_first, second]),
            Err(CommitGraphValidationError::Cycle { .. })
        ));
    }

    #[test]
    fn rejects_unknown_merge_tips() {
        let context_id = ContextId::new();
        let left = node(context_id, Vec::new());
        let left_id = left.id();
        let graph = graph([left]);
        assert!(matches!(
            MergePlan::resolve(&graph, left_id, CommitId::new()),
            Err(MergePlanError::UnknownCommit { .. })
        ));
    }
}
