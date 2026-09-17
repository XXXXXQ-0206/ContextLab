#!/usr/bin/env bash
set -euo pipefail

database_url="${CONTEXTLAB_TEST_DATABASE_URL:-}"
expected_database_name="${CONTEXTLAB_TEST_DATABASE_NAME:-contextlab_test}"
expected_database_role="${CONTEXTLAB_TEST_DATABASE_ROLE:-contextlab_test_runner}"

if [[ -z "$database_url" ]]; then
  echo "CONTEXTLAB_TEST_DATABASE_URL must name a disposable PostgreSQL database." >&2
  exit 1
fi

if [[ "${CONTEXTLAB_ALLOW_DISPOSABLE_DATABASE_RESET:-}" != "1" ]]; then
  echo "CONTEXTLAB_ALLOW_DISPOSABLE_DATABASE_RESET=1 is required before schema reset." >&2
  exit 1
fi

case "$database_url" in
  postgres://*@localhost:*|postgresql://*@localhost:*|postgres://*@127.0.0.1:*|postgresql://*@127.0.0.1:*|postgres://*@\[::1\]:*|postgresql://*@\[::1\]:*)
    ;;
  *)
    echo "CONTEXTLAB_TEST_DATABASE_URL must use a loopback PostgreSQL host." >&2
    exit 1
    ;;
esac

if ! command -v psql >/dev/null 2>&1; then
  echo "psql is required to reset the disposable PostgreSQL schema." >&2
  exit 1
fi

active_database_name="$(psql --dbname="$database_url" -X -At -v ON_ERROR_STOP=1 -c 'SELECT current_database()')"
if [[ "$active_database_name" != "$expected_database_name" ]]; then
  echo "The disposable database name does not match CONTEXTLAB_TEST_DATABASE_NAME." >&2
  exit 1
fi

active_database_role="$(psql --dbname="$database_url" -X -At -v ON_ERROR_STOP=1 -c 'SELECT current_user')"
if [[ "$active_database_role" != "$expected_database_role" ]]; then
  echo "The disposable database role does not match CONTEXTLAB_TEST_DATABASE_ROLE." >&2
  exit 1
fi

storage_tests=(
  postgres::tests::principal_identity_migration_preserves_legacy_memberships_and_checks_new_writes
  postgres::tests::audit_retention_migration_defaults_existing_events_to_hold_and_preserves_append_only
  postgres::tests::postgres_restricted_audit_purge_requires_executor_role_and_preserves_manifest_evidence
  postgres::tests::production_like_migration_rehearsal_upgrades_history_and_preserves_ledger
  postgres::tests::postgres_shared_protected_route_rate_limiter_is_atomic_across_independent_pools
  postgres::tests::postgres_shared_protected_route_rate_limiter_isolates_identities_recovers_expiry_and_rejects_policy_drift
  postgres::tests::postgres_shared_protected_route_rate_limiter_does_not_block_existing_keys_on_global_admission_lock
  postgres::tests::postgres_shared_protected_route_rate_limiter_fails_closed_for_a_future_state_timestamp
  postgres::tests::projects_seed_workspace_graph_from_postgres
  postgres::tests::postgres_benchmark_definition_binding_replays_and_reads_exact_scope
  postgres::tests::postgres_group_editor_can_write_but_direct_reader_overrides_the_group
  postgres::tests::postgres_guarded_writer_replays_the_same_key_concurrently
  postgres::tests::postgres_guarded_component_content_update_replays_and_projects_revision
  postgres::tests::postgres_guarded_component_content_creation_replays_and_projects_initial_revision
  postgres::tests::component_content_initial_revision_migration_preserves_history_and_enforces_null_prior
  postgres::tests::component_content_initial_revision_integrity_rejects_malformed_null_prior
  postgres::tests::parent_scope_migration_backfills_valid_commit_parent_history
  postgres::tests::parent_scope_migration_rejects_legacy_cross_context_parent_history
  postgres::tests::postgres_context_commit_parent_scope_rejects_a_cross_context_direct_insert
  postgres::tests::postgres_component_content_at_commit_replays_nearest_normal_parent_revision
  postgres::tests::postgres_guarded_writer_allows_only_one_concurrent_stale_head_writer
  postgres::tests::postgres_guarded_scope_constraints_reject_cross_context_references
  postgres::tests::postgres_authorizer_reports_membership_database_failures_as_unavailable
  postgres::tests::postgres_authorization_audit_sink_persists_safe_decisions
  postgres::tests::postgres_authorization_audit_review_is_redacted_and_cursor_paginated
  postgres::tests::guarded_writer_replays_and_advances_a_matching_branch_head
)

for storage_test in "${storage_tests[@]}"; do
  psql --dbname="$database_url" -X -v ON_ERROR_STOP=1 <<'SQL'
DROP SCHEMA public CASCADE;
CREATE SCHEMA public;
GRANT ALL ON SCHEMA public TO CURRENT_USER;
SQL

  cargo test -p contextlab-storage "$storage_test" --lib -- --ignored --exact --test-threads=1
done
