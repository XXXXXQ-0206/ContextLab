use crate::{
    ContextCommitSource, WorkflowCapabilitySnapshot, WorkflowCapabilitySnapshotSchemaVersion,
    WorkflowCapabilitySnapshotV1, WorkflowCapabilityValidationError, WorkflowContextBinding,
    WorkflowContextBindingId, WorkflowExecutionError, WorkflowNodeClaim, WorkflowNodeId,
    WorkflowNodeState, WorkflowReplayError, WorkflowRevision, WorkflowRunId, WorkflowRunState,
    WorkflowScheduler,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The explicit schema version for a deterministic workflow execution log.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowExecutionSchemaVersion {
    /// The first stable execution-log schema.
    V1,
}

/// One canonical event emitted by the provider-free execution state machine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "event", deny_unknown_fields)]
pub enum WorkflowExecutionEventV1 {
    /// A run was initialized from its exact immutable Workflow binding.
    RunStarted {
        /// The canonical event sequence number.
        sequence: u64,
    },
    /// A ready node was exclusively claimed by the scheduler.
    NodeClaimed {
        /// The canonical event sequence number.
        sequence: u64,
        /// The claimed node.
        node_id: WorkflowNodeId,
        /// The claimed attempt.
        attempt: u32,
    },
    /// A claimed node completed successfully.
    NodeSucceeded {
        /// The canonical event sequence number.
        sequence: u64,
        /// The completed node.
        node_id: WorkflowNodeId,
        /// The completed attempt.
        attempt: u32,
    },
    /// A claimed node failed with its typed provider-free failure.
    NodeFailed {
        /// The canonical event sequence number.
        sequence: u64,
        /// The failed node.
        node_id: WorkflowNodeId,
        /// The failed attempt.
        attempt: u32,
        /// The typed failure retained by the core log.
        failure: crate::WorkflowFailure,
    },
    /// The run reached a successful terminal state.
    RunSucceeded {
        /// The canonical event sequence number.
        sequence: u64,
    },
    /// The run reached a failed terminal state.
    RunFailed {
        /// The canonical event sequence number.
        sequence: u64,
        /// The node whose failure made the run terminal.
        failed_node_id: WorkflowNodeId,
    },
}

impl WorkflowExecutionEventV1 {
    fn sequence(&self) -> u64 {
        match self {
            Self::RunStarted { sequence }
            | Self::RunSucceeded { sequence }
            | Self::RunFailed { sequence, .. }
            | Self::NodeClaimed { sequence, .. }
            | Self::NodeSucceeded { sequence, .. }
            | Self::NodeFailed { sequence, .. } => *sequence,
        }
    }
}

/// A schema-versioned event log for one exact immutable Workflow binding and run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowExecutionLogV1 {
    schema_version: WorkflowExecutionSchemaVersion,
    binding_id: WorkflowContextBindingId,
    workflow_id: crate::WorkflowId,
    workflow_revision: WorkflowRevision,
    context_source: ContextCommitSource,
    capability_snapshot: WorkflowCapabilitySnapshotV1,
    capability_snapshot_digest: String,
    run_id: WorkflowRunId,
    replay_of: Option<WorkflowRunId>,
    events: Vec<WorkflowExecutionEventV1>,
}

impl WorkflowExecutionLogV1 {
    fn started(
        binding: &WorkflowContextBinding,
        run_id: WorkflowRunId,
        capability_snapshot: WorkflowCapabilitySnapshotV1,
    ) -> Self {
        let capability_snapshot_digest = capability_snapshot.canonical_digest();
        Self {
            schema_version: WorkflowExecutionSchemaVersion::V1,
            binding_id: binding.id(),
            workflow_id: binding.workflow_id(),
            workflow_revision: binding.workflow_revision(),
            context_source: binding.context_source(),
            capability_snapshot,
            capability_snapshot_digest,
            run_id,
            replay_of: None,
            events: vec![WorkflowExecutionEventV1::RunStarted { sequence: 1 }],
        }
    }

    /// Returns the canonical events in their append-only sequence order.
    #[must_use]
    pub fn events(&self) -> &[WorkflowExecutionEventV1] {
        &self.events
    }

    /// Returns the stable run identifier represented by this log.
    #[must_use]
    pub const fn run_id(&self) -> WorkflowRunId {
        self.run_id
    }

    /// Returns the source run for a fresh terminal replay, if applicable.
    #[must_use]
    pub const fn replay_of(&self) -> Option<WorkflowRunId> {
        self.replay_of
    }

    /// Returns the exact provider-free capability snapshot sealed into this log.
    #[must_use]
    pub const fn capability_snapshot(&self) -> &WorkflowCapabilitySnapshotV1 {
        &self.capability_snapshot
    }

    /// Returns the deterministic digest of the sealed capability snapshot.
    #[must_use]
    pub fn capability_snapshot_digest(&self) -> &str {
        &self.capability_snapshot_digest
    }
}

/// A provider-free deterministic execution state machine for one bound Workflow run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowExecution {
    binding: WorkflowContextBinding,
    scheduler: WorkflowScheduler,
    log: WorkflowExecutionLogV1,
}

impl WorkflowExecution {
    /// Starts one execution from an immutable Workflow-to-Context binding.
    pub fn start(
        binding: WorkflowContextBinding,
        run_id: WorkflowRunId,
        snapshot: &WorkflowCapabilitySnapshot,
    ) -> Result<Self, WorkflowExecutionStartError> {
        let scheduler =
            WorkflowScheduler::new(binding.workflow_definition().clone(), run_id, snapshot)
                .map_err(WorkflowExecutionStartError::CapabilityValidation)?;
        let log = WorkflowExecutionLogV1::started(&binding, run_id, snapshot.canonical_v1());
        Ok(Self {
            binding,
            scheduler,
            log,
        })
    }

    /// Claims the next ready node and appends its canonical event.
    pub fn claim_next(&mut self) -> Result<Option<WorkflowNodeClaim>, WorkflowExecutionError> {
        if matches!(
            self.scheduler.run_state(),
            WorkflowRunState::Succeeded | WorkflowRunState::Failed
        ) {
            return Ok(None);
        }
        let claim = self.scheduler.claim_next()?;
        if let Some(claim) = claim {
            self.push(WorkflowExecutionEventV1::NodeClaimed {
                sequence: self.next_sequence(),
                node_id: claim.node_id(),
                attempt: claim.attempt(),
            });
        } else if self.scheduler.run_state() == WorkflowRunState::Succeeded {
            self.push(WorkflowExecutionEventV1::RunSucceeded {
                sequence: self.next_sequence(),
            });
        }
        Ok(claim)
    }

    /// Records a successful node completion and an explicit terminal success when applicable.
    pub fn mark_succeeded(
        &mut self,
        node_id: WorkflowNodeId,
    ) -> Result<(), WorkflowExecutionError> {
        let attempt = running_attempt(&self.scheduler, node_id)?;
        self.scheduler.mark_succeeded(node_id)?;
        self.push(WorkflowExecutionEventV1::NodeSucceeded {
            sequence: self.next_sequence(),
            node_id,
            attempt,
        });
        if self.scheduler.run_state() == WorkflowRunState::Succeeded {
            self.push(WorkflowExecutionEventV1::RunSucceeded {
                sequence: self.next_sequence(),
            });
        }
        Ok(())
    }

    /// Records a failed node completion and an explicit terminal failure.
    pub fn mark_failed(
        &mut self,
        node_id: WorkflowNodeId,
        failure: crate::WorkflowFailure,
    ) -> Result<(), WorkflowExecutionError> {
        let attempt = running_attempt(&self.scheduler, node_id)?;
        self.scheduler.mark_failed(node_id, failure.clone())?;
        self.push(WorkflowExecutionEventV1::NodeFailed {
            sequence: self.next_sequence(),
            node_id,
            attempt,
            failure,
        });
        self.push(WorkflowExecutionEventV1::RunFailed {
            sequence: self.next_sequence(),
            failed_node_id: node_id,
        });
        Ok(())
    }

    /// Reconstructs one execution only when every event matches the bound scheduler transition.
    pub fn from_log(
        binding: WorkflowContextBinding,
        snapshot: &WorkflowCapabilitySnapshot,
        log: WorkflowExecutionLogV1,
    ) -> Result<Self, WorkflowExecutionReplayError> {
        validate_log_scope(&binding, &log)?;
        if let Some(claimed_source_run_id) = log.replay_of {
            if claimed_source_run_id == log.run_id {
                return Err(WorkflowExecutionReplayError::ReplaySourceIsCurrentRun {
                    run_id: log.run_id,
                });
            }
            return Err(
                WorkflowExecutionReplayError::ReplaySourceValidationRequired {
                    claimed_source_run_id,
                },
            );
        }
        validate_capability_snapshot_provenance(&log, snapshot)?;
        Self::reconstruct_log(binding, snapshot, log)
    }

    /// Reconstructs a replay log only from its exact validated terminal source execution.
    pub fn from_replay_log(
        binding: WorkflowContextBinding,
        snapshot: &WorkflowCapabilitySnapshot,
        log: WorkflowExecutionLogV1,
        source: &Self,
    ) -> Result<Self, WorkflowExecutionReplayError> {
        validate_log_scope(&binding, &log)?;
        let claimed_source_run_id = log
            .replay_of
            .ok_or(WorkflowExecutionReplayError::ReplaySourceMissing)?;
        if claimed_source_run_id == log.run_id {
            return Err(WorkflowExecutionReplayError::ReplaySourceIsCurrentRun {
                run_id: log.run_id,
            });
        }
        if source.binding != binding {
            return Err(WorkflowExecutionReplayError::ReplaySourceBindingMismatch {
                expected: binding.id(),
                provided: source.binding.id(),
            });
        }
        if claimed_source_run_id != source.log.run_id {
            return Err(WorkflowExecutionReplayError::ReplaySourceMismatch {
                expected: source.log.run_id,
                provided: claimed_source_run_id,
            });
        }
        validate_capability_snapshot_provenance(&source.log, snapshot)?;
        source
            .scheduler
            .replay(log.run_id)
            .map_err(WorkflowExecutionReplayError::SourceReplay)?;
        validate_capability_snapshot_provenance(&log, snapshot)?;
        Self::reconstruct_log(binding, snapshot, log)
    }

    fn reconstruct_log(
        binding: WorkflowContextBinding,
        snapshot: &WorkflowCapabilitySnapshot,
        log: WorkflowExecutionLogV1,
    ) -> Result<Self, WorkflowExecutionReplayError> {
        let mut execution = Self::start(binding, log.run_id, snapshot)
            .map_err(WorkflowExecutionReplayError::Start)?;
        execution.log.replay_of = log.replay_of;

        let mut terminal_marker_seen = false;
        for (index, event) in log.events.iter().enumerate() {
            let expected_sequence = (index as u64) + 1;
            if event.sequence() != expected_sequence {
                return Err(WorkflowExecutionReplayError::SequenceMismatch {
                    expected: expected_sequence,
                    provided: event.sequence(),
                });
            }
            if terminal_marker_seen {
                return Err(WorkflowExecutionReplayError::EventAfterTerminal {
                    sequence: event.sequence(),
                });
            }
            if index == 0 {
                if !matches!(event, WorkflowExecutionEventV1::RunStarted { .. }) {
                    return Err(WorkflowExecutionReplayError::MissingRunStarted);
                }
                continue;
            }

            match event {
                WorkflowExecutionEventV1::RunStarted { .. } => {
                    return Err(WorkflowExecutionReplayError::UnexpectedRunStarted {
                        sequence: event.sequence(),
                    });
                }
                WorkflowExecutionEventV1::NodeClaimed {
                    sequence,
                    node_id,
                    attempt,
                } => {
                    let claim = execution
                        .scheduler
                        .claim_next()
                        .map_err(|source| WorkflowExecutionReplayError::Transition {
                            sequence: *sequence,
                            source,
                        })?
                        .ok_or(WorkflowExecutionReplayError::MissingClaim {
                            sequence: *sequence,
                        })?;
                    if claim.node_id() != *node_id || claim.attempt() != *attempt {
                        return Err(WorkflowExecutionReplayError::ClaimMismatch {
                            sequence: *sequence,
                            expected_node_id: claim.node_id(),
                            expected_attempt: claim.attempt(),
                            provided_node_id: *node_id,
                            provided_attempt: *attempt,
                        });
                    }
                }
                WorkflowExecutionEventV1::NodeSucceeded {
                    sequence,
                    node_id,
                    attempt,
                } => {
                    validate_running_attempt(&execution.scheduler, *node_id, *attempt, *sequence)?;
                    execution
                        .scheduler
                        .mark_succeeded(*node_id)
                        .map_err(|source| WorkflowExecutionReplayError::Transition {
                            sequence: *sequence,
                            source,
                        })?;
                }
                WorkflowExecutionEventV1::NodeFailed {
                    sequence,
                    node_id,
                    attempt,
                    failure,
                } => {
                    validate_running_attempt(&execution.scheduler, *node_id, *attempt, *sequence)?;
                    execution
                        .scheduler
                        .mark_failed(*node_id, failure.clone())
                        .map_err(|source| WorkflowExecutionReplayError::Transition {
                            sequence: *sequence,
                            source,
                        })?;
                }
                WorkflowExecutionEventV1::RunSucceeded { sequence } => {
                    if execution.scheduler.run_state() != WorkflowRunState::Succeeded {
                        return Err(WorkflowExecutionReplayError::TerminalStateMismatch {
                            sequence: *sequence,
                            expected: WorkflowRunState::Succeeded,
                            actual: execution.scheduler.run_state(),
                        });
                    }
                    terminal_marker_seen = true;
                }
                WorkflowExecutionEventV1::RunFailed {
                    sequence,
                    failed_node_id,
                } => {
                    if execution.scheduler.run_state() != WorkflowRunState::Failed {
                        return Err(WorkflowExecutionReplayError::TerminalStateMismatch {
                            sequence: *sequence,
                            expected: WorkflowRunState::Failed,
                            actual: execution.scheduler.run_state(),
                        });
                    }
                    if !matches!(
                        execution.scheduler.node_state(*failed_node_id),
                        Some(WorkflowNodeState::Failed { .. })
                    ) {
                        return Err(WorkflowExecutionReplayError::FailedNodeMismatch {
                            sequence: *sequence,
                            node_id: *failed_node_id,
                        });
                    }
                    terminal_marker_seen = true;
                }
            }
            execution.log.events.push(event.clone());
        }

        if !matches!(
            execution.log.events.first(),
            Some(WorkflowExecutionEventV1::RunStarted { .. })
        ) {
            return Err(WorkflowExecutionReplayError::MissingRunStarted);
        }
        if matches!(
            execution.scheduler.run_state(),
            WorkflowRunState::Succeeded | WorkflowRunState::Failed
        ) && !terminal_marker_seen
        {
            return Err(WorkflowExecutionReplayError::MissingTerminalMarker {
                state: execution.scheduler.run_state(),
            });
        }
        if execution.log.events != log.events {
            return Err(WorkflowExecutionReplayError::LogMismatch);
        }
        Ok(execution)
    }

    /// Starts a fresh run from this terminal execution using the scheduler's existing replay rule.
    pub fn replay_as(&self, run_id: WorkflowRunId) -> Result<Self, WorkflowExecutionReplayError> {
        let scheduler = self
            .scheduler
            .replay(run_id)
            .map_err(WorkflowExecutionReplayError::SourceReplay)?;
        let mut log = WorkflowExecutionLogV1::started(
            &self.binding,
            run_id,
            self.log.capability_snapshot.clone(),
        );
        log.replay_of = Some(self.scheduler.run_id());
        Ok(Self {
            binding: self.binding.clone(),
            scheduler,
            log,
        })
    }

    /// Returns the immutable, schema-versioned execution log.
    #[must_use]
    pub const fn log(&self) -> &WorkflowExecutionLogV1 {
        &self.log
    }

    /// Returns the current state delegated from the existing scheduler.
    #[must_use]
    pub const fn run_state(&self) -> WorkflowRunState {
        self.scheduler.run_state()
    }

    /// Returns one node state delegated from the existing scheduler.
    #[must_use]
    pub fn node_state(&self, node_id: WorkflowNodeId) -> Option<&WorkflowNodeState> {
        self.scheduler.node_state(node_id)
    }

    fn next_sequence(&self) -> u64 {
        self.log
            .events
            .last()
            .map_or(1, |event| event.sequence() + 1)
    }

    fn push(&mut self, event: WorkflowExecutionEventV1) {
        self.log.events.push(event);
    }
}

fn validate_log_scope(
    binding: &WorkflowContextBinding,
    log: &WorkflowExecutionLogV1,
) -> Result<(), WorkflowExecutionReplayError> {
    if log.binding_id != binding.id() {
        return Err(WorkflowExecutionReplayError::BindingIdMismatch {
            expected: binding.id(),
            provided: log.binding_id,
        });
    }
    if log.workflow_id != binding.workflow_id()
        || log.workflow_revision != binding.workflow_revision()
    {
        return Err(WorkflowExecutionReplayError::WorkflowMismatch {
            expected_id: binding.workflow_id(),
            expected_revision: binding.workflow_revision(),
            provided_id: log.workflow_id,
            provided_revision: log.workflow_revision,
        });
    }
    if log.context_source != binding.context_source() {
        return Err(WorkflowExecutionReplayError::ContextSourceMismatch {
            expected: binding.context_source(),
            provided: log.context_source,
        });
    }
    Ok(())
}

fn validate_capability_snapshot_provenance(
    log: &WorkflowExecutionLogV1,
    snapshot: &WorkflowCapabilitySnapshot,
) -> Result<(), WorkflowExecutionReplayError> {
    if log.capability_snapshot.schema_version() != WorkflowCapabilitySnapshotSchemaVersion::V1 {
        return Err(WorkflowExecutionReplayError::CapabilitySnapshotSchemaMismatch);
    }

    for entries in log.capability_snapshot.capabilities().windows(2) {
        let previous = entries[0].capability();
        let current = entries[1].capability();
        match previous.cmp(current) {
            std::cmp::Ordering::Equal => {
                return Err(WorkflowExecutionReplayError::CapabilitySnapshotDuplicate {
                    capability: current.clone(),
                });
            }
            std::cmp::Ordering::Greater => {
                return Err(
                    WorkflowExecutionReplayError::CapabilitySnapshotOrderingMismatch {
                        previous: previous.clone(),
                        current: current.clone(),
                    },
                );
            }
            std::cmp::Ordering::Less => {}
        }
    }

    let canonical_wire_digest = log.capability_snapshot.canonical_digest();
    if log.capability_snapshot_digest != canonical_wire_digest {
        return Err(
            WorkflowExecutionReplayError::CapabilitySnapshotDigestMismatch {
                expected: canonical_wire_digest,
                provided: log.capability_snapshot_digest.clone(),
            },
        );
    }

    let expected = snapshot.canonical_v1();
    if log.capability_snapshot != expected {
        return Err(WorkflowExecutionReplayError::CapabilitySnapshotMismatch {
            expected_digest: expected.canonical_digest(),
            provided_digest: log.capability_snapshot_digest.clone(),
        });
    }
    Ok(())
}

fn validate_running_attempt(
    scheduler: &WorkflowScheduler,
    node_id: WorkflowNodeId,
    attempt: u32,
    sequence: u64,
) -> Result<(), WorkflowExecutionReplayError> {
    match scheduler.node_state(node_id) {
        Some(WorkflowNodeState::Running {
            attempt: actual_attempt,
        }) if *actual_attempt == attempt => Ok(()),
        Some(WorkflowNodeState::Running {
            attempt: actual_attempt,
        }) => Err(WorkflowExecutionReplayError::AttemptMismatch {
            sequence,
            node_id,
            expected: *actual_attempt,
            provided: attempt,
        }),
        Some(state) => Err(WorkflowExecutionReplayError::Transition {
            sequence,
            source: WorkflowExecutionError::NodeNotRunning {
                node_id,
                state: state.clone(),
            },
        }),
        None => Err(WorkflowExecutionReplayError::Transition {
            sequence,
            source: WorkflowExecutionError::UnknownNode { node_id },
        }),
    }
}

fn running_attempt(
    scheduler: &WorkflowScheduler,
    node_id: WorkflowNodeId,
) -> Result<u32, WorkflowExecutionError> {
    match scheduler.node_state(node_id) {
        Some(WorkflowNodeState::Running { attempt }) => Ok(*attempt),
        Some(state) => Err(WorkflowExecutionError::NodeNotRunning {
            node_id,
            state: state.clone(),
        }),
        None => Err(WorkflowExecutionError::UnknownNode { node_id }),
    }
}

/// Errors raised before a bound execution can begin.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WorkflowExecutionStartError {
    /// The injected existing capability snapshot cannot satisfy the binding definition.
    #[error("workflow execution cannot start because capabilities are invalid: {0}")]
    CapabilityValidation(WorkflowCapabilityValidationError),
}

/// Errors raised while reconstructing or creating a fresh replay of an execution log.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WorkflowExecutionReplayError {
    /// A serialized replay log claimed its own run as the source.
    #[error("workflow execution replay source cannot equal the current run {run_id:?}")]
    ReplaySourceIsCurrentRun {
        /// The self-referential run identifier.
        run_id: WorkflowRunId,
    },
    /// A replay log was submitted without a validated terminal source execution.
    #[error("workflow execution replay source requires terminal-source validation")]
    ReplaySourceValidationRequired {
        /// The unverified source run claimed by the serialized log.
        claimed_source_run_id: WorkflowRunId,
    },
    /// Replay-specific reconstruction was requested for a root execution log.
    #[error("workflow execution replay log does not declare a source run")]
    ReplaySourceMissing,
    /// The supplied source execution belongs to a different immutable binding.
    #[error("workflow execution replay source binding does not match the supplied binding")]
    ReplaySourceBindingMismatch {
        /// The binding required by the replay log.
        expected: WorkflowContextBindingId,
        /// The binding carried by the supplied source execution.
        provided: WorkflowContextBindingId,
    },
    /// The serialized replay source does not match the supplied source execution.
    #[error("workflow execution replay source run does not match the supplied source execution")]
    ReplaySourceMismatch {
        /// The run identifier from the supplied source execution.
        expected: WorkflowRunId,
        /// The source run identifier claimed by the replay log.
        provided: WorkflowRunId,
    },
    /// The log is for a different immutable binding record.
    #[error("workflow execution log binding does not match the supplied binding")]
    BindingIdMismatch {
        /// The supplied binding identifier.
        expected: WorkflowContextBindingId,
        /// The log binding identifier.
        provided: WorkflowContextBindingId,
    },
    /// The log workflow identity does not match the supplied binding definition.
    #[error("workflow execution log workflow identity does not match the supplied binding")]
    WorkflowMismatch {
        /// The supplied workflow identifier.
        expected_id: crate::WorkflowId,
        /// The supplied workflow revision.
        expected_revision: WorkflowRevision,
        /// The log workflow identifier.
        provided_id: crate::WorkflowId,
        /// The log workflow revision.
        provided_revision: WorkflowRevision,
    },
    /// The log Context commit source does not match the supplied binding.
    #[error("workflow execution log Context source does not match the supplied binding")]
    ContextSourceMismatch {
        /// The exact Context source from the supplied binding.
        expected: ContextCommitSource,
        /// The Context source declared by the log.
        provided: ContextCommitSource,
    },
    /// The log uses an unsupported capability snapshot schema.
    #[error("workflow execution log capability snapshot schema is unsupported")]
    CapabilitySnapshotSchemaMismatch,
    /// The sealed capability entries are not in canonical identifier order.
    #[error("workflow execution log capability snapshot ordering is not canonical")]
    CapabilitySnapshotOrderingMismatch {
        /// The preceding capability identifier.
        previous: crate::WorkflowCapabilityId,
        /// The following capability identifier.
        current: crate::WorkflowCapabilityId,
    },
    /// The sealed capability snapshot contains a duplicate identifier.
    #[error(
        "workflow execution log capability snapshot contains duplicate capability {capability}"
    )]
    CapabilitySnapshotDuplicate {
        /// The duplicated capability identifier.
        capability: crate::WorkflowCapabilityId,
    },
    /// The sealed snapshot digest does not match its canonical representation.
    #[error("workflow execution log capability snapshot digest does not match its representation")]
    CapabilitySnapshotDigestMismatch {
        /// The digest calculated from the supplied representation.
        expected: String,
        /// The digest retained in the log.
        provided: String,
    },
    /// The sealed snapshot is valid but differs from the snapshot supplied for replay.
    #[error("workflow execution log capability snapshot does not match the replay snapshot")]
    CapabilitySnapshotMismatch {
        /// The digest of the snapshot supplied for replay.
        expected_digest: String,
        /// The digest retained in the log.
        provided_digest: String,
    },
    /// The retained capability snapshot cannot start the bound Workflow.
    #[error("workflow execution log cannot start: {0}")]
    Start(WorkflowExecutionStartError),
    /// A canonical sequence number did not match its append-only position.
    #[error("workflow execution event sequence mismatch: expected {expected}, found {provided}")]
    SequenceMismatch {
        /// The expected contiguous sequence number.
        expected: u64,
        /// The supplied sequence number.
        provided: u64,
    },
    /// The first canonical event was not `run_started`.
    #[error("workflow execution log must start with run_started")]
    MissingRunStarted,
    /// A second `run_started` event appeared after the first event.
    #[error(
        "workflow execution log contains an unexpected run_started event at sequence {sequence}"
    )]
    UnexpectedRunStarted {
        /// The invalid sequence number.
        sequence: u64,
    },
    /// A claim event did not match the scheduler's deterministic next claim.
    #[error(
        "workflow execution claim does not match the deterministic scheduler at sequence {sequence}"
    )]
    ClaimMismatch {
        /// The invalid event sequence.
        sequence: u64,
        /// The scheduler-selected node.
        expected_node_id: WorkflowNodeId,
        /// The scheduler-selected attempt.
        expected_attempt: u32,
        /// The event node.
        provided_node_id: WorkflowNodeId,
        /// The event attempt.
        provided_attempt: u32,
    },
    /// The log claimed a node when the scheduler had no ready node.
    #[error("workflow execution log claims a node when none is ready at sequence {sequence}")]
    MissingClaim {
        /// The invalid event sequence.
        sequence: u64,
    },
    /// A completion event supplied a different attempt from the running scheduler attempt.
    #[error("workflow execution attempt does not match at sequence {sequence}")]
    AttemptMismatch {
        /// The invalid event sequence.
        sequence: u64,
        /// The completed node.
        node_id: WorkflowNodeId,
        /// The scheduler attempt.
        expected: u32,
        /// The event attempt.
        provided: u32,
    },
    /// A node transition was not accepted by the existing scheduler.
    #[error("workflow execution transition failed at sequence {sequence}: {source}")]
    Transition {
        /// The invalid event sequence.
        sequence: u64,
        /// The scheduler's structured transition error.
        source: WorkflowExecutionError,
    },
    /// A terminal marker disagreed with the scheduler's terminal state.
    #[error(
        "workflow execution terminal marker does not match the scheduler at sequence {sequence}"
    )]
    TerminalStateMismatch {
        /// The invalid event sequence.
        sequence: u64,
        /// The state implied by the marker.
        expected: WorkflowRunState,
        /// The actual scheduler state.
        actual: WorkflowRunState,
    },
    /// A failed terminal marker named a node that did not fail.
    #[error(
        "workflow execution failed terminal marker names a non-failed node at sequence {sequence}"
    )]
    FailedNodeMismatch {
        /// The invalid event sequence.
        sequence: u64,
        /// The node named by the marker.
        node_id: WorkflowNodeId,
    },
    /// A terminal marker was followed by another event.
    #[error("workflow execution log contains an event after terminal state at sequence {sequence}")]
    EventAfterTerminal {
        /// The invalid event sequence.
        sequence: u64,
    },
    /// A terminal scheduler state was not followed by its required terminal marker.
    #[error("workflow execution log is missing a terminal marker for state {state:?}")]
    MissingTerminalMarker {
        /// The scheduler terminal state.
        state: WorkflowRunState,
    },
    /// The reconstructed canonical log did not match the supplied log.
    #[error("workflow execution log does not match its canonical reconstruction")]
    LogMismatch,
    /// The source execution is not terminal or reused its source run identifier.
    #[error("workflow execution cannot start a fresh replay: {0}")]
    SourceReplay(WorkflowReplayError),
}
