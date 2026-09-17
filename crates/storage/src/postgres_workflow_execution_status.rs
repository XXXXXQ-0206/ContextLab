//! SQLx adapter for immutable Workflow execution status projections.

use super::PostgresContextGraphRepository;
use crate::{
    PersistWorkflowExecutionStatusV1, WorkflowExecutionStatusPersistenceError,
    WorkflowExecutionStatusRepository, WorkflowExecutionStatusWriteDisposition,
    WorkflowExecutionStatusWriteResult,
};
use async_trait::async_trait;
use contextlab_context_core::ContextId;
use contextlab_workflow::{
    WorkflowExecutionStatusProjectionSchemaVersion, WorkflowExecutionStatusProjectionV1,
    WorkflowRunId,
};
use serde_json::Value;
use sqlx::Postgres;
use uuid::Uuid;

const WORKFLOW_EXECUTION_STATUS_SCHEMA_V1: &str = "v1";

const EXISTING_PROJECTION_SQL: &str = r#"
SELECT project_id, context_id, context_commit_id, run_id, schema_version, projection
FROM workflow_execution_status_projections
WHERE context_id = $1 AND run_id = $2
FOR UPDATE
"#;

const INSERT_PROJECTION_SQL: &str = r#"
INSERT INTO workflow_execution_status_projections
    (project_id, context_id, context_commit_id, run_id, schema_version, projection)
SELECT contexts.project_id, $1, $2, $3, $4, $5
FROM contexts
WHERE contexts.id = $1
RETURNING project_id, context_id, context_commit_id, run_id, schema_version, projection
"#;

const READ_PROJECTION_SQL: &str = r#"
SELECT project_id, context_id, context_commit_id, run_id, schema_version, projection
FROM workflow_execution_status_projections
WHERE context_id = $1 AND run_id = $2
"#;

fn invalid_scope(reason: &'static str) -> WorkflowExecutionStatusPersistenceError {
    WorkflowExecutionStatusPersistenceError::InvalidScope { reason }
}

fn validate_projection(
    projection: &WorkflowExecutionStatusProjectionV1,
) -> Result<(), WorkflowExecutionStatusPersistenceError> {
    if projection.schema_version() != WorkflowExecutionStatusProjectionSchemaVersion::V1 {
        return Err(invalid_scope("unsupported projection schema"));
    }
    if projection.context_source().context_id().as_uuid().is_nil() {
        return Err(invalid_scope("Context identifier is nil"));
    }
    if projection.context_source().commit_id().as_uuid().is_nil() {
        return Err(invalid_scope("Context commit identifier is nil"));
    }
    if projection.run_id().as_uuid().is_nil() {
        return Err(invalid_scope("workflow run identifier is nil"));
    }
    Ok(())
}

fn database_error(_error: sqlx::Error) -> WorkflowExecutionStatusPersistenceError {
    WorkflowExecutionStatusPersistenceError::RepositoryUnavailable
}

fn decode_projection(
    project_id: Uuid,
    context_id: Uuid,
    context_commit_id: Uuid,
    run_id: Uuid,
    schema_version: String,
    projection: Value,
) -> Result<WorkflowExecutionStatusProjectionV1, WorkflowExecutionStatusPersistenceError> {
    if project_id.is_nil() {
        return Err(invalid_scope("stored project identifier is nil"));
    }
    if schema_version != WORKFLOW_EXECUTION_STATUS_SCHEMA_V1 {
        return Err(invalid_scope("stored projection schema is unsupported"));
    }
    let projection = serde_json::from_value::<WorkflowExecutionStatusProjectionV1>(projection)
        .map_err(|_| invalid_scope("stored projection JSON is invalid"))?;
    validate_projection(&projection)?;
    if projection.context_source().context_id().as_uuid() != context_id
        || projection.context_source().commit_id().as_uuid() != context_commit_id
        || projection.run_id().as_uuid() != run_id
    {
        return Err(invalid_scope(
            "stored projection scope does not match database columns",
        ));
    }
    Ok(projection)
}

type ProjectionRow = (Uuid, Uuid, Uuid, Uuid, String, Value);

fn decode_row(
    row: ProjectionRow,
) -> Result<WorkflowExecutionStatusProjectionV1, WorkflowExecutionStatusPersistenceError> {
    decode_projection(row.0, row.1, row.2, row.3, row.4, row.5)
}

async fn lock_projection_scope(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    context_id: Uuid,
    run_id: Uuid,
) -> Result<(), WorkflowExecutionStatusPersistenceError> {
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
        .bind(format!("workflow-execution-status:{context_id}:{run_id}"))
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    Ok(())
}

#[async_trait]
impl WorkflowExecutionStatusRepository for PostgresContextGraphRepository {
    async fn persist_workflow_execution_status(
        &self,
        command: PersistWorkflowExecutionStatusV1,
    ) -> Result<WorkflowExecutionStatusWriteResult, WorkflowExecutionStatusPersistenceError> {
        validate_projection(command.projection())?;
        let projection = command.projection().clone();
        let context_id = projection.context_source().context_id().as_uuid();
        let context_commit_id = projection.context_source().commit_id().as_uuid();
        let run_id = projection.run_id().as_uuid();
        let payload = serde_json::to_value(&projection)
            .map_err(|_| invalid_scope("projection JSON serialization failed"))?;

        let mut transaction = self.pool().begin().await.map_err(database_error)?;
        lock_projection_scope(&mut transaction, context_id, run_id).await?;

        let existing = sqlx::query_as::<_, ProjectionRow>(EXISTING_PROJECTION_SQL)
            .bind(context_id)
            .bind(run_id)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(database_error)?;

        if let Some(row) = existing {
            let stored = match decode_row(row) {
                Ok(stored) => stored,
                Err(error) => {
                    transaction.rollback().await.map_err(database_error)?;
                    return Err(error);
                }
            };
            if stored != projection {
                transaction.rollback().await.map_err(database_error)?;
                return Err(WorkflowExecutionStatusPersistenceError::Conflict {
                    context_id: context_id.to_string(),
                    run_id: run_id.to_string(),
                });
            }
            transaction.commit().await.map_err(database_error)?;
            return Ok(WorkflowExecutionStatusWriteResult::new(
                projection,
                WorkflowExecutionStatusWriteDisposition::Replayed,
            ));
        }

        let stored_row = sqlx::query_as::<_, ProjectionRow>(INSERT_PROJECTION_SQL)
            .bind(context_id)
            .bind(context_commit_id)
            .bind(run_id)
            .bind(WORKFLOW_EXECUTION_STATUS_SCHEMA_V1)
            .bind(payload)
            .fetch_one(&mut *transaction)
            .await
            .map_err(database_error)?;
        let stored = match decode_row(stored_row) {
            Ok(stored) => stored,
            Err(error) => {
                transaction.rollback().await.map_err(database_error)?;
                return Err(error);
            }
        };
        if stored != projection {
            transaction.rollback().await.map_err(database_error)?;
            return Err(invalid_scope(
                "stored projection does not match the requested projection",
            ));
        }
        transaction.commit().await.map_err(database_error)?;
        Ok(WorkflowExecutionStatusWriteResult::new(
            projection,
            WorkflowExecutionStatusWriteDisposition::Created,
        ))
    }

    async fn read_workflow_execution_status(
        &self,
        context_id: ContextId,
        run_id: WorkflowRunId,
    ) -> Result<Option<WorkflowExecutionStatusProjectionV1>, WorkflowExecutionStatusPersistenceError>
    {
        if context_id.as_uuid().is_nil() || run_id.as_uuid().is_nil() {
            return Err(invalid_scope("read scope contains a nil identifier"));
        }

        let mut transaction = super::begin_consistent_read_transaction(self.pool())
            .await
            .map_err(|_| WorkflowExecutionStatusPersistenceError::RepositoryUnavailable)?;
        let row = sqlx::query_as::<_, ProjectionRow>(READ_PROJECTION_SQL)
            .bind(context_id.as_uuid())
            .bind(run_id.as_uuid())
            .fetch_optional(&mut *transaction)
            .await
            .map_err(database_error)?;
        let projection = row.map(decode_row).transpose()?;
        transaction.commit().await.map_err(database_error)?;
        Ok(projection)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextlab_context_core::ContextId;
    use contextlab_versioning::CommitId;
    use contextlab_workflow::{
        ContextCommitSource, WorkflowCapabilitySnapshot, WorkflowContextBinding,
        WorkflowContextBindingId, WorkflowDefinition, WorkflowExecution, WorkflowId, WorkflowNode,
        WorkflowNodeId, WorkflowRevision,
    };

    fn uuid(value: u128) -> Uuid {
        Uuid::from_u128(value)
    }

    fn projection() -> WorkflowExecutionStatusProjectionV1 {
        let revision = WorkflowRevision::new(1).expect("revision");
        let binding = WorkflowContextBinding::new(
            WorkflowContextBindingId::from_uuid(uuid(101)),
            WorkflowDefinition::new(
                WorkflowId::from_uuid(uuid(102)),
                revision,
                vec![WorkflowNode::new(
                    WorkflowNodeId::from_uuid(uuid(103)),
                    revision,
                )],
                Vec::new(),
            )
            .expect("definition"),
            ContextCommitSource::new(
                ContextId::from_uuid(uuid(104)),
                CommitId::from_uuid(uuid(105)),
            ),
        );
        let execution = WorkflowExecution::start(
            binding.clone(),
            WorkflowRunId::from_uuid(uuid(106)),
            &WorkflowCapabilitySnapshot::default(),
        )
        .expect("execution");
        WorkflowExecutionStatusProjectionV1::from_log(
            binding,
            &WorkflowCapabilitySnapshot::default(),
            execution.log().clone(),
        )
        .expect("projection")
    }

    #[test]
    fn projection_json_rehydrates_with_exact_database_scope() {
        let projection = projection();
        let payload = serde_json::to_value(&projection).expect("serialize projection");
        let restored = decode_projection(
            uuid(100),
            projection.context_source().context_id().as_uuid(),
            projection.context_source().commit_id().as_uuid(),
            projection.run_id().as_uuid(),
            WORKFLOW_EXECUTION_STATUS_SCHEMA_V1.to_owned(),
            payload,
        )
        .expect("decode projection");
        assert_eq!(restored, projection);
    }

    #[test]
    fn decode_rejects_unknown_schema_unknown_fields_and_scope_drift() {
        let projection = projection();
        let payload = serde_json::to_value(&projection).expect("serialize projection");

        assert!(matches!(
            decode_projection(
                uuid(100),
                projection.context_source().context_id().as_uuid(),
                projection.context_source().commit_id().as_uuid(),
                projection.run_id().as_uuid(),
                "v99".to_owned(),
                payload.clone(),
            ),
            Err(WorkflowExecutionStatusPersistenceError::InvalidScope { .. })
        ));

        let mut unknown = payload.clone();
        unknown
            .as_object_mut()
            .expect("projection object")
            .insert("unexpected".to_owned(), Value::Bool(true));
        assert!(matches!(
            decode_projection(
                uuid(100),
                projection.context_source().context_id().as_uuid(),
                projection.context_source().commit_id().as_uuid(),
                projection.run_id().as_uuid(),
                WORKFLOW_EXECUTION_STATUS_SCHEMA_V1.to_owned(),
                unknown,
            ),
            Err(WorkflowExecutionStatusPersistenceError::InvalidScope { .. })
        ));

        assert!(matches!(
            decode_projection(
                uuid(100),
                uuid(999),
                projection.context_source().commit_id().as_uuid(),
                projection.run_id().as_uuid(),
                WORKFLOW_EXECUTION_STATUS_SCHEMA_V1.to_owned(),
                payload,
            ),
            Err(WorkflowExecutionStatusPersistenceError::InvalidScope { .. })
        ));
    }

    #[test]
    fn sql_reads_only_the_redacted_projection_and_preserves_exact_scope() {
        for sql in [
            EXISTING_PROJECTION_SQL,
            INSERT_PROJECTION_SQL,
            READ_PROJECTION_SQL,
        ] {
            assert!(sql.contains("context_id"));
            assert!(sql.contains("run_id"));
            assert!(sql.contains("projection"));
            assert!(!sql.contains("execution_events"));
            assert!(!sql.contains("failure_payload"));
        }
        assert!(EXISTING_PROJECTION_SQL.contains("FOR UPDATE"));
        assert!(INSERT_PROJECTION_SQL.contains("contexts.project_id"));
    }
}
